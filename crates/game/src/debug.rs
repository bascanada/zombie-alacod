use bevy::prelude::*;

#[derive(Resource, Default)]
struct SpriteDebugOverlayState {
    is_visible: bool,
}


fn toggle_sprite_debug_visibility_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<SpriteDebugOverlayState>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyP) {
        state.is_visible = !state.is_visible;
        if state.is_visible {
            info!("Sprite debug overlay: ON");
        } else {
            info!("Sprite debug overlay: OFF");
        }
    }
}


fn draw_sprite_debug_rects_system(
    mut gizmos: Gizmos, // System parameter to draw gizmos
    // Query for sprite entities. Add With<YourPlayerMarker> or similar if you only want it for specific sprites.
    sprite_query: Query<(&GlobalTransform, &Sprite)>,
    image_assets: Res<Assets<Image>>, // To get image dimensions
) {
    // Define the color for your debug rectangles (e.g., semi-transparent orange)
    let debug_rect_color = Color::srgba(1.0, 0.4, 0.0, 0.35);

    for (global_transform, sprite) in sprite_query.iter() {
        let transform = global_transform.compute_transform();

        // 1. Determine the sprite's base size (before scaling)
        let base_size: Vec2 = if let Some(custom_size) = sprite.custom_size {
            custom_size
        } else if let Some(image) = image_assets.get(&sprite.image) {
            // Use the image's actual dimensions
            image.size_f32()
        } else {
            // Image asset not yet loaded or handle is invalid, skip this sprite
            println!("CAND");
            continue;
        };

        // 2. Calculate the final visual size after applying the transform's scale
        // The scale from GlobalTransform is the final world scale.
        let final_visual_size = base_size * transform.scale.truncate();

        // 3. Get the 2D world position (center of the sprite)
        let world_position = transform.translation.truncate();

        // 4. Get the 2D rotation (angle around the Z-axis)
        // The transform.rotation is a Quaternion. We need to extract the Z-axis rotation.
        let z_rotation_angle = transform.rotation.to_euler(EulerRot::ZYX).0; // .0 gives Z for ZYX order

        // 5. Draw the 2D rectangle using gizmos
        gizmos.rect_2d(
            world_position,
            final_visual_size,
            debug_rect_color,
        );
    }
}


pub struct SpriteDebugOverlayPlugin;

impl Plugin for SpriteDebugOverlayPlugin {
    fn build(&self, app: &mut App) {
        // Ensure GizmoPlugin is added. It's part of DefaultPlugins in recent Bevy versions.
        // If not using DefaultPlugins or on an older version, you might need:
        if !app.is_plugin_added::<bevy::gizmos::GizmoPlugin>() {
             app.add_plugins(bevy::gizmos::GizmoPlugin);
        }

        app
            .init_resource::<SpriteDebugOverlayState>()
            .add_systems(Update,
                (
                    toggle_sprite_debug_visibility_system,
                    // Apply the run condition for the drawing system
                    draw_sprite_debug_rects_system
                        .run_if(|state: Res<SpriteDebugOverlayState>| state.is_visible),
                )
            );
    }
}
