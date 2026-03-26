//! Heltec D20 — Hardware True Random Dice Roller
//!
//! Runs on Heltec WiFi LoRa 32 V3.2 (ESP32-S3)
//! Drives the built-in SSD1306 128x64 OLED via I2C
//! Uses hardware TRNG for cryptographically fair dice rolls
//! Renders via ratatui + mousefood (same UI as simulator!)

#![no_std]
#![no_main]

extern crate alloc;

use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::gpio::{Input, InputConfig, Level, Output, OutputConfig, Pull};
use esp_hal::i2c::master::{Config as I2cConfig, I2c};
use esp_hal::main;
use esp_hal::rng::Rng;
use esp_hal::time::Rate;

use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use ssd1306::mode::BufferedGraphicsMode;
use ssd1306::prelude::*;
use ssd1306::rotation::DisplayRotation;
use ssd1306::size::DisplaySize128x64;
use ssd1306::I2CDisplayInterface;
use ssd1306::Ssd1306;

use mousefood::prelude::*;
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

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    loop {
        log::error!("{info:?}");
    }
}

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    esp_println::logger::init_logger_from_env();
    log::info!("⚡ Heltec D20 starting...");

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(size: 64 * 1024);

    let delay = Delay::new();

    // ----- OLED Power (Vext) -----
    // Heltec V3.2: GPIO36 controls OLED power (active LOW)
    let _vext = Output::new(peripherals.GPIO36, Level::Low, OutputConfig::default());
    delay.delay_millis(100);

    // ----- OLED Reset -----
    // GPIO21 is the OLED reset pin
    let mut rst = Output::new(peripherals.GPIO21, Level::Low, OutputConfig::default());
    delay.delay_millis(10);
    rst.set_high();
    delay.delay_millis(10);

    // ----- I2C for OLED -----
    // Heltec V3.2: SDA=GPIO17, SCL=GPIO18
    let i2c_config = I2cConfig::default().with_frequency(Rate::from_khz(400));
    let i2c = I2c::new(peripherals.I2C0, i2c_config)
        .unwrap()
        .with_sda(peripherals.GPIO17)
        .with_scl(peripherals.GPIO18);

    // ----- SSD1306 Display -----
    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().expect("OLED init failed");
    display.clear_buffer();
    display.flush().expect("OLED flush failed");

    log::info!("OLED initialized");

    // ----- PRG Button (GPIO0) -----
    let button = Input::new(
        peripherals.GPIO0,
        InputConfig::default().with_pull(Pull::Up),
    );

    // ----- Hardware RNG -----
    let rng = Rng::new(peripherals.RNG);
    let mut hw_rng = HardwareRng(rng);

    // ----- Mousefood + Ratatui Terminal -----
    let backend = EmbeddedBackend::new(&mut display, EmbeddedBackendConfig::default());
    let mut terminal = Terminal::new(backend).expect("Terminal init failed");

    // ----- App State -----
    let mut state = AppState::new();
    let mut button_was_pressed = false;
    let mut press_start: u64 = 0;
    let mut tick: u64 = 0;

    log::info!("Entering main loop — press PRG to roll!");

    // ----- Main Loop -----
    loop {
        // --- Button Handling ---
        let pressed = button.is_low(); // Active low

        if pressed && !button_was_pressed {
            press_start = tick;
        }

        if !pressed && button_was_pressed {
            let hold = tick - press_start;
            if hold > 10 {
                // Long press (>500ms) — cycle die type
                state.cycle_die();
                log::info!("Switched to {}", state.current_die.name());
            } else if state.animation.is_idle() {
                // Short press — roll!
                state.animation = AnimationState::start_roll();
                log::info!("Rolling {}...", state.current_die.name());
            }
        }
        button_was_pressed = pressed;

        // --- Animation Tick ---
        let was_animating = !state.animation.is_idle();
        state.animation.tick(&mut hw_rng, state.current_die);

        if was_animating {
            if let Some(result) = state.animation.final_result() {
                state.last_result = Some(result);
                state.history.push(state.current_die, result);
                state.animation = AnimationState::Idle;
                log::info!("Rolled {}: {}", state.current_die.name(), result);
            }
        }

        // --- Render via ratatui + mousefood ---
        terminal
            .draw(|frame| {
                ui::draw(frame, &state);
            })
            .expect("Draw failed");

        // --- Frame Rate ---
        delay.delay_millis(50); // 20fps
        tick += 1;
    }
}
