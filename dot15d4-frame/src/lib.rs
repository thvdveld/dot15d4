//! Zero-copy read and write structures for handling IEEE 802.15.4 MAC frames.
//!
//! Each reader contains the following functions:
//! - [`new`]: Create a new reader.
//! - [`check_len`]: Check if the buffer is long enough to contain a valid
//!   frame.
//! - [`new_unchecked`]: Create a new reader without checking the buffer length.
//!
//! The most important reader is the [`Frame`] reader, which is used to read a
//! full IEEE 802.15.4 frame. The reader provides the following functions:
//! - [`frame_control`]: returns a [`FrameControl`] reader.
//! - [`sequence_number`]: returns the sequence number if not suppressed.
//! - [`addressing`]: returns an [`AddressingFields`] reader.
//! - [`auxiliary_security_header`]: returns an [`AuxiliarySecurityHeader`]
//!   reader.
//! - [`information_elements`]: returns an [`InformationElements`] reader.
//! - [`payload`]: returns the payload of the frame.
//!
//! ## Reading a frame
//! For an incoming frame, use the [`Frame`] structure to read its content.
//! ```
//! # use dot15d4_frame::{
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
//! let src_addr = frame.addressing().unwrap().src_address();
//! let dst_addr = frame.addressing().unwrap().dst_address();
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
//!                      let time_slot = TschTimeslot::new(nested.content()).unwrap();
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
//! The IEEE 802.15.4 standard defines a set of Information Elements (IEs) that
//! can be included in the frame. These IEs are used to provide additional
//! information about the frame, such as timestamping, channel hopping, and
//! more. The IEs are divided into two groups: Header IEs and Payload IEs.
//! Calling [`information_elements`] on a [`Frame`] reader returns an
//! [`InformationElements`] reader. The reader provides access to the Header and
//! Payload IEs, via the [`header_information_elements`] and
//! [`payload_information_elements`] functions.
//!
//! ### Header Information Elements
//!
//! The Header IEs are located in the frame header, and are used to provide
//! information about the frame itself. The following IEs are defined in the
//! standard:
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
//! The Payload IEs are located in the frame payload, and are used to provide
//! information about the payload itself. The following IEs are defined in the
//! standard:
//!
//! - [ ] `Esdu`
//! - [x] `Mlme`: The MLME group contains a set of nested IEs. Call
//!   [`nested_information_elements`]
//!   to get an iterator over the nested IEs.
//! - [ ] `VendorSpecific`
//! - [ ] `PayloadTermination`
//!
//! ### Nested Information Elements
//!
//! Some IEs contain nested IEs. The [`NestedInformationElementsIterator`]
//! provides an iterator over the nested IEs. The iterator is used to parse the
//! nested IEs.
//!
//! The Nested IEs are split into two groups: Short and Long. The following
//! short IEs are defined in the standard:
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

#![cfg_attr(not(any(test, feature = "std")), no_std)]
#![deny(missing_docs)]
#![deny(unsafe_code)]

#[cfg(test)]
mod tests;

mod time;

mod frame_control;
pub use frame_control::*;

mod aux_sec_header;
pub use aux_sec_header::*;

mod addressing;
pub use addressing::*;

mod ie;
pub use ie::*;

mod repr;
pub use repr::*;

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
    /// Create a new [`Frame`] reader/writer from a given buffer.
    ///
    /// # Errors
    ///
    /// Returns an error if the buffer is too short to contain a valid frame.
    pub fn new(buffer: T) -> Result<Self> {
        let b = Self::new_unchecked(buffer);

        if !b.check_len() {
            return Err(Error);
        }

        let fc = b.frame_control();

        if fc.security_enabled() {
            return Err(Error);
        }

        if fc.frame_type() == FrameType::Unknown {
            return Err(Error);
        }

        if fc.frame_version() == FrameVersion::Unknown {
            return Err(Error);
        }

        if fc.dst_addressing_mode() == AddressingMode::Unknown {
            return Err(Error);
        }

        if fc.src_addressing_mode() == AddressingMode::Unknown {
            return Err(Error);
        }

        Ok(b)
    }

    /// Returns `false` if the buffer is too short to contain a valid frame.
    fn check_len(&self) -> bool {
        let buffer = self.buffer.as_ref();

        if buffer.len() < 2 || buffer.len() > 127 {
            return false;
        }

        let fc = self.frame_control();

        if !fc.sequence_number_suppression() && buffer.len() < 3 {
            return false;
        }

        true
    }

    /// Create a new [`Frame`] reader/writer from a given buffer without length
    /// checking.
    pub fn new_unchecked(buffer: T) -> Self {
        Self { buffer }
    }

    /// Return a [`FrameControl`] reader.
    pub fn frame_control(&self) -> FrameControl<&'_ [u8]> {
        FrameControl::new_unchecked(&self.buffer.as_ref()[..2])
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
    pub fn addressing(&self) -> Option<AddressingFields<&'_ [u8], &'_ [u8]>> {
        let fc = self.frame_control();

        if matches!(fc.frame_type(), FrameType::Ack)
            && matches!(
                fc.frame_version(),
                FrameVersion::Ieee802154_2003 | FrameVersion::Ieee802154_2006
            )
        {
            // Immediate Acks don't have addressing fields.
            return None;
        }

        if fc.sequence_number_suppression() {
            AddressingFields::new(&self.buffer.as_ref()[2..], fc).ok()
        } else {
            AddressingFields::new(&self.buffer.as_ref()[3..], fc).ok()
        }
    }

    /// Return an [`AuxiliarySecurityHeader`] reader.
    pub fn auxiliary_security_header(&self) -> Option<AuxiliarySecurityHeader<&'_ [u8]>> {
        let fc = self.frame_control();

        if fc.security_enabled() {
            let mut offset = 2;

            offset += !fc.sequence_number_suppression() as usize;

            if let Some(af) = self.addressing() {
                offset += af.len();
            }

            Some(AuxiliarySecurityHeader::new(
                &self.buffer.as_ref()[offset..],
            ))
        } else {
            None
        }
    }

    /// Return an [`InformationElements`] reader.
    pub fn information_elements(&self) -> Option<InformationElements<&'_ [u8]>> {
        let fc = self.frame_control();
        if fc.information_elements_present() {
            let mut offset = 2;
            offset += !fc.sequence_number_suppression() as usize;

            if let Some(af) = self.addressing() {
                offset += af.len();
            }

            Some(InformationElements::new(&self.buffer.as_ref()[offset..]).ok()?)
        } else {
            None
        }
    }
}

impl<'f, T: AsRef<[u8]> + ?Sized> Frame<&'f T> {
    /// Return the payload of the frame.
    pub fn payload(&self) -> Option<&'f [u8]> {
        let fc = self.frame_control();

        let mut offset = 0;
        offset += 2;

        if !fc.sequence_number_suppression() {
            offset += 1;
        }

        if let Some(af) = self.addressing() {
            offset += af.len();
        }

        if fc.security_enabled() {
            offset += self.auxiliary_security_header().unwrap().len();
        }

        if fc.information_elements_present() {
            if let Some(ie) = self.information_elements() {
                offset += ie.len();
            }
        }

        if self.buffer.as_ref().len() <= offset {
            return None;
        }

        Some(&self.buffer.as_ref()[offset..])
    }
}

impl<T: AsRef<[u8]> + AsMut<[u8]>> Frame<T> {
    /// Set the Frame Control field values in the buffer, based on the given
    /// [`FrameControlRepr`].
    pub fn set_frame_control(&mut self, fc: &FrameControlRepr) {
        let mut w = FrameControl::new_unchecked(&mut self.buffer.as_mut()[..2]);
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

    /// Get a mutable reference to the Frame Control fields
    pub fn frame_control_mut(&mut self) -> FrameControl<&'_ mut [u8]> {
        FrameControl::new_unchecked(&mut self.buffer.as_mut()[..2])
    }

    /// Set the Sequence Number field value in the buffer.
    pub fn set_sequence_number(&mut self, sequence_number: u8) {
        // Set the sequence number suppression bit to false.
        let mut w = FrameControl::new_unchecked(&mut self.buffer.as_mut()[..2]);
        w.set_sequence_number_suppression(false);

        self.buffer.as_mut()[2] = sequence_number;
    }

    /// Set the Addressing field values in the buffer, based on the given
    /// [`AddressingFieldsRepr`].
    pub fn set_addressing_fields(&mut self, addressing_fields: &AddressingFieldsRepr) {
        let start = 2 + (!self.frame_control().sequence_number_suppression() as usize);

        let (fc, addressing) = self.buffer.as_mut().split_at_mut(start);
        let mut w = AddressingFields::new_unchecked(addressing, FrameControl::new_unchecked(fc));
        w.write_fields(addressing_fields);
    }

    /// Set the Auxiliary Security Header field values in the buffer, based on
    /// the given _.
    pub fn set_aux_sec_header(&mut self) {
        todo!();
    }

    /// Set the Information Elements field values in the buffer, based on the
    /// given _.
    pub fn set_information_elements(
        &mut self,
        ie: &InformationElementsRepr,
        contains_payload: bool,
    ) {
        let mut offset = 2;
        offset += !self.frame_control().sequence_number_suppression() as usize;

        if let Some(af) = self.addressing() {
            offset += af.len();
        }

        ie.emit(&mut self.buffer.as_mut()[offset..], contains_payload);
    }

    /// Set the payload of the frame.
    pub fn set_payload(&mut self, payload: &[u8]) {
        let mut offset = 0;
        offset += 2;

        if !self.frame_control().sequence_number_suppression() {
            offset += 1;
        }

        if let Some(af) = self.addressing() {
            offset += af.len();
        }

        if self.frame_control().security_enabled() {
            offset += self.auxiliary_security_header().unwrap().len();
        }

        if self.frame_control().information_elements_present() {
            offset += self.information_elements().unwrap().len();
        }

        self.buffer.as_mut()[offset..].copy_from_slice(payload);
    }
}

impl<T: AsRef<[u8]> + ?Sized> core::fmt::Display for Frame<&T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let fc = self.frame_control();
        write!(f, "{}", fc)?;
        if !fc.sequence_number_suppression() {
            writeln!(f, "Sequence number: {}", self.sequence_number().unwrap())?;
        }

        if let Some(af) = self.addressing() {
            write!(f, "{af}")?;
        }

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
