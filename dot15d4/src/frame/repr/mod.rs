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
    pub payload: Option<&'p [u8]>,
}

impl<'f> FrameRepr<'f> {
    /// Parse an IEEE 802.15.4 frame.
    pub fn parse(reader: &Frame<&'f [u8]>) -> Self {
        Self {
            frame_control: FrameControlRepr::parse(reader.frame_control()),
            sequence_number: reader.sequence_number(),
            addressing_fields: AddressingFieldsRepr::parse(
                reader.addressing(),
                // Frame control is needed to determine the addressing modes
                reader.frame_control(),
            ),
            information_elements: reader
                .information_elements()
                .map(InformationElementsRepr::parse),
            payload: reader.payload(),
        }
    }

    pub fn buffer_len(&self) -> usize {
        let mut len = 2; // Frame control

        if self.sequence_number.is_some() {
            len += 1;
        }

        len += self.addressing_fields.buffer_len(&self.frame_control);

        if let Some(ie) = &self.information_elements {
            len += ie.buffer_len(self.payload.is_some());
        }

        if let Some(payload) = self.payload {
            len += payload.len();
        }

        len
    }

    pub fn emit(&self, frame: &mut Frame<&'_ mut [u8]>) {
        frame.set_frame_control(&self.frame_control);

        if let Some(sequence_number) = self.sequence_number {
            frame.set_sequence_number(sequence_number);
        }

        frame.set_addressing_fields(&self.addressing_fields);

        if let Some(ie) = &self.information_elements {
            frame.set_information_elements(ie, self.payload.is_some());
        }

        if let Some(payload) = self.payload {
            frame.set_payload(payload);
        }
    }
}

pub struct FrameBuilder<'p> {
    frame: FrameRepr<'p>,
}

impl<'p> FrameBuilder<'p> {
    pub fn new_ack() -> Self {
        Self {
            frame: FrameRepr {
                frame_control: FrameControlRepr {
                    frame_type: super::FrameType::Ack,
                    security_enabled: false,
                    frame_pending: false,
                    ack_request: false,
                    pan_id_compression: false,
                    sequence_number_suppression: true,
                    information_elements_present: false,
                    dst_addressing_mode: super::AddressingMode::Absent,
                    src_addressing_mode: super::AddressingMode::Absent,
                    frame_version: super::FrameVersion::Ieee802154_2020,
                },
                sequence_number: None,
                addressing_fields: AddressingFieldsRepr {
                    dst_pan_id: None,
                    src_pan_id: None,
                    dst_address: None,
                    src_address: None,
                },
                information_elements: None,
                payload: None,
            },
        }
    }

    pub fn set_sequence_number(mut self, sequence_number: u8) -> Self {
        self.frame.sequence_number = Some(sequence_number);
        self.frame.frame_control.sequence_number_suppression = false;
        self
    }

    pub fn set_dst_pan_id(mut self, pan_id: u16) -> Self {
        self.frame.addressing_fields.dst_pan_id = Some(pan_id);
        self
    }

    pub fn set_src_pan_id(mut self, pan_id: u16) -> Self {
        self.frame.addressing_fields.src_pan_id = Some(pan_id);
        self
    }

    pub fn set_dst_address(mut self, address: super::Address) -> Self {
        self.frame.frame_control.dst_addressing_mode = address.into();
        self.frame.addressing_fields.dst_address = Some(address);
        self
    }

    pub fn set_src_address(mut self, address: super::Address) -> Self {
        self.frame.frame_control.src_addressing_mode = address.into();
        self.frame.addressing_fields.src_address = Some(address);
        self
    }

    pub fn add_header_information_element(mut self, ie: HeaderInformationElementRepr) -> Self {
        self.frame.frame_control.information_elements_present = true;
        self.frame
            .information_elements
            .get_or_insert_with(InformationElementsRepr::default)
            .header_information_elements
            .push(ie)
            .unwrap();

        self
    }

    pub fn add_payload_information_element(mut self, ie: PayloadInformationElementRepr) -> Self {
        self.frame.frame_control.information_elements_present = true;
        self.frame
            .information_elements
            .get_or_insert_with(InformationElementsRepr::default)
            .payload_information_elements
            .push(ie)
            .unwrap();

        self
    }

    pub fn set_payload(mut self, payload: &'p [u8]) -> Self {
        self.frame.payload = Some(payload);
        self
    }

    pub fn finalize(self) -> FrameRepr<'p> {
        self.frame
    }
}
