//! IEEE 802.15.4 Header Information Element reader and writers.

use crate::time::Duration;
use crate::{Error, Result};
use dot15d4_macros::frame;

/// A reader/writer for the IEEE 802.15.4 Header Information Elements
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct HeaderInformationElement<T: AsRef<[u8]>> {
    data: T,
}

impl<T: AsRef<[u8]>> HeaderInformationElement<T> {
    /// Create a new [`HeaderInformationElement`] reader/writer from a given
    /// buffer.
    ///
    /// # Errors
    ///
    /// Returns an error if the length field is less than 2.
    pub fn new(data: T) -> Result<Self> {
        let ie = Self::new_unchecked(data);

        if !ie.check_len() {
            return Err(Error);
        }

        Ok(ie)
    }

    /// Returns `false` if the buffer is too short to contain the Header
    /// Information Element.
    fn check_len(&self) -> bool {
        self.data.as_ref().len() >= 2
    }

    /// Create a new [`HeaderInformationElement`] reader/writer from a given
    /// buffer without length checking.
    pub fn new_unchecked(data: T) -> Self {
        Self { data }
    }

    /// Returns `true` when the length field is 0.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Return the length field value.
    pub fn len(&self) -> usize {
        let b = &self.data.as_ref()[0..2];
        u16::from_le_bytes([b[0], b[1]]) as usize & 0b1111_1110
    }

    /// Return the [`HeaderElementId`].
    pub fn element_id(&self) -> HeaderElementId {
        let b = &self.data.as_ref()[0..2];
        let id = (u16::from_le_bytes([b[0], b[1]]) >> 7) & 0b1111_1111;
        HeaderElementId::from(id as u8)
    }

    /// Return the content of this Header Information Element.
    pub fn content(&self) -> &[u8] {
        &self.data.as_ref()[2..][..self.len()]
    }
}

impl<T: AsRef<[u8]> + AsMut<[u8]>> HeaderInformationElement<T> {
    /// Clear the content of this Header Information Element.
    pub fn clear(&mut self) {
        self.data.as_mut().fill(0);
    }

    /// Set the length field.
    pub fn set_length(&mut self, len: u16) {
        const MASK: u16 = 0b1111_1110;

        let b = &mut self.data.as_mut()[0..2];
        let value = u16::from_le_bytes([b[0], b[1]]) & !MASK;
        let value = value | (len & MASK);
        b[0..2].copy_from_slice(&value.to_le_bytes());
    }

    /// Set the element ID field.
    pub fn set_element_id(&mut self, id: HeaderElementId) {
        const SHIFT: u16 = 7;
        const MASK: u16 = 0b0111_1111_1000_0000;

        let b = &mut self.data.as_mut()[0..2];
        let value = u16::from_le_bytes([b[0], b[1]]) & !MASK;
        let value = value | (((id as u16) << SHIFT) & MASK);
        b[0..2].copy_from_slice(&value.to_le_bytes());
    }

    /// Return the content of this Header Information Element.
    pub fn content_mut(&mut self) -> &mut [u8] {
        &mut self.data.as_mut()[2..]
    }
}

impl<T: AsRef<[u8]>> core::fmt::Display for HeaderInformationElement<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let id = self.element_id();
        match id {
            HeaderElementId::HeaderTermination1 | HeaderElementId::HeaderTermination2 => {
                write!(f, "{:?}", id)
            }
            HeaderElementId::SimplifiedSuperframeSpecification => {
                write!(
                    f,
                    "{} {:?}",
                    id,
                    SimplifiedSuperframeSpecification::new(self.content())
                )
            }
            HeaderElementId::TimeCorrection => {
                let Ok(tc) = TimeCorrection::new(self.content()) else {
                    return write!(f, "{:?}({:0x?})", id, self.content());
                };
                write!(f, "{} {}", id, tc)
            }
            id => write!(f, "{:?}({:0x?})", id, self.content()),
        }
    }
}

/// Header Information Element ID.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum HeaderElementId {
    /// Vendor specific header.
    VendorSpecificHeader = 0x00,
    /// Csl header.
    Csl = 0x1a,
    /// Rit header.
    Rit = 0x1b,
    /// Dsme Pan Descriptor header.
    DsmePanDescriptor = 0x1c,
    /// Rendezvous Time header.
    RendezvousTime = 0x1d,
    /// Time Correction header.
    TimeCorrection = 0x1e,
    /// Extended Dsme Pan Descriptor header.
    ExtendedDsmePanDescriptor = 0x21,
    /// Fragment Sequence Context Description header.
    FragmentSequenceContextDescription = 0x22,
    /// Simplified Superframe Specification header.
    SimplifiedSuperframeSpecification = 0x23,
    /// Simplified Gts Specification header.
    SimplifiedGtsSpecification = 0x24,
    /// Lecim Capabilities header.
    LecimCapabilities = 0x25,
    /// Trle Descriptor header.
    TrleDescriptor = 0x26,
    /// Rcc Capabilities header.
    RccCapabilities = 0x27,
    /// Rccn Descriptor header.
    RccnDescriptor = 0x28,
    /// Global Time header.
    GlobalTime = 0x29,
    /// Da header.
    Da = 0x2b,
    /// Header Termination 1.
    HeaderTermination1 = 0x7e,
    /// Header Termination 2.
    HeaderTermination2 = 0x7f,
    /// Unkown header.
    Unkown,
}

impl From<u8> for HeaderElementId {
    fn from(value: u8) -> Self {
        match value {
            0x00 => Self::VendorSpecificHeader,
            0x1a => Self::Csl,
            0x1b => Self::Rit,
            0x1c => Self::DsmePanDescriptor,
            0x1d => Self::RendezvousTime,
            0x1e => Self::TimeCorrection,
            0x21 => Self::ExtendedDsmePanDescriptor,
            0x22 => Self::FragmentSequenceContextDescription,
            0x23 => Self::SimplifiedSuperframeSpecification,
            0x24 => Self::SimplifiedGtsSpecification,
            0x25 => Self::LecimCapabilities,
            0x26 => Self::TrleDescriptor,
            0x27 => Self::RccCapabilities,
            0x28 => Self::RccnDescriptor,
            0x29 => Self::GlobalTime,
            0x2b => Self::Da,
            0x7e => Self::HeaderTermination1,
            0x7f => Self::HeaderTermination2,
            _ => Self::Unkown,
        }
    }
}

impl core::fmt::Display for HeaderElementId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::TimeCorrection => write!(f, "Time Correction"),
            _ => write!(f, "{:?}", self),
        }
    }
}

/// An [`Iterator`] over [`HeaderInformationElement`].
#[derive(Debug)]
pub struct HeaderInformationElementsIterator<'f> {
    pub(crate) data: &'f [u8],
    pub(crate) offset: usize,
    pub(crate) terminated: bool,
}

impl HeaderInformationElementsIterator<'_> {
    /// Returns the offset of the next Header Information Element.
    pub fn offset(&self) -> usize {
        self.offset
    }
}

impl<'f> Iterator for HeaderInformationElementsIterator<'f> {
    type Item = HeaderInformationElement<&'f [u8]>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.terminated {
            None
        } else {
            let ie = HeaderInformationElement::new(&self.data[self.offset..]).ok()?;

            self.terminated = matches!(
                ie.element_id(),
                HeaderElementId::HeaderTermination1 | HeaderElementId::HeaderTermination2
            );

            self.offset += ie.len() + 2;

            if self.offset >= self.data.len() {
                self.terminated = true;
            }

            Some(ie)
        }
    }
}

/// Vendor Specific Header Information Element.
#[frame]
#[derive(Debug)]
pub struct VendorSpecific {
    #[bytes(3)]
    /// Returns the vendor OUI field.
    vendor_oui: u32,

    #[bytes(0)]
    /// Returns the vendor specific payload.
    vendor_specific_payload: &[u8],
}

/// CSL Header Information Element.
#[frame]
#[derive(Debug)]
pub struct Csl {
    /// Return the CSL phase field value.
    csl_phase: u16,
    /// Return the CSL period field value.
    csl_period: u16,
    #[condition(self.buffer.as_ref().len() > 4)]
    /// Return the rendezvous time field value.
    rendezvous_time: u16,
}

/// RIT Header Information Element.
#[frame]
#[derive(Debug)]
pub struct Rit {
    /// Return the time to first listen field value.
    time_to_first_listen: u8,
    /// Return the number of repeat listen field value.
    number_of_repeat_listen: u8,
    /// Return the repeat listen interval field value.
    repeat_listen_interval: u16,
}

/// DSME Superframe Specification Header Information Element.
#[frame]
pub struct DsmeSuperframeSpecification {
    #[bits(4)]
    /// Return the multi superframe order field value.
    multi_superframe_order: u8,
    #[bits(1)]
    /// Return the channel diversity mode field value.
    channel_diversity_mode: bool,
    #[bits(1)]
    _reserved: bool,
    #[bits(1)]
    /// Return the cap reduction field value.
    cap_reduction: bool,
    #[bits(1)]
    /// Return the deferred beacon field value.
    deferred_beacon: bool,
}

/// Time Synchronization Specification Header Information Element.
#[frame]
pub struct TimeSynchronizationSpecification {
    #[bytes(8)]
    // TODO: use a Duration type
    /// Return the beacon timestamp field value.
    beacon_timestamp: &[u8],
    /// Return the beacon offset timestamp field value.
    beacon_offset_timestamp: u16,
}

/// Channel Hopping Specification Header Information Element.
#[frame]
pub struct ChannelHoppingSpecification {
    /// Return the hopping sequence ID field value.
    hopping_sequence_id: u8,
    /// Return the PAN coordinator BSN field value.
    pan_coordinator_bsn: u8,
    /// Return the channel offset field value.
    channel_offset: u16,
    /// Return the channel offset bitmap length field value.
    channel_offset_bitmap_length: u8,
    #[bytes(0)]
    /// Return the channel offset bitmap field value.
    channel_offset_bitmap: &[u8],
}

/// Renzdevous Time Header Information Element.
#[frame]
pub struct RendezvousTime {
    /// Return the rendezvous time field value.
    rendezvous_time: u16,
    /// Return the wake-up interval field value.
    wake_up_interval: u16,
}

/// A reader/writer for the IEEE 802.15.4 Time Correction Header Information
/// Element.
pub struct TimeCorrection<T: AsRef<[u8]>> {
    buffer: T,
}

impl<T: AsRef<[u8]>> TimeCorrection<T> {
    /// Create a new [`TimeCorrection`] reader/writer from a given buffer.
    ///
    /// # Errors
    ///
    /// Returns an error if the buffer is too short.
    pub fn new(buffer: T) -> Result<Self> {
        let ie = Self::new_unchecked(buffer);

        if !ie.check_len() {
            return Err(Error);
        }

        Ok(ie)
    }

    /// Returns `false` if the buffer is too short to contain the Time
    /// Correction field.
    fn check_len(&self) -> bool {
        self.buffer.as_ref().len() >= 2
    }

    /// Create a new [`TimeCorrection`] reader/writer from a given buffer
    /// without length checking.
    pub fn new_unchecked(buffer: T) -> Self {
        Self { buffer }
    }

    #[allow(clippy::len_without_is_empty)]
    /// Returns the length of the Time Correction field.
    pub const fn len(&self) -> usize {
        2
    }

    /// Return the time correction value in us.
    pub fn time_correction(&self) -> Duration {
        let b = &self.buffer.as_ref()[0..2];
        let time = ((u16::from_le_bytes([b[0], b[1]]) & 0x0fff) << 4) as i16;
        Duration::from_us((time >> 4) as i64)
    }

    /// Returns `true` when the frame is not acknowledged.
    pub fn nack(&self) -> bool {
        let b = &self.buffer.as_ref()[0..2];
        i16::from_le_bytes([b[0], b[1]]) & (0x8000u16 as i16) != 0
    }
}

impl<T: AsRef<[u8]> + AsMut<[u8]>> TimeCorrection<T> {
    /// Set the time correction value.
    pub fn set_time_correction(&mut self, time_correction: Duration) {
        let time = (((time_correction.as_us() as i16) << 4) >> 4) & 0x0fff;
        let b = &mut self.buffer.as_mut()[0..2];
        b[0..2].copy_from_slice(&time.to_le_bytes());
    }

    /// Set the NACK field.
    pub fn set_nack(&mut self, nack: bool) {
        let b = &mut self.buffer.as_mut()[0..2];
        let value = i16::from_le_bytes([b[0], b[1]]);
        if nack {
            b[0..2].copy_from_slice(&((value | (0x8000_u16 as i16)) as u16).to_le_bytes());
        } else {
            b[0..2].copy_from_slice(&((value & 0x7fff) as u16).to_le_bytes());
        }
    }
}

impl<T: AsRef<[u8]>> core::fmt::Display for TimeCorrection<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{}, nack: {}",
            self.time_correction(),
            self.nack() as usize
        )
    }
}

#[frame]
#[derive(Debug)]
/// A reader/writer for the IEEE 802.15.4 Simplified Superframe Specification
/// Header Information Element.
pub struct SimplifiedSuperframeSpecification {
    /// Returns the timestamp field value.
    timestamp: u16,
    #[bytes(2)]
    /// Returns the superframe specification field value.
    superframe_specification: SuperframeSpecification,
    #[bytes(2)]
    /// Returns the CFP specification field value.
    cfp_specification: CfpSpecification,
}

#[frame]
#[derive(Debug)]
/// A reader/writer for the IEEE 802.15.4 Superframe Specification Header
/// Information Element.
pub struct SuperframeSpecification {
    #[bits(4)]
    /// Return the beacon order field value.
    beacon_order: u8,
    #[bits(4)]
    /// Return the superframe order field value.
    superframe_order: u8,
    #[bits(4)]
    /// Return the final cap slot field value.
    final_cap_slot: u8,
    #[bits(1)]
    /// Return the battery life extension field value.
    battery_life_extension: bool,
    #[bits(1)]
    _reserved: bool,
    #[bits(1)]
    /// Return the PAN coordinator field value.
    pan_coordinator: bool,
    #[bits(1)]
    /// Return the association permit field value.
    association_permit: bool,
}

#[frame]
#[derive(Debug)]
/// A reader/writer for the IEEE 802.15.4 CFP Specification Header Information
/// Element.
pub struct CfpSpecification {
    #[bits(3)]
    /// Return the GTS field value.
    gts_count: u8,
    #[bits(5)]
    /// Return the first CFP slot field value.
    first_cfp_slot: u8,
    #[bits(4)]
    /// Return the last CFP slot field value.
    last_cfp_slot: u8,
    #[bits(1)]
    /// Return the CFP field value.
    gts_permit: bool,
}

bitflags::bitflags! {
    /// Supported Frequency Bands values.
    pub struct SupportedFrequencyBands: u16 {
        /// 169 MHz band.
        const BAND_161_MHZ = 0b0000_0000_0000_0001;
        /// 216 MHz band.
        const BAND_216_MHZ = 0b0000_0000_0000_0010;
        /// 217 MHz band.
        const BAND_217_MHZ = 0b0000_0000_0000_0100;
        /// 220 MHz band.
        const BAND_220_MHZ = 0b0000_0000_0000_1000;
        /// 450 MHz band.
        const BAND_450_MHZ = 0b0000_0000_0001_0000;
        /// 779 MHz band.
        const BAND_779_MHZ = 0b0000_0000_0010_0000;
        /// 800 MHz band.
        const BAND_800_MHZ = 0b0000_0000_0100_0000;
        /// 806 MHz band.
        const BAND_806_MHZ = 0b0000_0000_1000_0000;
        /// 896 MHz band.
        const BAND_896_MHZ = 0b0000_0001_0000_0000;
        /// 915 MHz band.
        const BAND_915_MHZ = 0b0000_0010_0000_0000;
        /// 928 MHz band.
        const BAND_928_MHZ = 0b0000_0100_0000_0000;
        /// 2450 MHz band.
        const BAND_2450_MHZ = 0b0000_1000_0000_0000;
        /// 4965 MHz band.
        const BAND_4965_MHZ = 0b0001_0000_0000_0000;
        /// 5800 MHz band.
        const BAND_5800_MHZ = 0b0010_0000_0000_0000;
        /// Reserved.
        const BAND_RESERVED = 0b1100_0000_0000_0000;
    }
}

bitflags::bitflags! {
    /// Supported Modulation values.
    pub struct SupportedRccPhyAndModulation: u16 {
        /// GMSK 9.6 kbps.
        const GMSK_9_6_KBPS = 0b0000_0000_0000_0001;
        /// GMSK 19.2 kbps.
        const GMSK_19_2_KBPS = 0b0000_0000_0000_0010;
        /// 4 FSK 9.6 kbps.
        const FOUR_FSK_9_6_KBPS = 0b0000_0000_0000_0100;
        /// 4 FSK 19.2 kbps.
        const FOUR_FSK_19_2_KBPS = 0b0000_0000_0000_1000;
        /// 4 FSK 38.4 kbps.
        const FOUR_FSK_38_4_KBPS = 0b0000_0000_0001_0000;
        /// QPSK 16 kbps.
        const QPSK_16_KBPS = 0b0000_0000_0010_0000;
        /// QPSK 32 kbps.
        const QPSK_32_KBPS = 0b0000_0000_0100_0000;
        /// PI/4 DQPSK 16 kbps.
        const PI_4_DQPSK_16_KBPS = 0b0000_0000_1000_0000;
        /// PI/4 DQPSK 32 kbps.
        const PI_4_DQPSK_32_KBPS = 0b0000_0001_0000_0000;
        /// PI/4 DQPSK 64 kbps.
        const PI_4_DQPSK_64_KBPS = 0b0000_0010_0000_0000;
        /// DSSS DPSK.
        const DSSS_DPSK = 0b0000_0100_0000_0000;
        /// DSSS BPSK.
        const DSSS_BPSK = 0b0000_1000_0000_0000;
        /// Reserved.
        const RESERVED = 0b1111_0000_0000_0000;
    }
}

bitflags::bitflags! {
    /// Supported DSSS DPSK Modulation values.
    pub struct SupportedDsssDpskModulation: u16 {
        /// 100 Kcps.
        const RATE_300_KCPS = 0b0000_0000_0000_0001;
        /// 600 Kcps.
        const RATE_600_KCPS = 0b0000_0000_0000_0010;
        /// 800 Kcps.
        const RATE_800_KCPS = 0b0000_0000_0000_0100;
        /// 1 Mcps.
        const RATE_1_MCPS = 0b0000_0000_0000_1000;
        /// 1.6 Mcps.
        const RATE_1_6_MCPS = 0b0000_0000_0001_0000;
        /// 2 Mcps.
        const RATE_2_MCPS = 0b0000_0000_0010_0000;
        /// 3 Mcps.
        const RATE_3_MCPS = 0b0000_0000_0100_0000;
        /// 4 Mcps.
        const RATE_4_MCPS = 0b0000_0000_1000_0000;
        /// 11 chip spreading.
        const SPREADING_11_CHIP = 0b0000_0001_0000_0000;
        /// 15 chip spreading.
        const SPREADING_15_CHIP = 0b0000_0010_0000_0000;
        /// 20 chip spreading.
        const SPREADING_20_CHIP = 0b0000_0100_0000_0000;
        /// 40 chip spreading.
        const SPREADING_40_CHIP = 0b0000_1000_0000_0000;
        /// DSSS DBPSK.
        const DSSS_DBPSK = 0b0001_0000_0000_0000;
        /// DSSS DQPSK.
        const DSSS_DQPSK = 0b0010_0000_0000_0000;
        /// Reserved.
        const RESERVED = 0b1100_0000_0000_0000;
    }
}
