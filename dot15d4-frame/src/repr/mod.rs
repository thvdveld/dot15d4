use crate::FrameType;

use super::{Error, Frame, Result};

mod addressing;
pub use addressing::AddressingFieldsRepr;

mod frame_control;
pub use frame_control::FrameControlRepr;

mod ie;
pub use ie::*;

mod builder;
pub use builder::FrameBuilder;

/// A high-level representation of an IEEE 802.15.4 frame.
#[derive(Debug)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct FrameRepr<'p> {
    /// The frame control field.
    pub frame_control: FrameControlRepr,
    /// The sequence number.
    pub sequence_number: Option<u8>,
    /// The addressing fields.
    pub addressing_fields: Option<AddressingFieldsRepr>,
    /// The information elements.
    pub information_elements: Option<InformationElementsRepr>,
    /// The payload.
    pub payload: Option<&'p [u8]>,
}

impl<'f> FrameRepr<'f> {
    /// Parse an IEEE 802.15.4 frame.
    pub fn parse(reader: &Frame<&'f [u8]>) -> Result<Self> {
        let frame_control = FrameControlRepr::parse(reader.frame_control())?;
        let addressing_fields = reader
            .addressing()
            .map(|af| AddressingFieldsRepr::parse(af));
        let information_elements = reader
            .information_elements()
            .map(InformationElementsRepr::parse)
            .transpose()?;

        Ok(Self {
            frame_control,
            sequence_number: reader.sequence_number(),
            addressing_fields,
            information_elements,
            payload: reader.payload(),
        })
    }

    /// Validate the frame.
    pub fn validate(&self) -> Result<()> {
        // If the frame type is data, then the addressing fields must be present.
        if self.frame_control.frame_type == FrameType::Data {
            if self.addressing_fields.is_none() {
                return Err(Error);
            }

            if self.payload.is_none() {
                return Err(Error);
            }
        }

        // If the addressing fields are present, they must be valid.
        if let Some(af) = &self.addressing_fields {
            af.validate(&self.frame_control)?;
        }

        // If the payload is present, it must not be empty.
        if let Some(payload) = self.payload {
            if payload.is_empty() {
                return Err(Error);
            }
        }

        Ok(())
    }

    /// Return the length of the frame when emitted into a buffer.
    pub fn buffer_len(&self) -> usize {
        let mut len = 2; // Frame control

        if self.sequence_number.is_some() {
            len += 1;
        }

        if let Some(af) = &self.addressing_fields {
            len += af.buffer_len(&self.frame_control);
        }

        if let Some(ie) = &self.information_elements {
            len += ie.buffer_len(self.payload.is_some());
        }

        if let Some(payload) = self.payload {
            len += payload.len();
        }

        len
    }

    /// Emit the frame into a buffer.
    pub fn emit(&self, frame: &mut Frame<&'_ mut [u8]>) {
        frame.set_frame_control(&self.frame_control);

        if let Some(sequence_number) = self.sequence_number {
            frame.set_sequence_number(sequence_number);
        }

        if let Some(af) = &self.addressing_fields {
            frame.set_addressing_fields(af);
        }

        if let Some(ie) = &self.information_elements {
            frame.set_information_elements(ie, self.payload.is_some());
        }

        if let Some(payload) = self.payload {
            frame.set_payload(payload);
        }
    }
}
