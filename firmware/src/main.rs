//! Heltec D20 — Hardware True Random Dice Roller
//!
//! Runs on Heltec WiFi LoRa 32 V3.2 (ESP32-S3)
//! Drives the built-in SSD1306 128x64 OLED via I2C
//! Uses hardware TRNG for cryptographically fair dice rolls

#![no_std]
#![no_main]

extern crate alloc;
use alloc::format;

use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::delay::Delay;
use esp_hal::gpio::{Input, Level, Output, Pull};
use esp_hal::i2c::master::I2c;
use esp_hal::prelude::*;
use esp_hal::rng::Rng;
use log::info;

use embedded_graphics::mono_font::ascii::{FONT_6X10, FONT_10X20};
use embedded_graphics::mono_font::MonoTextStyleBuilder;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{
    Circle, Line, PrimitiveStyle, Rectangle, Triangle,
};
use embedded_graphics::text::{Alignment, Text};
use ssd1306::mode::BufferedGraphicsMode;
use ssd1306::prelude::*;
use ssd1306::rotation::DisplayRotation;
use ssd1306::size::DisplaySize128x64;
use ssd1306::I2CDisplayInterface;
use ssd1306::Ssd1306;

use heltec_d20_shared::animation::AnimationState;
use heltec_d20_shared::dice::{DieType, RngSource};
use heltec_d20_shared::AppState;

/// ESP32-S3 Hardware TRNG wrapper
struct HardwareRng(Rng);

impl RngSource for HardwareRng {
    fn random_u32(&mut self) -> u32 {
        self.0.random()
    }
}

// Heap allocator — 64KB
const HEAP_SIZE: usize = 64 * 1024;
static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

#[esp_hal::entry]
fn main() -> ! {
    unsafe {
        esp_alloc::HEAP.add_region(esp_alloc::HeapRegion::new(
            HEAP.as_mut_ptr(),
            HEAP_SIZE,
            esp_alloc::MemoryCapability::Internal.into(),
        ));
    }

    let peripherals = esp_hal::init(esp_hal::Config::default());
    esp_println::logger::init_logger_from_env();
    let delay = Delay::new();

    info!("⚡ Heltec D20 starting...");

    // OLED Power (Vext) — GPIO36, active LOW
    let mut _vext = Output::new(peripherals.GPIO36, Level::Low);
    delay.delay_millis(100);

    // OLED Reset — GPIO21
    let mut rst = Output::new(peripherals.GPIO21, Level::Low);
    delay.delay_millis(10);
    rst.set_high();
    delay.delay_millis(10);

    // I2C — SDA=GPIO17, SCL=GPIO18
    let i2c = I2c::new(peripherals.I2C0, {
        let mut config = esp_hal::i2c::master::Config::default();
        config.frequency = 400.kHz();
        config
    })
    .with_sda(peripherals.GPIO17)
    .with_scl(peripherals.GPIO18);

    // SSD1306 display
    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().expect("OLED init failed");
    display.clear_buffer();
    display.flush().expect("OLED flush failed");

    info!("OLED initialized");

    // PRG Button — GPIO0, active low
    let button = Input::new(peripherals.GPIO0, Pull::Up);

    // Hardware RNG
    let rng = Rng::new(peripherals.RNG);
    let mut hw_rng = HardwareRng(rng);

    // App state
    let mut state = AppState::new();
    let mut button_was_pressed = false;
    let mut press_start: u64 = 0;
    let mut tick: u64 = 0;

    info!("Entering main loop");

    loop {
        // --- Button handling ---
        let pressed = button.is_low();

        if pressed && !button_was_pressed {
            press_start = tick;
        }
        if !pressed && button_was_pressed {
            let hold = tick - press_start;
            if hold > 10 {
                // Long press — cycle die
                state.cycle_die();
                info!("Switched to {}", state.current_die.name());
            } else if state.animation.is_idle() {
                // Short press — roll
                state.animation = AnimationState::start_roll();
                info!("Rolling {}...", state.current_die.name());
            }
        }
        button_was_pressed = pressed;

        // --- Animation tick ---
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

        // --- Render ---
        display.clear_buffer();
        draw_oled(&mut display, &state);
        display.flush().expect("flush failed");

        delay.delay_millis(50);
        tick += 1;
    }
}

// ═══════════════════════════════════════════════════════
// OLED Rendering — embedded-graphics (128x64 pixels)
// ═══════════════════════════════════════════════════════

fn draw_oled<D>(display: &mut D, state: &AppState)
where
    D: DrawTarget<Color = BinaryColor>,
{
    let white = BinaryColor::On;
    let _black = BinaryColor::Off;

    let small = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(white)
        .build();

    let big = MonoTextStyleBuilder::new()
        .font(&FONT_10X20)
        .text_color(white)
        .build();

    // Header: die type name
    let header = state.current_die.name();
    let _ = Text::with_alignment(
        header,
        Point::new(64, 10),
        small,
        Alignment::Center,
    )
    .draw(display);

    // Die shape + number (centered, y=15..50)
    let display_val = match &state.animation {
        AnimationState::Idle => state.last_result,
        AnimationState::Rolling { display_value, .. } => Some(*display_value),
        AnimationState::Landed { result, .. } => Some(*result),
    };

    // Draw die shape
    draw_die_shape(display, state.current_die, white);

    // Draw number inside shape
    if let Some(val) = display_val {
        let num_str = format!("{}", val);
        let _ = Text::with_alignment(
            &num_str,
            Point::new(64, 40),
            big,
            Alignment::Center,
        )
        .draw(display);
    } else {
        let _ = Text::with_alignment(
            "?",
            Point::new(64, 40),
            big,
            Alignment::Center,
        )
        .draw(display);
    }

    // Die selector bar (bottom)
    draw_die_selector(display, state, small);

    // Roll history dots (very bottom)
    draw_history_dots(display, state);
}

fn draw_die_shape<D>(display: &mut D, die: DieType, color: BinaryColor)
where
    D: DrawTarget<Color = BinaryColor>,
{
    let style = PrimitiveStyle::with_stroke(color, 1);
    let cx = 64i32; // center x
    let cy = 32i32; // center y

    match die {
        DieType::D4 => {
            // Tetrahedron — equilateral triangle
            let _ = Triangle::new(
                Point::new(cx, cy - 18),      // top
                Point::new(cx - 22, cy + 12), // bottom-left
                Point::new(cx + 22, cy + 12), // bottom-right
            )
            .into_styled(style)
            .draw(display);
        }
        DieType::D6 => {
            // Cube — square with perspective hint
            let _ = Rectangle::new(Point::new(cx - 16, cy - 16), Size::new(32, 32))
                .into_styled(style)
                .draw(display);
            // Top face perspective lines
            let _ = Line::new(Point::new(cx - 16, cy - 16), Point::new(cx - 10, cy - 22))
                .into_styled(style)
                .draw(display);
            let _ = Line::new(Point::new(cx + 16, cy - 16), Point::new(cx + 22, cy - 22))
                .into_styled(style)
                .draw(display);
            let _ = Line::new(Point::new(cx - 10, cy - 22), Point::new(cx + 22, cy - 22))
                .into_styled(style)
                .draw(display);
            // Right face perspective
            let _ = Line::new(Point::new(cx + 16, cy + 16), Point::new(cx + 22, cy + 10))
                .into_styled(style)
                .draw(display);
            let _ = Line::new(Point::new(cx + 22, cy - 22), Point::new(cx + 22, cy + 10))
                .into_styled(style)
                .draw(display);
        }
        DieType::D8 => {
            // Octahedron — diamond (two triangles)
            let _ = Triangle::new(
                Point::new(cx, cy - 20),
                Point::new(cx - 20, cy),
                Point::new(cx + 20, cy),
            )
            .into_styled(style)
            .draw(display);
            let _ = Triangle::new(
                Point::new(cx, cy + 20),
                Point::new(cx - 20, cy),
                Point::new(cx + 20, cy),
            )
            .into_styled(style)
            .draw(display);
        }
        DieType::D10 => {
            // Kite shape — elongated diamond
            let _ = Line::new(Point::new(cx, cy - 22), Point::new(cx - 18, cy - 4))
                .into_styled(style).draw(display);
            let _ = Line::new(Point::new(cx, cy - 22), Point::new(cx + 18, cy - 4))
                .into_styled(style).draw(display);
            let _ = Line::new(Point::new(cx - 18, cy - 4), Point::new(cx, cy + 18))
                .into_styled(style).draw(display);
            let _ = Line::new(Point::new(cx + 18, cy - 4), Point::new(cx, cy + 18))
                .into_styled(style).draw(display);
            // Horizontal midline
            let _ = Line::new(Point::new(cx - 18, cy - 4), Point::new(cx + 18, cy - 4))
                .into_styled(style).draw(display);
        }
        DieType::D12 => {
            // Dodecahedron — pentagon
            // 5 vertices of a regular pentagon
            let r = 20i32;
            let pts: [(i32, i32); 5] = [
                (cx + 0,                    cy - r),
                (cx + (r * 95 / 100),       cy - (r * 31 / 100)),
                (cx + (r * 59 / 100),       cy + (r * 81 / 100)),
                (cx - (r * 59 / 100),       cy + (r * 81 / 100)),
                (cx - (r * 95 / 100),       cy - (r * 31 / 100)),
            ];
            for i in 0..5 {
                let j = (i + 1) % 5;
                let _ = Line::new(
                    Point::new(pts[i].0, pts[i].1),
                    Point::new(pts[j].0, pts[j].1),
                )
                .into_styled(style)
                .draw(display);
            }
        }
        DieType::D20 => {
            // Icosahedron — triangle with inner facets
            let _ = Triangle::new(
                Point::new(cx, cy - 22),
                Point::new(cx - 24, cy + 14),
                Point::new(cx + 24, cy + 14),
            )
            .into_styled(style)
            .draw(display);
            // Inner triangle (inverted)
            let _ = Triangle::new(
                Point::new(cx, cy + 8),
                Point::new(cx - 12, cy - 8),
                Point::new(cx + 12, cy - 8),
            )
            .into_styled(style)
            .draw(display);
        }
        DieType::D100 => {
            // Two overlapping circles (percentile dice)
            let _ = Circle::new(Point::new(cx - 18, cy - 14), 24)
                .into_styled(style)
                .draw(display);
            let _ = Circle::new(Point::new(cx + 2, cy - 10), 24)
                .into_styled(style)
                .draw(display);
        }
    }
}

fn draw_die_selector<D>(display: &mut D, state: &AppState, style: embedded_graphics::mono_font::MonoTextStyle<BinaryColor>)
where
    D: DrawTarget<Color = BinaryColor>,
{
    let names = ["d4", "d6", "d8", "d10", "d12", "d20", "%"];
    let start_x = 4i32;
    let y = 60i32;

    for (i, name) in names.iter().enumerate() {
        let x = start_x + (i as i32) * 18;
        if i == state.die_index {
            // Draw selection indicator
            let _ = Text::new(">", Point::new(x - 6, y), style).draw(display);
        }
        let _ = Text::new(name, Point::new(x, y), style).draw(display);
    }
}

fn draw_history_dots<D>(display: &mut D, state: &AppState)
where
    D: DrawTarget<Color = BinaryColor>,
{
    let dot_style = PrimitiveStyle::with_fill(BinaryColor::On);
    let max = state.current_die.max_value();

    // Draw last 8 rolls as dots at bottom — height proportional to result
    for (i, record) in state.history.recent(8).enumerate() {
        let x = 16 + (i as i32) * 12;
        let height = (record.result as i32 * 4) / max as i32;
        let _ = Rectangle::new(
            Point::new(x, 54 - height),
            Size::new(3, height as u32 + 1),
        )
        .into_styled(dot_style)
        .draw(display);
    }
}
