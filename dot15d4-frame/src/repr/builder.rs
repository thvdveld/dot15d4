use super::*;
use crate::{Address, AddressingMode, FrameType, FrameVersion};
use crate::{Error, Result};

pub struct Beacon;
pub struct EnhancedBeacon;
pub struct Ack;
pub struct Data;

/// A helper for building IEEE 802.15.4 frames.
pub struct FrameBuilder<'p, T> {
    frame: FrameRepr<'p>,
    r#type: core::marker::PhantomData<T>,
}

impl<'p> FrameBuilder<'p, Ack> {
    /// Create a new builder for an immediate acknowledgment frame.
    pub fn new_imm_ack(sequence_number: u8) -> Self {
        Self {
            frame: FrameRepr {
                frame_control: FrameControlRepr {
                    frame_type: FrameType::Ack,
                    security_enabled: false,
                    frame_pending: false,
                    ack_request: false,
                    pan_id_compression: false,
                    sequence_number_suppression: false,
                    information_elements_present: false,
                    dst_addressing_mode: AddressingMode::Absent,
                    src_addressing_mode: AddressingMode::Absent,
                    frame_version: FrameVersion::Ieee802154_2006,
                },
                sequence_number: Some(sequence_number),
                addressing_fields: None,
                information_elements: None,
                payload: None,
            },
            r#type: Default::default(),
        }
    }

    /// Create a new builder for an acknowledgment frame.
    pub fn new_ack() -> Self {
        Self {
            frame: FrameRepr {
                frame_control: FrameControlRepr {
                    frame_type: FrameType::Ack,
                    security_enabled: false,
                    frame_pending: false,
                    ack_request: false,
                    pan_id_compression: false,
                    sequence_number_suppression: true,
                    information_elements_present: false,
                    dst_addressing_mode: AddressingMode::Absent,
                    src_addressing_mode: AddressingMode::Absent,
                    frame_version: FrameVersion::Ieee802154_2020,
                },
                sequence_number: None,
                addressing_fields: None,
                information_elements: None,
                payload: None,
            },
            r#type: Default::default(),
        }
    }
}

impl<'p> FrameBuilder<'p, Beacon> {
    /// Create a new builder for a beacon frame.
    pub fn new_beacon() -> Self {
        Self {
            frame: FrameRepr {
                frame_control: FrameControlRepr {
                    frame_type: FrameType::Beacon,
                    security_enabled: false,
                    frame_pending: false,
                    ack_request: false,
                    pan_id_compression: false,
                    sequence_number_suppression: true,
                    information_elements_present: false,
                    dst_addressing_mode: AddressingMode::Absent,
                    src_addressing_mode: AddressingMode::Absent,
                    frame_version: FrameVersion::Ieee802154_2006,
                },
                sequence_number: None,
                addressing_fields: None,
                information_elements: None,
                payload: None,
            },
            r#type: core::marker::PhantomData,
        }
    }
}

impl<'p> FrameBuilder<'p, EnhancedBeacon> {
    /// Create a new builder for an enhanced beacon frame.
    pub fn new_enhanced_beacon() -> Self {
        Self {
            frame: FrameRepr {
                frame_control: FrameControlRepr {
                    frame_type: FrameType::Beacon,
                    security_enabled: false,
                    frame_pending: false,
                    ack_request: false,
                    pan_id_compression: false,
                    sequence_number_suppression: true,
                    information_elements_present: false,
                    dst_addressing_mode: AddressingMode::Absent,
                    src_addressing_mode: AddressingMode::Absent,
                    frame_version: FrameVersion::Ieee802154_2020,
                },
                sequence_number: None,
                addressing_fields: None,
                information_elements: None,
                payload: None,
            },
            r#type: core::marker::PhantomData,
        }
    }
}

impl<'p> FrameBuilder<'p, Data> {
    /// Create a new builder for a data frame.
    pub fn new_data(payload: &'p [u8]) -> Self {
        Self {
            frame: FrameRepr {
                frame_control: FrameControlRepr {
                    frame_type: FrameType::Data,
                    security_enabled: false,
                    frame_pending: false,
                    ack_request: false,
                    pan_id_compression: false,
                    sequence_number_suppression: true,
                    information_elements_present: false,
                    dst_addressing_mode: AddressingMode::Absent,
                    src_addressing_mode: AddressingMode::Absent,
                    frame_version: FrameVersion::Ieee802154_2006,
                },
                sequence_number: None,
                addressing_fields: None,
                information_elements: None,
                payload: Some(payload),
            },
            r#type: core::marker::PhantomData,
        }
    }
}

impl<'p, T> FrameBuilder<'p, T> {
    /// Set the frame sequence number.
    ///
    /// # Note
    /// This method disables sequence number suppression.
    pub fn set_sequence_number(mut self, sequence_number: u8) -> Self {
        self.frame.sequence_number = Some(sequence_number);
        self.frame.frame_control.sequence_number_suppression = false;
        self
    }

    /// Set the destination PAN ID.
    pub fn set_dst_pan_id(mut self, pan_id: u16) -> Self {
        self.frame
            .addressing_fields
            .get_or_insert_with(AddressingFieldsRepr::default)
            .dst_pan_id = Some(pan_id);

        self
    }

    /// Set the source PAN ID.
    pub fn set_src_pan_id(mut self, pan_id: u16) -> Self {
        self.frame
            .addressing_fields
            .get_or_insert_with(AddressingFieldsRepr::default)
            .src_pan_id = Some(pan_id);
        self
    }

    /// Set the destination address.
    ///
    /// # Note
    /// Based on the address, the addressing mode will be set.
    pub fn set_dst_address(mut self, address: Address) -> Self {
        self.frame.frame_control.dst_addressing_mode = address.into();
        self.frame
            .addressing_fields
            .get_or_insert_with(AddressingFieldsRepr::default)
            .dst_address = Some(address);
        self
    }

    /// Set the source address.
    ///
    /// # Note
    /// Based on the address, the addressing mode will be set.
    pub fn set_src_address(mut self, address: Address) -> Self {
        self.frame.frame_control.src_addressing_mode = address.into();
        self.frame
            .addressing_fields
            .get_or_insert_with(AddressingFieldsRepr::default)
            .src_address = Some(address);
        self
    }

    /// Add a header Information Element.
    ///
    /// # Note
    /// This method will enable the Information Elements Present bit in the
    /// frame control. The frame version will be set to IEEE 802.15.4-2020.
    pub fn add_header_information_element(mut self, ie: HeaderInformationElementRepr) -> Self {
        self.frame.frame_control.information_elements_present = true;
        self.frame
            .information_elements
            .get_or_insert_with(InformationElementsRepr::default)
            .header_information_elements
            .push(ie)
            .unwrap();

        self.frame.frame_control.frame_version = FrameVersion::Ieee802154_2020;

        self
    }

    /// Add a payload Information Element.
    ///
    /// # Note
    /// This method will enable the Information Elements Present bit in the
    /// frame control. The frame version will be set to IEEE 802.15.4-2020.
    pub fn add_payload_information_element(mut self, ie: PayloadInformationElementRepr) -> Self {
        self.frame.frame_control.information_elements_present = true;
        self.frame
            .information_elements
            .get_or_insert_with(InformationElementsRepr::default)
            .payload_information_elements
            .push(ie)
            .unwrap();

        self.frame.frame_control.frame_version = FrameVersion::Ieee802154_2020;

        self
    }

    /// Set the frame payload.
    pub fn set_payload(mut self, payload: &'p [u8]) -> Self {
        self.frame.payload = Some(payload);
        self
    }

    /// Finalize the frame builder, returning the frame representation.
    ///
    /// # Note
    /// This method will check and set if PAN ID compression is possible,
    /// depending on the frame version.
    pub fn finalize(mut self) -> Result<FrameRepr<'p>> {
        // Check if PAN ID compression is possible, depending on the frame version.
        if self.frame.frame_control.frame_version == FrameVersion::Ieee802154_2020 {
            let Some(addr) = self.frame.addressing_fields.as_mut() else {
                return Err(Error);
            };

            self.frame.frame_control.pan_id_compression = match (
                addr.dst_address,
                addr.src_address,
                addr.dst_pan_id,
                addr.src_pan_id,
            ) {
                (None, None, None, None) => false,
                (None, None, Some(_), None) => true,
                (Some(_), None, Some(_), None) => false,
                (None, Some(_), None, Some(_)) => false,
                (None, Some(_), None, None) => true,
                (Some(Address::Extended(_)), Some(Address::Extended(_)), Some(_), None) => false,
                (Some(Address::Extended(_)), Some(Address::Extended(_)), None, None) => true,
                (Some(Address::Short(_)), Some(Address::Short(_)), Some(dst), Some(src)) => {
                    if dst == src {
                        addr.src_pan_id = None;
                    }

                    dst == src
                }
                (Some(Address::Short(_)), Some(Address::Extended(_)), Some(dst), Some(src)) => {
                    if dst == src {
                        addr.src_pan_id = None;
                    }

                    dst == src
                }
                (Some(Address::Extended(_)), Some(Address::Short(_)), Some(dst), Some(src)) => {
                    if dst == src {
                        addr.src_pan_id = None;
                    }

                    dst == src
                }
                (Some(Address::Short(_)), Some(Address::Extended(_)), Some(_), None) => true,
                (Some(Address::Extended(_)), Some(Address::Short(_)), Some(_), None) => true,
                (Some(Address::Short(_)), Some(Address::Short(_)), Some(_), None) => true,
                _ => return Err(Error),
            };
        } else {
            if matches!(self.frame.frame_control.frame_type, FrameType::Ack) {
                // The sequence number is required for immediate acknowledgment frames.
                if self.frame.sequence_number.is_none() {
                    return Err(Error);
                }

                // The addressing fields are not present in acknowledgment frames.
                self.frame.addressing_fields = None;

                return Ok(self.frame);
            }

            // - If both destination and source addresses are present, and the PAN IDs are
            //   equal, then PAN ID compression is possible. In this case, the source PAN ID
            //   field is omitted and the PAN ID compression bit is set to 1. If PAN IDs are
            //   different, the PAN ID compression bit is set to 0.
            // - If only either the destination or source address is present, the PAN ID
            //   compression bit is set to 0. The PAN ID field of the single address shall
            //   be included in the frame.
            let Some(addr) = self.frame.addressing_fields.as_mut() else {
                return Err(Error);
            };

            match (
                addr.dst_address,
                addr.src_address,
                addr.dst_pan_id,
                addr.src_pan_id,
            ) {
                (Some(_), Some(_), Some(dst_pan_id), Some(src_pan_id)) => {
                    if dst_pan_id == src_pan_id {
                        self.frame.frame_control.pan_id_compression = true;
                        addr.src_pan_id = None;
                    }
                }
                (Some(_), None, Some(_), _) => {
                    self.frame.frame_control.pan_id_compression = false;
                    addr.src_pan_id = None;
                }
                (None, Some(_), _, Some(_)) => {
                    self.frame.frame_control.pan_id_compression = false;
                    addr.dst_pan_id = None;
                }
                _ => return Err(Error),
            }
        }

        Ok(self.frame)
    }
}
