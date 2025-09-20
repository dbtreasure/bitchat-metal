use defmt::info;
use heapless::Vec;
use crate::protocol::message::{Message, MessageType};

pub struct MessageRouter {
    device_id: [u8; 6],
    relay_enabled: bool,
    seen_messages: Vec<(u16, [u8; 6]), 16>, // Track recent messages to prevent relay loops
}

impl MessageRouter {
    pub fn new(device_id: [u8; 6]) -> Self {
        Self {
            device_id,
            relay_enabled: true,
            seen_messages: Vec::new(),
        }
    }

    pub fn should_relay(&mut self, message: &Message) -> bool {
        // Don't relay our own messages
        if message.header.sender_id == self.device_id {
            return false;
        }

        // Don't relay if TTL is exhausted
        if message.header.ttl == 0 {
            info!("Message TTL exhausted, not relaying");
            return false;
        }

        // Don't relay if we've already relayed this message (prevent loops)
        let key = (message.header.sequence, message.header.sender_id);
        if self.seen_messages.iter().any(|seen| *seen == key) {
            info!("Already relayed message seq {}, not relaying again", message.header.sequence);
            return false;
        }

        // Don't relay ACK messages (they're point-to-point)
        if message.header.msg_type == MessageType::Ack {
            return false;
        }

        // Check if relay is enabled
        if !self.relay_enabled {
            info!("Relay disabled, not forwarding");
            return false;
        }

        // Record that we're relaying this message
        if self.seen_messages.len() >= 16 {
            self.seen_messages.remove(0);
        }
        let _ = self.seen_messages.push(key);

        info!("Will relay message seq {} with TTL {}",
            message.header.sequence, message.header.ttl - 1);

        true
    }

    pub fn prepare_for_relay(&self, message: &mut Message) {
        // Decrement TTL
        message.header.ttl = message.header.ttl.saturating_sub(1);

        // Could add relay node info to flags if needed
        message.header.flags |= 0x01; // Mark as relayed
    }

    pub fn is_for_us(&self, message: &Message) -> bool {
        // For now, all messages are considered "for us" since we're in a broadcast mesh
        // In the future, we might add targeted messaging
        match message.header.msg_type {
            MessageType::Text | MessageType::Announce => true, // Broadcast messages
            MessageType::Ack => {
                // ACKs might be targeted (future enhancement)
                true
            }
            MessageType::Relay => true, // Process all relay messages
        }
    }

    pub fn set_relay_enabled(&mut self, enabled: bool) {
        self.relay_enabled = enabled;
        info!("Relay mode: {}", if enabled { "enabled" } else { "disabled" });
    }
}