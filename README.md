# Bluetooth Audio Optimizer & Sentinel for Linux

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-2024%20edition-orange.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20PipeWire-lightgrey.svg)]()
[![Status](https://img.shields.io/badge/stability-production%20ready-green.svg)]()

A modular, low-overhead Linux audio engineering daemon and CLI tool written in **Rust**. 

Designed to cure sudden volume drops, audio dropouts, channel desynchronization (left/right earbud cutting out), and microphone profile downgrades on laptops equipped with **Realtek Wi-Fi/Bluetooth combo chips** (such as the RTL8821CE, RTL8822CE, RTL8723DE) using **True Wireless Stereo (TWS) earbuds** (such as JBL Wave/Tune/Live series, Sony, Galaxy Buds, etc.) on **PipeWire** and **WirePlumber 0.5+**.

---

## 🎯 Target Hardware & Compatibility

> [!IMPORTANT]
> **Scope Notice:** This software is specifically engineered for **Linux** environments running **PipeWire** and **WirePlumber 0.5+**. It targets machines with hybrid Wi-Fi/Bluetooth combo cards sharing a single physical antenna (1x1 SISO).
>
> - **Tested Distribution:** Arch Linux (Kernel 6.x / 7.x, Hyprland / Wayland / X11).
> - **Tested Laptops:** Dell, Lenovo, HP, Asus featuring Realtek PCIe Wi-Fi (`rtw88_8821ce`, `rtw88_8822ce`) and USB Bluetooth (`0bda:c829`).
> - **Tested Audio Devices:** JBL Wave Beam 2, JBL Tune, JBL Live, and other modern TWS earphones.

---

## 💡 The Problem Explained

If you use Bluetooth earbuds on a Linux laptop with a Realtek combo card, you may encounter:
1. **Abrupt Volume & Gain Drops:** The volume suddenly drops by 20–40 dB or jumps erratically between the left and right earbuds.
   - *Cause:* A bug in the earbud firmware when synchronizing AVRCP Absolute Hardware Volume (`bluez5.enable-hw-volume`) with Linux BlueZ/PipeWire.
2. **Audio Stutters & TWS Desync:** The audio chokes intermittently, alternating between ears.
   - *Cause:* Aggressive USB autosuspend (`enable_autosuspend=Y` in `btusb`) putting the Bluetooth radio into micro-sleeps, combined with high-bitrate codecs (like SBC-XQ ~500kbps) congesting the shared 1x1 antenna.
3. **Unexpected Mono Headset Degrade:** Sound quality plummets to 8 kHz telephone quality whenever a browser, Steam, or Discord queries recording inputs.
   - *Cause:* WirePlumber's automatic headset autoswitch forcing HFP/HSP mono duplex routing even when a dedicated USB microphone is available.

---

## 🛠️ Architectural Highlights & Design Patterns

The project follows clean architecture principles with strict separation of concerns, high cohesion, low coupling, and zero magic numbers:

- **Specification / Strategy Pattern:** The diagnostic engine (`diagnostics/rule.rs`) evaluates isolated, atomic rules (`diagnostics/rules/*`), making it easy to contribute new hardware checks without altering the core engine.
- **Hexagonal Ports & Adapters:** Hardware probes (`sysfs`, `procfs`, `pactl`, `nmcli`) are cleanly decoupled from the domain and UI layers.
- **Embedded Templates:** Long configuration templates (`wireplumber`, `modprobe`, `udev`) are compiled directly into the binary using `include_str!()`, keeping the Rust source clean and avoiding runtime asset lookup failures.
- **Safe Rollback System:** Full built-in recovery mechanism (`bt-dell-audio rollback` and `scripts/rollback.sh`) to restore factory configurations cleanly at any time.
- **Airbags & Circuit Breakers:** Built-in fault tolerance preventing infinite loops or bus-hammering. If failures repeat 3 times in 60s, the sentinel trips open into safe mode cooldown.
- **Doctor Command:** Comprehensive stability and kernel lockup audit (`bt-dell-audio doctor`) catching dangerous controller reset loops.
- **Defensive CLI:** Interactive confirmation prompts (`[s/N]`), with `--yes` (`-y`) for scripting and `--dry-run` for non-destructive inspection.

```
├── bin/
│   └── bt-dell-audio                  # Standalone compiled release binary
├── bt-dell-audio/                     # Rust source code
│   ├── templates/                     # Extracted configuration templates
│   ├── src/
│   │   ├── config/                    # Centralized constants and compiled templates
│   │   ├── domain/                    # Typed domain models, errors, and enums
│   │   ├── hardware/                  # Sysfs, PCI, USB, and Wi-Fi adapters
│   │   ├── audio/                     # PipeWire and WirePlumber 0.5 managers
│   │   ├── diagnostics/               # Diagnostic engine and atomic rules
│   │   ├── service/                   # Fixer, Sentinel daemon, and Rollback
│   │   └── ui/                        # Console theme, report formatting, and prompts
│   └── tests/                         # Automated integration and contract test suite
├── scripts/
│   ├── quick-fix.sh                   # Pure POSIX/Bash user-space fix (set -euo pipefail)
│   ├── apply-system-tuning.sh         # Idempotent root script for Kernel/UDEV tuning
│   ├── rollback.sh                    # Reverts Kernel/UDEV settings to factory state
│   └── status.sh                      # Non-invasive hardware telemetry reader
└── systemd/
    └── bt-dell-audio.service          # User-level background daemon unit
```

---

## 🚀 Installation & Quick Start

### 1. Prerequisites
Ensure you have PipeWire and WirePlumber installed on your Linux distribution:
```bash
# Arch Linux
sudo pacman -S pipewire wireplumber bluez bluez-utils
```

### 2. Build from Source
```bash
# Clone the repository
git clone https://github.com/<your-username>/dell-bt-audio-optimizer.git
cd dell-bt-audio-optimizer

# Build with Cargo
cargo build --release --manifest-path bt-dell-audio/Cargo.toml

# Copy binary to local bin directory
mkdir -p bin
cp bt-dell-audio/target/release/bt-dell-audio bin/
```

---

## 📖 Usage Guide

### 1. Perform a Full System Diagnostic
Scans PCI, USB, Wi-Fi frequency band, active codecs, and WirePlumber properties:
```bash
./bin/bt-dell-audio checkup
```

### 2. System Health, Stability & Airbag Doctor
Audits kernel safety parameters (such as dangerous `btusb.reset=1` locks on Intel Alder Lake xHCI controllers), RAM pressure, CPU load, and audio server daemon health:
```bash
./bin/bt-dell-audio doctor
```

### 2. Apply Audio Stabilization (User-space, No Root Needed)
Deploys optimal WirePlumber drop-ins, disables AVRCP hardware volume bugs, and locks output to clean stereo A2DP:
```bash
./bin/bt-dell-audio fix
```
*(Or use `./scripts/quick-fix.sh` directly).*

### 3. Apply Kernel & UDEV Rules (Root, Required Once)
Permanently disables USB autosuspend on Realtek Bluetooth radios and prevents Wi-Fi link power-saving collision:
```bash
sudo ./scripts/apply-system-tuning.sh
```

### 4. Interactive Live Watcher
Monitors battery, active codec, PipeWire node health, and USB power control in real time:
```bash
./bin/bt-dell-audio watch
```

### 5. Safe Rollback (Undo All Changes)
If you ever want to revert your audio setup back to factory defaults:
```bash
# Revert user-level WirePlumber configuration
./bin/bt-dell-audio rollback

# Revert system-level Kernel/UDEV rules (if applied)
sudo ./scripts/rollback.sh
```

### 6. Background Sentinel (Optional)
To run the auto-healing sentinel automatically on login:
```bash
./bin/bt-dell-audio install-service
```

---

## 🧪 Testing & Code Quality

The codebase enforces strict compiler checks, zero warnings, and unit/contract tests:

```bash
cd bt-dell-audio

# Run unit and contract tests
cargo test

# Run linter with strict warning enforcement
cargo clippy -- -D warnings
```

---

## 🤝 Contributing

Contributions from the open-source community are warmly welcome!
1. Fork the repository.
2. Create your feature branch (`git checkout -b feature/support-new-card`).
3. Commit your changes (`git commit -m 'feat: Add support for Intel AX210 coex rules'`).
4. Ensure all tests pass (`cargo test` and `cargo clippy -- -D warnings`).
5. Push to the branch and open a Pull Request.

---

## 📄 License

This project is open-source software licensed under the [MIT License](LICENSE).
