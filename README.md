# MX Mini Battery Manager

A Rust-based daemon that manages battery charging for Logitech MX Mini devices to extend battery lifecycle by preventing overcharging.

## Features

- **Smart Charging Control**: Automatically disables USB charging when battery > 80%, enables when < 80%
- **USB Device Detection**: Finds devices by vendor/product ID via sysfs
- **HID Communication**: Reads battery levels using HID++ protocol
- **Systemd Integration**: Runs as a systemd service with timer (every minute)
- **Structured Logging**: Logs to systemd journal with structured format
- **Configurable Thresholds**: Customize battery thresholds via JSON config

## Architecture

```
Hardware Layer    → USB/HID device communication via sysfs and hidapi
Business Layer    → Battery management logic and decision making  
Configuration     → JSON-based configuration with sensible defaults
Integration Layer → Systemd service with timer-based execution
```

## Prerequisites

- Linux system with systemd
- Root access (required for USB power control)
- Rust toolchain (for building)
- Logitech MX Mini device

## Installation

1. **Clone and build:**
   ```bash
   git clone <repository-url>
   cd mx-mini-battery-manager
   chmod +x install.sh
   ./install.sh
   ```

2. **Verify installation:**
   ```bash
   sudo systemctl status mx-mini-battery-manager.timer
   journalctl -u mx-mini-battery-manager -f
   ```

## Configuration

Edit `/etc/mx-mini-battery-manager/config.json`:

```json
{
  "device": {
    "vendor_id": 1133,
    "product_id": 45091,
    "name": "Logitech MX Mini"
  },
  "thresholds": {
    "high_threshold": 80,
    "low_threshold": 20
  },
  "logging": {
    "level": "info",
    "use_journal": true
  }
}
```

**Finding your device IDs:**
```bash
lsusb | grep -i logitech
# Look for your specific device and note vendor:product IDs
```

## Usage

**View logs:**
```bash
journalctl -u mx-mini-battery-manager -f
```

**Check timer status:**
```bash
sudo systemctl status mx-mini-battery-manager.timer
```

**Manual run (for testing):**
```bash
sudo /usr/local/bin/mx-mini-battery-manager
```

**Stop/start service:**
```bash
sudo systemctl stop mx-mini-battery-manager.timer
sudo systemctl start mx-mini-battery-manager.timer
```

## Log Format

The service logs structured entries:
- `is_connected_via_usb=true, battery_level=85, action_done=charging_disabled`
- `is_connected_via_usb=true, battery_level=75, action_done=charging_enabled`
- `device not found via USB

