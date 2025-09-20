# NOTES_FOR_CODEX: Bitchat nRF52840 Device Visibility Issue

## Current Status (2025-09-20)

The nRF52840 device successfully advertises BLE with the correct Bitchat service UUID and is visible in nRF Connect app, but **does NOT appear in the Bitchat iOS app's #mesh view**.

### What's Working
- ✅ Device advertises with correct mainnet UUID: `F47B5E2D-4A9E-4C5A-9B3F-8E1D2C3A4B5C`
- ✅ Advertisement packet format is correct (verified with nRF Connect)
- ✅ GATT service and characteristic UUIDs match iOS expectations
- ✅ Device accepts BLE connections from iOS
- ✅ Device queues announce packet immediately after connection
- ✅ Visible in nRF Connect app with correct UUID

### What's NOT Working
- ❌ Device does not appear in Bitchat #mesh view
- ❌ No evidence of iOS app actually connecting to our device
- ❌ Announce packet likely never gets sent (no "Sent X bytes" in logs)
- ❌ Notifications never get enabled by iOS

## What We've Tried

### 1. UUID Corrections
- Fixed UUID byte order (was reversed, now correct little-endian)
- Confirmed using mainnet UUID (ending in 5C) for App Store version
- Changed GATT service UUID from testnet (5A) to mainnet (5C)
- **Result**: Device visible in nRF Connect but not Bitchat

### 2. Advertisement Flags
- Tried multiple flag combinations: 0x1A, 0x02, 0x04, 0x06
- Settled on 0x06 (LE General Discoverable, BR/EDR not supported)
- **Result**: Advertisement works, but Bitchat doesn't connect

### 3. Protocol Implementation
- Implemented actual Bitchat binary protocol (13-byte header + fields)
- Created proper announce packets with PacketType::Announce
- **Result**: Packet creation works but never gets transmitted

### 4. Handshake Timing Fix
- Moved announce packet to send immediately after connection (not wait for notifications)
- Removed requirement for notifications_enabled before sending
- **Result**: Still no connection from Bitchat app

## Key Discoveries from iOS Code Analysis

### Critical Requirements (from BLEService.swift)
1. **Service UUID Filtering**: iOS scans ONLY for specific UUID (debug vs release)
2. **Connection Required**: Device must be `isConnected` to appear in mesh
3. **Announce Packet Validation**: Must contain:
   - Nickname (string)
   - Noise public key (32 bytes, Curve25519)
   - Signing public key (32 bytes, Ed25519)
   - Valid signature using Noise private key
4. **TLV Format**: Announce uses Tag-Length-Value encoding, not just raw binary
5. **Immediate Send**: Announce should be sent on connection, not after notifications

### iOS Connection Flow (expected)
1. Scan for peripherals with service UUID
2. Connect to discovered peripheral
3. Discover services and characteristics
4. **Receive announce packet from device** (this is where we fail)
5. Verify announce packet signature
6. Enable notifications
7. Add to mesh view

## Current Hypothesis

The core issue is that **Bitchat iOS app is not even attempting to connect** to our device. Possible reasons:

### 1. Missing Advertisement Data
iOS might require additional advertisement data beyond just the service UUID:
- Manufacturer data?
- Service data?
- Local name (even though code suggests it's optional)?

### 2. RSSI Threshold
Default RSSI threshold is -90 dBm. Device might be:
- Too far away (weak signal)
- Signal strength not being reported correctly

### 3. iOS Filtering Beyond UUID
iOS might have additional undocumented filters:
- Checking for specific advertisement patterns
- Requiring certain peripheral properties
- Filtering based on address type (public vs random)

### 4. Announce Packet Format Wrong
Our announce packet might be malformed:
- Not using proper TLV encoding
- Missing required cryptographic keys
- Incorrect packet structure

## What We Haven't Tried Yet

### 1. Advertisement Enhancements
- Add manufacturer data to advertisement
- Include service data in advertisement
- Try advertising with a local name (even though privacy-focused Bitchat shouldn't need it)
- Use non-connectable advertisement first, then switch to connectable

### 2. Cryptographic Implementation
- Implement actual Noise protocol keys (Curve25519)
- Add Ed25519 signing keys
- Create properly signed announce packets
- Use correct TLV encoding for announce data

### 3. Protocol Debugging
- Capture BLE traffic between two real Bitchat devices
- Use PacketLogger on iOS to see what Bitchat is actually doing
- Monitor CoreBluetooth logs for connection attempts
- Check if iOS is even discovering our peripheral

### 4. Alternative Approaches
- Try advertising as a Bitchat "relay" instead of regular device
- Implement the testnet UUID and use a debug build of iOS app
- Add a delay between advertisement start and connection acceptance
- Try different BLE address types (random static vs public)

### 5. Signal Strength
- Move device closer to phone (< 1 meter)
- Increase TX power if possible
- Check if RSSI is being reported in advertisement

## Next Steps Recommendation

1. **Capture Real Traffic**: Use a BLE sniffer to capture packets between working Bitchat devices
2. **Implement Crypto**: Add proper Noise/Ed25519 keys even if dummy values
3. **TLV Encoding**: Rewrite announce packet to use proper TLV format
4. **iOS Debugging**: Enable CoreBluetooth logging to see why connection isn't attempted
5. **Advertisement Testing**: Systematically try different advertisement configurations

## Technical Details

### Current Implementation Files
- `/src/ble/advertise.rs` - BLE advertisement setup (lines 13-28)
- `/src/ble/service.rs` - GATT service and announce packet (lines 11-16, 60-79)
- `/src/bitchat/packet.rs` - Bitchat protocol implementation
- `/src/config.rs` - UUID definitions (lines 6-9)

### iOS Implementation References
- `BLEService.swift:18-22` - UUID selection based on debug/release
- `BLEService.swift:3171-3177` - Advertisement data builder
- `UnifiedPeerService.swift:154-158` - Mesh visibility requirements
- `Packets.swift` - TLV packet encoding format

## Conclusion

The device is technically advertising correctly from a BLE perspective (proven by nRF Connect visibility), but something about our implementation doesn't meet Bitchat's specific requirements for connection. The most likely issue is either:

1. Missing cryptographic components in the announce packet
2. Incorrect TLV encoding format
3. iOS-specific filtering we haven't discovered yet
4. Advertisement data missing required elements

Without packet captures from working Bitchat devices or iOS debug logs, we're essentially reverse-engineering blindly based on source code analysis alone.