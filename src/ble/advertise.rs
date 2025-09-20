use defmt::info;
use nrf_softdevice::ble::{peripheral, Connection};
use nrf_softdevice::ble::peripheral::ConnectableAdvertisement;
use nrf_softdevice::Softdevice;

use crate::config::BITCHAT_SERVICE_UUID;

pub async fn advertise(sd: &Softdevice) -> Result<Connection, peripheral::AdvertiseError> {
    let config = peripheral::Config::default();

    // Match Bitchat: No device name for privacy, only service UUID
    // BLE requires little-endian (reversed) byte order for 128-bit UUIDs
    let adv = ConnectableAdvertisement::ScannableUndirected {
        adv_data: &[
            0x02, 0x01, 0x06,  // Flags (LE General Discoverable Mode, BR/EDR not supported)
            0x11, 0x07,        // 17 bytes follow, type 0x07 (Complete 128-bit service UUID list)
            // UUID bytes in reverse order for BLE (little-endian)
            BITCHAT_SERVICE_UUID[15], BITCHAT_SERVICE_UUID[14],
            BITCHAT_SERVICE_UUID[13], BITCHAT_SERVICE_UUID[12],
            BITCHAT_SERVICE_UUID[11], BITCHAT_SERVICE_UUID[10],
            BITCHAT_SERVICE_UUID[9], BITCHAT_SERVICE_UUID[8],
            BITCHAT_SERVICE_UUID[7], BITCHAT_SERVICE_UUID[6],
            BITCHAT_SERVICE_UUID[5], BITCHAT_SERVICE_UUID[4],
            BITCHAT_SERVICE_UUID[3], BITCHAT_SERVICE_UUID[2],
            BITCHAT_SERVICE_UUID[1], BITCHAT_SERVICE_UUID[0],
        ],
        scan_data: &[0x00], // Minimal scan response to satisfy softdevice
    };

    info!("Starting BLE advertisement (no name for privacy)");
    info!("Service UUID: F47B5E2D-4A9E-4C5A-9B3F-8E1D2C3A4B5C (Mainnet/Release)");

    peripheral::advertise_connectable(sd, adv, &config).await
}