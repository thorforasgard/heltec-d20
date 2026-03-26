# Heltec D20 — Build Spec

## Project Structure

```
heltec-d20/
├── README.md
├── SPEC.md
├── simulator/          # Desktop simulator (runs on any machine)
│   ├── Cargo.toml
│   └── src/
│       └── main.rs
├── firmware/           # ESP32-S3 firmware (flashes to Heltec)
│   ├── Cargo.toml
│   ├── rust-toolchain.toml
│   └── src/
│       └── main.rs
└── shared/             # Shared library (dice logic, UI, animation)
    ├── Cargo.toml
    └── src/
        ├── lib.rs
        ├── dice.rs     # Dice types, roll logic, RNG trait
        ├── ui.rs       # Ratatui widgets and layout
        ├── animation.rs # Roll animation state machine
        └── history.rs  # Roll history and stats
```

## Shared Library (no_std compatible)

### dice.rs
- `DieType` enum: D4, D6, D8, D10, D12, D20, D100
- `Roll` struct: die_type, result, timestamp_ms
- `RngSource` trait: `fn random_u32(&mut self) -> u32`
  - Simulator impl: uses `rand` crate
  - Firmware impl: uses ESP32-S3 hardware RNG register
- `roll_die(rng: &mut impl RngSource, die: DieType) -> u8`
  - Rejection sampling for uniform distribution (no modulo bias)

### ui.rs
- `draw_dashboard(frame, app_state)` — main render function
- Layout for 128x64 (21x8 chars with 6x8 font):
  ```
  ┌─ D20 ──────────────┐
  │                     │
  │        17           │  ← Big centered number (result)
  │                     │
  │  ▸d4 d6 d8 d20     │  ← Die selector (highlighted = current)
  │  Last: 17 12 3 20  │  ← Recent rolls
  └─────────────────────┘
  ```
- During animation: rapidly cycling random numbers
- After roll: big number with brief flash/highlight

### animation.rs
- `AnimationState` enum: Idle, Rolling(frame_count), Landed(result)
- Rolling phase: 15 frames of random numbers (decelerating)
- Landed phase: hold result, brief invert flash, then idle
- Frame timing: ~50ms per frame during roll (~750ms total animation)

### history.rs
- `RollHistory`: circular buffer of last 20 rolls
- Stats: min, max, average, count per die type
- Display: condensed list for the small screen

## Simulator (simulator/)

**Dependencies:**
- `shared` (path dependency)
- `ratatui` 0.29
- `mousefood` (with `simulator` feature)
- `embedded-graphics-simulator`
- `rand` (for desktop RNG)
- `crossterm` (for keyboard input in simulator window)

**Behavior:**
- Opens a window showing the 128x64 display
- Keyboard: Space=roll, Tab=cycle die, q=quit, h=history
- Renders at 20fps during animation, idle otherwise

## Firmware (firmware/)

**Dependencies:**
- `shared` (path dependency)
- `ratatui` 0.29 (no default features, no_std)
- `mousefood` (no default features, no_std)
- `ssd1306` (I2C driver)
- `esp-hal` (ESP32-S3)
- `esp-alloc` (heap allocator)

**Heltec V3.2 Pin Mapping:**
- OLED SDA: GPIO17
- OLED SCL: GPIO18
- OLED RST: GPIO21
- PRG Button: GPIO0 (active low, internal pull-up)
- Vext (OLED power): GPIO36

**Boot sequence:**
1. Init heap allocator
2. Init I2C (400kHz)
3. Power on OLED via Vext
4. Reset OLED
5. Init SSD1306 display driver
6. Init mousefood backend
7. Draw initial UI
8. Enter main loop (poll button, update animation, render)

**Button handling:**
- Debounce: 50ms
- Short press (<500ms): roll
- Long press (>500ms): cycle die type
- Double press (<300ms gap): toggle history view

**RNG:**
- Read from ESP32-S3 RNG register (0x6003_5144)
- Or use `esp_hal::rng::Rng` if available in esp-hal

## Build Notes

### Simulator
```bash
cd simulator
cargo run
```
Standard Rust toolchain, no special setup needed.

### Firmware
```bash
# One-time setup
cargo install espup espflash
espup install
source ~/export-esp.sh

# Build + flash
cd firmware
cargo espflash flash --release --monitor
```

Requires `rust-toolchain.toml` with:
```toml
[toolchain]
channel = "esp"
```

## v0.2 Ideas
- LoRa broadcast rolls to nearby boards
- Advantage/disadvantage mode (roll 2, take high/low)
- Custom dice expressions (2d6+3)
- Roll verification (hash of TRNG output for provably fair rolls)
- Sound via buzzer (if wired)
