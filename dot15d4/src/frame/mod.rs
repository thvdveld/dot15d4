//! Zero-copy read and write structures for handling IEEE 802.15.4 MAC frames.
//!
//! Each reader contains the following functions:
//! - [`new`]: Create a new reader.
//! - [`check_len`]: Check if the buffer is long enough to contain a valid frame.
//! - [`new_unchecked`]: Create a new reader without checking the buffer length.
//!
//! The most important reader is the [`Frame`] reader, which is used to read a full IEEE 802.15.4
//! frame. The reader provides the following functions:
//! - [`frame_control`]: returns a [`FrameControl`] reader.
//! - [`sequence_number`]: returns the sequence number if not suppressed.
//! - [`addressing`]: returns an [`AddressingFields`] reader.
//! - [`auxiliary_security_header`]: returns an [`AuxiliarySecurityHeader`] reader.
//! - [`information_elements`]: returns an [`InformationElements`] reader.
//! - [`payload`]: returns the payload of the frame.
//!
//! ## Reading a frame
//! For an incoming frame, use the [`Frame`] structure to read its content.
//! ```
//! # use dot15d4::frame::{
//! #   Frame,
//! #   FrameControl,
//! #   FrameType,
//! #   AddressingFields,
//! #   NestedInformationElementsIterator,
//! #   PayloadGroupId,
//! #   NestedSubId,
//! #   NestedSubIdShort,
//! #   TschTimeslot,
//! # };
//! # let frame: [u8; 35] = [
//! #     0x40, 0xeb, 0xcd, 0xab, 0xff, 0xff, 0x01, 0x00, 0x01, 0x00, 0x01, 0x00, 0x01, 0x00,
//! #     0x00, 0x3f, 0x11, 0x88, 0x06, 0x1a, 0x0e, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x1c,
//! #     0x00, 0x01, 0xc8, 0x00, 0x01, 0x1b, 0x00,
//! # ];
//! let frame = Frame::new(&frame).unwrap();
//! let fc = frame.frame_control();
//! let src_addr = frame.addressing().src_address(&fc);
//! let dst_addr = frame.addressing().dst_address(&fc);
//!
//! assert_eq!(fc.frame_type(), FrameType::Beacon);
//!
//! let Some(ie) = frame.information_elements() else { return; };
//!
//! for payload in ie.payload_information_elements() {
//!      if matches!(payload.group_id(), PayloadGroupId::Mlme) {
//!         for nested in payload.nested_information_elements() {
//!              match nested.sub_id() {
//!                  NestedSubId::Short(NestedSubIdShort::TschTimeslot) => {
//!                      let time_slot = TschTimeslot::new(nested.content());
//!                      assert_eq!(time_slot.id(), 0);
//!                  }
//!                  _ => (),
//!              }
//!          }
//!      }
//!  }
//! ```
//!
//! ## Writing a frame
//!
//! __Work in progress!__
//!
//! ## Information Elements
//!
//! The IEEE 802.15.4 standard defines a set of Information Elements (IEs) that can be included in
//! the frame. These IEs are used to provide additional information about the frame, such as
//! timestamping, channel hopping, and more. The IEs are divided into two groups: Header IEs and
//! Payload IEs. Calling [`information_elements`] on a [`Frame`] reader returns an [`InformationElements`] reader.
//! The reader provides access to the Header and Payload IEs, via the [`header_information_elements`] and [`payload_information_elements`] functions.
//!
//! ### Header Information Elements
//!
//! The Header IEs are located in the frame header, and are used to provide information about the
//! frame itself. The following IEs are defined in the standard:
//!
//! - [x] [`VendorSpecific`]
//! - [x] [`Csl`]
//! - [x] [`Rit`]
//! - [ ] `DsmePanDescriptor`
//! - [x] [`RendezvousTime`]
//! - [x] [`TimeCorrection`]
//! - [ ] `ExtededDsmePanDescriptor`
//! - [ ] `FragmentSequencecontextDescription`
//! - [x] [`SimplifiedSuperframeSpecification`]
//! - [ ] `SimplifiedGtsSpecification`
//! - [ ] `LecimCapabilities`
//! - [ ] `TrleDescriptor`
//! - [ ] `RccCapabilities`
//! - [ ] `RccnDescriptor`
//! - [ ] `GlobalTime`
//! - [ ] `Da`
//! - [x] [`HeaderTermination1`]
//! - [x] [`HeaderTermination2`]
//!
//! ### Payload Information Elements
//!
//! The Payload IEs are located in the frame payload, and are used to provide information about the
//! payload itself. The following IEs are defined in the standard:
//!
//! - [ ] `Esdu`
//! - [x] `Mlme`: The MLME group contains a set of nested IEs. Call [`nested_information_elements`]
//! to get an iterator over the nested IEs.
//! - [ ] `VendorSpecific`
//! - [ ] `PayloadTermination`
//!
//! ### Nested Information Elements
//!
//! Some IEs contain nested IEs. The [`NestedInformationElementsIterator`] provides an iterator
//! over the nested IEs. The iterator is used to parse the nested IEs.
//!
//! The Nested IEs are split into two groups: Short and Long. The following short IEs are defined in
//! the standard:
//!
//! - [x] [`TschSynchronization`]
//! - [x] [`TschSlotframeAndLink`]
//! - [x] [`TschTimeslot`]
//! - [ ] `HoppingTiming`
//! - [ ] `EnhancedBeaconFilter`
//! - [ ] `MacMetrics`
//! - [ ] `AllMacMetrics`
//! - [ ] `CoexistenceSpecification`
//! - [ ] `SunDeviceCapabilities`
//! - [ ] `SunFskGenericPhy`
//! - [ ] `ModeSwitchParameter`
//! - [ ] `PhyParameterChange`
//! - [ ] `OQpskPhyMode`
//! - [ ] `PcaAllocation`
//! - [ ] `LecimDsssOperatingMode`
//! - [ ] `LecimFskOperatingMode`
//! - [ ] `TvwsPhyOperatingMode`
//! - [ ] `TvwsDeviceCapabilities`
//! - [ ] `TvwsDeviceCategory`
//! - [ ] `TvwsDeviceIdentification`
//! - [ ] `TvwsDeviceLocation`
//! - [ ] `TvwsChannelInformationQuery`
//! - [ ] `TvwsChannelInformationSource`
//! - [ ] `Ctm`
//! - [ ] `Timestamp`
//! - [ ] `TimestampDifference`
//! - [ ] `TmctpSpecification`
//! - [ ] `RccPhyOperatingMode`
//! - [ ] `LinkMargin`
//! - [ ] `RsGfskDeviceCapabilities`
//! - [ ] `MultiPhy`
//! - [ ] `VendorSpecific`
//! - [ ] `Srm`
//!
//! The following long IEs are defined in the standard:
//!
//! - [ ] `VendorSpecificNested`
//! - [x] [`ChannelHopping`]
//!
//! [`new`]: Frame::new
//! [`check_len`]: Frame::check_len
//! [`new_unchecked`]: Frame::new_unchecked
//! [`frame_control`]: Frame::frame_control
//! [`sequence_number`]: Frame::sequence_number
//! [`addressing`]: Frame::addressing
//! [`auxiliary_security_header`]: Frame::auxiliary_security_header
//! [`information_elements`]: Frame::information_elements
//! [`payload`]: Frame::payload
//! [`HeaderTermination1`]: HeaderElementId::HeaderTermination1
//! [`HeaderTermination2`]: HeaderElementId::HeaderTermination2
//! [`header_information_elements`]: InformationElements::header_information_elements
//! [`payload_information_elements`]: InformationElements::payload_information_elements
//! [`nested_information_elements`]: PayloadInformationElement::nested_information_elements

#[cfg(test)]
mod tests;

mod frame_control;
pub use frame_control::FrameControl;
pub use frame_control::FrameControlRepr;
pub use frame_control::FrameType;
pub use frame_control::FrameVersion;

mod aux_sec_header;
pub use aux_sec_header::AuxiliarySecurityHeader;

mod addressing;
pub use addressing::Address;
pub use addressing::AddressingFields;
pub use addressing::AddressingFieldsRepr;
pub use addressing::AddressingMode;

mod ie;
pub use ie::*;

use crate::time::Duration;
use heapless::Vec;

/// An error that can occur when reading or writing an IEEE 802.15.4 frame.
#[derive(Debug, Clone, Copy)]
pub struct Error;

/// A type alias for `Result<T, frame::Error>`.
pub type Result<T> = core::result::Result<T, Error>;

/// A reader/writer for an IEEE 802.15.4 frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Frame<T: AsRef<[u8]>> {
    buffer: T,
}

impl<T: AsRef<[u8]>> Frame<T> {
    /// Create a new IEEE 802.15.4 frame reader/writer.
    ///
    /// ## Note
    /// This is a combination of [`Frame::new_unchecked`] and [`Frame::check_len`].
    pub fn new(data: T) -> Result<Self> {
        let frame = Self::new_unchecked(data);

        if !frame.check_len() {
            return Err(Error);
        }

        Ok(frame)
    }

    /// Create a new IEEE 802.15.4 frame reader/writer, without checking the buffer length.
    pub fn new_unchecked(data: T) -> Self {
        Self { buffer: data }
    }

    /// Check if the buffer is long enough to contain a valid IEEE 802.15.4 frame.
    ///
    /// ## Note
    /// This function does not check the validity of the frame. It only checks if the buffer is
    /// long enough to contain a valid frame, such that frame accessors do not panic.
    pub fn check_len(&self) -> bool {
        let buffer = self.buffer.as_ref();

        if buffer.len() < 2 {
            return false;
        }

        let fc = self.frame_control();

        if !fc.sequence_number_suppression() && buffer.len() < 3 {
            return false;
        }

        true
    }

    /// Return a [`FrameControl`] reader.
    pub fn frame_control(&self) -> FrameControl<&'_ [u8]> {
        FrameControl::new(&self.buffer.as_ref()[..2])
    }

    /// Return the sequence number if not suppressed.
    pub fn sequence_number(&self) -> Option<u8> {
        if self.frame_control().sequence_number_suppression() {
            None
        } else {
            Some(self.buffer.as_ref()[2])
        }
    }

    /// Return an [`AddressingFields`] reader.
    pub fn addressing(&self) -> AddressingFields<&'_ [u8]> {
        if self.frame_control().sequence_number_suppression() {
            AddressingFields::new(&self.buffer.as_ref()[2..])
        } else {
            AddressingFields::new(&self.buffer.as_ref()[3..])
        }
    }

    /// Return an [`AuxiliarySecurityHeader`] reader.
    pub fn auxiliary_security_header(&self) -> Option<AuxiliarySecurityHeader<&'_ [u8]>> {
        if self.frame_control().security_enabled() {
            let start = 2
                + (!self.frame_control().sequence_number_suppression() as usize)
                + self.addressing().len(&self.frame_control());
            Some(AuxiliarySecurityHeader::new(&self.buffer.as_ref()[start..]))
        } else {
            None
        }
    }

    /// Return an [`InformationElements`] reader.
    pub fn information_elements(&self) -> Option<InformationElements<&'_ [u8]>> {
        if self.frame_control().information_elements_present() {
            let start = 2
                + (!self.frame_control().sequence_number_suppression() as usize)
                + self.addressing().len(&self.frame_control());
            Some(InformationElements::new(&self.buffer.as_ref()[start..]))
        } else {
            None
        }
    }
}

impl<'f, T: AsRef<[u8]> + ?Sized> Frame<&'f T> {
    /// Return the payload of the frame.
    pub fn payload(&self) -> Option<&'f [u8]> {
        let mut offset = 0;
        offset += 2;

        if !self.frame_control().sequence_number_suppression() {
            offset += 1;
        }

        offset += self.addressing().len(&self.frame_control());

        if self.frame_control().security_enabled() {
            offset += self.auxiliary_security_header().unwrap().len();
        }

        if self.frame_control().information_elements_present() {
            offset += self.information_elements().unwrap().len();
        }

        if self.buffer.as_ref().len() <= offset {
            return None;
        }

        Some(&self.buffer.as_ref()[offset..])
    }
}

impl<T: AsRef<[u8]> + AsMut<[u8]>> Frame<T> {
    /// Set the Frame Control field values in the buffer, based on the given [`FrameControlRepr`].
    pub fn set_frame_control(&mut self, fc: &FrameControlRepr) {
        let mut w = FrameControl::new(&mut self.buffer.as_mut()[..2]);
        w.set_frame_type(fc.frame_type);
        w.set_security_enabled(fc.security_enabled);
        w.set_frame_pending(fc.frame_pending);
        w.set_ack_request(fc.ack_request);
        w.set_pan_id_compression(fc.pan_id_compression);
        w.set_sequence_number_suppression(fc.sequence_number_suppression);
        w.set_information_elements_present(fc.information_elements_present);
        w.set_dst_addressing_mode(fc.dst_addressing_mode);
        w.set_src_addressing_mode(fc.src_addressing_mode);
        w.set_frame_version(fc.frame_version);
    }

    /// Set the Sequence Number field value in the buffer.
    pub fn set_sequence_number(&mut self, sequence_number: u8) {
        // Set the sequence number suppression bit to false.
        let mut w = FrameControl::new(&mut self.buffer.as_mut()[..2]);
        w.set_sequence_number_suppression(false);

        self.buffer.as_mut()[2] = sequence_number;
    }

    /// Set the Addressing field values in the buffer, based on the given [`AddressingFieldsRepr`].
    pub fn set_addressing_fields(&mut self, addressing_fields: &AddressingFieldsRepr) {
        let start = 2 + (!self.frame_control().sequence_number_suppression() as usize);

        let mut w = AddressingFields::new(&mut self.buffer.as_mut()[start..]);
        w.write_fields(addressing_fields);
    }

    /// Set the Auxiliary Security Header field values in the buffer, based on the given _.
    pub fn set_aux_sec_header(&mut self) {
        todo!();
    }

    /// Set the Information Elements field values in the buffer, based on the given _.
    pub fn set_information_elements(&mut self, ie: &InformationElementsRepr) {
        todo!();
    }

    /// Set the payload of the frame.
    pub fn set_payload(&mut self, payload: &[u8]) {
        let mut offset = 0;
        offset += 2;

        if !self.frame_control().sequence_number_suppression() {
            offset += 1;
        }

        offset += self.addressing().len(&self.frame_control());

        if self.frame_control().security_enabled() {
            offset += self.auxiliary_security_header().unwrap().len();
        }

        if self.frame_control().information_elements_present() {
            offset += self.information_elements().unwrap().len();
        }

        self.buffer.as_mut()[offset..].copy_from_slice(payload);
    }
}

impl<'f, T: AsRef<[u8]> + ?Sized> core::fmt::Display for Frame<&'f T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fc = self.frame_control();
        write!(f, "{}", fc)?;
        if !fc.sequence_number_suppression() {
            writeln!(f, "Sequence number: {}", self.sequence_number().unwrap())?;
        }

        let addressing = self.addressing();
        addressing.fmt(f, &fc)?;

        if fc.security_enabled() {
            todo!();
        }

        if let Some(ie) = self.information_elements() {
            writeln!(f, "Information Elements")?;
            for header_ie in ie.header_information_elements() {
                writeln!(f, "  {}", header_ie)?;
            }

            for payload_ie in ie.payload_information_elements() {
                writeln!(f, "  {}", payload_ie)?;
            }
        }

        if let Some(payload) = self.payload() {
            writeln!(f, "Payload")?;
            writeln!(f, "  {:0x?}", payload)?;
        }

        Ok(())
    }
}

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

impl<'r, 'f: 'r> FrameRepr<'f> {
    /// Parse an IEEE 802.15.4 frame into a high-level representation.
    pub fn parse(frame: &'r Frame<&'f [u8]>) -> FrameRepr<'f> {
        Self {
            frame_control: FrameControlRepr::parse(frame.frame_control()),
            sequence_number: frame.sequence_number(),
            addressing_fields: AddressingFieldsRepr::parse(
                frame.addressing(),
                frame.frame_control(),
            ),
            information_elements: frame.information_elements().map(|ie| {
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
            payload: frame.payload().unwrap_or(&[]),
        }
    }
}
