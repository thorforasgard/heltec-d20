# ⚡🎲 Heltec D20 — Hardware True Random Dice Roller

A beautiful polyhedral dice roller for the **Heltec WiFi LoRa 32 V3.2** (ESP32-S3) built with Rust, ratatui, and mousefood.

Uses the ESP32-S3's **hardware true random number generator (TRNG)** for cryptographically fair rolls — dice you literally can't cheat on.

## Features

- 🎲 **Full polyhedral set** — d4, d6, d8, d10, d12, d20, d100
- 🔐 **Hardware TRNG** — real entropy from ESP32-S3 silicon, not pseudorandom
- ✨ **Animated rolls** — spinning digits before landing on result
- 📊 **Roll history** — last 10 rolls with running stats
- 📻 **LoRa broadcast** — share rolls with other Heltec boards (future)
- 🖥️ **Simulator mode** — develop and test on desktop without hardware

## Hardware

- **Board:** Heltec WiFi LoRa 32 V3.2
- **MCU:** ESP32-S3 (Xtensa LX7, dual-core 240MHz)
- **Display:** 128×64 OLED (SSD1306, I2C)
- **Button:** PRG button (GPIO0) for rolling
- **LoRa:** SX1262 (future: broadcast rolls)

## Architecture

```
┌─────────────────────────────────┐
│  App Logic (dice, animation)    │
├─────────────────────────────────┤
│  ratatui (widgets, layout)      │
├─────────────────────────────────┤
│  mousefood (ratatui → e-g)      │
├─────────────────────────────────┤
│  ssd1306 (display driver)       │
├─────────────────────────────────┤
│  esp-hal (ESP32-S3 HAL)         │
└─────────────────────────────────┘
```

## Quick Start (Simulator)

Test on your desktop without hardware:

```bash
cd simulator
cargo run
```

Press `Space` to roll, `Tab` to switch dice type, `q` to quit.

## Flash to Heltec

```bash
# Install ESP toolchain
cargo install espup
espup install

# Build and flash
cd firmware
cargo espflash flash --release --monitor
```

## Controls

| Input | Action |
|-------|--------|
| PRG button (short press) | Roll current die |
| PRG button (long press) | Cycle die type |
| PRG button (double press) | Toggle roll history |

## Modes

- **Single Die** — roll one die, big number display
- **Multi Roll** — roll multiple dice (e.g., 4d6 for stats)
- **History** — view roll log with distribution chart

## Color Theme (Yggdrasil Nexus)

Monochrome on SSD1306, but the simulator renders with:
- Verdandi Green (#2dd881) on Root Dark (#0a0e14)

## Why Hardware RNG?

Most dice apps use pseudorandom number generators (PRNGs) seeded from time or entropy pools. The ESP32-S3 has a dedicated hardware random number generator that produces true random numbers from thermal noise in the silicon. This is the same quality of randomness used in cryptographic operations.

Your rolls are as fair as physics allows.

## License

MIT

## Credits

- [ratatui](https://ratatui.rs) — Terminal UI framework
- [mousefood](https://github.com/ratatui/mousefood) — embedded-graphics backend for ratatui
- [esp-hal](https://github.com/esp-rs/esp-hal) — ESP32 Hardware Abstraction Layer
- Built with ⚡ by [Asgard Security](https://www.asgardsec.com)
