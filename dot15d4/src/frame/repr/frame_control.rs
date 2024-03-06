use super::super::{AddressingMode, FrameControl, FrameType, FrameVersion};
use super::super::{Error, Result};

/// A high-level representation of the IEEE 802.15.4 Frame Control field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameControlRepr {
    pub frame_type: FrameType,
    pub security_enabled: bool,
    pub frame_pending: bool,
    pub ack_request: bool,
    pub pan_id_compression: bool,
    pub sequence_number_suppression: bool,
    pub information_elements_present: bool,
    pub dst_addressing_mode: AddressingMode,
    pub src_addressing_mode: AddressingMode,
    pub frame_version: FrameVersion,
}

impl FrameControlRepr {
    /// Parse an IEEE 802.15.4 Frame Control field.
    pub fn parse(fc: FrameControl<&[u8]>) -> Result<Self> {
        Ok(Self {
            frame_type: fc.frame_type(),
            security_enabled: fc.security_enabled(),
            frame_pending: fc.frame_pending(),
            ack_request: fc.ack_request(),
            pan_id_compression: fc.pan_id_compression(),
            sequence_number_suppression: fc.sequence_number_suppression(),
            information_elements_present: fc.information_elements_present(),
            dst_addressing_mode: fc.dst_addressing_mode(),
            src_addressing_mode: fc.src_addressing_mode(),
            frame_version: fc.frame_version(),
        })
    }

    /// Return the length of the frame control field when emitted into a buffer.
    pub const fn buffer_len(&self) -> usize {
        2
    }

    /// Emit the frame control field into a buffer.
    pub fn emit(&self, fc: &mut FrameControl<&mut [u8]>) {
        fc.set_frame_type(self.frame_type);
        fc.set_security_enabled(self.security_enabled);
        fc.set_frame_pending(self.frame_pending);
        fc.set_ack_request(self.ack_request);
        fc.set_pan_id_compression(self.pan_id_compression);
        fc.set_sequence_number_suppression(self.sequence_number_suppression);
        fc.set_information_elements_present(self.information_elements_present);
        fc.set_dst_addressing_mode(self.dst_addressing_mode);
        fc.set_src_addressing_mode(self.src_addressing_mode);
        fc.set_frame_version(self.frame_version);
    }
}
