use std::ops::Mul;

use bevy::ecs::component::Component;
use bevy::ecs::system::{Commands, Resource};
use bevy::math::{Vec2, Vec3};
use serde::{Deserialize, Serialize};

use fixed_trigonometry::*;
use fixed::types::extra::U16; // For 16 fractional bits
use fixed::FixedI32;


// UNSIGNED NUMBER SCALED TO 100 TO REPLACE FLOAT

const SCALING_FACTOR: i32 = 100;
const SCALING_FACTOR_F32: f32 = SCALING_FACTOR as f32;

pub type R32 = i32;

#[derive(Clone, Copy, Serialize, Deserialize, Default, Debug)]
pub struct RVec2 {
    pub x: R32,
    pub y: R32,
}

impl RVec2 {
    pub fn new(x: R32, y: R32) -> Self {
        Self { x, y }
    }
    
}

impl Mul<i32> for RVec2 {
    type Output = RVec2;
    fn mul(self, rhs: i32) -> Self::Output {
        RVec2 { x: self.x * rhs , y: self.y * rhs}
    }
}

impl Into<Vec2> for RVec2 {
    fn into(self) -> Vec2 {
        Vec2::new(to_f32(self.x), to_f32(self.y))
    }
}

impl Into<Vec3> for RVec2 {
    fn into(self) -> Vec3 {
        Vec3::new(to_f32(self.x), to_f32(self.y), 0.0)
    }
}

impl PartialEq for RVec2 {
    fn eq(&self, other: &Self) -> bool {
        return self.x == other.x && self.y == other.y;
    }

    fn ne(&self, other: &Self) -> bool {
        return self.x != other.x || self.y != other.y;
    }
    
}

pub fn to_f32(r: R32) -> f32 {
    r as f32 / SCALING_FACTOR_F32
}






// Trait for a deterministic RNG that GGRS can use
pub trait DeterministicRng {
    /// Creates a new RNG instance with a specific seed.
    /// The seed itself must be deterministic across clients.
    fn new_with_seed(seed: u32) -> Self
    where
        Self: Sized;

    /// Generates a scaled random factor, e.g., for spread.
    /// Expected to return a value in a specific range, e.g., [-10000, 10000].
    fn gen_scaled_random_factor(&mut self) -> i32;

    // You might want to add other methods like gen_u32, gen_range, etc.
    // For now, we'll stick to the one needed for the spread example.
}

#[derive(Resource, Clone, Debug)] // Add Copy if your GGRS setup benefits from it for state
pub struct Xorshift32Rng {
    /// The internal state of the RNG. This is what GGRS would need to
    /// save and restore if the RNG state is part of the synchronized game state.
    state: u32,
}

impl Xorshift32Rng {
    /// Generates the next u32 random number.
    fn next_u32(&mut self) -> u32 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.state = x;
        x
    }
}

impl DeterministicRng for Xorshift32Rng {
    /// Creates a new Xorshift32Rng instance.
    ///
    /// # Panics
    /// The seed for Xorshift32 **must not be zero**. This implementation
    /// will use 1 if a seed of 0 is provided, or you can panic/assert.
    /// For GGRS, ensure your seeding strategy avoids zero or handles it.
    fn new_with_seed(seed: u32) -> Self {
        if seed == 0 {
            // eprintln!("Warning: Xorshift32Rng seed was 0, using 1 instead.");
            Xorshift32Rng { state: 1 } // Or some other non-zero default
        } else {
            Xorshift32Rng { state: seed }
        }
    }

    /// Generates a random i32 value scaled to the range [-10000, 10000].
    fn gen_scaled_random_factor(&mut self) -> i32 {
        // Generate a u32 random number
        let random_u32 = self.next_u32();

        // Map it to the desired range: [-10000, 10000]
        // The range has 20001 possible values.
        const TARGET_RANGE_SIZE: u32 = 20001; // -10000 to +10000 inclusive
        const OFFSET: i32 = 10000;

        let scaled_value = random_u32 % TARGET_RANGE_SIZE; // Result is in [0, 20000]
        scaled_value as i32 - OFFSET // Map to [-10000, 10000]
    }
}




pub fn calculate_deterministic_spread_direction(
    rng: &mut Xorshift32Rng,
    spread_angle: R32,
    aim_dir: &RVec2,
) -> RVec2 {
    // 1. Get a deterministic random factor for the spread amount
    // Let's say gen_scaled_random_factor returns a value from -10000 to 10000.
    // We want to map this to roughly -0.5 to 0.5 as a fixed-point number.
    let random_int_factor = rng.gen_scaled_random_factor(); // e.g., -5000 if it means -0.5

    // Convert this integer factor to a fixed-point number representing [-0.5 to 0.5]
    // If random_int_factor is -5000 and you want it to mean -0.5, then divide by 10000.
    let random_fixed_factor =
        FixedI32::<U16>::from_num(random_int_factor) / FixedI32::<U16>::from_num(10000);

    // 2. Calculate the actual spread_angle as a fixed-point number
    // spread_angle = (random_factor_approx_neg_0_5_to_0_5) * weapon_config.spread_max_angle
    let spread_angle_fixed: FixedI32<U16> =
        random_fixed_factor * spread_angle;

    // 3. Calculate deterministic sin and cos of the spread_angle_fixed
    // These will also be FixedI32<U16> numbers, representing sin/cos values from -1.0 to 1.0
    let cos_spread_fixed: FixedI32<U16> = cos(spread_angle_fixed); // From fixed_trigonometry
    let sin_spread_fixed: FixedI32<U16> = sin(spread_angle_fixed); // From fixed_trigonometry

    // 4. Get the raw integer bits of sin/cos. These are scaled by TRIG_INTERNAL_SCALE_FACTOR (2^16)
    let cos_val_scaled_int: i32 = cos_spread_fixed.to_bits();
    let sin_val_scaled_int: i32 = sin_spread_fixed.to_bits();

    // 5. Perform the 2D rotation using fixed-point style multiplication
    // aim_dir components are already scaled by COMPONENT_SCALE_FACTOR (100)
    // direction_x = aim_dir.x * cos_spread - aim_dir.y * sin_spread
    // direction_y = aim_dir.x * sin_spread + aim_dir.y * cos_spread

    // Intermediate products need i64 to avoid overflow.
    // Current scaling: (aim_dir.x scaled by 100) * (cos_val_scaled_int scaled by 2^16)
    // So, product is scaled by (100 * 2^16)
    let num_x: i64 = (aim_dir.x as i64 * cos_val_scaled_int as i64)
        - (aim_dir.y as i64 * sin_val_scaled_int as i64);
    let num_y: i64 = (aim_dir.x as i64 * sin_val_scaled_int as i64)
        + (aim_dir.y as i64 * cos_val_scaled_int as i64);

    // 6. Rescale to the desired output scale (COMPONENT_SCALE_FACTOR = 100)
    // To do this, we divide by TRIG_INTERNAL_SCALE_FACTOR (2^16)
    // (num_x scaled by 100 * 2^16) / (2^16) = direction_x scaled by 100
    const TRIG_INTERNAL_SCALE_SHIFT: i32 = FixedI32::<U16>::FRAC_NBITS as i32; // This is 16

    // Integer division provides deterministic truncation.
    let final_direction_x: i32 = (num_x >> TRIG_INTERNAL_SCALE_SHIFT) as i32;
    let final_direction_y: i32 = (num_y >> TRIG_INTERNAL_SCALE_SHIFT) as i32;

    RVec2::new(final_direction_x, final_direction_y)
}






pub fn system_setup_rng(mut commands: Commands) {
    commands.insert_resource(Xorshift32Rng::new_with_seed(12345));
}