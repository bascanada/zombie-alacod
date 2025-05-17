

use bevy::prelude::*;
use bevy_ggrs::{Session};
use bevy_kira_audio::{Audio, AudioControl, AudioInstance, AudioSource, AudioTween, PlaybackState, SpatialAudioEmitter, SpatialRadius};
use ggrs::SessionState;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use crate::{character::player::jjrs::PeerConfig, frame::FrameCount};

// ======== SOUND MODIFIERS ========

/// Modifiers that can be applied to a sound when playing
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SoundModifiers {
    /// Name of the modifier preset (if it's a named preset)
    name: Option<String>,
    
    /// Pitch multiplier (1.0 = normal pitch)
    pitch: Option<f64>,
    
    /// Panning (-1.0 = left, 0.0 = center, 1.0 = right)
    panning: Option<f64>,
    
    /// Custom volume override (0.0 - 1.0)
    volume_override: Option<f32>,
    
    /// Fade-in duration in seconds
    fade_in: Option<f64>,
    
    /// Start position in seconds from beginning of audio
    start_position: Option<f64>,
    
    /// Custom position offset relative to entity
    position_offset: Option<Vec3>,
}

impl Default for SoundModifiers {
    fn default() -> Self {
        Self {
            name: None,
            pitch: None,
            panning: None,
            volume_override: None,
            fade_in: None,
            start_position: None,
            position_offset: None,
        }
    }
}

impl SoundModifiers {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
    
    pub fn with_pitch(mut self, pitch: f64) -> Self {
        self.pitch = Some(pitch);
        self
    }
    
    pub fn with_panning(mut self, panning: f64) -> Self {
        self.panning = Some(panning.clamp(-1.0, 1.0));
        self
    }
    
    pub fn with_volume(mut self, volume: f32) -> Self {
        self.volume_override = Some(volume.clamp(0.0, 1.0));
        self
    }
    
    pub fn with_fade_in(mut self, seconds: f64) -> Self {
        self.fade_in = Some(seconds.max(0.0));
        self
    }
    
    pub fn with_start_position(mut self, seconds: f64) -> Self {
        self.start_position = Some(seconds.max(0.0));
        self
    }
    
    pub fn with_position_offset(mut self, offset: Vec3) -> Self {
        self.position_offset = Some(offset);
        self
    }
    
    /// Get the name of this modifier preset (if any)
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}

// ======== COMPONENTS ========

/// A generic audio configuration component
#[derive(Component, Clone, Serialize, Deserialize)]
pub struct GameAudioEmitter {
    /// Current active sound instances by sound ID
    #[serde(skip)]
    instances: HashMap<String, Handle<AudioInstance>>,
    
    /// Sound configurations by sound ID
    sound_configs: HashMap<String, SoundConfig>,
    
    /// Predefined modifier presets by name
    modifier_presets: HashMap<String, SoundModifiers>,
    
    /// Active state and modifiers for each sound
    #[serde(skip)]
    active_sounds: HashMap<String, Option<SoundModifiers>>,
    
    /// Maximum concurrent sounds (0 = unlimited)
    max_concurrent: usize,
    
    /// Default radius for all sounds (if not specified in SoundConfig)
    default_radius: f32,
}

impl Default for GameAudioEmitter {
    fn default() -> Self {
        Self {
            instances: HashMap::new(),
            sound_configs: HashMap::new(),
            modifier_presets: HashMap::new(),
            active_sounds: HashMap::new(),
            max_concurrent: 0,
            default_radius: 10.0,
        }
    }
}

/// Configuration for a specific sound
#[derive(Clone, Serialize, Deserialize)]
pub struct SoundConfig {
    /// Path to the sound file
    path: String,
    
    /// Base volume of the sound (0.0 - 1.0)
    volume: f64,
    
    /// Spatial radius for this sound
    radius: Option<f32>,
    
    /// Whether this sound should loop
    looped: bool,
    
    /// Default modifiers for this sound (either name reference or actual modifiers)
    #[serde(skip_serializing_if = "Option::is_none")]
    default_modifier_name: Option<String>,
    
    /// Default modifiers for this sound (direct values)
    #[serde(skip_serializing_if = "Option::is_none")]
    default_modifiers: Option<SoundModifiers>,
}

impl Default for SoundConfig {
    fn default() -> Self {
        Self {
            path: String::new(),
            volume: 1.0,
            radius: None,
            looped: false,
            default_modifier_name: None,
            default_modifiers: None,
        }
    }
}

/// Holds information about sound variant handling
#[derive(Component, Default, Serialize, Deserialize)]
struct ActiveSoundVariants {
    /// Maps base sound ID to active variant (if any)
    #[serde(skip)]
    active_variants: HashMap<String, String>,
}

// Builder pattern for fluent configuration
impl SoundConfig {
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            ..Default::default()
        }
    }
    
    pub fn with_volume(mut self, volume: f32) -> Self {
        self.volume = volume.clamp(0.0, 1.0);
        self
    }
    
    pub fn with_radius(mut self, radius: f32) -> Self {
        self.radius = Some(radius);
        self
    }
    
    pub fn looped(mut self) -> Self {
        self.looped = true;
        self
    }
    
    pub fn with_default_modifiers(mut self, modifiers: SoundModifiers) -> Self {
        self.default_modifiers = Some(modifiers);
        // Clear any modifier name reference
        self.default_modifier_name = None;
        self
    }
    
    pub fn with_default_modifier_name(mut self, name: impl Into<String>) -> Self {
        self.default_modifier_name = Some(name.into());
        // Clear any direct modifiers
        self.default_modifiers = None;
        self
    }
    
    /// Get the effective default modifiers for this sound config
    pub fn get_default_modifiers(&self, emitter: &GameAudioEmitter) -> Option<SoundModifiers> {
        if let Some(modifiers) = &self.default_modifiers {
            return Some(modifiers.clone());
        }
        
        if let Some(name) = &self.default_modifier_name {
            return emitter.modifier_presets.get(name).cloned();
        }
        
        None
    }
}

// ======== PLUGIN ========

pub struct GameAudioPlugin;

impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<GameAudioEmitter>()
            .register_type::<ActiveSoundVariants>()
            // Process audio changes after GGRS systems
            .add_systems(PostUpdate, process_audio_states.after(GGRSSchedule))
            // Cleanup finished audio instances
            .add_systems(PostUpdate, cleanup_audio_instances);
    }
}

// ======== API METHODS ========

impl GameAudioEmitter {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn with_max_concurrent(mut self, max: usize) -> Self {
        self.max_concurrent = max;
        self
    }
    
    pub fn with_default_radius(mut self, radius: f32) -> Self {
        self.default_radius = radius;
        self
    }
    
    /// Register a sound with its configuration
    pub fn register_sound(&mut self, id: impl Into<String>, config: SoundConfig) {
        let id = id.into();
        self.sound_configs.insert(id.clone(), config);
        self.active_sounds.insert(id, None); // Not active initially
    }
    
    /// Register multiple sounds at once
    pub fn register_sounds(&mut self, configs: Vec<(impl Into<String>, SoundConfig)>) {
        for (id, config) in configs {
            self.register_sound(id, config);
        }
    }
    
    /// Register a modifier preset
    pub fn register_modifier(&mut self, name: impl Into<String>, modifiers: SoundModifiers) {
        let name = name.into();
        self.modifier_presets.insert(name, modifiers);
    }
    
    /// Register multiple modifier presets at once
    pub fn register_modifiers(&mut self, modifiers: Vec<(impl Into<String>, SoundModifiers)>) {
        for (name, modifier) in modifiers {
            self.register_modifier(name, modifier);
        }
    }
    
    /// Get a modifier preset by name
    pub fn get_modifier(&self, name: &str) -> Option<&SoundModifiers> {
        self.modifier_presets.get(name)
    }
    
    /// Play a registered sound with default modifiers
    pub fn play(&mut self, id: impl Into<String>) {
        let id = id.into();
        self.active_sounds.insert(id, None); // Use default modifiers
    }
    
    /// Play a registered sound with custom modifiers
    pub fn play_with_modifiers(&mut self, id: impl Into<String>, modifiers: SoundModifiers) {
        let id = id.into();
        self.active_sounds.insert(id, Some(modifiers));
    }
    
    /// Play a registered sound with a modifier preset by name
    pub fn play_with_modifier_name(&mut self, id: impl Into<String>, modifier_name: impl Into<String>) {
        let id = id.into();
        let name = modifier_name.into();
        
        if let Some(modifier) = self.modifier_presets.get(&name) {
            let mut modifier_clone = modifier.clone();
            // Set the name reference in the clone
            modifier_clone.name = Some(name);
            self.active_sounds.insert(id, Some(modifier_clone));
        } else {
            // Fall back to default modifiers if the named preset doesn't exist
            self.active_sounds.insert(id, None);
        }
    }
    
    /// Stop a registered sound
    pub fn stop(&mut self, id: impl Into<String>) {
        let id = id.into();
        if let Some(Some(_)) = self.active_sounds.get(&id) {
            self.active_sounds.insert(id, None);
        }
    }
    
    /// Check if a sound is currently active
    pub fn is_active(&self, id: impl Into<String>) -> bool {
        self.active_sounds.get(&id.into()).is_some_and(|m| m.is_some())
    }
    
    /// Get all currently active sound IDs
    pub fn active_sounds(&self) -> Vec<String> {
        self.active_sounds
            .iter()
            .filter_map(|(id, modifiers)| {
                modifiers.is_some().then(|| id.clone())
            })
            .collect()
    }
}

// ======== SYSTEMS ========

/// Process sound state changes and update SpatialAudioEmitter
fn process_audio_states(
    frame: FrameCount,
    session: Option<Res<Session<PeerConfig>>>,
    mut commands: Commands,
    audio: Res<Audio>,
    asset_server: Res<AssetServer>,
    mut query: Query<(
        Entity, 
        &mut GameAudioEmitter, 
        Option<&mut SpatialAudioEmitter>,
        Option<&mut SpatialRadius>,
        Option<&Transform>
    )>,
) {
    // Skip if we're in a GGRS rollback and not on a confirmed frame
    if let Some(session) = session.as_ref() {
        match session.as_ref() {
            Session::P2P(s) => {
                println!("Confirmated {} {}", s.confirmed_frame(), frame.frame);
            },
            _ => {
                return;
            }
        };
    }
    
    for (entity, mut game_audio, spatial_emitter, spatial_radius, transform) in query.iter_mut() {
        let mut sound_instances = Vec::new();
        let mut spatial_radius_value = game_audio.default_radius;
        
        // Process each potentially active sound
        for (id, modifiers_opt) in game_audio.active_sounds.iter() {
            let already_playing = game_audio.instances.contains_key(id);
            let should_play = modifiers_opt.is_some();
            
            // Start sounds that should be active but aren't playing
            if should_play && !already_playing {
                // Check if we've hit the concurrent sound limit
                if game_audio.max_concurrent > 0 && game_audio.instances.len() >= game_audio.max_concurrent {
                    continue;
                }
                
                // Get sound config
                if let Some(config) = game_audio.sound_configs.get(id) {
                    // Use the largest radius of any active sound
                    if let Some(radius) = config.radius {
                        spatial_radius_value = spatial_radius_value.max(radius);
                    }
                    
                    // Start building the audio playback
                    let mut audio_playback = audio.play(asset_server.load(&config.path));
                    
                    // Apply base configuration
                    let effective_volume = config.volume;
                    let mut audio_playback = audio_playback.with_volume(effective_volume);
                    
                    if config.looped {
                        audio_playback = audio_playback.looped();
                    }
                    
                    // Get effective default modifiers - either direct or from a named preset
                    let default_modifiers = config.get_default_modifiers(&game_audio);
                    
                    // Apply default modifiers if present
                    if let Some(ref default_mods) = default_modifiers {
                        if let Some(pitch) = default_mods.pitch {
                            audio_playback = audio_playback.with_playback_rate(pitch);
                        }
                        
                        if let Some(panning) = default_mods.panning {
                            audio_playback = audio_playback.with_panning(panning);
                        }
                        
                        if let Some(fade_in) = default_mods.fade_in {
                            audio_playback = audio_playback.fade_in(AudioTween::linear(std::time::Duration::from_secs_f64(fade_in)));
                        }
                        
                        if let Some(start_pos) = default_mods.start_position {
                            //audio_playback = audio_playback.seek_to(start_pos);
                        }
                    }
                    
                    // Override with custom modifiers if provided
                    if let Some(modifiers) = modifiers_opt {
                        // First check if this is a named modifier preset
                        let effective_modifiers = if let Some(name) = &modifiers.name {
                            // Use the named preset with any explicit overrides from modifiers
                            if let Some(preset) = game_audio.modifier_presets.get(name) {
                                merge_modifiers(preset, modifiers)
                            } else {
                                modifiers.clone()
                            }
                        } else {
                            modifiers.clone()
                        };
                        
                        // Apply the effective modifiers
                        if let Some(pitch) = effective_modifiers.pitch {
                            audio_playback = audio_playback.with_playback_rate(pitch);
                        }
                        
                        if let Some(panning) = effective_modifiers.panning {
                            audio_playback = audio_playback.with_panning(panning);
                        }
                        
                        if let Some(volume) = effective_modifiers.volume_override {
                            audio_playback = audio_playback.with_volume(volume);
                        }
                        
                        if let Some(fade_in) = effective_modifiers.fade_in {
                            audio_playback = audio_playback.fade_in(std::time::Duration::from_secs_f64(fade_in));
                        }
                        
                        if let Some(start_pos) = effective_modifiers.start_position {
                            audio_playback = audio_playback.seek_to(start_pos);
                        }
                        
                        // Handle position offset for spatial audio
                        if let Some(position_offset) = effective_modifiers.position_offset {
                            if let Some(transform) = transform {
                                // Calculate world position based on entity transform + offset
                                let world_pos = transform.translation + position_offset;
                                audio_playback = audio_playback.with_spatial_position(world_pos);
                            }
                        }
                    }
                    
                    // Get instance handle and store it
                    let instance_handle = audio_playback.handle();
                    game_audio.instances.insert(id.clone(), instance_handle.clone());
                    sound_instances.push(instance_handle);
                }
            }
            // Stop sounds that shouldn't be active but are playing
            else if !should_play && already_playing {
                if let Some(handle) = game_audio.instances.get(id) {
                    audio.stop_handle(handle.clone());
                    game_audio.instances.remove(id);
                }
            }
            // For sounds that are already playing and should continue
            else if should_play && already_playing {
                if let Some(handle) = game_audio.instances.get(id) {
                    // Add to list of active instances
                    sound_instances.push(handle.clone());
                    
                    // Determine the effective modifiers (named or direct)
                    let effective_modifiers = if let Some(modifiers) = modifiers_opt {
                        if let Some(name) = &modifiers.name {
                            if let Some(preset) = game_audio.modifier_presets.get(name) {
                                merge_modifiers(preset, modifiers)
                            } else {
                                modifiers.clone()
                            }
                        } else {
                            modifiers.clone()
                        }
                    } else {
                        SoundModifiers::default()
                    };
                    
                    // Update modifiers if needed
                    if let Some(volume) = effective_modifiers.volume_override {
                        audio.set_volume(handle.clone(), volume);
                    }
                    
                    if let Some(pitch) = effective_modifiers.pitch {
                        audio.set_playback_rate(handle.clone(), pitch);
                    }
                    
                    if let Some(panning) = effective_modifiers.panning {
                        audio.set_panning(handle.clone(), panning);
                    }
                    
                    // For spatial position updating
                    if let (Some(position_offset), Some(transform)) = (effective_modifiers.position_offset, transform) {
                        let world_pos = transform.translation + position_offset;
                        audio.set_spatial_position(handle.clone(), world_pos);
                    }
                    
                    // Update radius if needed
                    if let Some(config) = game_audio.sound_configs.get(id) {
                        if let Some(radius) = config.radius {
                            spatial_radius_value = spatial_radius_value.max(radius);
                        }
                    }
                }
            }
        }
        
        // Update or create SpatialAudioEmitter component
        if let Some(mut spatial_emitter) = spatial_emitter {
            spatial_emitter.instances = sound_instances;
        } else if !sound_instances.is_empty() {
            commands.entity(entity).insert(SpatialAudioEmitter {
                instances: sound_instances,
            });
        }
        
        // Update or create SpatialRadius component
        if let Some(mut radius) = spatial_radius {
            radius.radius = spatial_radius_value;
        } else {
            commands.entity(entity).insert(SpatialRadius {
                radius: spatial_radius_value,
            });
        }
    }
}

/// Helper function to merge two sets of modifiers, with the second taking precedence
fn merge_modifiers(base: &SoundModifiers, overrides: &SoundModifiers) -> SoundModifiers {
    SoundModifiers {
        name: overrides.name.clone().or_else(|| base.name.clone()),
        pitch: overrides.pitch.or(base.pitch),
        panning: overrides.panning.or(base.panning),
        volume_override: overrides.volume_override.or(base.volume_override),
        fade_in: overrides.fade_in.or(base.fade_in),
        start_position: overrides.start_position.or(base.start_position),
        position_offset: overrides.position_offset.or(base.position_offset),
    }
}

/// System to clean up finished audio instances
fn cleanup_audio_instances(
    audio: Res<Audio>,
    mut query: Query<&mut GameAudioEmitter>,
) {
    for mut emitter in query.iter_mut() {
        // Check each instance and remove if finished
        let mut to_remove = Vec::new();
        
        for (id, handle) in emitter.instances.iter() {
            if audio.state(handle) == PlaybackState::Stopped {
                to_remove.push(id.clone());
                // Also update the active state
                emitter.active_sounds.insert(id.clone(), None);
            }
        }
        
        // Remove finished instances
        for id in to_remove {
            emitter.instances.remove(&id);
        }
    }
}

// ======== SOUND VARIANT SYSTEM ========

/// System to handle sound variants (different versions of the same sound)
pub fn register_sound_variant(
    emitter: &mut GameAudioEmitter,
    base_id: impl Into<String>,
    variant_id: impl Into<String>,
    config: SoundConfig,
) {
    let base_id = base_id.into();
    let variant_id = format!("{}:{}", base_id, variant_id.into());
    
    // Register this as a normal sound
    emitter.register_sound(variant_id, config);
}

/// Play a specific variant of a sound
pub fn play_sound_variant(
    emitter: &mut GameAudioEmitter,
    variants: &mut ActiveSoundVariants,
    base_id: impl Into<String>,
    variant_id: impl Into<String>,
    modifiers: Option<SoundModifiers>,
) {
    let base_id = base_id.into();
    let variant_id = format!("{}:{}", base_id, variant_id.into());
    
    // Update the active variant map
    variants.active_variants.insert(base_id, variant_id.clone());
    
    // Play the sound with or without modifiers
    if let Some(mods) = modifiers {
        emitter.play_with_modifiers(variant_id, mods);
    } else {
        emitter.play(variant_id);
    }
}

/// Play a specific variant of a sound with a named modifier preset
pub fn play_sound_variant_with_modifier_name(
    emitter: &mut GameAudioEmitter,
    variants: &mut ActiveSoundVariants,
    base_id: impl Into<String>,
    variant_id: impl Into<String>,
    modifier_name: impl Into<String>,
) {
    let base_id = base_id.into();
    let variant_id = format!("{}:{}", base_id, variant_id.into());
    
    // Update the active variant map
    variants.active_variants.insert(base_id, variant_id.clone());
    
    // Play with the named modifier preset
    emitter.play_with_modifier_name(variant_id, modifier_name);
}

// ======== USAGE EXAMPLES ========

// Example system setting up audio emitters with variants and named modifiers
fn setup_player_audio(mut commands: Commands) {
    // Create a new audio emitter with configuration
    let mut audio_emitter = GameAudioEmitter::new()
        .with_max_concurrent(8)
        .with_default_radius(10.0);
    
    // Register modifier presets
    audio_emitter.register_modifiers(vec![
        // Footstep modifiers for different speeds
        ("walk_normal", SoundModifiers::new()
            .with_volume(0.6)
            .with_pitch(1.0)),
            
        ("walk_fast", SoundModifiers::new()
            .with_volume(0.8)
            .with_pitch(1.2)),
            
        ("walk_slow", SoundModifiers::new()
            .with_volume(0.4)
            .with_pitch(0.8)),
            
        // Jump modifiers for different heights
        ("jump_small", SoundModifiers::new()
            .with_volume(0.5)
            .with_pitch(1.1)),
            
        ("jump_medium", SoundModifiers::new()
            .with_volume(0.7)
            .with_pitch(1.0)),
            
        ("jump_large", SoundModifiers::new()
            .with_volume(0.9)
            .with_pitch(0.9)),
    ]);
    
    // Register sounds with their configurations
    audio_emitter.register_sounds(vec![
        ("footstep", SoundConfig::new("sounds/footstep.ogg")
            .with_volume(0.6)
            .with_radius(8.0)
            .with_default_modifier_name("walk_normal")),
            
        ("jump", SoundConfig::new("sounds/jump.ogg")
            .with_volume(0.8)
            .with_radius(12.0)
            .with_default_modifier_name("jump_medium")),
    ]);
    
    // Register sound variants (different surfaces for footsteps)
    register_sound_variant(
        &mut audio_emitter,
        "footstep", 
        "concrete",
        SoundConfig::new("sounds/footstep_concrete.ogg")
            .with_volume(0.7)
            .with_radius(9.0)
            .with_default_modifier_name("walk_normal")
    );
    
    register_sound_variant(
        &mut audio_emitter,
        "footstep", 
        "metal",
        SoundConfig::new("sounds/footstep_metal.ogg")
            .with_volume(0.8)
            .with_radius(12.0)
            .with_default_modifiers(SoundModifiers::new().with_pitch(1.1))
    );
    
    register_sound_variant(
        &mut audio_emitter,
        "footstep", 
        "water",
        SoundConfig::new("sounds/footstep_water.ogg")
            .with_volume(0.7)
            .with_radius(7.0)
            .with_default_modifier_name("walk_slow")
    );
    
    // Spawn the player with this audio emitter
    commands.spawn((
        // Standard player components
        PlayerBundle::default(),
        Transform::default(),
        
        // Audio components
        audio_emitter,
        ActiveSoundVariants::default(),
    ));
}

// Example weapon setup with modifiers
fn setup_weapon_audio(mut commands: Commands) {
    let mut audio_emitter = GameAudioEmitter::new()
        .with_max_concurrent(3)
        .with_default_radius(20.0);
    
    // Register modifier presets
    audio_emitter.register_modifiers(vec![
        ("shot_normal", SoundModifiers::new()
            .with_volume(1.0)
            .with_position_offset(Vec3::new(0.0, 0.1, 0.5))),
            
        ("shot_silenced", SoundModifiers::new()
            .with_volume(0.7)
            .with_position_offset(Vec3::new(0.0, 0.1, 0.7))),
            
        ("shot_burst", SoundModifiers::new()
            .with_volume(1.2)
            .with_position_offset(Vec3::new(0.0, 0.1, 0.5))),
    ]);
    
    audio_emitter.register_sounds(vec![
        ("fire", SoundConfig::new("sounds/shot.ogg")
            .with_volume(1.0)
            .with_radius(25.0)
            .with_default_modifier_name("shot_normal")),
            
        ("reload", SoundConfig::new("sounds/reload.ogg")
            .with_volume(0.8)
            .with_radius(15.0)
            .with_default_modifiers(SoundModifiers::new()
                .with_position_offset(Vec3::new(0.2, -0.1, 0.3)))),
            
        ("empty", SoundConfig::new("sounds/empty.ogg")
            .with_volume(0.6)
            .with_radius(8.0)),
    ]);
    
    // Register sound variants for different firing modes
    register_sound_variant(
        &mut audio_emitter,
        "fire", 
        "silenced",
        SoundConfig::new("sounds/shot_silenced.ogg")
            .with_volume(0.7)
            .with_radius(12.0)
            .with_default_modifier_name("shot_silenced")
    );
    
    register_sound_variant(
        &mut audio_emitter,
        "fire", 
        "burst",
        SoundConfig::new("sounds/shot_burst.ogg")
            .with_volume(1.0)
            .with_radius(28.0)
            .with_default_modifier_name("shot_burst")
    );
    
    commands.spawn((
        // Standard weapon components
        WeaponBundle::default(),
        Transform::default(),
        
        // Audio components
        audio_emitter,
        ActiveSoundVariants::default(),
    ));
}

// Example GGRS rollback system that triggers sounds with modifiers
fn player_movement_system(
    mut query: Query<(
        &PlayerState, 
        &mut GameAudioEmitter, 
        &mut ActiveSoundVariants, 
        &Transform, 
        &Terrain
    )>
) {
    for (player_state, mut audio, mut variants, transform, terrain) in query.iter_mut() {
        // Toggle footstep sounds based on walking state
        if player_state.is_walking {
            // Choose appropriate modifier based on speed
            let modifier_name = if player_state.speed_factor > 1.3 {
                "walk_fast"
            } else if player_state.speed_factor < 0.7 {
                "walk_slow"
            } else {
                "walk_normal"
            };
            
            // Choose footstep variant based on terrain
            match terrain.surface_type {
                SurfaceType::Concrete => {
                    play_sound_variant_with_modifier_name(
                        &mut audio, 
                        &mut variants, 
                        "footstep", 
                        "concrete", 
                        modifier_name
                    );
                },
                SurfaceType::Metal => {
                    // For metal, we'll use the named modifier but add a pitch override
                    let mut modifiers = SoundModifiers::new()
                        .with_name(modifier_name);
                        
                    // Add extra pitch for metal surfaces based on speed
                    if player_state.speed_factor > 1.0 {
                        modifiers = modifiers.with_pitch(1.2);
                    }
                    
                    play_sound_variant(
                        &mut audio, 
                        &mut variants, 
                        "footstep", 
                        "metal", 
                        Some(modifiers)
                    );
                },
                SurfaceType::Water => {
                    // For water, we'll always use the slow modifier regardless of speed
                    play_sound_variant_with_modifier_name(
                        &mut audio, 
                        &mut variants, 
                        "footstep", 
                        "water", 
                        "walk_slow"
                    );
                },
                _ => {
                    // Default footstep sound with the standard modifier
                    audio.play_with_modifier_name("footstep", modifier_name);
                }
            }
        } else {
            audio.stop("footstep");
            // Also stop all variants
            for variant_id in variants.active_variants.values() {
                audio.stop(variant_id);
            }
        }
        
        // Play jump sound with modifier based on jump charge
        if player_state.just_jumped {
            let jump_modifier_name = if player_state.jump_charge > 0.8 {
                "jump_large"
            } else if player_state.jump_charge > 0.4 {
                "jump_medium"
            } else {
                "jump_small"
            };
            
            audio.play_with_modifier_name("jump", jump_modifier_name);
        }
    }
}

// Example weapon sound system using named modifiers
fn weapon_system(mut query: Query<(&WeaponState, &mut GameAudioEmitter, &mut ActiveSoundVariants, &Transform)>) {
    for (weapon_state, mut audio, mut variants, transform) in query.iter_mut() {
        match weapon_state {
            WeaponState::Firing => {
                // Choose firing sound based on weapon mode
                match weapon_state.fire_mode {
                    FireMode::Single => {
                        audio.play_with_modifier_name("fire", "shot_normal");
                    },
                    FireMode::Silenced => {
                        play_sound_variant_with_modifier_name(
                            &mut audio, 
                            &mut variants, 
                            "fire", 
                            "silenced", 
                            "shot_silenced"
                        );
                    },
                    FireMode::Burst => {
                        play_sound_variant_with_modifier_name(
                            &mut audio, 
                            &mut variants, 
                            "fire", 
                            "burst", 
                            "shot_burst"
                        );
                    }
                }
            },
            WeaponState::Reloading => {
                // Use the default configured modifiers for reload
                audio.play("reload");
            },
            WeaponState::Empty => {
                // Create a one-time custom modifier for empty sound
                let empty_modifiers = SoundModifiers::new()
                    .with_panning(0.2) // Slight right panning
                    .with_volume(0.5);
                
                audio.play_with_modifiers("empty", empty_modifiers);
            },
            _ => {
                // Stop all sounds
                audio.stop("fire");
                audio.stop("reload");
                audio.stop("empty");
                
                // Stop variants too
                for variant_id in variants.active_variants.values() {
                    audio.stop(variant_id);
                }
            }
        }
    }
}

// Example of serializing and deserializing the audio configuration
fn save_audio_config(world: &mut World) {
    use bevy::asset::{AssetServer, Assets};
    use bevy::prelude::*;
    use std::fs::File;
    use std::io::Write;
    
    // Get all GameAudioEmitter components
    let mut emitters = Vec::new();
    for (entity, emitter) in world.query::<(Entity, &GameAudioEmitter)>().iter(world) {
        emitters.push((entity, emitter.clone()));
    }
    
    // Serialize each emitter to JSON
    for (entity, emitter) in emitters {
        let serialized = serde_json::to_string_pretty(&emitter).unwrap();
        let filename = format!("assets/audio/emitter_{}.json", entity.index());
        
        let mut file = File::create(filename).unwrap();
        file.write_all(serialized.as_bytes()).unwrap();
    }
}

// Example of loading a serialized audio configuration
fn load_audio_config(path: &str) -> Result<GameAudioEmitter, Box<dyn std::error::Error>> {
    use std::fs::File;
    use std::io::Read;
    
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    
    let emitter: GameAudioEmitter = serde_json::from_str(&contents)?;
    Ok(emitter)
}

// Example system that dynamically loads audio configurations from asset files
fn load_audio_configs_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    query: Query<(Entity, &AudioConfigPath), Added<AudioConfigPath>>,
) {
    for (entity, config_path) in &query {
        match load_audio_config(&config_path.0) {
            Ok(emitter) => {
                commands.entity(entity).insert(emitter);
            },
            Err(err) => {
                eprintln!("Failed to load audio config from {}: {}", config_path.0, err);
            }
        }
    }
}

// Helper component to reference audio config files
#[derive(Component)]
struct AudioConfigPath(String);

// Example plugin registration with serialization support
pub struct SerializableGameAudioPlugin;

impl Plugin for SerializableGameAudioPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<GameAudioEmitter>()
           .register_type::<ActiveSoundVariants>()
           .register_type::<AudioConfigPath>()
           // Add the base plugin functionality
           .add_plugins(GameAudioPlugin)
           // Add serialization systems
           .add_systems(Update, load_audio_configs_system);
    }
}


pub struct ZAudioPlugin {}

impl Plugin for ZAudioPlugin {
   fn build(&self, app: &mut App) {
       app.add_plugins(AudioPlugin);
       app.add_plugins(SpatialAudioPlugin);
       //app.add_systems(Startup, play_loop);
   } 
    
}

fn play_loop(asset_server: Res<AssetServer>, audio: Res<Audio>) {
    audio.play(asset_server.load("sounds/loop.ogg")).looped();
}
