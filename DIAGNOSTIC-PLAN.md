# Heltec D20 — Diagnostic Plan

**Goal:** Find why the ESP-IDF bootloader reads garbage at the app descriptor offset

## Step 1: Verify toolchain with a minimal project

Generate a known-working template project and flash it to prove the toolchain + board work:

```bash
# Install if needed
cargo install esp-generate --locked

# Generate minimal ESP32-S3 project with alloc + log
esp-generate --chip esp32s3 -o alloc -o log --headless test-flash
cd test-flash

# Build and flash
cargo run --release
```

**If this works:** The toolchain and board are fine. Issue is in our project specifically.
**If this fails the same way:** Toolchain/espflash version issue.

## Step 2: Inspect the ELF sections

From the `firmware/` directory, after a successful build:

```bash
cd ~/path/to/heltec-d20/firmware

# Build without flashing
cargo build --release

# Find the ELF
ELF="target/xtensa-esp32s3-none-elf/release/heltec-d20-firmware"

# Check if .rodata_desc section exists
xtensa-esp32s3-elf-readelf -S "$ELF" | grep -i rodata

# Full section listing (look for rodata_desc and its offset)
xtensa-esp32s3-elf-readelf -S "$ELF"

# Dump the app descriptor section specifically
xtensa-esp32s3-elf-objdump -s -j .rodata_desc "$ELF"

# Also check with espflash's image generation
espflash save-image --chip esp32s3 "$ELF" test.bin
# (If this fails, try with --ignore-app-descriptor and note the error)
```

**What to look for:**
- Does `.rodata_desc` section exist?
- What offset is it at?
- Does `espflash save-image` find the descriptor?

## Step 3: Compare ELF layouts

If Step 1 produced a working binary, compare the section layouts:

```bash
# Working template
xtensa-esp32s3-elf-readelf -S test-flash/target/xtensa-esp32s3-none-elf/release/test-flash > working.txt

# Our project
xtensa-esp32s3-elf-readelf -S firmware/target/xtensa-esp32s3-none-elf/release/heltec-d20-firmware > ours.txt

diff working.txt ours.txt
```

**Key difference to find:** Is `.rodata_desc` at the same relative offset in both?

## Step 4: Try building without shared library

Create a stripped-down main.rs that doesn't import `heltec_d20_shared`. Just blink or print:

```rust
#![no_std]
#![no_main]

use esp_hal::clock::CpuClock;
use esp_hal::main;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! { loop {} }

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    esp_println::logger::init_logger_from_env();
    log::info!("Minimal test!");
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let _peripherals = esp_hal::init(config);
    loop {
        log::info!("tick");
        esp_hal::delay::Delay::new().delay_millis(1000);
    }
}
```

Remove `heltec-d20-shared`, `ssd1306`, `embedded-graphics`, `ratatui`, `mousefood` from deps.

**If this works:** One of our dependencies is pulling in something that interferes with section layout.
**If this fails:** Issue is in base config.

## Step 5: Try without LTO

Edit `Cargo.toml`:
```toml
[profile.release]
codegen-units = 1
opt-level = "s"
# lto = "fat"   # COMMENT THIS OUT
```

LTO can reorder sections. The template doesn't use `lto = "fat"` in release.

## Step 6: Try debug build (not release)

```bash
cargo run   # (not --release)
```

Debug builds don't apply LTO. If debug works but release doesn't, it's an LTO section ordering bug.

## Expected Outcome

Most likely culprit (based on research): **LTO is reordering the `.rodata_desc` section**, placing it after other rodata, which shifts the app descriptor away from where the bootloader expects it. The official template doesn't use `lto = "fat"`.

Second most likely: One of our larger dependencies (ratatui, mousefood) is adding rodata that gets placed before the descriptor section.
