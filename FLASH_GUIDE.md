# Flashing Heltec D20 to Your Board

Step-by-step guide to get the dice roller running on a Heltec WiFi LoRa 32 V3.2.

## Prerequisites

- Heltec WiFi LoRa 32 V3.2 board
- USB-C cable
- A computer (Mac, Linux, or Windows)
- ~15 minutes

## Step 1: Install Rust (if you don't have it)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

## Step 2: Install ESP Rust Toolchain

The ESP32-S3 uses the Xtensa architecture, which needs a special Rust fork:

```bash
cargo install espup
espup install
```

This downloads the ESP Rust toolchain (~500MB). When it finishes, source the env:

```bash
# On Mac/Linux:
source ~/export-esp.sh

# Add to your shell profile so it loads automatically:
echo 'source ~/export-esp.sh' >> ~/.bashrc   # or ~/.zshrc
```

## Step 3: Install espflash

```bash
cargo install espflash
```

This is the tool that flashes firmware to your ESP32.

## Step 4: Clone the Repo

```bash
git clone https://github.com/thorforasgard/heltec-d20.git
cd heltec-d20
```

## Step 5: Plug In the Board

1. Connect the Heltec V3.2 to your computer via USB-C
2. Verify it shows up:

```bash
# Mac:
ls /dev/cu.usbserial-* /dev/cu.wchusbserial-*

# Linux:
ls /dev/ttyUSB* /dev/ttyACM*
```

You should see a serial device. If not:
- **Mac**: You may need the CH343 driver: https://www.wch-ic.com/downloads/CH343SER_MAC_ZIP.html
- **Linux**: Usually works out of the box. If not: `sudo apt install linux-modules-extra-$(uname -r)`
- **Windows**: Install CH343 driver from WCH website

## Step 6: Build & Flash

```bash
cd firmware
cargo run --release
```

This will:
1. Compile the firmware for ESP32-S3 (~1-2 minutes first time)
2. Auto-detect your board on USB
3. Flash the firmware
4. Open a serial monitor so you can see logs

If `cargo run` doesn't auto-detect the port, specify it:
```bash
espflash flash --release --monitor --port /dev/ttyUSB0 target/xtensa-esp32s3-none-elf/release/heltec-d20-firmware
```

## Step 7: Use It!

Once flashed, the OLED should light up with the dice roller UI:

```
┌─ D20 ──────────────┐
│                     │
│       ROLL!         │
│                     │
│ ▸d4 d6 d8 d20      │
│ Press SPACE to roll │
└─────────────────────┘
```

**Controls (PRG button on the board):**
- **Short press** → Roll the current die
- **Long press (>0.5s)** → Cycle to next die type (d4→d6→d8→...→d100→d4)

The number will spin with a quick animation, then land on the result.

## Troubleshooting

### "Permission denied" on serial port (Linux)
```bash
sudo usermod -a -G dialout $USER
# Log out and back in
```

### Board not detected
- Try a different USB-C cable (some are charge-only, no data)
- Try a different USB port
- Hold the board's BOOT button while pressing RST to force bootloader mode

### Compilation errors with esp-hal versions
The ESP Rust ecosystem moves fast. If you get dependency conflicts:
```bash
cd firmware
cargo update
```

If that doesn't help, check https://github.com/esp-rs/esp-hal for the latest compatible versions and update `Cargo.toml`.

### OLED doesn't turn on
- The Vext pin (GPIO36) must be driven LOW to power the OLED
- The reset pin (GPIO21) must be pulsed low then high
- Both are handled in the firmware, but if you're seeing nothing, check your board revision

### "espflash: No serial ports detected"
The Heltec V3.2 uses a CH343 USB-UART chip. Install the driver:
- **Mac**: https://www.wch-ic.com/downloads/CH343SER_MAC_ZIP.html
- **Windows**: https://www.wch-ic.com/downloads/CH343SER_ZIP.html

## Development: Test Without Hardware

Run the simulator on any computer (no board needed):

```bash
cd simulator
cargo run
```

Controls: `Space`=roll, `Tab`=cycle die, `h`=history, `q`=quit

## What's Next

- **v0.2**: LoRa broadcast — share rolls with other boards over mesh
- **v0.2**: Advantage/disadvantage mode (D&D 5e)
- **v0.2**: Custom roll expressions (2d6+3)
