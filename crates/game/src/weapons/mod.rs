pub mod ui;

use animation::{AnimationBundle, FacingDirection};
use bevy::{prelude::*, utils::HashMap};
use bevy_ggrs::{AddRollbackCommandExtension, PlayerInputs, Rollback};
use ggrs::PlayerHandle;
use serde::{Deserialize, Serialize};
use utils::{bmap, rng::RollbackRng};

use crate::{character::player::{input::{CursorPosition, INPUT_RELOAD}, jjrs::{BoxConfig, PeerConfig}, Player}, frame::FrameCount, global_asset::GlobalAsset};

// ROOLBACL

// COMPONENTS

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum FiringMode {
    Automatic{},  // Hold trigger to continuously fire
    Manual{},     // One shot per trigger pull
    Burst{pellets_per_shot: u32},      // Fire a fixed number of shots per trigger pull
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BulletType {
    Standard {
        damage: f32,
        speed: f32,
    },
    Explosive {
        damage: f32,
        speed: f32,
        blast_radius: f32,
    },
    Piercing {
        damage: f32,
        speed: f32,
        penetration: u8,
    },
}

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
}


/// Component to track the player's weapon inventory
#[derive(Component, Debug, Clone)]
pub struct WeaponInventory {
    pub active_weapon_index: usize,
    pub frame_switched: u32,
    pub weapons: Vec<(Entity, Weapon)>,  // Store entity handles and weapon data

    pub reloading_ending_frame: Option<u32>,
}

impl Default for WeaponInventory {
    fn default() -> Self {
        Self {
            active_weapon_index: 0,
            frame_switched: 0,
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

    active: bool,

    player_entity: Entity,
    weapon: WeaponAsset,
    inventory: &mut WeaponInventory,
) -> Entity {

    let map_layers = global_assets.spritesheets.get(&weapon.sprite_config.name).unwrap().clone();
    let animation_handle = global_assets.animations.get(&weapon.sprite_config.name).unwrap().clone();

    let animation_bundle =
        AnimationBundle::new(map_layers, animation_handle.clone(), weapon.sprite_config.index, bmap!("body" => String::new()));

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
    damage: f32,
    range: f32,
    player_handle: PlayerHandle,
    current_frame: u32,
) -> Entity {
    let speed = match &bullet_type {
        BulletType::Standard { speed, .. } => *speed / 60., // Convert to per-frame speed
        BulletType::Explosive { speed, .. } => *speed / 60.,
        BulletType::Piercing { speed, .. } => *speed / 60.,
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
    

    commands.spawn((
        Sprite::from_color(color, Vec2::new(10.0, 10.0)),
        Bullet {
            velocity: direction * speed,
            bullet_type,
            damage,
            range,
            distance_traveled: 0.,
            player_handle,
        },
        BulletRollbackState {
            spawn_frame: current_frame,
            initial_position: firing_position_v2,
            direction,
        },
        transform,
    )).add_rollback().id()

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

    mut inventory_query: Query<(Entity, &mut WeaponInventory, &Player)>,
    mut weapon_query: Query<(&mut Weapon, &mut WeaponState, &mut WeaponModesState, &GlobalTransform, &Parent)>,

    player_query: Query<(&GlobalTransform, &FacingDirection, &Player)>,
) {
    // Process weapon firing for all players
    for (entity,  mut inventory, player) in inventory_query.iter_mut() {
        let (input, _input_status) = inputs[player.handle];

        // Do nothing if no weapons
        if inventory.weapons.is_empty() {
            continue;
        }
        
        // Get active weapon
        let (weapon_entity, _) = inventory.weapons[inventory.active_weapon_index];


        // Get the entity for the active weapon
        if let Ok((mut weapon, mut weapon_state, mut weapon_modes_state, weapon_transform, parent)) = weapon_query.get_mut(weapon_entity) {
            let active_mode = weapon_state.active_mode.clone();
            let weapon_config = weapon.config.firing_modes.get(&active_mode).unwrap();
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
                    inventory.frame_switched + 20 < frame.frame {
                    inventory.active_weapon_index = new_index;
                    inventory.frame_switched = frame.frame;

                    continue;
                }
            }

            if input.fire {
                // Calculate fire rate in frames (60 FPS assumed) , need to be configure via ressource instead
                let frame_per_shot = ((60. / weapon_config.firing_rate)) as u32;
                let current_frame = frame.frame;
                let frames_since_last_shot = current_frame - weapon_state.last_fire_frame;


                let (can_fire, empty) = match weapon_config.firing_mode {
                    FiringMode::Automatic { .. } => {
                        (frames_since_last_shot >= frame_per_shot, weapon_mode_state.mag_ammo <= 0)
                    },
                    FiringMode::Manual { .. } => {
                        (!weapon_state.is_firing && frames_since_last_shot >= frame_per_shot,
                        weapon_mode_state.mag_ammo <= 0)
                    },
                    FiringMode::Burst { pellets_per_shot } => {
                        if weapon_mode_state.burst_shots_left > 0 && frames_since_last_shot >= frame_per_shot && weapon_mode_state.mag_ammo > 0 {
                            (true, true)
                        } else if weapon_mode_state.burst_shots_left == 0 && !weapon_state.is_firing && frames_since_last_shot >= (frame_per_shot * pellets_per_shot) && weapon_mode_state.mag_ammo > 0 {
                            weapon_mode_state.burst_shots_left = pellets_per_shot;
                            (true, true)
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

                        // need to replace with fix seed random number
                        let spread_angle = (rng.next_f32_symmetric() - 0.5)  * weapon_config.spread;
                        let spread_rotation = Mat2::from_angle(spread_angle);
                        let direction = spread_rotation * aim_dir;

                        spawn_bullet_rollback(
                            &mut commands,
                            &weapon,
                            weapon_transform,
                            facing_direction,
                            direction,
                            weapon_config.bullet_type.clone(),
                            match &weapon_config.bullet_type {
                                BulletType::Standard { damage, .. } => *damage,
                                BulletType::Explosive { damage, .. } => *damage,
                                BulletType::Piercing { damage, .. } => *damage,
                            },
                            weapon_config.range,
                            player.handle,
                            frame.frame,
                        );


                        weapon_state.last_fire_frame = frame.frame;
                        weapon_mode_state.mag_ammo -= 1;
                        
                        if matches!(weapon_config.firing_mode, FiringMode::Burst { .. }) && weapon_mode_state.burst_shots_left > 0 {
                            weapon_mode_state.burst_shots_left -= 1;
                        }
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