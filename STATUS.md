# Heltec D20 — Project Status

**Last Updated:** 2026-03-26
**Status:** PAUSED — blocked on esp-hal toolchain issue

## What Works
- Simulator builds and runs perfectly on desktop (`cd simulator && cargo run`)
- Firmware compiles clean for ESP32-S3 (xtensa target)
- Code logic is solid: OLED via ratatui+mousefood, hardware TRNG, button handling, animation

## What's Broken: Flash/Boot Failure

### The Problem
The firmware flashes to the Heltec V3.2 board but the **ESP-IDF 2nd stage bootloader rejects the image** with garbage efuse revision values:

```
E boot_comm: Image requires efuse blk rev >= v206.44, but chip is v1.3
E boot_comm: Image requires efuse blk rev <= v0.64, but chip is v1.3
E boot: Factory app partition is not bootable
```

### Root Cause Analysis
- `esp_bootloader_esp_idf::esp_app_desc!()` macro IS in the code
- The macro places the descriptor in `.rodata_desc.appdesc` section
- esp-hal's linker script has `KEEP(*(.rodata_desc.*))` in `.rodata_desc` section — names match
- **espflash 4.3.0 cannot find the app descriptor in the ELF** (even `save-image` fails without `--ignore-app-descriptor`)
- Using `--ignore-app-descriptor` bypasses espflash's check, but the on-chip bootloader still reads garbage at the descriptor offset
- The garbage values change between builds, confirming the bootloader is reading whatever data happens to land at that offset — not the actual descriptor

### What We Tried
1. ✅ BOOT+RST dance — fixed initial "Error while connecting to device"
2. ❌ Removing `lto = "fat"` from release profile — same garbage values
3. ❌ Using git main of `esp-bootloader-esp-idf` — `esp-rom-sys` link conflict with crates.io `esp-hal` 1.0.0
4. ❌ `--ignore-app-descriptor` flag — flashes but bootloader rejects

### Likely Issue
Version mismatch or linker script bug between:
- `esp-hal` 1.0.0 (crates.io)
- `esp-bootloader-esp-idf` 0.4.0 (crates.io)  
- `espflash` 4.3.0

The `.rodata_desc` section may not be placed at the offset where the ESP-IDF bootloader expects the app descriptor. This is an upstream ecosystem issue.

### Next Steps to Try (When Resuming)
1. **Inspect the ELF sections** — `xtensa-esp32s3-elf-readelf -S` to see if `.rodata_desc` section exists and where
2. **Try esp-generate template** — `esp-generate --chip esp32s3` to create a known-working minimal project, then compare its output/config
3. **Try downgrading espflash** to 3.x — the 4.x series may have changed image generation
4. **Check esp-hal issue tracker** more carefully for this specific combination
5. **Try esp-idf (not esp-hal)** as alternative — more mature toolchain, less bleeding edge

### Environment
- **Build machine:** Mac (Apple Silicon)
- **Board:** Heltec WiFi LoRa 32 V3.2 (ESP32-S3, chip rev v0.2, efuse blk rev v1.3)
- **Serial:** `/dev/cu.usbserial-0001` (CH343 USB-UART)
- **Toolchain:** espup (Xtensa Rust fork), espflash 4.3.0
- **Repo:** github.com/thorforasgard/heltec-d20

### .cargo/config.toml
```toml
[target.xtensa-esp32s3-none-elf]
runner = "espflash flash --monitor --chip esp32s3"

[env]
ESP_LOG = "info"

[build]
target = "xtensa-esp32s3-none-elf"
rustflags = ["-C", "link-arg=-nostartfiles"]

[unstable]
build-std = ["alloc", "core"]
```
