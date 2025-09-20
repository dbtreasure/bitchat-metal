use defmt::Format;
use heapless::Vec;

pub const PROTOCOL_VERSION: u8 = 0x01;
pub const HEADER_SIZE: usize = 16;
pub const MAX_PAYLOAD_SIZE: usize = 228; // 244 - 16 header
pub const MAX_MESSAGE_SIZE: usize = 1024; // Maximum size for a complete message

#[derive(Debug, Clone, Copy, Format, PartialEq)]
pub enum MessageType {
    Text = 0x01,
    Ack = 0x02,
    Announce = 0x03,
    Relay = 0x04,
}

impl TryFrom<u8> for MessageType {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(MessageType::Text),
            0x02 => Ok(MessageType::Ack),
            0x03 => Ok(MessageType::Announce),
            0x04 => Ok(MessageType::Relay),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, Format)]
pub struct MessageHeader {
    pub version: u8,
    pub msg_type: MessageType,
    pub sender_id: [u8; 6],
    pub sequence: u16,
    pub fragment_index: u8,
    pub total_fragments: u8,
    pub ttl: u8,
    pub flags: u8,
    pub checksum: u16,
}

impl MessageHeader {
    pub fn new(msg_type: MessageType, sender_id: [u8; 6], sequence: u16) -> Self {
        Self {
            version: PROTOCOL_VERSION,
            msg_type,
            sender_id,
            sequence,
            fragment_index: 0,
            total_fragments: 1,
            ttl: 3, // Default TTL for mesh relay
            flags: 0,
            checksum: 0,
        }
    }

    pub fn serialize(&self) -> [u8; HEADER_SIZE] {
        let mut bytes = [0u8; HEADER_SIZE];
        bytes[0] = self.version;
        bytes[1] = self.msg_type as u8;
        bytes[2..8].copy_from_slice(&self.sender_id);
        bytes[8] = (self.sequence >> 8) as u8;
        bytes[9] = (self.sequence & 0xFF) as u8;
        bytes[10] = self.fragment_index;
        bytes[11] = self.total_fragments;
        bytes[12] = self.ttl;
        bytes[13] = self.flags;
        bytes[14] = (self.checksum >> 8) as u8;
        bytes[15] = (self.checksum & 0xFF) as u8;
        bytes
    }

    pub fn deserialize(bytes: &[u8]) -> Result<Self, ()> {
        if bytes.len() < HEADER_SIZE {
            return Err(());
        }

        let version = bytes[0];
        if version != PROTOCOL_VERSION {
            return Err(());
        }

        let msg_type = MessageType::try_from(bytes[1])?;
        let mut sender_id = [0u8; 6];
        sender_id.copy_from_slice(&bytes[2..8]);

        let sequence = ((bytes[8] as u16) << 8) | (bytes[9] as u16);
        let checksum = ((bytes[14] as u16) << 8) | (bytes[15] as u16);

        Ok(Self {
            version,
            msg_type,
            sender_id,
            sequence,
            fragment_index: bytes[10],
            total_fragments: bytes[11],
            ttl: bytes[12],
            flags: bytes[13],
            checksum,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Message {
    pub header: MessageHeader,
    pub payload: Vec<u8, MAX_MESSAGE_SIZE>,
}

impl Message {
    pub fn new(msg_type: MessageType, sender_id: [u8; 6], sequence: u16, payload: &[u8]) -> Result<Self, ()> {
        if payload.len() > MAX_MESSAGE_SIZE {
            return Err(());
        }

        let header = MessageHeader::new(msg_type, sender_id, sequence);
        let mut msg_payload = Vec::new();
        msg_payload.extend_from_slice(payload).map_err(|_| ())?;

        Ok(Self {
            header,
            payload: msg_payload,
        })
    }

    pub fn calculate_fragments(&self) -> u8 {
        let payload_len = self.payload.len();
        if payload_len == 0 {
            return 1;
        }
        ((payload_len + MAX_PAYLOAD_SIZE - 1) / MAX_PAYLOAD_SIZE) as u8
    }

    pub fn get_fragment(&self, index: u8) -> Option<Vec<u8, 244>> {
        let total_fragments = self.calculate_fragments();
        if index >= total_fragments {
            return None;
        }

        let start = (index as usize) * MAX_PAYLOAD_SIZE;
        let end = ((index as usize + 1) * MAX_PAYLOAD_SIZE).min(self.payload.len());

        let mut header = self.header;
        header.fragment_index = index;
        header.total_fragments = total_fragments;

        let mut fragment = Vec::new();
        let header_bytes = header.serialize();
        fragment.extend_from_slice(&header_bytes).ok()?;
        fragment.extend_from_slice(&self.payload[start..end]).ok()?;

        // Calculate and update checksum
        let checksum = calculate_crc16(&fragment[0..14], &fragment[16..]);
        fragment[14] = (checksum >> 8) as u8;
        fragment[15] = (checksum & 0xFF) as u8;

        Some(fragment)
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