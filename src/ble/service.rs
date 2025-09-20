use defmt::{info, warn};
use nrf_softdevice::ble::{gatt_server, Connection};
use nrf_softdevice::ble::gatt_server::RegisterError;
use nrf_softdevice::Softdevice;
use heapless::Vec;
use embassy_time::Timer;

use crate::bitchat::{BitchatPacket, PacketType};

// Using actual Bitchat UUIDs from iOS app
#[nrf_softdevice::gatt_service(uuid = "F47B5E2D-4A9E-4C5A-9B3F-8E1D2C3A4B5C")]
pub struct BitchatService {
    // Bitchat uses a single characteristic for bidirectional communication
    #[characteristic(uuid = "A1B2C3D4-E5F6-4A5B-8C9D-0E1F2A3B4C5D", write_without_response, notify)]
    pub data: [u8; 244],
}

#[nrf_softdevice::gatt_server]
pub struct Server {
    pub bitchat: BitchatService,
}

pub struct BitchatServer {
    server: Server,
    device_id: [u8; 8],
}

impl BitchatServer {
    pub fn new(sd: &mut Softdevice) -> Result<Self, RegisterError> {
        let server = Server::new(sd)?;

        // Generate device ID from BLE address (pad to 8 bytes)
        let addr = nrf_softdevice::ble::get_address(sd);
        let mut device_id = [0u8; 8];
        device_id[..6].copy_from_slice(&addr.bytes);
        // Last 2 bytes are 0x00 padding

        info!("Device ID: {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            device_id[0], device_id[1], device_id[2], device_id[3],
            device_id[4], device_id[5], device_id[6], device_id[7]);

        Ok(Self {
            server,
            device_id,
        })
    }

    pub async fn run(&mut self, conn: &Connection) {
        info!("Client connected, starting GATT server with protocol support");

        // Handle system attributes to fix BleGattsSysAttrMissing
        match gatt_server::set_sys_attrs(conn, None) {
            Ok(_) => info!("System attributes cleared"),
            Err(e) => warn!("Failed to set system attributes: {:?}", e),
        }

        let mut notifications_enabled = false;
        let mut outgoing_queue: Vec<Vec<u8, 244>, 4> = Vec::new();

        // Send announce immediately after connection (iOS expects this)
        match BitchatPacket::create_announce(
            self.device_id,
            b"bitchat-metal"
        ) {
            Ok(announce) => {
                match announce.encode() {
                    Ok(data) => {
                        let _ = outgoing_queue.push(data);
                        info!("Queued initial announce packet");
                    }
                    Err(e) => {
                        warn!("Failed to encode announce: {}", e);
                    }
                }
            }
            Err(e) => {
                warn!("Failed to create announce: {}", e);
            }
        }

        loop {
            // Process events
            let result = gatt_server::run(conn, &mut self.server, |e| {
                match e {
                    ServerEvent::Bitchat(BitchatServiceEvent::DataWrite(val)) => {
                        info!("RX: {} bytes", val.len());

                        // Try to decode as Bitchat packet
                        match BitchatPacket::decode(&val) {
                            Ok(mut packet) => {
                                info!("Received packet type: {:?}, TTL: {}", packet.packet_type, packet.ttl);

                                // Handle different packet types
                                match packet.packet_type {
                                    PacketType::Announce => {
                                        info!("Device announce from peer");
                                        // Could respond with our own announce
                                    }
                                    PacketType::Text => {
                                        info!("Text message received");
                                        // Log the message content
                                        if let Ok(text) = core::str::from_utf8(&packet.payload) {
                                            info!("Message: {}", text);
                                        }
                                    }
                                    PacketType::Discovery => {
                                        info!("Discovery packet");
                                        // Respond with announce
                                    }
                                    _ => {
                                        info!("Other packet type");
                                    }
                                }

                                // Relay if TTL > 0 (mesh functionality)
                                if packet.decrement_ttl() {
                                    info!("Would relay packet with TTL: {}", packet.ttl);
                                    // In mesh mode, encode and relay to other connections
                                }
                            }
                            Err(e) => {
                                warn!("Failed to decode packet: {}", e);
                            }
                        }
                    }
                    ServerEvent::Bitchat(BitchatServiceEvent::DataCccdWrite { notifications }) => {
                        info!("Data notifications: {}", notifications);
                        notifications_enabled = notifications;

                        if notifications {
                            // Send device announce when notifications enabled
                            match BitchatPacket::create_announce(
                                self.device_id,
                                b"bitchat-metal online"
                            ) {
                                Ok(announce) => {
                                    match announce.encode() {
                                        Ok(data) => {
                                            let _ = outgoing_queue.push(data);
                                            info!("Queued device announce");
                                        }
                                        Err(e) => {
                                            warn!("Failed to encode announce: {}", e);
                                        }
                                    }
                                }
                                Err(e) => {
                                    warn!("Failed to create announce: {}", e);
                                }
                            }
                        }
                    }
                }
            }).await;

            // Try to send queued messages (even without notifications for initial announce)
            if !outgoing_queue.is_empty() {
                if let Some(data) = outgoing_queue.get(0) {
                    let mut tx_data = [0u8; 244];
                    let len = data.len().min(244);
                    tx_data[..len].copy_from_slice(&data[..len]);
                    match self.server.bitchat.data_notify(conn, &tx_data) {
                        Ok(_) => {
                            info!("Sent {} bytes", data.len());
                            outgoing_queue.remove(0);
                        }
                        Err(e) => {
                            warn!("Failed to send: {:?}", e);
                            // Keep in queue to retry
                            Timer::after_millis(100).await;
                        }
                    }
                }
                continue;
            }

            // If run() returned, the connection was closed
            warn!("Disconnected: {:?}", result);
            break;
        }

        info!("Connection closed");
    }
}