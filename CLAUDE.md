# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

### Build and Flash
```bash
# Build and flash to connected nRF52840 board
cargo run --release

# Build only
cargo build --release

# Monitor RTT logs (in separate terminal)
probe-rs attach --chip nRF52840_xxAA
```

### Development
```bash
# Check compilation without flashing
cargo check

# Format code
cargo fmt

# Run clippy lints
cargo clippy
```

## Architecture Overview

This is an embedded Rust firmware for nRF52840 microcontrollers implementing the Bitchat protocol for BLE mesh networking.

### Key Technical Stack
- **Embassy**: Async/await runtime for embedded systems - all tasks should use async patterns
- **nrf-softdevice**: Nordic's BLE stack wrapper (S140) - will be integrated for BLE functionality
- **defmt**: Logging framework - use `defmt::info!()`, `defmt::warn!()`, etc. for debug output
- **probe-rs**: Flashing and debugging - configured to auto-flash on `cargo run`

### Memory Layout
- Flash: 1MB starting at 0x00000000
- RAM: 256KB starting at 0x20000000
- When integrating softdevice (M1), expect ~112KB flash and ~8KB RAM reserved

### Hardware Configuration
- Primary LED: P0.13 (LED1 on nRF52840 DK)
- Target: thumbv7em-none-eabi (Cortex-M4F)
- Debug probe: J-Link compatible via probe-rs

### Project Milestones
Currently implementing M1 (BLE Setup) as defined in M1_BLE_SETUP.md:
- Integrate nrf-softdevice for BLE support
- Implement GATT service with RX/TX characteristics
- Enable phone ↔ device communication

### Code Organization Pattern
When implementing BLE (M1 and beyond):
```
src/
├── main.rs           # Embassy executor entry, task spawning
├── ble/
│   ├── mod.rs        # BLE module exports
│   ├── service.rs    # GATT service definitions
│   └── advertise.rs  # Advertisement configuration
└── config.rs         # UUIDs, constants
```

### Critical Constraints
- No heap allocation (use `heapless` collections)
- All async tasks must be spawned from main with `#[embassy_executor::task]`
- Softdevice requires specific memory regions - update memory.x when enabling BLE
- Use `Write Without Response` for RX characteristic, `Notify` for TX characteristic