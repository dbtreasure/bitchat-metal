use heapless::Vec;
use defmt::Format;

const HEADER_SIZE: usize = 13;
const SENDER_ID_SIZE: usize = 8;
const RECIPIENT_ID_SIZE: usize = 8;
const SIGNATURE_SIZE: usize = 64;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Format)]
pub enum PacketType {
    Text = 1,
    Announce = 2,
    File = 3,
    Ack = 4,
    Discovery = 5,
    KeepAlive = 6,
    Error = 7,
}

impl From<u8> for PacketType {
    fn from(value: u8) -> Self {
        match value {
            1 => PacketType::Text,
            2 => PacketType::Announce,
            3 => PacketType::File,
            4 => PacketType::Ack,
            5 => PacketType::Discovery,
            6 => PacketType::KeepAlive,
            7 => PacketType::Error,
            _ => PacketType::Text,
        }
    }
}

pub struct Flags;
impl Flags {
    pub const HAS_RECIPIENT: u8 = 0x01;
    pub const HAS_SIGNATURE: u8 = 0x02;
    pub const IS_COMPRESSED: u8 = 0x04;
}

pub struct BitchatPacket {
    pub version: u8,
    pub packet_type: PacketType,
    pub ttl: u8,
    pub timestamp: u64,
    pub flags: u8,
    pub sender_id: [u8; 8],
    pub recipient_id: Option<[u8; 8]>,
    pub payload: Vec<u8, 244>,
    pub signature: Option<[u8; 64]>,
}

impl BitchatPacket {
    pub fn new(
        packet_type: PacketType,
        sender_id: [u8; 8],
        payload: &[u8],
    ) -> Result<Self, &'static str> {
        let mut payload_vec = Vec::new();
        if payload_vec.extend_from_slice(payload).is_err() {
            return Err("Payload too large");
        }

        Ok(Self {
            version: 1,
            packet_type,
            ttl: 3,
            timestamp: 0,
            flags: 0,
            sender_id,
            recipient_id: None,
            payload: payload_vec,
            signature: None,
        })
    }

    pub fn encode(&self) -> Result<Vec<u8, 244>, &'static str> {
        let mut data = Vec::new();

        data.push(self.version).map_err(|_| "Buffer full")?;
        data.push(self.packet_type as u8).map_err(|_| "Buffer full")?;
        data.push(self.ttl).map_err(|_| "Buffer full")?;

        for i in (0..8).rev() {
            data.push((self.timestamp >> (i * 8)) as u8).map_err(|_| "Buffer full")?;
        }

        let mut flags = self.flags;
        if self.recipient_id.is_some() {
            flags |= Flags::HAS_RECIPIENT;
        }
        if self.signature.is_some() {
            flags |= Flags::HAS_SIGNATURE;
        }
        data.push(flags).map_err(|_| "Buffer full")?;

        let payload_length = self.payload.len() as u16;
        data.push((payload_length >> 8) as u8).map_err(|_| "Buffer full")?;
        data.push((payload_length & 0xFF) as u8).map_err(|_| "Buffer full")?;

        data.extend_from_slice(&self.sender_id).map_err(|_| "Buffer full")?;

        if let Some(recipient) = self.recipient_id {
            data.extend_from_slice(&recipient).map_err(|_| "Buffer full")?;
        }

        data.extend_from_slice(&self.payload).map_err(|_| "Buffer full")?;

        if let Some(signature) = self.signature {
            data.extend_from_slice(&signature).map_err(|_| "Buffer full")?;
        }

        Ok(data)
    }

    pub fn decode(data: &[u8]) -> Result<Self, &'static str> {
        if data.len() < HEADER_SIZE + SENDER_ID_SIZE {
            return Err("Packet too small");
        }

        let mut offset = 0;

        let version = data[offset];
        if version != 1 {
            return Err("Unsupported version");
        }
        offset += 1;

        let packet_type = PacketType::from(data[offset]);
        offset += 1;

        let ttl = data[offset];
        offset += 1;

        let mut timestamp: u64 = 0;
        for _ in 0..8 {
            timestamp = (timestamp << 8) | data[offset] as u64;
            offset += 1;
        }

        let flags = data[offset];
        offset += 1;

        let payload_length = ((data[offset] as u16) << 8) | data[offset + 1] as u16;
        offset += 2;

        let mut sender_id = [0u8; 8];
        sender_id.copy_from_slice(&data[offset..offset + 8]);
        offset += 8;

        let recipient_id = if flags & Flags::HAS_RECIPIENT != 0 {
            if data.len() < offset + 8 {
                return Err("Incomplete recipient ID");
            }
            let mut recipient = [0u8; 8];
            recipient.copy_from_slice(&data[offset..offset + 8]);
            offset += 8;
            Some(recipient)
        } else {
            None
        };

        if data.len() < offset + payload_length as usize {
            return Err("Incomplete payload");
        }

        let mut payload = Vec::new();
        if payload.extend_from_slice(&data[offset..offset + payload_length as usize]).is_err() {
            return Err("Payload too large");
        }
        offset += payload_length as usize;

        let signature = if flags & Flags::HAS_SIGNATURE != 0 {
            if data.len() < offset + 64 {
                return Err("Incomplete signature");
            }
            let mut sig = [0u8; 64];
            sig.copy_from_slice(&data[offset..offset + 64]);
            Some(sig)
        } else {
            None
        };

        Ok(Self {
            version,
            packet_type,
            ttl,
            timestamp,
            flags,
            sender_id,
            recipient_id,
            payload,
            signature,
        })
    }

    pub fn create_announce(sender_id: [u8; 8], message: &[u8]) -> Result<Self, &'static str> {
        let mut packet = Self::new(PacketType::Announce, sender_id, message)?;
        packet.timestamp = Self::current_timestamp_millis();
        Ok(packet)
    }

    pub fn create_text(sender_id: [u8; 8], text: &[u8]) -> Result<Self, &'static str> {
        let mut packet = Self::new(PacketType::Text, sender_id, text)?;
        packet.timestamp = Self::current_timestamp_millis();
        Ok(packet)
    }

    fn current_timestamp_millis() -> u64 {
        0
    }

    pub fn decrement_ttl(&mut self) -> bool {
        if self.ttl > 0 {
            self.ttl -= 1;
            true
        } else {
            false
        }
    }
}