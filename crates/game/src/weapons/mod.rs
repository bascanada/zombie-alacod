use animation::{AnimationBundle, FacingDirection};
use bevy::{prelude::*, sprite::Anchor, utils::HashMap};
use bevy_ggrs::{AddRollbackCommandExtension, PlayerInputs, Rollback};
use ggrs::PlayerHandle;
use serde::{Deserialize, Serialize};
use utils::bmap;

use crate::{character::player::{jjrs::{BoxConfig, PeerConfig}, Player}, frame::FrameCount, global_asset::GlobalAsset};

// COMPONENTS

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum FiringMode {
    Automatic{},  // Hold trigger to continuously fire
    Manual{},     // One shot per trigger pull
    Burst{pellets_per_shot: u32},      // Fire a fixed number of shots per trigger pull
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
pub struct WeaponConfig {
    pub name: String,
    pub firing_rate: f32,        // bullets per second
    pub firing_mode: FiringMode,
    pub spread: f32,             // in radians
    pub recoil: f32,             // force applied when firing
    pub mag_size: u32,
    pub bullet_type: BulletType,
    pub range: f32,              // how far bullets travel

    pub reload_time_seconds: f32,
}



#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WeaponSpriteConfig {
    pub name: String,
    pub index: usize,
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

/// Component for the weapon sprite's position relative to player
#[derive(Component, Clone, Copy)]
pub struct WeaponPosition {
    pub angle_offset: f32,          // Additional angle offset for visual style
}

impl Default for WeaponPosition {
    fn default() -> Self {
        Self {
            angle_offset: 0.0,
        }
    }
}

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
    pub burst_shots_left: u32,
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
    weapon_state.mag_ammo = weapon.config.mag_size;

    let weapon: Weapon = weapon.into();

    let entity = commands.spawn((
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)).with_rotation(Quat::IDENTITY),
        WeaponPosition {
            angle_offset: 0.0,
        },
        weapon_state,
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
    position: Vec2,
    direction: Vec2,
    bullet_type: BulletType,
    damage: f32,
    range: f32,
    player_handle: PlayerHandle,
    current_frame: u32,
) -> Entity {
    let speed = match &bullet_type {
        BulletType::Standard { speed, .. } => *speed / 60.0, // Convert to per-frame speed
        BulletType::Explosive { speed, .. } => *speed / 60.0,
        BulletType::Piercing { speed, .. } => *speed / 60.0,
    };

    let color = match &bullet_type {
        BulletType::Standard { .. } => Color::BLACK,
        BulletType::Explosive { .. } => Color::WHITE,
        BulletType::Piercing { .. } => Color::BLACK,
    };


    commands.spawn((
        Sprite::from_color(color, Vec2::new(10.0, 10.0)),
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
        Transform::from_translation(Vec3::new(position.x , position.y, 5.0)),
    )).add_rollback().id()

}


// Function to calculate the weapon position based on the player cursor location
pub fn update_weapon_position(x: i16, y: i16, weapon_position: &mut WeaponPosition) {
    let vec = Vec2::new(x as f32, y as f32);
    let angle_radians = vec.y.atan2(vec.x);

    weapon_position.angle_offset = angle_radians;
}




// SYSTEMS

// Rollback system to correctly transform the weapon based on the position
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

// rollback system for weapon action , firing and all
pub fn weapon_rollback_system(
    mut commands: Commands,
    inputs: Res<PlayerInputs<PeerConfig>>,
    frame: Res<FrameCount>,

    mut inventory_query: Query<(&mut WeaponInventory, &Player)>,
    mut weapon_query: Query<(&mut Weapon, &mut WeaponState, &Transform, &Parent)>,

    player_query: Query<(&GlobalTransform, &Player)>,
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
        
        let (weapon_entity, _) = inventory.weapons[inventory.active_weapon_index];

        if let Ok((mut weapon, mut weapon_state, weapon_transform, parent)) = weapon_query.get_mut(weapon_entity) {
            if input.fire {
                // Calculate fire rate in frames (60 FPS assumed) , need to be configure via ressource instead
                let frame_per_shot = (60.0 / weapon.config.firing_rate) as u32;
                let current_frame = frame.frame;
                let frames_since_last_shot = current_frame - weapon_state.last_fire_frame;

                let can_fire = match weapon.config.firing_mode {
                    FiringMode::Automatic { .. } => {
                        frames_since_last_shot >= frame_per_shot && weapon_state.mag_ammo > 0
                    },
                    FiringMode::Manual { .. } => {
                        !weapon_state.is_firing && frames_since_last_shot >= frame_per_shot &&
                        weapon_state.mag_ammo > 0
                    },
                    FiringMode::Burst { pellets_per_shot } => {
                        if weapon_state.burst_shots_left > 0 && frames_since_last_shot >= frame_per_shot && weapon_state.mag_ammo > 0 {
                            true
                        } else if weapon_state.burst_shots_left == 0 && !weapon_state.is_firing && frames_since_last_shot >= (frame_per_shot * pellets_per_shot) && weapon_state.mag_ammo > 0 {
                            weapon_state.burst_shots_left = pellets_per_shot;
                            true
                        } else {
                            false
                        }
                    }
                };


                weapon_state.is_firing = true;

                if can_fire {
                    if let Ok((player_transform, _)) = player_query.get(**parent) {
                        let aim_dir = Vec2::new(
                            input.pan_x as f32 / 127.0,
                            input.pan_y as f32 / 127.0
                        ).normalize();

                        let firing_position = player_transform.translation().truncate() + weapon_transform.translation.truncate();

                        // need to replace with fix seed random number
                        let spread_angle = (rand::random::<f32>() - 0.5)  * weapon.config.spread;
                        let spread_rotation = Mat2::from_angle(spread_angle);
                        let direction = spread_rotation * aim_dir;


                        // SPAWN BULLET
                        spawn_bullet_rollback(
                            &mut commands,
                            firing_position,
                            direction,
                            weapon.config.bullet_type.clone(),
                            match &weapon.config.bullet_type {
                                BulletType::Standard { damage, .. } => *damage,
                                BulletType::Explosive { damage, .. } => *damage,
                                BulletType::Piercing { damage, .. } => *damage,
                            },
                            weapon.config.range,
                            player.handle,
                            frame.frame,
                        );


                        weapon_state.last_fire_frame = frame.frame;
                        weapon_state.mag_ammo -= 1;
                        
                        if matches!(weapon.config.firing_mode, FiringMode::Burst { .. }) && weapon_state.burst_shots_left > 0 {
                            weapon_state.burst_shots_left -= 1;
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