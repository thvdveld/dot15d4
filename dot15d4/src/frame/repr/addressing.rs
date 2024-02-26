use super::super::{Address, AddressingFields, AddressingMode, FrameControl};
use super::FrameControlRepr;

/// A high-level representation of the IEEE 802.15.4 Addressing Fields.
#[derive(Debug)]
pub struct AddressingFieldsRepr {
    pub dst_pan_id: Option<u16>,
    pub src_pan_id: Option<u16>,
    pub dst_address: Option<Address>,
    pub src_address: Option<Address>,
}

impl AddressingFieldsRepr {
    /// Parse the Addressing Fields from the given buffer.
    pub fn parse<'f>(addressing: AddressingFields<&'f [u8]>, fc: FrameControl<&'f [u8]>) -> Self {
        Self {
            dst_pan_id: addressing.dst_pan_id(&fc),
            src_pan_id: addressing.src_pan_id(&fc),
            dst_address: addressing.dst_address(&fc),
            src_address: addressing.src_address(&fc),
        }
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
    pub fn emit<'f>(
        &self,
        buffer: &AddressingFields<&'f mut [u8]>,
        fc: &FrameControl<&'f mut [u8]>,
    ) {
        todo!();
    }
}
