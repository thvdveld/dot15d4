use super::super::{AddressingMode, FrameControl, FrameType, FrameVersion};

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
    pub fn parse<'f>(fc: FrameControl<&'f [u8]>) -> Self {
        Self {
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
        }
    }
}
