//! Vendor-specific mailbox header encoding.

/// VoE protocol header followed by opaque vendor data.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Packet<'a> {
    /// Vendor identifier.
    pub vendor_id: u32,
    /// Vendor-defined message type.
    pub vendor_type: u16,
    /// Opaque vendor-owned payload.
    pub payload: &'a [u8],
}

impl Packet<'_> {
    /// Encode the six-byte VoE header and opaque payload.
    pub fn encode(self, output: &mut [u8]) -> Option<usize> {
        let end = 6usize.checked_add(self.payload.len())?;
        let header = output.get_mut(..6)?;
        header[..4].copy_from_slice(&self.vendor_id.to_le_bytes());
        header[4..].copy_from_slice(&self.vendor_type.to_le_bytes());
        output.get_mut(6..end)?.copy_from_slice(self.payload);
        Some(end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vendor_echo_vector_uses_standard_six_byte_header() {
        let packet = Packet { vendor_id: 0x1337, vendor_type: 1, payload: &[0xaa] };
        let mut bytes = [0; 7];
        assert_eq!(packet.encode(&mut bytes), Some(7));
        assert_eq!(bytes, [0x37, 0x13, 0, 0, 1, 0, 0xaa]);
    }
}
