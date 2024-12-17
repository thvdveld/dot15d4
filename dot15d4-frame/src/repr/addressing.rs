use super::FrameControlRepr;

use crate::{Address, AddressingFields, AddressingMode, Error, FrameType, Result};

/// A high-level representation of the IEEE 802.15.4 Addressing Fields.
#[derive(Debug, Default)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct AddressingFieldsRepr {
    /// Destination PAN identifier.
    pub dst_pan_id: Option<u16>,
    /// Destination address.
    pub dst_address: Option<Address>,
    /// Source PAN identifier.
    pub src_pan_id: Option<u16>,
    /// Source address.
    pub src_address: Option<Address>,
}

impl AddressingFieldsRepr {
    /// Parse the Addressing Fields from the given buffer.
    pub fn parse(addressing: AddressingFields<&'_ [u8], &'_ [u8]>) -> Self {
        Self {
            dst_pan_id: addressing.dst_pan_id(),
            dst_address: addressing.dst_address(),
            src_pan_id: addressing.src_pan_id(),
            src_address: addressing.src_address(),
        }
    }

    /// Validate the Addressing Fields.
    pub fn validate(&self, fc: &FrameControlRepr) -> Result<()> {
        if fc.frame_type == FrameType::Data
            && matches!(
                (
                    self.dst_pan_id,
                    self.dst_address,
                    self.src_pan_id,
                    self.src_address
                ),
                (None, None, None, None)
            )
        {
            return Err(Error);
        }
        Ok(())
    }

    /// Return the length of the Addressing Fields in octets.
    pub fn buffer_len(&self, fc: &FrameControlRepr) -> usize {
        (match self.dst_pan_id {
            Some(_) => 2,
            None => 0,
        }) + match fc.dst_addressing_mode {
            AddressingMode::Absent => 0,
            AddressingMode::Short => 2,
            AddressingMode::Extended => 8,
            _ => unreachable!(),
        } + match self.src_pan_id {
            Some(_) => 2,
            None => 0,
        } + match fc.src_addressing_mode {
            AddressingMode::Absent => 0,
            AddressingMode::Short => 2,
            AddressingMode::Extended => 8,
            _ => unreachable!(),
        }
    }

    /// Emit the Addressing Fields into the given buffer.
    pub fn emit(&self, _buffer: &AddressingFields<&'_ mut [u8], &'_ [u8]>) {
        todo!();
    }
}
