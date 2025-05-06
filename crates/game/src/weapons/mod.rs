use std::f32::consts::FRAC_PI_2;

use animation::{AnimationBundle, FacingDirection};
use bevy::{prelude::*, sprite::Anchor};
use bevy_ggrs::{AddRollbackCommandExtension, PlayerInputs, Rollback};
use ggrs::PlayerHandle;
use utils::bmap;

use crate::{character::player::{jjrs::{BoxConfig, PeerConfig}, Player}, frame::FrameCount, global_asset::GlobalAsset};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FiringMode {
    Automatic,  // Hold trigger to continuously fire
    Manual,     // One shot per trigger pull
    Burst,      // Fire a fixed number of shots per trigger pull
}

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Component, Debug, Clone)]
pub struct Weapon {
    pub name: String,
    pub firing_rate: f32,        // bullets per second
    pub firing_mode: FiringMode,
    pub spread: f32,             // in radians
    pub recoil: f32,             // force applied when firing
    pub mag_size: u32,
    pub bullets_in_mag: u32,
    pub bullet_type: BulletType,
    pub range: f32,              // how far bullets travel
    
    // Internal weapon state
    pub last_shot_time: f32,
    pub burst_shots_left: u8,    // for burst mode
    pub is_firing: bool,
}

impl Default for Weapon {
    fn default() -> Self {
        Self {
            name: "weapon_1".to_string(),
            firing_rate: 2.0,
            firing_mode: FiringMode::Manual,
            spread: 0.02,
            recoil: 2.0,
            mag_size: 12,
            bullets_in_mag: 12,
            bullet_type: BulletType::Standard { damage: 10.0, speed: 800.0 },
            range: 800.0,
            last_shot_time: 0.0,
            burst_shots_left: 0,
            is_firing: false,
        }
    }
}



/// Component to mark an entity as the active weapon
#[derive(Component)]
pub struct ActiveWeapon;

/// Component for the weapon sprite's position relative to player
#[derive(Component, Clone, Copy)]
pub struct WeaponPosition {
    pub distance_from_player: f32,  // Distance from player center
    pub angle_offset: f32,          // Additional angle offset for visual style
}

impl Default for WeaponPosition {
    fn default() -> Self {
        Self {
            distance_from_player: 00.0,
            angle_offset: 0.0,
        }
    }
}

/// Component for bullets
#[derive(Component)]
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
}

impl Default for WeaponInventory {
    fn default() -> Self {
        Self {
            active_weapon_index: 0,
            frame_switched: 0,
            weapons: Vec::new(),
        }
    }
}



// Component to track rollbackable state for weapons
#[derive(Component, Reflect, Default, Clone)]
pub struct WeaponState {
    pub last_fire_frame: u32,
    pub mag_ammo: u32,
    pub is_firing: bool,
    pub burst_shots_left: u8,
}

// Rollback state for bullets
#[derive(Component, Clone)]
struct BulletRollbackState {
    spawn_frame: u32,
    initial_position: Vec2,
    direction: Vec2,
}

#[derive(Event)]
pub struct FireWeaponEvent {
    pub player_entity: Entity,
}


pub fn spawn_weapon_for_player(
    commands: &mut Commands,
    global_assets: &Res<GlobalAsset>,

    active: bool,
    starting_index: usize,

    player_entity: Entity,
    weapon: Weapon,
    inventory: &mut WeaponInventory,
) -> Entity {

    let map_layers = global_assets.spritesheets.get(&weapon.name).unwrap().clone();
    let animation_handle = global_assets.animations.get(&weapon.name).unwrap().clone();

    let animation_bundle =
        AnimationBundle::new(map_layers, animation_handle.clone(), starting_index, bmap!("body" => String::new()));
    

    let entity = commands.spawn((
        Transform::from_translation(Vec3::new(0.0, -5.0, 0.0)).with_rotation(Quat::IDENTITY),
        WeaponPosition {
            distance_from_player: 0.0,
            angle_offset: 0.0,
        },
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


    entity
}


pub fn update_weapon_position(x: i16, y: i16, weapon_position: &mut WeaponPosition) {
    let vec = Vec2::new(x as f32, y as f32);
    let angle_radians = vec.y.atan2(vec.x);
    let angle_degrees = angle_radians.to_degrees();


    weapon_position.angle_offset = angle_radians;
}


pub fn system_weapon_position(
    query: Query<(&Children, &WeaponPosition, &FacingDirection), With<Rollback>>,
    mut query_weapon: Query<(&Children, &mut Transform), With<ActiveWeapon>>,
    mut query_sprite: Query<(&mut Sprite)>,

) {
    for (childs, weapon_position, direction) in query.iter() {
        for child in childs.iter() {
            if let Ok((childs, mut transform)) = query_weapon.get_mut(*child) {
                for child in childs.iter() {
                    if let Ok((mut sprite)) = query_sprite.get_mut(*child) {
                        
                        transform.rotation = Quat::from_rotation_z(weapon_position.angle_offset);

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

pub fn weapon_rollback_system(
    mut commands: Commands,
    inputs: Res<PlayerInputs<PeerConfig>>,
    frame: Res<FrameCount>,

    mut inventory_query: Query<(&mut WeaponInventory, &Player)>,
    mut weapon_query: Query<(&mut Weapon, &mut WeaponState, &Transform, &Parent)>,

    player_query: Query<(&Transform, &Player)>,
) {
    // Process weapon firing for all players
    for (mut inventory, player) in inventory_query.iter_mut() {
        let (input, _input_status) = inputs[player.handle];

        if input.switch_weapon && !inventory.weapons.is_empty() {
            let new_index = (inventory.active_weapon_index + 1) % inventory.weapons.len();

            if new_index != inventory.active_weapon_index &&
                inventory.frame_switched + 20 < frame.frame {
                inventory.active_weapon_index = new_index;
                inventory.frame_switched = frame.frame;
            }
        }

        if inventory.weapons.is_empty() {
            continue;
        }
    }
}

pub fn weapon_inventory_system(
    mut commands: Commands,
    query: Query<(Entity, &mut WeaponInventory)>,
    mut weapon_entities: Query<(Entity, &mut Visibility),  With<Weapon>>,
) {
    for (player_entity, inventory) in query.iter() {
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

/*
// System to handle weapon firing and state with rollback
pub fn weapon_rollback_system(
    mut commands: Commands,
    inputs: Res<bevy_ggrs::PlayerInputs<BoxConfig>>,
    frame: Res<bevy_ggrs::FrameCount>,
    mut inventory_query: Query<(&mut WeaponInventory, &Parent, &Player)>,
    mut weapon_query: Query<(&mut Weapon, &mut WeaponState, &Transform, &Parent)>,
    player_query: Query<(&Transform, &Player)>,
) {
    // Process weapon firing for all players
    for (mut inventory, parent, player) in inventory_query.iter_mut() {
        // Get this player's input
        if let Some(input) = inputs[player.handle as usize].0.latest() {
            let game_input: GameInput = *input;
            
            // Handle weapon switching
            if game_input.switch_weapon && !inventory.weapons.is_empty() {
                inventory.active_weapon_index = (inventory.active_weapon_index + 1) % inventory.weapons.len();
            }
            
            // Skip if no weapons
            if inventory.weapons.is_empty() {
                continue;
            }
            
            // Get active weapon
            let (weapon_entity, _) = inventory.weapons[inventory.active_weapon_index];
            
            if let Ok((mut weapon, mut weapon_state, weapon_transform, _)) = weapon_query.get_mut(weapon_entity) {
                // Handle weapon firing
                if game_input.fire {
                    // Calculate fire rate in frames (60 FPS assumed)
                    let frames_per_shot = (60.0 / weapon.firing_rate) as u64;
                    let current_frame = *frame as u64;
                    let frames_since_last_shot = current_frame - weapon_state.last_fire_frame as u64;
                    
                    let can_fire = match weapon.firing_mode {
                        FiringMode::Automatic => {
                            frames_since_last_shot >= frames_per_shot && weapon_state.mag_ammo > 0
                        },
                        FiringMode::Manual => {
                            // Only fire if we weren't firing last frame (for single shots)
                            !weapon_state.is_firing && 
                            frames_since_last_shot >= frames_per_shot && 
                            weapon_state.mag_ammo > 0
                        },
                        FiringMode::Burst => {
                            if weapon_state.burst_shots_left > 0 && 
                               frames_since_last_shot >= frames_per_shot && 
                               weapon_state.mag_ammo > 0 {
                                true
                            } else if weapon_state.burst_shots_left == 0 && 
                                    !weapon_state.is_firing && 
                                    frames_since_last_shot >= (frames_per_shot * 3) && 
                                    weapon_state.mag_ammo > 0 {
                                // Start a new burst (3 bullets)
                                weapon_state.burst_shots_left = 3;
                                true
                            } else {
                                false
                            }
                        }
                    };
                    
                    // Update firing state
                    weapon_state.is_firing = true;
                    
                    if can_fire {
                        // Get player transform
                        if let Ok((player_transform, _)) = player_query.get(parent.get()) {
                            // Get cursor position for this player
                            let aim_dir = Vec2::new(
                                game_input.cursor_x as f32 / 127.0,
                                game_input.cursor_y as f32 / 127.0
                            ).normalize();
                            
                            // Calculate firing position (from weapon)
                            let firing_pos = weapon_transform.translation.truncate();
                            
                            // Apply weapon spread
                            let spread_angle = (rand::random::<f32>() - 0.5) * weapon.spread;
                            let spread_rotation = Mat2::from_angle(spread_angle);
                            let direction = spread_rotation * aim_dir;
                            
                            // Spawn bullet
                            /*spawn_bullet_rollback(
                                &mut commands,
                                firing_pos,
                                direction,
                                weapon.bullet_type.clone(),
                                match &weapon.bullet_type {
                                    BulletType::Standard { damage, .. } => *damage,
                                    BulletType::Explosive { damage, .. } => *damage,
                                    BulletType::Piercing { damage, .. } => *damage,
                                },
                                weapon.range,
                                player.handle,
                                frame.0,
                            );*/
                            println!("SPWANIN BUTLLING");
                            
                            // Update weapon state
                            weapon_state.last_fire_frame = frame.0;
                            weapon_state.mag_ammo -= 1;
                            
                            if weapon.firing_mode == FiringMode::Burst && weapon_state.burst_shots_left > 0 {
                                weapon_state.burst_shots_left -= 1;
                            }
                        }
                    }
                } else {
                    // Reset firing state when not pressing fire
                    weapon_state.is_firing = false;
                }
            }
        }
    }
}
*/

/*
// Spawn a bullet with rollback properties
fn spawn_bullet_rollback(
    commands: &mut Commands,
    position: Vec2,
    direction: Vec2,
    bullet_type: BulletType,
    damage: f32,
    range: f32,
    player_handle: PlayerHandle,
    current_frame: Frame,
) -> Entity {
    // Get bullet speed based on type
    let speed = match &bullet_type {
        BulletType::Standard { speed, .. } => *speed / 60.0, // Convert to per-frame speed
        BulletType::Explosive { speed, .. } => *speed / 60.0,
        BulletType::Piercing { speed, .. } => *speed / 60.0,
    };
    
    // Get bullet color based on type
    let color = match &bullet_type {
        BulletType::Standard { .. } => Color::BLACK,
        BulletType::Explosive { .. } => Color::WHITE,
        BulletType::Piercing { .. } => Color::BLACK,
    };

    println!("spawning bullet");
    
    // Spawn bullet entity
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color,
                custom_size: Some(Vec2::new(6.0, 6.0)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(position.x, position.y, 5.0)),
            ..default()
        },
        Bullet {
            velocity: direction * speed,
            bullet_type,
            damage,
            range,
            distance_traveled: 0.0,
            player_handle,
        },
        BulletRollbackState {
            spawn_frame: current_frame,
            initial_position: position,
            direction,
        },
        Rollback::new(0), // Bullet IDs will be generated by the caller
    )).id()
}*/


pub fn bullet_rollback_system(
    mut commands: Commands,
    frame: Res<FrameCount>,
    mut bullet_query: Query<(Entity, &mut Transform, &mut Bullet, &BulletRollbackState)>,
) {
    for (entity, mut transform, mut bullet, bullet_state) in bullet_query.iter_mut() {
        // Calculate frames since spawn
        let frames_alive = frame.frame - bullet_state.spawn_frame;
        
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