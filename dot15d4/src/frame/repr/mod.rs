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

    pub fn buffer_len(&self) -> usize {
        let mut len = 2; // Frame control

        if self.sequence_number.is_some() {
            len += 1;
        }

        len += self.addressing_fields.buffer_len(&self.frame_control);

        if let Some(ie) = &self.information_elements {
            len += ie.buffer_len();
        }

        len += self.payload.len();

        len
    }

    pub fn emit(&self, frame: &mut Frame<&'_ mut [u8]>) {
        frame.set_frame_control(&self.frame_control);

        if let Some(sequence_number) = self.sequence_number {
            frame.set_sequence_number(sequence_number);
        }

        frame.set_addressing_fields(&self.addressing_fields);

        if let Some(ie) = &self.information_elements {
            frame.set_information_elements(ie);
        }

        if !self.payload.is_empty() {
            frame.set_payload(self.payload);
        }
    }
}
