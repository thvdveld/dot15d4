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

    /// Set the Addressing field values in the buffer, based on the given [`AddressingFieldsRepr`].
    pub fn set_addressing_fields(&mut self, addressing_fields: &AddressingFieldsRepr) {
        let start = 2 + (!self.frame_control().sequence_number_suppression() as usize);

        let mut writer = AddressingFields::new(&mut self.buffer.as_mut()[start..]);
        writer.write_fields(addressing_fields);
    }

    /// Set the Auxiliary Security Header field values in the buffer, based on the given _.
    pub fn set_aux_secu_header(&mut self) {
        todo!();
    }

    /// Set the Information Elements field values in the buffer, based on the given _.
    pub fn set_information_elements(&mut self) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ack_frame() {
        let frame = [
            0x02, 0x2e, 0x37, 0xcd, 0xab, 0x02, 0x00, 0x02, 0x00, 0x02, 0x00, 0x02, 0x00, 0x02,
            0x0f, 0xe1, 0x8f,
        ];

        let frame = Frame::new(&frame).unwrap();

        let fc = frame.frame_control();
        assert_eq!(fc.frame_type(), FrameType::Ack);
        assert!(!fc.security_enabled());
        assert!(!fc.frame_pending());
        assert!(!fc.ack_request());
        assert!(!fc.pan_id_compression());
        assert!(!fc.sequence_number_suppression());
        assert!(fc.information_elements_present());
        assert!(fc.dst_addressing_mode() == AddressingMode::Extended);
        assert!(fc.frame_version() == FrameVersion::Ieee802154);
        assert!(fc.src_addressing_mode() == AddressingMode::Absent);

        assert!(frame.sequence_number() == Some(55));

        let addressing = frame.addressing();
        assert_eq!(addressing.dst_pan_id(&fc), Some(0xabcd));
        assert_eq!(
            addressing.dst_address(&fc),
            Some(Address::Extended([
                0x00, 0x02, 0x00, 0x02, 0x00, 0x02, 0x00, 0x02
            ]))
        );
        assert_eq!(addressing.src_pan_id(&fc), None);
        assert_eq!(addressing.src_address(&fc), Some(Address::Absent));

        let ie = frame.information_elements().unwrap();
        let mut headers = ie.header_information_elements();

        let time_correction = headers.next().unwrap();
        assert_eq!(
            time_correction.element_id(),
            HeaderElementId::TimeCorrection
        );

        let time_correction = TimeCorrection::new(time_correction.content());
        assert_eq!(time_correction.len(), 2);
        assert_eq!(
            time_correction.time_correction(),
            crate::time::Duration::from_us(-31)
        );
        assert!(time_correction.nack());
    }

    #[test]
    fn data_frame() {
        let frame = [
            0x41, 0xd8, 0x01, 0xcd, 0xab, 0xff, 0xff, 0xc7, 0xd9, 0xb5, 0x14, 0x00, 0x4b, 0x12,
            0x00, 0x2b, 0x00, 0x00, 0x00,
        ];

        let frame = Frame::new(&frame).unwrap();

        let fc = frame.frame_control();
        assert_eq!(fc.frame_type(), FrameType::Data);
        assert!(!fc.security_enabled());
        assert!(!fc.frame_pending());
        assert!(!fc.ack_request());
        assert!(fc.pan_id_compression());

        assert!(fc.dst_addressing_mode() == AddressingMode::Short);
        assert!(fc.frame_version() == FrameVersion::Ieee802154_2006);
        assert!(fc.src_addressing_mode() == AddressingMode::Extended);

        assert!(frame.payload().unwrap() == &[0x2b, 0x00, 0x00, 0x00][..]);
    }

    #[test]
    fn enhanced_beacon() {
        let frame: [u8; 35] = [
            0x40, 0xeb, 0xcd, 0xab, 0xff, 0xff, 0x01, 0x00, 0x01, 0x00, 0x01, 0x00, 0x01, 0x00,
            0x00, 0x3f, 0x11, 0x88, 0x06, 0x1a, 0x0e, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x1c,
            0x00, 0x01, 0xc8, 0x00, 0x01, 0x1b, 0x00,
        ];

        let frame = Frame::new(&frame).unwrap();
        let fc = frame.frame_control();
        assert_eq!(fc.frame_type(), FrameType::Beacon);
        assert!(!fc.security_enabled());
        assert!(!fc.frame_pending());
        assert!(!fc.ack_request());
        assert!(fc.pan_id_compression());
        assert!(fc.sequence_number_suppression());
        assert!(fc.information_elements_present());
        assert_eq!(fc.dst_addressing_mode(), AddressingMode::Short);
        assert_eq!(fc.src_addressing_mode(), AddressingMode::Extended);
        assert_eq!(fc.frame_version(), FrameVersion::Ieee802154);

        let addressing = frame.addressing();
        assert_eq!(addressing.dst_pan_id(&fc), Some(0xabcd),);
        assert_eq!(addressing.src_pan_id(&fc), None,);
        assert_eq!(addressing.dst_address(&fc), Some(Address::BROADCAST));
        assert_eq!(
            addressing.src_address(&fc),
            Some(Address::Extended([
                0x00, 0x01, 0x00, 0x01, 0x00, 0x01, 0x00, 0x01
            ]))
        );
        assert_eq!(addressing.len(&fc), 12);

        let ie = frame.information_elements().unwrap();

        let mut headers = ie.header_information_elements();
        let terminator = headers.next().unwrap();
        assert_eq!(terminator.element_id(), HeaderElementId::HeaderTermination1);
        assert_eq!(terminator.len(), 0);

        assert_eq!(headers.next(), None);

        let mut payloads = ie.payload_information_elements();

        let mlme = payloads.next().unwrap();
        assert_eq!(mlme.group_id(), PayloadGroupId::Mlme);
        assert_eq!(mlme.len() + 2, 19);
        assert_eq!(payloads.next(), None);

        let mut nested_iterator = NestedInformationElementsIterator::new(mlme.content());

        let tsch_sync = nested_iterator.next().unwrap();
        assert_eq!(
            tsch_sync.sub_id(),
            NestedSubId::Short(NestedSubIdShort::TschSynchronization)
        );
        assert_eq!(
            TschSynchronization::new(tsch_sync.content()).absolute_slot_number(),
            14
        );
        assert_eq!(
            TschSynchronization::new(tsch_sync.content()).join_metric(),
            0
        );

        let tsch_timeslot = nested_iterator.next().unwrap();
        assert_eq!(
            tsch_timeslot.sub_id(),
            NestedSubId::Short(NestedSubIdShort::TschTimeslot)
        );
        assert_eq!(TschTimeslot::new(tsch_timeslot.content()).id(), 0);

        let channel_hopping = nested_iterator.next().unwrap();
        assert_eq!(
            channel_hopping.sub_id(),
            NestedSubId::Long(NestedSubIdLong::ChannelHopping)
        );
        assert_eq!(
            ChannelHopping::new(channel_hopping.content()).hopping_sequence_id(),
            0
        );

        let slotframe = nested_iterator.next().unwrap();
        assert_eq!(
            slotframe.sub_id(),
            NestedSubId::Short(NestedSubIdShort::TschSlotframeAndLink)
        );
        assert_eq!(
            TschSlotframeAndLink::new(slotframe.content()).number_of_slot_frames(),
            0
        );

        assert_eq!(nested_iterator.next(), None);
        assert!(frame.payload().is_none());
    }

    #[test]
    fn write_buffer() {
        let mut buffer = [0u8; 127];

        let mut writer = Frame::new_unchecked(&mut buffer);
        writer.set_frame_control(&FrameControlRepr {
            frame_type: FrameType::Beacon,
            security_enabled: false,
            frame_pending: false,
            ack_request: false,
            pan_id_compression: true,
            sequence_number_suppression: true,
            information_elements_present: true,
            dst_addressing_mode: AddressingMode::Short,
            src_addressing_mode: AddressingMode::Extended,
            frame_version: FrameVersion::Ieee802154,
        });

        writer.set_addressing_fields(&AddressingFieldsRepr {
            dst_pan_id: None,
            src_pan_id: None,
            dst_address: None,
            src_address: None,
        });

        assert_eq!(&buffer[..2], &[0x40, 0xeb]);
    }
}
