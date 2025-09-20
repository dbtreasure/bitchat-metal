use defmt::{info, warn, Format};
use heapless::Vec;
use crate::protocol::message::{Message, MessageHeader, MessageType, HEADER_SIZE, MAX_MESSAGE_SIZE};
use crate::protocol::fragmentation::FragmentAssembler;
use crate::protocol::text::TextMessage;

#[derive(Debug, Format)]
pub enum HandlerError {
    InvalidMessage,
    FragmentationError,
    ChecksumError,
}

pub struct MessageHandler {
    device_id: [u8; 6],
    sequence_counter: u16,
    fragment_assembler: FragmentAssembler,
    last_seen_sequences: Vec<(u16, [u8; 6]), 32>, // Track last 32 messages to prevent duplicates
}

impl MessageHandler {
    pub fn new(device_id: [u8; 6]) -> Self {
        Self {
            device_id,
            sequence_counter: 0,
            fragment_assembler: FragmentAssembler::new(),
            last_seen_sequences: Vec::new(),
        }
    }

    pub fn get_next_sequence(&mut self) -> u16 {
        let seq = self.sequence_counter;
        self.sequence_counter = self.sequence_counter.wrapping_add(1);
        seq
    }

    pub fn process_incoming(&mut self, data: &[u8]) -> Result<Option<Message>, HandlerError> {
        if data.len() < HEADER_SIZE {
            warn!("Received data too small for header: {} bytes", data.len());
            return Err(HandlerError::InvalidMessage);
        }

        // Parse header
        let header = MessageHeader::deserialize(data)
            .map_err(|_| HandlerError::InvalidMessage)?;

        info!("Received message type {:?} from {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}, seq {}, frag {}/{}",
            header.msg_type,
            header.sender_id[0], header.sender_id[1], header.sender_id[2],
            header.sender_id[3], header.sender_id[4], header.sender_id[5],
            header.sequence,
            header.fragment_index + 1, header.total_fragments
        );

        // Check for duplicate
        if self.is_duplicate(&header) {
            info!("Duplicate message detected, ignoring");
            return Ok(None);
        }

        // Verify checksum
        if !self.verify_checksum(data) {
            warn!("Checksum verification failed");
            return Err(HandlerError::ChecksumError);
        }

        // Extract payload
        let payload = &data[HEADER_SIZE..];

        // Handle fragmentation
        if header.total_fragments > 1 {
            match self.fragment_assembler.add_fragment(header, payload) {
                Ok(Some(complete_message)) => {
                    self.record_sequence(&header);
                    Ok(Some(complete_message))
                }
                Ok(None) => {
                    // Fragment stored, waiting for more
                    Ok(None)
                }
                Err(_) => Err(HandlerError::FragmentationError),
            }
        } else {
            // Single fragment message
            self.record_sequence(&header);

            let mut payload_vec = Vec::new();
            payload_vec.extend_from_slice(payload)
                .map_err(|_| HandlerError::InvalidMessage)?;

            Ok(Some(Message {
                header,
                payload: payload_vec,
            }))
        }
    }

    pub fn handle_message(&mut self, message: &Message) -> Result<Option<Vec<u8, MAX_MESSAGE_SIZE>>, HandlerError> {
        match message.header.msg_type {
            MessageType::Text => {
                info!("Text message received: {} bytes", message.payload.len());

                // Parse and log the text
                if let Ok(text) = TextMessage::parse(&message.payload) {
                    info!("Text: \"{}\"", text.as_str());
                }

                // Could generate ACK here if needed
                Ok(None)
            }
            MessageType::Announce => {
                info!("Device announce from {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                    message.header.sender_id[0], message.header.sender_id[1],
                    message.header.sender_id[2], message.header.sender_id[3],
                    message.header.sender_id[4], message.header.sender_id[5]
                );
                // Could respond with our own announce
                Ok(None)
            }
            MessageType::Ack => {
                info!("ACK received for sequence {}", message.header.sequence);
                Ok(None)
            }
            MessageType::Relay => {
                if message.header.ttl > 0 {
                    info!("Relay message with TTL {}", message.header.ttl);
                    // Relay logic would go here
                }
                Ok(None)
            }
        }
    }

    fn is_duplicate(&self, header: &MessageHeader) -> bool {
        self.last_seen_sequences.iter().any(|(seq, sender)| {
            *seq == header.sequence && *sender == header.sender_id
        })
    }

    fn record_sequence(&mut self, header: &MessageHeader) {
        if self.last_seen_sequences.len() >= 32 {
            self.last_seen_sequences.remove(0);
        }
        let _ = self.last_seen_sequences.push((header.sequence, header.sender_id));
    }

    fn verify_checksum(&self, data: &[u8]) -> bool {
        if data.len() < HEADER_SIZE {
            return false;
        }

        // Extract the checksum from header
        let stored_checksum = ((data[14] as u16) << 8) | (data[15] as u16);

        // Calculate checksum on header (excluding checksum bytes) and payload
        let header_part = &data[0..14];
        let payload_part = &data[16..];

        let calculated = calculate_crc16(header_part, payload_part);

        calculated == stored_checksum
    }
}

fn calculate_crc16(header: &[u8], payload: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;

    for &byte in header.iter().chain(payload.iter()) {
        crc ^= (byte as u16) << 8;
        for _ in 0..8 {
            if (crc & 0x8000) != 0 {
                crc = (crc << 1) ^ 0x1021;
            } else {
                crc <<= 1;
            }
        }
    }

    crc
}