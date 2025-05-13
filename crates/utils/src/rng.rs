use bevy::ecs::system::Resource;



#[derive(Debug, Resource, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RollbackRng {
    pub seed: u32,
}

impl RollbackRng {
    // Constants for the LCG algorithm. These are common choices.
    const A: u32 = 1664525;  // Multiplier
    const C: u32 = 1013904223; // Increment
    // Modulus M is implicitly 2^32 because we are using u32 and letting overflow happen.

    /// Creates a new RNG instance with a given seed.
    /// This seed should ideally be synchronized across all players at the start of the game.
    pub fn new(initial_seed: u32) -> Self {
        RollbackRng { seed: initial_seed }
    }

    /// Generates the next u32 random number.
    /// This method advances the RNG state.
    pub fn next_u32(&mut self) -> u32 {
        // LCG formula: X_n+1 = (a * X_n + c) mod m
        // Here, we use wrapping arithmetic for `mod 2^32`.
        self.seed = self.seed.wrapping_mul(Self::A).wrapping_add(Self::C);
        self.seed
    }

    /// Generates a random f32 value between 0.0 (inclusive) and 1.0 (exclusive).
    pub fn next_f32(&mut self) -> f32 {
        // Convert the u32 to f32 in the range [0,1)
        // We divide by 2^32.
        // U32_MAX is 2^32 - 1. To get a value strictly less than 1.0,
        // we can use the formula: random_u32 / (U32_MAX + 1.0), which is random_u32 / 2^32.
        self.next_u32() as f32 / 4294967296.0 // 2.0f32.powi(32)
    }

    /// Generates a random f32 value between -1.0 (inclusive) and 1.0 (exclusive).
    pub fn next_f32_symmetric(&mut self) -> f32 {
        (self.next_f32() * 2.0) - 1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*; // Import items from the parent module (RollbackRng)

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
}
