/// Trait for random number sources.
/// Simulator uses `rand`, firmware uses ESP32-S3 hardware TRNG.
pub trait RngSource {
    fn random_u32(&mut self) -> u32;
}

/// Supported die types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DieType {
    D4,
    D6,
    D8,
    D10,
    D12,
    D20,
    D100,
}

impl DieType {
    /// Maximum value for this die type
    pub fn max_value(self) -> u16 {
        match self {
            DieType::D4 => 4,
            DieType::D6 => 6,
            DieType::D8 => 8,
            DieType::D10 => 10,
            DieType::D12 => 12,
            DieType::D20 => 20,
            DieType::D100 => 100,
        }
    }

    /// Display name
    pub fn name(self) -> &'static str {
        match self {
            DieType::D4 => "d4",
            DieType::D6 => "d6",
            DieType::D8 => "d8",
            DieType::D10 => "d10",
            DieType::D12 => "d12",
            DieType::D20 => "d20",
            DieType::D100 => "d100",
        }
    }
}

/// Roll a die using rejection sampling (no modulo bias)
pub fn roll_die(rng: &mut impl RngSource, die: DieType) -> u16 {
    let max = die.max_value() as u32;
    // Find the largest multiple of max that fits in u32
    let limit = u32::MAX - (u32::MAX % max);

    loop {
        let val = rng.random_u32();
        if val < limit {
            return (val % max + 1) as u16;
        }
        // Reject and retry — prevents modulo bias
    }
}

/// Generate a random "animation frame" number (for the spinning effect)
pub fn random_display_value(rng: &mut impl RngSource, die: DieType) -> u16 {
    roll_die(rng, die)
}
