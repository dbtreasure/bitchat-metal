# bitchat-metal 🤘

Run Bitchat on bare metal. Your keys, your hardware, your mesh.

## What is this?

Open-source firmware for nRF52840 devices that speaks the Bitchat protocol. Run your own BLE mesh node that interoperates with the Bitchat iOS app, but with your hardware in full control.

## Features

- **BLE Mesh Communication**: Connect with Bitchat apps and other nodes
- **True Ownership**: Keys never leave your device
- **Embassy Async**: Modern Rust with async/await for embedded
- **Hardware Agnostic**: Runs on any nRF52840 board

## Hardware Support

Primary development on:
- Nordic nRF52840 DK (PCA10056)
- Adafruit Feather nRF52840 Express

Should work on any nRF52840-based board with minor pin adjustments.

## Quick Start

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add ARM target
rustup target add thumbv7em-none-eabi

# Install probe-rs for flashing/debugging
cargo install probe-rs-tools
```

### Build & Flash

```bash
# Clone the repo
git clone https://github.com/yourusername/bitchat-metal
cd bitchat-metal

# Build and flash to nRF52840 DK
cargo run --release
```

Make sure your nRF52840 DK is:
- Connected via J2 (Debug USB port)
- Power switch set to VDD

## Project Status

### Completed
- [x] M0: Board bring-up, RTT logging, LED blink

### In Progress
- [ ] M1: BLE advertising with GATT service

### Planned
- [ ] M2: Bitchat protocol implementation
- [ ] M3: Message fragmentation/reassembly
- [ ] M4: Multi-peer mesh relay
- [ ] M5: Full Bitchat app interoperability

## Architecture

Built with:
- **Embassy**: Async embedded framework
- **nrf-softdevice**: BLE stack for nRF chips
- **defmt**: Efficient embedded logging
- **heapless**: Static memory collections

## Contributing

This is an open protocol implementation. PRs welcome! Please read the PRD in `/docs` for protocol details.

## License

MIT - Because your hardware should be truly yours.

## Security

- Public chat is plaintext by design (for mesh discovery)
- DMs will use Noise protocol (coming in v1)
- Keys are generated on-device and never leave

## Links

- [Bitchat Protocol Spec](docs/protocol.md) (TBD)
- [Embassy Framework](https://embassy.dev)
- [Hardware Setup Guide](docs/hardware.md) (TBD)