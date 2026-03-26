//! Heltec D20 — Hardware True Random Dice Roller
//!
//! Runs on Heltec WiFi LoRa 32 V3.2 (ESP32-S3)
//! Drives the built-in SSD1306 128x64 OLED via I2C
//! Uses hardware TRNG for cryptographically fair dice rolls

#![no_std]
#![no_main]

extern crate alloc;

use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::delay::Delay;
use esp_hal::gpio::{Input, Level, Output, Pull};
use esp_hal::i2c::master::I2c;
use esp_hal::prelude::*;
use esp_hal::rng::Rng;
use log::info;

use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use ssd1306::mode::BufferedGraphicsMode;
use ssd1306::prelude::*;
use ssd1306::rotation::DisplayRotation;
use ssd1306::size::DisplaySize128x64;
use ssd1306::I2CDisplayInterface;
use ssd1306::Ssd1306;

use mousefood::{EmbeddedBackend, EmbeddedBackendConfig};
use ratatui::Terminal;

use heltec_d20_shared::animation::AnimationState;
use heltec_d20_shared::dice::RngSource;
use heltec_d20_shared::ui;
use heltec_d20_shared::AppState;

/// ESP32-S3 Hardware TRNG wrapper
struct HardwareRng(Rng);

impl RngSource for HardwareRng {
    fn random_u32(&mut self) -> u32 {
        self.0.random()
    }
}

// Heap allocator — 64KB should be plenty for ratatui + mousefood
const HEAP_SIZE: usize = 64 * 1024;
static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

#[esp_hal::entry]
fn main() -> ! {
    // Initialize heap allocator
    unsafe {
        esp_alloc::HEAP.add_region(esp_alloc::HeapRegion::new(
            HEAP.as_mut_ptr(),
            HEAP_SIZE,
            esp_alloc::MemoryCapability::Internal.into(),
        ));
    }

    // Initialize peripherals
    let peripherals = esp_hal::init(esp_hal::Config::default());
    esp_println::logger::init_logger_from_env();
    let delay = Delay::new();

    info!("⚡ Heltec D20 starting...");

    // ----- OLED Power (Vext) -----
    // Heltec V3.2: GPIO36 controls OLED power (active LOW)
    let mut vext = Output::new(peripherals.GPIO36, Level::Low);
    delay.delay_millis(100); // Let OLED power stabilize

    // ----- OLED Reset -----
    // GPIO21 is the OLED reset pin
    let mut rst = Output::new(peripherals.GPIO21, Level::Low);
    delay.delay_millis(10);
    rst.set_high();
    delay.delay_millis(10);

    // ----- I2C for OLED -----
    // Heltec V3.2: SDA=GPIO17, SCL=GPIO18
    let i2c = I2c::new(peripherals.I2C0, {
        let mut config = esp_hal::i2c::master::Config::default();
        config.frequency = 400.kHz();
        config
    })
    .with_sda(peripherals.GPIO17)
    .with_scl(peripherals.GPIO18);

    // ----- SSD1306 Display -----
    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().expect("OLED init failed");
    display.clear_buffer();
    display.flush().expect("OLED flush failed");

    info!("OLED initialized");

    // ----- PRG Button (GPIO0) -----
    let button = Input::new(peripherals.GPIO0, Pull::Up);

    // ----- Hardware RNG -----
    let rng = Rng::new(peripherals.RNG);
    let mut hw_rng = HardwareRng(rng);

    // ----- App State -----
    let mut state = AppState::new();

    // ----- Button State -----
    let mut button_was_pressed = false;
    let mut press_start_ms: u64 = 0;
    let mut tick_count: u64 = 0;

    info!("Entering main loop");

    // ----- Main Loop -----
    loop {
        // --- Button Handling ---
        let button_pressed = button.is_low(); // Active low

        if button_pressed && !button_was_pressed {
            // Button just pressed
            press_start_ms = tick_count;
        }

        if !button_pressed && button_was_pressed {
            // Button just released
            let hold_duration = tick_count - press_start_ms;

            if hold_duration > 10 {
                // Long press (>500ms at 50ms/tick) — cycle die
                state.cycle_die();
                info!("Switched to {}", state.current_die.name());
            } else {
                // Short press — roll!
                if state.animation.is_idle() {
                    state.animation = AnimationState::start_roll();
                    info!("Rolling {}...", state.current_die.name());
                }
            }
        }

        button_was_pressed = button_pressed;

        // --- Animation Tick ---
        let was_animating = !state.animation.is_idle();
        state.animation.tick(&mut hw_rng, state.current_die);

        if was_animating {
            if let Some(result) = state.animation.final_result() {
                state.last_result = Some(result);
                state.history.push(state.current_die, result);
                state.animation = AnimationState::Idle;
                info!("Rolled {}: {}", state.current_die.name(), result);
            }
        }

        // --- Render to OLED via mousefood ---
        display.clear_buffer();
        {
            let config = EmbeddedBackendConfig::default();
            let backend = EmbeddedBackend::new(&mut display, config);
            let mut terminal = Terminal::new(backend).expect("Terminal init failed");

            terminal
                .draw(|frame| {
                    ui::draw(frame, &state);
                })
                .expect("Draw failed");
        }
        display.flush().expect("Display flush failed");

        // --- Frame Rate ---
        // 50ms per tick = 20fps during animation, same during idle (simple)
        delay.delay_millis(50);
        tick_count += 1;
    }
}
