use crate::dice::{DieType, RngSource, random_display_value, roll_die};

/// Animation phases
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationState {
    /// Waiting for input
    Idle,
    /// Spinning digits — frame is current frame number (0..TOTAL_FRAMES)
    Rolling { frame: u8, display_value: u16 },
    /// Showing final result with flash effect
    Landed { result: u16, flash_frames: u8 },
}

/// Total frames in the roll animation
const ROLL_FRAMES: u8 = 15;
/// Frames to show the flash/highlight effect after landing
const FLASH_FRAMES: u8 = 4;

impl AnimationState {
    /// Start a new roll animation
    pub fn start_roll() -> Self {
        AnimationState::Rolling {
            frame: 0,
            display_value: 1,
        }
    }

    /// Advance the animation by one frame. Returns true if animation is still active.
    pub fn tick(&mut self, rng: &mut impl RngSource, die: DieType) -> bool {
        match self {
            AnimationState::Idle => false,

            AnimationState::Rolling { frame, display_value } => {
                if *frame >= ROLL_FRAMES {
                    // Animation complete — do the real roll
                    let result = roll_die(rng, die);
                    *self = AnimationState::Landed {
                        result,
                        flash_frames: FLASH_FRAMES,
                    };
                    true
                } else {
                    // Show a random number (spinning effect)
                    *display_value = random_display_value(rng, die);
                    *frame += 1;
                    true
                }
            }

            AnimationState::Landed { flash_frames, .. } => {
                if *flash_frames > 0 {
                    *flash_frames -= 1;
                    true
                } else {
                    false
                }
            }
        }
    }

    /// Get the current display value (for rendering)
    pub fn display_value(&self) -> Option<u16> {
        match self {
            AnimationState::Idle => None,
            AnimationState::Rolling { display_value, .. } => Some(*display_value),
            AnimationState::Landed { result, .. } => Some(*result),
        }
    }

    /// Is the result flashing? (for invert effect)
    pub fn is_flashing(&self) -> bool {
        matches!(self, AnimationState::Landed { flash_frames, .. } if *flash_frames > 0 && *flash_frames % 2 == 0)
    }

    /// Is the animation complete?
    pub fn is_idle(&self) -> bool {
        matches!(self, AnimationState::Idle)
    }

    /// Get the final result (if landed)
    pub fn final_result(&self) -> Option<u16> {
        match self {
            AnimationState::Landed { result, flash_frames } if *flash_frames == 0 => Some(*result),
            _ => None,
        }
    }
}
