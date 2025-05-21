use bevy::math::{Vec2, Vec3};

use crate::rng::RollbackRng;


pub fn calculate_spread_angle(
    rng: &mut RollbackRng,
    spread: f32,
) -> f32 {
    // Original line:
    // let spread_angle = (rand::random::<f32>() - 0.5)  * .spread;

    // New rollback-appropriate line:
    // Generate a random f32 between 0.0 and 1.0
    let random_val_0_to_1 = rng.next_f32();

    // Convert to a range of -0.5 to 0.5 (similar to original logic)
    let random_val_neg_0_5_to_0_5 = random_val_0_to_1 - 0.5;

    let spread_angle = random_val_neg_0_5_to_0_5 * spread;
    spread_angle
}

pub fn calculate_time_remaining_seconds(ending_frame_number: u32, current_frame: u32) -> f32 {
    // Ensure current_frame does not exceed ending_frame_number to prevent underflow
    // and negative time. If it does, no time is remaining.
    if current_frame >= ending_frame_number {
        return 0.0;
    }

    // Calculate the number of frames remaining
    let frames_remaining: u32 = ending_frame_number - current_frame;

    // Convert frames remaining to seconds
    // We cast frames_remaining to f32 for the division
    let time_remaining_seconds: f32 = frames_remaining as f32 / 60.0;

    time_remaining_seconds
}


pub fn round(n: f32) -> f32 {
    (n * 1000.0).round() / 1000.0
}

pub fn round_vec2(v: Vec2) -> Vec2 {
    Vec2::new(
        (v.x * 1000.0).round() / 1000.0, 
        (v.y * 1000.0).round() / 1000.0
    )
}

pub fn round_vec3(v: Vec3) -> Vec3 {
    Vec3::new(
        (v.x * 1000.0).round() / 1000.0, 
        (v.y * 1000.0).round() / 1000.0,
        (v.z * 1000.0).round() / 1000.0,
    )
}




#[cfg(test)]
mod tests {
    use super::*; // Import items from the parent module (RollbackRng)
    use std::f32::consts::PI; // For testing spread angle

    #[test]
    fn test_rng_new() {
        let rng = RollbackRng::new(42);
        assert_eq!(rng.seed, 42, "RNG seed should be initialized correctly.");
    }

    #[test]
    fn test_rng_determinism_u32() {
        let mut rng1 = RollbackRng::new(12345);
        let mut rng2 = RollbackRng::new(12345);

        let mut sequence1 = Vec::new();
        let mut sequence2 = Vec::new();

        for _ in 0..100 {
            sequence1.push(rng1.next_u32());
            sequence2.push(rng2.next_u32());
        }

        assert_eq!(sequence1, sequence2, "Two RNGs with the same seed should produce the same sequence of u32s.");
        assert_ne!(rng1.seed, 12345, "RNG seed should change after generation.");
    }

    #[test]
    fn test_rng_determinism_f32() {
        let mut rng1 = RollbackRng::new(54321);
        let mut rng2 = RollbackRng::new(54321);

        let mut sequence1 = Vec::new();
        let mut sequence2 = Vec::new();

        for _ in 0..100 {
            // Pushing f32 directly can have precision issues with assert_eq! on Vecs.
            // For testing determinism, comparing the bit patterns of f32s is more robust if needed,
            // but direct comparison should work for this LCG.
            sequence1.push(rng1.next_f32());
            sequence2.push(rng2.next_f32());
        }
        assert_eq!(sequence1, sequence2, "Two RNGs with the same seed should produce the same sequence of f32s.");
    }

    #[test]
    fn test_rng_f32_range() {
        let mut rng = RollbackRng::new(98765);
        for _ in 0..1000 {
            let val = rng.next_f32();
            assert!(val >= 0.0 && val < 1.0, "next_f32() output {} was not in range [0.0, 1.0)", val);
        }
    }

    #[test]
    fn test_rng_f32_symmetric_range() {
        let mut rng = RollbackRng::new(112233);
        for _ in 0..1000 {
            let val = rng.next_f32_symmetric();
            assert!(val >= -1.0 && val < 1.0, "next_f32_symmetric() output {} was not in range [-1.0, 1.0)", val);
        }
    }

    #[test]
    fn test_rng_different_seeds_produce_different_sequences() {
        let mut rng1 = RollbackRng::new(100);
        let mut rng2 = RollbackRng::new(200); // Different seed

        let val1 = rng1.next_u32();
        let val2 = rng2.next_u32();

        assert_ne!(val1, val2, "RNGs with different seeds should produce different first values (highly likely).");

        // Further check a short sequence
        let mut seq1 = vec![val1];
        let mut seq2 = vec![val2];
        for _ in 0..10 {
            seq1.push(rng1.next_u32());
            seq2.push(rng2.next_u32());
        }
        assert_ne!(seq1, seq2, "RNGs with different seeds should produce different sequences.");
    }

     #[test]
    fn test_rng_state_changes() {
        let mut rng = RollbackRng::new(777);
        let initial_seed = rng.seed;
        rng.next_u32();
        assert_ne!(rng.seed, initial_seed, "Seed should change after calling next_u32.");
        let seed_after_u32 = rng.seed;
        rng.next_f32();
        assert_ne!(rng.seed, seed_after_u32, "Seed should change after calling next_f32.");
        let seed_after_f32 = rng.seed;
        rng.next_f32_symmetric();
        assert_ne!(rng.seed, seed_after_f32, "Seed should change after calling next_f32_symmetric.");
    }

    #[test]
    fn test_calculate_spread_angle_range() {
        let mut rng = RollbackRng::new(88888);
        let max_spread_rad = PI / 4.0; // e.g., 45 degrees max spread in one direction from center

        for _ in 0..1000 {
            let angle = calculate_spread_angle(&mut rng, max_spread_rad);
            // The angle should be between -max_spread_rad / 2 and +max_spread_rad / 2
            // if random_val_neg_0_5_to_0_5 is in [-0.5, 0.5)
            // So, angle is in [-max_spread_rad * 0.5, max_spread_rad * 0.5)
            let half_max_spread = max_spread_rad * 0.5;
            assert!(angle >= -half_max_spread && angle < half_max_spread,
                    "Calculated spread angle {} was not in range [{}, {}) for max_spread_rad {}",
                    angle, -half_max_spread, half_max_spread, max_spread_rad);
        }

        // Test with zero spread
        let angle_zero_spread = calculate_spread_angle(&mut rng, 0.0);
        assert_eq!(angle_zero_spread, 0.0, "Angle with zero spread should be zero.");
    }
}