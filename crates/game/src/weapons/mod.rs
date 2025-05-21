pub mod ui;

use animation::{create_child_sprite, AnimationBundle, FacingDirection, SpriteSheetConfig};
use bevy::{math::VectorSpace, prelude::*, utils::{HashMap, HashSet}};
use bevy_ggrs::{AddRollbackCommandExtension, PlayerInputs, Rollback};
use ggrs::PlayerHandle;
use serde::{Deserialize, Serialize};
use utils::{bmap, rng::RollbackRng, math::round};

use crate::{character::{dash::DashState, health::{self, DamageAccumulator, Health}, movement::SprintState, player::{input::{CursorPosition, INPUT_DASH, INPUT_RELOAD, INPUT_SPRINT, INPUT_SWITCH_WEAPON_MODE}, jjrs::PeerConfig, Player}}, collider::{is_colliding, Collider, ColliderShape, CollisionLayer, CollisionSettings, Wall}, frame::FrameCount, global_asset::GlobalAsset};

// ROOLBACL

// COMPONENTS

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum FiringMode {
    Automatic{},  // Hold trigger to continuously fire
    Manual{},     // One shot per trigger pull
    Burst{pellets_per_shot: u32, cooldown_frames: u32},      // Fire a fixed number of shots per trigger pull
    Shotgun{pellet_count: u32, spread_angle: f32},
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MagBulletConfig {
    Mag {
        mag_size: u32,
        mag_limit: u32,
    },
    Magless {
        bullet_limit: u32,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum BulletType {
    Standard {
        damage: f32,
        speed: f32,
    },
    Explosive {
        damage: f32,
        speed: f32,
        blast_radius: f32,
        explosive_damage_multiplier: f32,
    },
    Piercing {
        damage: f32,
        speed: f32,
        penetration: u8,
    },
}

#[derive(Component)]
pub struct ExplosiveTag;

#[derive(Component)]
pub struct PiercingTag;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FiringModeConfig {
    pub firing_rate: f32,
    pub firing_mode: FiringMode,
    pub spread: f32,
    pub recoil: f32,
    pub bullet_type: BulletType,
    pub range: f32,

    pub reload_time_seconds: f32,
    pub mag: MagBulletConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WeaponConfig {
    pub name: String,
    pub default_firing_mode: String,
    pub firing_modes: HashMap<String, FiringModeConfig>,
}



#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WeaponSpriteConfig {
    pub name: String,
    pub index: usize,

    pub weapon_offset: Vec2,

    pub bullet_offset_left: Vec2,
    pub bullet_offset_right: Vec2,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct WeaponAsset {
    pub config: WeaponConfig,
    pub sprite_config: WeaponSpriteConfig,
}

// Component for a weapon
#[derive(Component, Debug, Clone)]
pub struct Weapon {
    pub config: WeaponConfig,
    pub sprite_config: WeaponSpriteConfig,
}

impl From<WeaponAsset> for Weapon {
    fn from(value: WeaponAsset) -> Self {
        Self { config: value.config, sprite_config: value.sprite_config }
    }
}

#[derive(Component, Clone, Serialize, Deserialize)]
pub struct HitMarker {
    pub target: Entity,
    pub damage: f32,
}


#[derive(Component)]
pub struct VisualEffectRequest {
    pub effect_type: EffectType,
    pub position: Vec2,
    pub scale: f32,
}

#[derive(Clone)]
pub enum EffectType {
    BulletHit,
    Explosion,
    Piercing,
}

#[derive(Component, Clone, Serialize, Deserialize)]
pub struct ExplosionMarker {
    pub radius: f32,
    pub damage: f32,
    pub player_handle: PlayerHandle,
    pub processed: bool, // Flag to ensure one-time processing
}

/// Component to mark an entity as the active weapon
#[derive(Component)]
pub struct ActiveWeapon;



/// Component for bullets
#[derive(Component, Clone)]
pub struct Bullet {
    pub velocity: Vec2,
    pub bullet_type: BulletType,
    pub damage: f32,
    pub range: f32,
    pub distance_traveled: f32,
    pub player_handle: PlayerHandle,
    pub created_at: u32,
}


/// Component to track the player's weapon inventory
#[derive(Component, Debug, Clone)]
pub struct WeaponInventory {
    pub active_weapon_index: usize,
    pub frame_switched: u32,
    pub frame_switched_mode: u32,
    pub weapons: Vec<(Entity, Weapon)>,  // Store entity handles and weapon data

    pub reloading_ending_frame: Option<u32>,
}

impl Default for WeaponInventory {
    fn default() -> Self {
        Self {
            active_weapon_index: 0,
            frame_switched: 0,
            frame_switched_mode: 0,
            reloading_ending_frame: None,
            weapons: Vec::new(),
        }
    }
}

impl WeaponInventory {
    pub fn active_weapon(&self) -> &(Entity, Weapon) {
        return self.weapons.get(self.active_weapon_index).unwrap();
    }
}


#[derive(Reflect, Default, Clone)]
pub struct WeaponModeState {
    pub mag_ammo: u32,
    pub mag_quantity: u32,

    pub burst_shots_left: u32,

    pub mag_size: u32,
    pub burst_cooldown: bool,
}


#[derive(Component, Reflect, Default, Clone)]
pub struct WeaponModesState {
    pub modes: HashMap<String, WeaponModeState>,
}

// Component to track rollbackable state for weapons
#[derive(Component, Reflect, Default, Clone)]
pub struct WeaponState {
    pub last_fire_frame: u32,
    pub is_firing: bool,
    pub active_mode: String,
}

// Rollback state for bullets
#[derive(Component, Clone)]
pub struct BulletRollbackState {
    spawn_frame: u32,
    initial_position: Vec2,
    direction: Vec2,
}

#[derive(Event)]
pub struct FireWeaponEvent {
    pub player_entity: Entity,
}

// ASSETS

#[derive(Asset, TypePath, Serialize, Deserialize)]
pub struct WeaponsConfig(pub HashMap<String, WeaponAsset>);


// UTILITY FUNCTION


impl WeaponModeState {
    // Do the reloading of the ammo when the reloading process is over or some other event
    pub fn reload(&mut self) {
        if self.mag_quantity > 0 {
            self.mag_quantity -= 1;
            self.mag_ammo = self.mag_size;
        }
    }

    pub fn is_mag_full(&self) -> bool {
        self.mag_ammo == self.mag_size
    }
}

impl WeaponModesState {
    pub fn reload(&mut self, mode: &String) {
        if let Some(mode) = self.modes.get_mut(mode) {
            mode.reload();
        }
    }
}


impl WeaponInventory {

    pub fn is_reloading(&self) -> bool {
        self.reloading_ending_frame.is_some()
    }

    pub fn is_reloading_over(&self, current_frame: u32) -> bool {
        self.reloading_ending_frame.map_or_else(|| true, |f| current_frame >= f)
    }

    pub fn clear_reloading(&mut self) {
        self.reloading_ending_frame = None;
    }

    pub fn start_reload(
        &mut self,
        current_game_frame: u32,
        reload_time_seconds: f32,
    ) {
        self.reloading_ending_frame = {
            if reload_time_seconds <= 0.0 {
                None
            } else {
                let frames_to_reload = (reload_time_seconds * 60.0).ceil() as u32;
                if frames_to_reload == 0 { // Ensure at least one frame for very short reload times
                    Some(current_game_frame + 1)
                } else {
                    Some(current_game_frame + frames_to_reload)
                }
            }
        };
    }
}


// start the reload process


// Function to spawn weapon , all weapon should be spawn on the user when they got them
pub fn spawn_weapon_for_player(
    commands: &mut Commands,
    global_assets: &Res<GlobalAsset>,

    asset_server: &Res<AssetServer>,
    texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
    sprint_sheet_assets: &Res<Assets<SpriteSheetConfig>>,

    active: bool,

    player_entity: Entity,
    weapon: WeaponAsset,
    inventory: &mut WeaponInventory,
) -> Entity {

    let map_layers = global_assets.spritesheets.get(&weapon.sprite_config.name).unwrap().clone();
    let animation_handle = global_assets.animations.get(&weapon.sprite_config.name).unwrap().clone();

    let animation_bundle =
        AnimationBundle::new(map_layers.clone(), animation_handle.clone(), weapon.sprite_config.index, bmap!("body" => String::new()));

    let mut weapon_state = WeaponState::default();
    let mut weapon_modes_state = WeaponModesState::default();
    weapon_state.active_mode = weapon.config.default_firing_mode.clone();
    for (k, v) in weapon.config.firing_modes.iter() {
        let mut weapon_mode_state = WeaponModeState::default();
        match v.mag {
            MagBulletConfig::Mag { mag_size, mag_limit } => {
                weapon_mode_state.mag_ammo = mag_size;
                weapon_mode_state.mag_quantity = mag_limit;
                weapon_mode_state.mag_size = mag_size;
            },
            MagBulletConfig::Magless {  bullet_limit } => {
                weapon_mode_state.mag_ammo = bullet_limit;
            },
        };

        weapon_modes_state.modes.insert(k.clone(), weapon_mode_state);
    }

    let weapon: Weapon = weapon.into();

    let entity = commands.spawn((
        Transform::from_translation(Vec3::new(weapon.sprite_config.weapon_offset.x, weapon.sprite_config.weapon_offset.y, 0.0)).with_rotation(Quat::IDENTITY),
        weapon_state,
        weapon_modes_state,
        weapon.clone(),
        animation_bundle
    )).add_rollback().id();


    let spritesheet_config = sprint_sheet_assets.get(map_layers.get("body").unwrap()).unwrap();
    create_child_sprite(
        commands,
        &asset_server,
        texture_atlas_layouts,
        entity.clone(), &spritesheet_config, 0);

    inventory.weapons.push((entity.clone(), weapon));

    if active {
        commands.entity(entity).insert((ActiveWeapon{}, Visibility::Inherited));
        inventory.active_weapon_index = inventory.weapons.len() - 1;
    } else {
        commands.entity(entity).insert(Visibility::Hidden);
    }

    commands.entity(player_entity).add_child(entity);

    entity
}


fn spawn_bullet_rollback(
    commands: &mut Commands,
    weapon: &Weapon,
    weapon_transform: &GlobalTransform,
    facing_direction: &FacingDirection,
    direction: Vec2,
    bullet_type: BulletType,
    range: f32,
    player_handle: PlayerHandle,
    current_frame: u32,
    collision_settings: &Res<CollisionSettings>,
    parent_layer: &CollisionLayer,
) -> Entity {
    let (velocity, damage, range, radius) = match &bullet_type {
        BulletType::Standard { speed, damage: damage_bullet } => {
            (direction * (speed / 60.0 ), *damage_bullet, range, 5.0)
        },
        BulletType::Explosive { speed, damage: damage_bullet, blast_radius, explosive_damage_multiplier } => {
            (direction * (speed / 60.0), *damage_bullet, range, 8.0)
        },
        BulletType::Piercing { speed, damage: damage_bullet, penetration } => {
            (direction * (speed / 60.0), *damage_bullet, range, 5.0)
        }
    };

    let color = match &bullet_type {
        BulletType::Standard { .. } => Color::BLACK,
        BulletType::Explosive { .. } => Color::WHITE,
        BulletType::Piercing { .. } => Color::BLACK,
    };

    let firing_position_v2 = if matches!(facing_direction, FacingDirection::Right) {
        weapon.sprite_config.bullet_offset_right
    } else {
        weapon.sprite_config.bullet_offset_left
    };
    let firing_position = weapon_transform.transform_point(firing_position_v2.extend(0.));
    let (_, weapon_world_rotation, _) = weapon_transform.affine().to_scale_rotation_translation();

    let transform = Transform::from_translation(firing_position)
            .with_rotation(weapon_world_rotation);
    

    let mut entity_commands = commands.spawn((
        Sprite::from_color(color, Vec2::new(10.0, 10.0)),
        Bullet {
            velocity,
            bullet_type,
            damage,
            range,
            distance_traveled: 0.,
            player_handle,
            created_at: current_frame
        },
        BulletRollbackState {
            spawn_frame: current_frame,
            initial_position: firing_position_v2,
            direction,
        },
        Collider {
            offset: Vec2::ZERO,
            shape: ColliderShape::Circle { radius },
        },
        CollisionLayer(parent_layer.0),
        transform,
    ));

    match bullet_type {
        BulletType::Explosive { .. } =>  { entity_commands.insert(ExplosiveTag); },
        BulletType::Piercing { .. } => { entity_commands.insert(PiercingTag); },
        _ => {}
    };

    entity_commands.add_rollback().id()

}



// SYSTEMS

// Rollback system to correctly transform the weapon based on the position
pub fn system_weapon_position(
    query: Query<(&Children, &CursorPosition, &FacingDirection), With<Rollback>>,
    mut query_weapon: Query<(&Children, &mut Transform), With<ActiveWeapon>>,
    mut query_sprite: Query<(&mut Sprite)>,

) {
    for (childs, cursor_position, direction) in query.iter() {
        for child in childs.iter() {
            if let Ok((childs, mut transform)) = query_weapon.get_mut(*child) {
                for child in childs.iter() {
                    if let Ok((mut sprite)) = query_sprite.get_mut(*child) {

                        let vec = Vec2::new(cursor_position.x as f32, cursor_position.y as f32);
                        let angle_radians = vec.y.atan2(vec.x);
                        
                        transform.rotation = Quat::from_rotation_z(angle_radians);

                        match direction {
                            FacingDirection::Left => {
                                sprite.flip_y = true;
                            }
                            FacingDirection::Right => {
                                sprite.flip_y = false;
                            }
                        };
                    }
                }
            }
        }
    }
}

// rollback system for weapon action , firing and all
pub fn weapon_rollback_system(
    mut commands: Commands,
    mut rng: ResMut<RollbackRng>,
    inputs: Res<PlayerInputs<PeerConfig>>,
    frame: Res<FrameCount>,

    mut inventory_query: Query<(Entity, &mut WeaponInventory, &SprintState, &DashState, &CollisionLayer, &Player)>,
    mut weapon_query: Query<(&mut Weapon, &mut WeaponState, &mut WeaponModesState, &GlobalTransform, &Parent)>,

    player_query: Query<(&GlobalTransform, &FacingDirection, &Player)>,

    collision_settings: Res<CollisionSettings>,
) {
    // Process weapon firing for all players
    for (entity,  mut inventory, sprint_state, dash_state , collision_layer, player) in inventory_query.iter_mut() {
        let (input, _input_status) = inputs[player.handle];

        // Do nothing if no weapons
        if inventory.weapons.is_empty() {
            continue;
        }

        if sprint_state.is_sprinting || dash_state.is_dashing || input.buttons & INPUT_SPRINT != 0 || input.buttons & INPUT_DASH != 0 {
            continue;
        }


        // Nothing to do for weapon if we are sprinting
        
        // Get active weapon
        let (weapon_entity, _) = inventory.weapons[inventory.active_weapon_index];


        // Get the entity for the active weapon
        if let Ok((mut weapon, mut weapon_state, mut weapon_modes_state, weapon_transform, parent)) = weapon_query.get_mut(weapon_entity) {
            let active_mode = weapon_state.active_mode.clone();
            let weapon_config = weapon.config.firing_modes.get(&active_mode).unwrap();

            if input.buttons & INPUT_SWITCH_WEAPON_MODE != 0 {
                if let Some(new_mode) = weapon_modes_state.modes.keys().find(|&x| *x != weapon_state.active_mode) {
                    if inventory.frame_switched_mode + 20 < frame.frame && inventory.frame_switched + 20 < frame.frame {
                        inventory.frame_switched_mode = frame.frame;
                        weapon_state.active_mode = new_mode.clone();

                        continue;
                    }

                }
            }

            let weapon_mode_state = weapon_modes_state.modes.get_mut(&active_mode).unwrap();


            // Check if reloading and update progress,
            if inventory.is_reloading() {
                if inventory.is_reloading_over(frame.frame) {
                    weapon_mode_state.reload();
                    inventory.clear_reloading();
                } else {
                    continue;
                }
            } else if input.buttons & INPUT_RELOAD != 0  && !weapon_mode_state.is_mag_full() {
                inventory.start_reload(frame.frame, weapon_config.reload_time_seconds);
                continue;
            }

            // Handle switching of weapons, will start firing on the next frame
            if input.switch_weapon && !inventory.weapons.is_empty(){
                let new_index = (inventory.active_weapon_index + 1) % inventory.weapons.len();

                if new_index != inventory.active_weapon_index &&
                    inventory.frame_switched + 20 < frame.frame &&
                    inventory.frame_switched_mode + 20 < frame.frame  {

                    inventory.active_weapon_index = new_index;
                    inventory.frame_switched = frame.frame;

                    continue;
                }
            }

            // TODO: fix only support two mode, take the first that is not the current
            if input.fire {
                // Calculate fire rate in frames (60 FPS assumed) , need to be configure via ressource instead
                let frame_per_shot = ((60. / weapon_config.firing_rate)) as u32;
                let current_frame = frame.frame;
                let frames_since_last_shot = current_frame - weapon_state.last_fire_frame;

               let (can_fire, empty) = match weapon_config.firing_mode {
                    FiringMode::Automatic { .. } => {
                        (frames_since_last_shot >= frame_per_shot, weapon_mode_state.mag_ammo == 0)
                    },
                    
                    FiringMode::Manual { .. } => {
                        (!weapon_state.is_firing && frames_since_last_shot >= frame_per_shot,
                        weapon_mode_state.mag_ammo == 0)
                    },
                    
                    FiringMode::Burst { pellets_per_shot, cooldown_frames } => {
                        if weapon_mode_state.burst_shots_left > 0 && frames_since_last_shot >= frame_per_shot {
                            // Continue ongoing burst
                            (true, weapon_mode_state.mag_ammo == 0)
                        } else if weapon_mode_state.burst_shots_left == 0 {
                            if !weapon_state.is_firing && !weapon_mode_state.burst_cooldown && frames_since_last_shot >= cooldown_frames {
                                // Start new burst when trigger is pulled
                                weapon_mode_state.burst_shots_left = pellets_per_shot;
                                (true, weapon_mode_state.mag_ammo == 0)
                            } else if weapon_mode_state.burst_cooldown && frames_since_last_shot >= cooldown_frames {
                                // Reset cooldown
                                weapon_mode_state.burst_cooldown = false;
                                (false, false)
                            } else {
                                (false, false)
                            }
                        } else {
                            (false, false)
                        }
                    },
                    
                    FiringMode::Shotgun { pellet_count, spread_angle } => {
                        if !weapon_state.is_firing && frames_since_last_shot >= frame_per_shot {
                            // Shotgun fires all pellets at once, so we don't need burst_shots_left
                            (true, weapon_mode_state.mag_ammo == 0)
                        } else {
                            (false, false)
                        }
                    }
                }; 


                if empty {
                    inventory.start_reload(frame.frame, weapon_config.reload_time_seconds);
                    continue;
                }

                weapon_state.is_firing = true;

                if can_fire {
                    if let Ok((_, facing_direction, _)) = player_query.get(**parent) {
                        let aim_dir = Vec2::new(
                            input.pan_x as f32 / 127.0,
                            input.pan_y as f32 / 127.0
                        ).normalize();
                                match weapon_config.firing_mode {
                                    FiringMode::Shotgun { pellet_count, spread_angle } => {
                                        // Fire multiple pellets in a spread pattern
                                        for _ in 0..pellet_count {
                                            // Calculate a random angle within the spread range
                                            let pellet_angle = (round(rng.next_f32()) - 0.5) * spread_angle;
                                            let spread_rotation = Mat2::from_angle(pellet_angle);
                                            let direction = spread_rotation * aim_dir;

                                            spawn_bullet_rollback(
                                                &mut commands,
                                                &weapon,
                                                weapon_transform,
                                                facing_direction,
                                                direction,
                                                weapon_config.bullet_type.clone(),
                                                weapon_config.range,
                                                player.handle,
                                                frame.frame,
                                                &collision_settings,
                                                collision_layer,
                                            );
                                        }
                                        weapon_mode_state.mag_ammo -= 1; // Shotgun uses one ammo for all pellets
                                        inventory.start_reload(frame.frame, weapon_config.reload_time_seconds);
                                    },
                                    _ => {
                                        // Standard firing for Automatic, Manual, and Burst
                                        let spread_angle = (rng.next_f32_symmetric() - 0.5) * weapon_config.spread;
                                        let spread_rotation = Mat2::from_angle(spread_angle);
                                        let direction = spread_rotation * aim_dir;

                                        spawn_bullet_rollback(
                                            &mut commands,
                                            &weapon,
                                            weapon_transform,
                                            facing_direction,
                                            direction,
                                            weapon_config.bullet_type.clone(),
                                            weapon_config.range,
                                            player.handle,
                                            frame.frame,
                                            &collision_settings,
                                            collision_layer,
                                        );
                                        weapon_mode_state.mag_ammo -= 1;

                                        if matches!(weapon_config.firing_mode, FiringMode::Burst { .. }) && weapon_mode_state.burst_shots_left > 0 {
                                            weapon_mode_state.burst_shots_left -= 1;
                                            
                                            // Set cooldown when burst finishes
                                            if weapon_mode_state.burst_shots_left == 0 {
                                                weapon_mode_state.burst_cooldown = true;
                                            }
                                        }
                                    }
                                }
                                weapon_state.last_fire_frame = frame.frame;
                    }
                }
            } else {
                weapon_state.is_firing = false;
            }
        }
    }
}



pub fn bullet_rollback_system(
    mut commands: Commands,
    frame: Res<FrameCount>,
    mut bullet_query: Query<(Entity, &mut Transform, &mut Bullet, &BulletRollbackState)>,
) {
    for (entity, mut transform, mut bullet, bullet_state) in bullet_query.iter_mut() {
        // Move bullet based on velocity (fixed timestep)
        let delta = bullet.velocity;
        transform.translation.x += delta.x;
        transform.translation.y += delta.y;
        
        // Track distance traveled (fixed timestep version)
        bullet.distance_traveled += delta.length();
        
        // Destroy bullet if it exceeds its range
        if bullet.distance_traveled >= bullet.range {
            commands.entity(entity).despawn();
        }

        // Maybe also lifetime frame
    }
}


fn apply_bullet_dommage(
    commands: &mut Commands,
    target_entity: Entity,
    bullet: &Bullet,
    
    mut opt_dmg_accumulator: Option<Mut<'_, DamageAccumulator, >>
) {
    if let Some(accumulator) = opt_dmg_accumulator.as_mut() {
        // Update existing accumulator
        accumulator.total_damage += bullet.damage;
        accumulator.hit_count += 1;
        accumulator.last_hit_by = Some(health::HitBy::Player(bullet.player_handle))
    } else {
        commands.entity(target_entity).insert(DamageAccumulator{
            hit_count: 1,
            total_damage: bullet.damage,
            last_hit_by: Some(health::HitBy::Player(bullet.player_handle)),
        });
    }
}

// Collision detection system (inside rollback)
pub fn bullet_rollback_collision_system(
    mut commands: Commands,
    settings: Res<CollisionSettings>,
    bullet_query: Query<(Entity, &Transform, &Bullet, &Collider, &CollisionLayer), With<Rollback>>,
    mut collider_query: Query<(Entity, &Transform, &Collider, &CollisionLayer, Option<&Wall>, Option<&Health>, Option<&mut DamageAccumulator>), (Without<Bullet>, With<Rollback>)>,
) {
    // Track bullets to despawn after processing
    let mut bullets_to_despawn = Vec::new();

    for (bullet_entity, bullet_transform, bullet, bullet_collider, bullet_layer) in bullet_query.iter() {
        // Skip already processed bullets
        if bullets_to_despawn.contains(&bullet_entity) {
            continue;
        }

        let mut collision_targets: Vec<_> = collider_query.iter_mut().collect();
        collision_targets.sort_by(|(_, transform_a, _, _, _, _, _), (_, transform_b, _, _, _, _, _)| {
            let pos_a = transform_a.translation.truncate();
            let pos_b = transform_b.translation.truncate();
            
            let x_cmp = ((pos_a.x * 1000.0) as i32).cmp(&((pos_b.x * 1000.0) as i32));
            if x_cmp == std::cmp::Ordering::Equal {
                ((pos_a.y * 1000.0) as i32).cmp(&((pos_b.y * 1000.0) as i32))
            } else {
                x_cmp
            }
        });
            
        for (target_entity, target_transform, target_collider, target_layer, opt_wall, opt_health, opt_accumulator) in collider_query.iter_mut() {
            // Check if these layers should collide
            if !settings.layer_matrix[bullet_layer.0 as usize][target_layer.0 as usize] {
                continue;
            }

            // Check for collision using our new helper function
            if is_colliding(bullet_transform, bullet_collider, target_transform, target_collider) { 
                if opt_health.is_some() {
                    apply_bullet_dommage(&mut commands, target_entity, bullet, opt_accumulator);
                }

                match bullet.bullet_type {
                    BulletType::Standard { .. } => {
                       
                        // Standard bullets are destroyed on impact
                        bullets_to_despawn.push(bullet_entity);
                        break;
                    },
                    BulletType::Explosive { blast_radius, .. } => {
 
                        // Add explosion marker to bullet entity
                        /*
                        commands.entity(bullet_entity).insert(ExplosionMarker {
                            radius: blast_radius,
                            damage: bullet.damage,
                            player_handle: bullet.player_handle,
                            processed: false,
                        });
                        */
                        
                        // Explosive bullets are destroyed on impact
                        bullets_to_despawn.push(bullet_entity);
                        break;
                    },
                    BulletType::Piercing { .. } => {
                        // Mark target as hit

                        if opt_wall.is_some() {
                            bullets_to_despawn.push(bullet_entity);
                            break;
                        }
                        /*
                        if let Some(counter) = penetration_counter {
                            if counter.remaining <= 1 {
                                bullets_to_despawn.insert(bullet_entity);
                                break;
                            }
                            // Reduce counter in next system
                        }*/
                    },
                }
            }
        }
    }

    // Sort by entity ID for Deterministic delete ?????
    bullets_to_despawn.sort_by_key(|entity| entity.index());
    
    // Despawn bullets
    for entity in bullets_to_despawn {
        commands.entity(entity).despawn();
    }
}



// Non rollback system to display the weapon correct sprite
pub fn weapon_inventory_system(
    mut commands: Commands,
    query: Query<(Entity, &mut WeaponInventory)>,
    mut weapon_entities: Query<(Entity, &mut Visibility),  With<Weapon>>,
) {
    for (_player_entity, inventory) in query.iter() {
        if inventory.weapons.is_empty() {
            continue;
        }

        // Update active/inactive weapon visibility
        for (i, (weapon_entity, _)) in inventory.weapons.iter().enumerate() {
            let is_active = i == inventory.active_weapon_index;

            
            // For simplicity, we're using commands to add/remove components
            // In a real implementation, you might want to use a Visibility component
            if let Ok((_, mut visibility)) = weapon_entities.get_mut(*weapon_entity) {
                if is_active {
                    commands.entity(*weapon_entity)
                        .insert(ActiveWeapon);
                    *visibility = Visibility::Visible;
                } else {
                    commands.entity(*weapon_entity)
                        .remove::<ActiveWeapon>();
                    *visibility = Visibility::Hidden;
                }
            }

        }
    }
}

pub fn weapons_config_update_system(
    _asset_server: Res<AssetServer>,
    
    weapons_config: Res<Assets<WeaponsConfig>>,

    mut ev_asset: EventReader<AssetEvent<WeaponsConfig>>,

    mut query_weapons: Query<(&Children, Entity, &mut Weapon)>,
) {

    for event in ev_asset.read() {
        if let AssetEvent::Modified { id } = event {

            if let Some(weapons_config) = weapons_config.get(*id) {
                for (_childs, _entity, mut weapon) in query_weapons.iter_mut() {
                    if let Some(config) = weapons_config.0.get(&weapon.config.name) {
                        weapon.config = config.config.clone();
                        weapon.sprite_config = config.sprite_config.clone();
                    }
                
                }
            }

        }
    }

}