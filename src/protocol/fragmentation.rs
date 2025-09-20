use defmt::{info, warn, Format};
use heapless::{FnvIndexMap, Vec};
use crate::protocol::message::{Message, MessageHeader};

const MAX_CONCURRENT_MESSAGES: usize = 4;
const MAX_FRAGMENTS_PER_MESSAGE: usize = 8;

#[derive(Debug, Format, Clone)]
pub struct FragmentKey {
    sender_id: [u8; 6],
    sequence: u16,
}

impl FragmentKey {
    fn new(sender_id: [u8; 6], sequence: u16) -> Self {
        Self { sender_id, sequence }
    }
}

impl PartialEq for FragmentKey {
    fn eq(&self, other: &Self) -> bool {
        self.sender_id == other.sender_id && self.sequence == other.sequence
    }
}

impl Eq for FragmentKey {}

impl core::hash::Hash for FragmentKey {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.sender_id.hash(state);
        self.sequence.hash(state);
    }
}

struct FragmentBuffer {
    header: MessageHeader,
    fragments: Vec<Option<Vec<u8, 228>>, MAX_FRAGMENTS_PER_MESSAGE>,
    received_count: u8,
    total_expected: u8,
}

impl FragmentBuffer {
    fn new(header: MessageHeader) -> Self {
        let mut fragments = Vec::new();
        for _ in 0..MAX_FRAGMENTS_PER_MESSAGE {
            let _ = fragments.push(None);
        }

        Self {
            header,
            fragments,
            received_count: 0,
            total_expected: header.total_fragments,
        }
    }

    fn add_fragment(&mut self, index: u8, data: &[u8]) -> Result<bool, ()> {
        if index >= self.total_expected || index >= MAX_FRAGMENTS_PER_MESSAGE as u8 {
            warn!("Fragment index {} out of bounds (expected {})", index, self.total_expected);
            return Err(());
        }

        // Check if we already have this fragment
        if self.fragments[index as usize].is_some() {
            info!("Duplicate fragment {} ignored", index);
            return Ok(false);
        }

        // Store the fragment
        let mut fragment_data = Vec::new();
        fragment_data.extend_from_slice(data).map_err(|_| ())?;
        self.fragments[index as usize] = Some(fragment_data);
        self.received_count += 1;

        info!("Stored fragment {}/{} for message seq {}",
            self.received_count, self.total_expected, self.header.sequence);

        // Return true if we have all fragments
        Ok(self.received_count == self.total_expected)
    }

    fn assemble(self) -> Result<Message, ()> {
        if self.received_count != self.total_expected {
            return Err(());
        }

        let mut payload = Vec::new();

        for i in 0..self.total_expected as usize {
            if let Some(fragment) = &self.fragments[i] {
                payload.extend_from_slice(fragment).map_err(|_| {
                    warn!("Failed to assemble: payload too large");
                })?;
            } else {
                warn!("Missing fragment {} during assembly", i);
                return Err(());
            }
        }

        info!("Assembled complete message: {} bytes from {} fragments",
            payload.len(), self.total_expected);

        Ok(Message {
            header: self.header,
            payload,
        })
    }
}

pub struct FragmentAssembler {
    buffers: FnvIndexMap<FragmentKey, FragmentBuffer, MAX_CONCURRENT_MESSAGES>,
}

impl FragmentAssembler {
    pub fn new() -> Self {
        Self {
            buffers: FnvIndexMap::new(),
        }
    }

    pub fn add_fragment(&mut self, header: MessageHeader, payload: &[u8]) -> Result<Option<Message>, ()> {
        let key = FragmentKey::new(header.sender_id, header.sequence);

        // Get or create buffer for this message
        if !self.buffers.contains_key(&key) {
            // Make room if necessary
            if self.buffers.len() >= MAX_CONCURRENT_MESSAGES {
                // Remove oldest (first) entry
                if let Some((old_key, _)) = self.buffers.iter().next() {
                    let old_key = old_key.clone();
                    self.buffers.remove(&old_key);
                    warn!("Dropped incomplete message to make room");
                }
            }

            let buffer = FragmentBuffer::new(header);
            self.buffers.insert(key.clone(), buffer).map_err(|_| ())?;
        }

        // Add fragment to buffer
        let buffer = self.buffers.get_mut(&key).ok_or(())?;
        let is_complete = buffer.add_fragment(header.fragment_index, payload)?;

        if is_complete {
            // Remove and assemble
            let buffer = self.buffers.remove(&key).ok_or(())?;
            buffer.assemble().map(Some)
        } else {
            Ok(None)
        }
    }

    pub fn pending_count(&self) -> usize {
        self.buffers.len()
    }

    pub fn clear(&mut self) {
        self.buffers.clear();
    }
}