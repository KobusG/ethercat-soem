//! FoE wire format shared by the wrappers.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u16)]
/// FoE operation code.
pub enum Opcode {
    /// Read request.
    Read = 1,
    /// Write request.
    Write = 2,
    /// Data packet.
    Data = 3,
    /// Packet acknowledgement.
    Ack = 4,
    /// Transfer error.
    Error = 5,
    /// Busy response.
    Busy = 6,
}

/// FoE packet fields after the EtherCAT mailbox header.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Packet<'a> {
    /// Operation code.
    pub opcode: Opcode,
    /// Password, packet number, or error code.
    pub value: u32,
    /// Filename bytes for RRQ/WRQ or packet payload.
    pub data: &'a [u8],
}

/// FoE packet decoding failure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecodeError {
    /// Fewer than six bytes were present.
    TooShort,
    /// The opcode is not defined by FoE.
    UnknownOpcode,
}

impl<'a> Packet<'a> {
    /// Decode a FoE header and borrow its payload.
    pub fn decode(bytes: &'a [u8]) -> Result<Self, DecodeError> {
        if bytes.len() < 6 {
            return Err(DecodeError::TooShort);
        }
        let opcode = match u16::from_le_bytes([bytes[0], bytes[1]]) {
            1 => Opcode::Read,
            2 => Opcode::Write,
            3 => Opcode::Data,
            4 => Opcode::Ack,
            5 => Opcode::Error,
            6 => Opcode::Busy,
            _ => return Err(DecodeError::UnknownOpcode),
        };
        Ok(Self {
            opcode,
            value: u32::from_le_bytes(bytes[2..6].try_into().unwrap()),
            data: &bytes[6..],
        })
    }

    /// Encode the standard six-byte FoE header and data into `output`.
    pub fn encode(self, output: &mut [u8]) -> Option<usize> {
        let end = 6usize.checked_add(self.data.len())?;
        let header = output.get_mut(..6)?;
        header[..2].copy_from_slice(&(self.opcode as u16).to_le_bytes());
        header[2..].copy_from_slice(&self.value.to_le_bytes());
        output.get_mut(6..end)?.copy_from_slice(self.data);
        Some(end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rrq_wrq_match_soem_wire_bytes() {
        let expected_read = [1, 0, 0, 0, 0, 0, b'd', b'e', b'm', b'o', b'.', b'b', b'i', b'n'];
        let expected_write = [2, 0, 0, 0, 0, 0, b'd', b'e', b'm', b'o', b'.', b'b', b'i', b'n'];
        for (opcode, expected) in [(Opcode::Read, expected_read), (Opcode::Write, expected_write)] {
            let packet = Packet { opcode, value: 0, data: b"demo.bin" };
            let mut bytes = [0; 14];
            assert_eq!(packet.encode(&mut bytes), Some(14));
            assert_eq!(bytes, expected);
        }
        let mut data = [0; 8];
        Packet { opcode: Opcode::Data, value: 1, data: &[0xaa, 0xbb] }.encode(&mut data);
        assert_eq!(data, [3, 0, 1, 0, 0, 0, 0xaa, 0xbb]);
        let mut ack = [0; 6];
        Packet { opcode: Opcode::Ack, value: 1, data: &[] }.encode(&mut ack);
        assert_eq!(ack, [4, 0, 1, 0, 0, 0]);
        let error = [5, 0, 0, 0x80, 0, 0];
        assert_eq!(Packet::decode(&error).unwrap().value, 0x8000);
        assert_eq!(Packet::decode(&error[..5]), Err(DecodeError::TooShort));
        assert_eq!(Packet::decode(&[0, 0, 0, 0, 0, 0]), Err(DecodeError::UnknownOpcode));
    }
}
