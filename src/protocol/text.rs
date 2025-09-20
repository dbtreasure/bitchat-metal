use defmt::info;
use heapless::String;
use crate::protocol::message::{Message, MessageType, MAX_MESSAGE_SIZE};

pub struct TextMessage;

impl TextMessage {
    pub fn create(
        sender_id: [u8; 6],
        sequence: u16,
        text: &str,
    ) -> Result<Message, ()> {
        // Convert text to bytes
        let text_bytes = text.as_bytes();

        if text_bytes.len() > MAX_MESSAGE_SIZE {
            return Err(());
        }

        info!("Creating text message: {} bytes", text_bytes.len());
        Message::new(MessageType::Text, sender_id, sequence, text_bytes)
    }

    pub fn parse(payload: &[u8]) -> Result<String<MAX_MESSAGE_SIZE>, ()> {
        // Try to convert payload to UTF-8 string
        let mut result = String::new();

        // Find valid UTF-8 portion
        let text = core::str::from_utf8(payload)
            .unwrap_or_else(|e| {
                // If invalid UTF-8, use the valid portion
                core::str::from_utf8(&payload[..e.valid_up_to()]).unwrap_or("")
            });

        result.push_str(text).map_err(|_| ())?;
        Ok(result)
    }

    pub fn create_announce(
        device_id: [u8; 6],
        sequence: u16,
        device_name: &str,
    ) -> Result<Message, ()> {
        let announce_text = if device_name.is_empty() {
            "device online"
        } else {
            device_name
        };

        Message::new(MessageType::Announce, device_id, sequence, announce_text.as_bytes())
    }
}