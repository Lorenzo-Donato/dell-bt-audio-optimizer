# Bluetooth Audio Optimizer & Sentinel for Linux

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-2024%20edition-orange.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20PipeWire-lightgrey.svg)]()
[![Status](https://img.shields.io/badge/stability-production%20ready-green.svg)]()
[![Privacy](https://img.shields.io/badge/privacy-shield%20%26%20zero%20telemetry-blueviolet.svg)]()

A modular, low-overhead, privacy-first Linux audio engineering daemon and CLI tool written in **Rust**. 

Designed to cure sudden volume drops, audio dropouts, channel desynchronization (left/right earbud cutting out), and microphone profile downgrades on laptops equipped with **Wi-Fi/Bluetooth combo chips** (Realtek RTL8821CE, RTL8822CE, Intel, MediaTek) using **True Wireless Stereo (TWS) earbuds** (JBL Wave/Tune/Live series, Sony, Galaxy Buds, etc.) on **PipeWire** and **WirePlumber 0.5+**.

---

## 🎯 Target Hardware & Compatibility

> [!IMPORTANT]
> **Dynamic Hardware Discovery:** This software does not rely on hardcoded device identifiers. It dynamically inspects your Linux kernel `sysfs` tree (`/sys/class/bluetooth/`, `/sys/class/net/`, `/sys/bus/usb/devices/`) to discover your actual Bluetooth controller and Wi-Fi interface (whether named `wlan0`, `wlp2s0`, etc.), automatically tailoring UDEV and modprobe rules to your hardware.
>
> - **Tested Distributions:** Arch Linux, Fedora, Ubuntu/Debian (Kernel 6.x / 7.x, Hyprland / Wayland / X11).
> - **Tested Laptops:** Dell Inspiron, Lenovo ThinkPad/IdeaPad, HP Pavilion, Asus VivoBook.
> - **Tested Audio Devices:** JBL Wave Beam 2, JBL Tune, JBL Live, Sony WF-1000XM4/XM5, Galaxy Buds, and other TWS earbuds.

---

## 💡 The Problem Explained

If you use Bluetooth earbuds on a Linux laptop with a combo Wi-Fi/BT card sharing a single physical antenna, you may encounter:
1. **Abrupt Volume & Gain Drops:** The volume suddenly drops by 20–40 dB or jumps erratically between the left and right earbuds.
   - *Cause:* Earbud firmware desync when negotiating AVRCP Absolute Hardware Volume (`bluez5.enable-hw-volume`) with PipeWire.
2. **Audio Stutters & TWS Desync:** The audio chokes intermittently, alternating between ears.
   - *Cause:* Aggressive USB autosuspend (`enable_autosuspend=Y` in `btusb`) putting the Bluetooth radio into micro-sleeps, combined with high-bitrate codecs (like SBC-XQ ~500kbps) congesting the shared antenna.
3. **Unexpected Mono Headset Degrade:** Sound quality plummets to 8 kHz telephone quality whenever a browser, Steam, or Discord queries recording inputs.
   - *Cause:* WirePlumber's automatic headset autoswitch forcing HFP/HSP mono duplex routing even when a dedicated USB microphone is available.
4. **xHCI Deadlocks / System Freezes:** Resetting the USB controller (`reset=1` in `btusb.conf`) on Intel Alder Lake and newer chipsets causes bus deadlocks that hard freeze the operating system.

---

## 🛡️ Privacy, Security & Zero-Telemetry Guarantee

- **Zero Network Telemetry:** The tool operates 100% offline. No outbound connections, analytics, or phone-home requests are made.
- **Privacy Shield (`--anonymize` / `-a`):** Obscures unique physical hardware identifiers (masks device MAC addresses to retain only the IEEE OUI vendor prefix and masks Wi-Fi SSIDs) so terminal logs and screenshots can be shared safely on GitHub Issues or forums.
- **Strict POSIX Permissions (`0600`):** All local diagnostic reports and state snapshots are created with permissions `0600` (read/write only by file owner).
- **Ephemeral Snapshots & Garbage Collector:** Stored reports are capped to a maximum of 5 recent snapshots and automatically pruned after 7 days.
- **Zero-Footprint Wipe (`bt-dell-audio clean`):** Instantly purges all local state and cached reports with a single command.

---

## 🛠️ Architectural Highlights & Design Patterns

The project follows clean architecture principles with strict separation of concerns, high cohesion, low coupling, and zero magic numbers:

- **Dynamic Hardware Discovery:** Replaces static compile-time constants with dynamic runtime inspection of sysfs USB classes (`e0:01:01`), network devices, and PCI drivers.
- **Specification / Strategy Pattern:** The diagnostic engine (`diagnostics/rule.rs`) and health auditor (`service/health.rs`) evaluate isolated, atomic rules (`HealthCheck` trait), strictly adhering to SOLID's Open/Closed Principle.
- **Hexagonal Ports & Adapters:** Hardware probes (`sysfs`, `procfs`, `pactl`, `nmcli`) are cleanly decoupled from domain logic and UI presentation layers.
- **Embedded Templates:** Long configuration templates (`wireplumber`, `modprobe`, `udev`, `systemd`) are compiled directly into the binary using `include_str!()` with runtime placeholder substitution (`{{VENDOR}}`, `{{PRODUCT}}`).
- **Airbags & Circuit Breakers:** Built-in fault tolerance preventing infinite loops or bus-hammering. If operations fail repeatedly, the sentinel trips open into safe mode cooldown to prevent kernel instability.
- **Doctor Command:** Comprehensive stability and kernel lockup audit (`bt-dell-audio doctor`) catching dangerous controller reset loops (`reset=1`), high memory pressure, or runaway CPU load.
- **Safe Rollback System:** Full built-in recovery mechanism (`bt-dell-audio rollback` and `scripts/rollback.sh`) to restore factory configurations cleanly at any time.

```
├── bin/
│   └── bt-dell-audio                  # Standalone compiled release binary
├── bt-dell-audio/                     # Rust source code
│   ├── templates/                     # Extracted configuration templates
│   ├── src/
│   │   ├── config/                    # Centralized constants and compiled templates
│   │   ├── domain/                    # Typed domain models, errors, sanitizer, and enums
│   │   ├── hardware/                  # Sysfs, PCI, USB, OS, and Wi-Fi dynamic adapters
│   │   ├── audio/                     # PipeWire and WirePlumber 0.5 managers
│   │   ├── diagnostics/               # Diagnostic engine and atomic rules
│   │   ├── service/                   # Fixer, Health Doctor, History/GC, Sentinel, Rollback
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
git clone https://github.com/Lorenzo-Donato/dell-bt-audio-optimizer.git
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

# Or with Privacy Shield active (masks SSIDs and MAC addresses):
./bin/bt-dell-audio checkup --anonymize

# Or save an ephemeral snapshot to compare later:
./bin/bt-dell-audio checkup --save --anonymize
```

### 2. System Health, Stability & Airbag Doctor
Audits kernel safety parameters (such as dangerous `btusb.reset=1` locks on Intel Alder Lake xHCI controllers), RAM pressure, CPU load, and audio server daemon health:
```bash
./bin/bt-dell-audio doctor
```

### 3. Apply Audio Stabilization (User-space, No Root Needed)
Deploys optimal WirePlumber drop-ins, disables AVRCP hardware volume bugs, and locks output to clean stereo A2DP:
```bash
./bin/bt-dell-audio fix
```
*(Or use `./scripts/quick-fix.sh` directly).*

### 4. Apply Kernel & UDEV Rules (Root, Required Once)
Permanently disables USB autosuspend on the dynamically detected Bluetooth radio and prevents Wi-Fi link power-saving collision:
```bash
sudo ./scripts/apply-system-tuning.sh
```

### 5. Interactive Live Watcher
Monitors battery, active codec, PipeWire node health, and USB power control in real time:
```bash
./bin/bt-dell-audio watch
```

### 6. View Diagnostic History & Comparisons
Lists stored diagnostic snapshots:
```bash
./bin/bt-dell-audio history
```

### 7. Zero-Footprint Data Wipe (Clean)
Immediately removes all historical reports and logs from disk:
```bash
./bin/bt-dell-audio clean
```

### 8. Safe Rollback (Undo All Changes)
If you ever want to revert your audio setup back to factory defaults:
```bash
# Revert user-level WirePlumber configuration
./bin/bt-dell-audio rollback

# Revert system-level Kernel/UDEV rules (if applied)
sudo ./scripts/rollback.sh
```

### 9. Background Sentinel Daemon & Systemd Service
To run the auto-healing sentinel automatically on login:
```bash
./bin/bt-dell-audio install-service
```
Or run the daemon in foreground mode:
```bash
./bin/bt-dell-audio daemon
```

---

## 🧪 Testing & Code Quality

The codebase enforces strict compiler checks, zero warnings, and unit/contract tests:

```bash
cd bt-dell-audio

# Run unit and contract tests (19 automated tests)
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
