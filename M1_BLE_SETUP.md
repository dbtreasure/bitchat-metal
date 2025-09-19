# Milestone 1: BLE Advertising & GATT Service

## Objective

Set up BLE advertising and implement a basic GATT service that can be discovered by the Bitchat iOS app.

## Success Criteria

- [ ] Device advertises with a custom UUID that Bitchat app can discover
- [ ] GATT service implemented with RX/TX characteristics
- [ ] Can receive data from phone via Write Without Response
- [ ] Can send notifications to phone
- [ ] Visible in nRF Connect app for testing

## Implementation Plan

### Phase 1: Basic BLE Setup
1. Integrate nrf-softdevice
2. Configure S140 softdevice for nRF52840
3. Set up advertising with custom name "bitchat-metal"

### Phase 2: GATT Service Structure
```
Service: Bitchat Service (UUID: TBD from upstream)
├── RX Characteristic (Write Without Response)
│   └── Phone → Device messages
└── TX Characteristic (Notify)
    └── Device → Phone messages
```

### Phase 3: Echo Test
1. Receive data on RX characteristic
2. Log received bytes via RTT
3. Echo same data back via TX notify
4. Verify round-trip with nRF Connect

### Phase 4: MTU Negotiation
1. Request MTU of 247 bytes
2. Handle MTU exchange
3. Prepare for fragmentation layer (M2)

## Technical Notes

### Softdevice Configuration
- S140 v7.x.x for BLE 5 support
- Central + Peripheral roles enabled
- 1 connection max initially (will increase in M4)

### UUIDs (Placeholder - need upstream values)
```rust
const BITCHAT_SERVICE_UUID: Uuid = uuid!("00000000-0000-0000-0000-000000000000");
const RX_CHAR_UUID: Uuid = uuid!("00000001-0000-0000-0000-000000000000");
const TX_CHAR_UUID: Uuid = uuid!("00000002-0000-0000-0000-000000000000");
```

### Memory Considerations
- Softdevice uses ~112KB of flash
- Requires ~8KB RAM for BLE stack
- Update memory.x accordingly

## Testing

### With nRF Connect (iOS/Android)
1. Scan for "bitchat-metal"
2. Connect and discover services
3. Write test data to RX characteristic
4. Subscribe to TX notifications
5. Verify echo received

### With Bitchat App
1. Ensure service/characteristic UUIDs match
2. Device should appear in nearby devices
3. Test basic message exchange

## Code Structure

```
src/
├── main.rs           # Main entry, spawns BLE task
├── ble/
│   ├── mod.rs        # BLE module
│   ├── service.rs    # GATT service definition
│   └── advertise.rs  # Advertising configuration
└── config.rs         # UUIDs and constants
```

## Dependencies to Add

```toml
nrf-softdevice = { version = "0.1", features = [
    "defmt",
    "nrf52840",
    "s140",
    "ble-peripheral",
    "ble-central",
    "ble-gatt-server",
    "ble-gatt-client"
]}
futures = { version = "0.3", default-features = false }
```

## Next Steps After M1

- M2: Implement Bitchat message header format
- M3: Add fragmentation/reassembly
- M4: Multi-peer connection management
- M5: Full protocol compliance testing

## Open Questions

1. What are the exact UUIDs used by the Bitchat app?
2. Does the app expect any specific advertising data?
3. Are there any GATT security requirements (bonding/pairing)?

## References

- [nrf-softdevice examples](https://github.com/embassy-rs/nrf-softdevice/tree/master/examples)
- [Nordic S140 Softdevice Spec](https://infocenter.nordicsemi.com/topic/sds_s140/SDS/s140/s140.html)
- [BLE GATT Specifications](https://www.bluetooth.com/specifications/gatt/)