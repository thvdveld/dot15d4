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
//! #   EnhancedBeaconFrame,
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
//! let frame = EnhancedBeaconFrame::new(&frame).unwrap();
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
#![no_std]
#![deny(missing_docs)]
#![deny(unsafe_code)]
#![allow(unused)]

#[cfg(any(feature = "std", test))]
#[macro_use]
extern crate std;

#[cfg(test)]
mod tests;

mod frames;
pub use frames::BeaconFrame;
pub use frames::DataFrame;
pub use frames::EnhancedBeaconFrame;
pub use frames::Frame;

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
