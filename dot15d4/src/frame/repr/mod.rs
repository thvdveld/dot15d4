use super::Frame;

mod addressing;
pub use addressing::AddressingFieldsRepr;

mod frame_control;
pub use frame_control::FrameControlRepr;

mod ie;
pub use ie::*;

use heapless::Vec;

/// A high-level representation of an IEEE 802.15.4 frame.
#[derive(Debug)]
pub struct FrameRepr<'p> {
    /// The frame control field.
    pub frame_control: FrameControlRepr,
    /// The sequence number.
    pub sequence_number: Option<u8>,
    /// The addressing fields.
    pub addressing_fields: AddressingFieldsRepr,
    /// The information elements.
    pub information_elements: Option<InformationElementsRepr>,
    /// The payload.
    pub payload: &'p [u8],
}

impl<'f> FrameRepr<'f> {
    /// Parse an IEEE 802.15.4 frame.
    pub fn parse(reader: &Frame<&'f [u8]>) -> Self {
        Self {
            frame_control: FrameControlRepr::parse(reader.frame_control()),
            sequence_number: reader.sequence_number(),
            addressing_fields: AddressingFieldsRepr::parse(
                reader.addressing(),
                reader.frame_control(),
            ),
            information_elements: reader.information_elements().map(|ie| {
                let mut header_information_elements = Vec::new();
                let mut payload_information_elements = Vec::new();

                for header_ie in ie.header_information_elements() {
                    header_information_elements
                        .push(HeaderInformationElementRepr::parse(header_ie));
                }

                for payload_ie in ie.payload_information_elements() {
                    payload_information_elements
                        .push(PayloadInformationElementRepr::parse(payload_ie));
                }

                InformationElementsRepr {
                    header_information_elements,
                    payload_information_elements,
                }
            }),
            payload: reader.payload().unwrap_or(&[]),
        }
    }
}
