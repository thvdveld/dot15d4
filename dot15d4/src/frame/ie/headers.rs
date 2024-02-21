//! IEEE 802.15.4 Header Information Element reader and writers.

use crate::time::Duration;
use dot15d4_macros::frame;

/// A reader/writer for the IEEE 802.15.4 Header Information Elements
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct HeaderInformationElement<T: AsRef<[u8]>> {
    data: T,
}

impl<T: AsRef<[u8]>> HeaderInformationElement<T> {
    /// Return the length field value.
    pub fn len(&self) -> usize {
        let b = &self.data.as_ref()[0..2];
        u16::from_le_bytes([b[0], b[1]]) as usize & 0b111111
    }

    /// Returns `true` when the length field is 0.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Return the [`HeaderElementId`].
    pub fn element_id(&self) -> HeaderElementId {
        let b = &self.data.as_ref()[0..2];
        let id = (u16::from_le_bytes([b[0], b[1]]) >> 7) & 0b11111111;
        HeaderElementId::from(id as u8)
    }

    /// Return the content of this Header Information Element.
    pub fn content(&self) -> &[u8] {
        &self.data.as_ref()[2..][..self.len()]
    }
}

impl<T: AsRef<[u8]>> core::fmt::Display for HeaderInformationElement<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
                write!(f, "{} {}", id, TimeCorrection::new(self.content()))
            }
            id => write!(f, "{:?}({:0x?})", id, self.content()),
        }
    }
}

/// Header Information Element ID.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum HeaderElementId {
    VendorSpecificHeader = 0x00,
    Csl = 0x1a,
    Rit = 0x1b,
    DsmePanDescriptor = 0x1c,
    RendezvousTime = 0x1d,
    TimeCorrection = 0x1e,
    ExtendedDsmePanDescriptor = 0x21,
    FragmentSequenceContextDescription = 0x22,
    SimplifiedSuperframeSpecification = 0x23,
    SimplifiedGtsSpecification = 0x24,
    LecimCapabilities = 0x25,
    TrleDescriptor = 0x26,
    RccCapabilities = 0x27,
    RccnDescriptor = 0x28,
    GlobalTime = 0x29,
    Da = 0x2b,
    HeaderTermination1 = 0x7e,
    HeaderTermination2 = 0x7f,
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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
            let ie = HeaderInformationElement {
                data: &self.data[self.offset..],
            };

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

#[frame]
pub struct TimeSynchronizationSpecification {
    #[bytes(8)]
    // TODO: use a Duration type
    /// Return the beacon timestamp field value.
    beacon_timestamp: &[u8],
    /// Return the beacon offset timestamp field value.
    beacon_offset_timestamp: u16,
}

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

#[frame]
pub struct RendezvousTime {
    /// Return the rendezvous time field value.
    rendezvous_time: u16,
    /// Return the wake-up interval field value.
    wake_up_interval: u16,
}

/// A reader/writer for the IEEE 802.15.4 Time Correction Header Information Element.
pub struct TimeCorrection<T: AsRef<[u8]>> {
    buffer: T,
}

impl<T: AsRef<[u8]>> TimeCorrection<T> {
    pub fn new(buffer: T) -> Self {
        Self { buffer }
    }

    #[allow(clippy::len_without_is_empty)]
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

impl<T: AsRef<[u8]>> core::fmt::Display for TimeCorrection<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
/// A reader/writer for the IEEE 802.15.4 Simplified Superframe Specification Header
/// Information Element.
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
/// A reader/writer for the IEEE 802.15.4 Superframe Specification Header Information Element.
pub struct SuperframeSpecification {
    #[bits(4)]
    beacon_order: u8,
    #[bits(4)]
    superframe_order: u8,
    #[bits(4)]
    final_cap_slot: u8,
    #[bits(1)]
    battery_life_extension: bool,
    #[bits(1)]
    _reserved: bool,
    #[bits(1)]
    pan_coordinator: bool,
    #[bits(1)]
    association_permit: bool,
}

#[frame]
#[derive(Debug)]
/// A reader/writer for the IEEE 802.15.4 CFP Specification Header Information Element.
pub struct CfpSpecification {
    #[bits(3)]
    gts_count: u8,
    #[bits(5)]
    first_cfp_slot: u8,
    #[bits(4)]
    last_cfp_slot: u8,
    #[bits(1)]
    gts_permit: bool,
}

bitflags::bitflags! {
    pub struct SupportedFrequencyBands: u16 {
        const BAND_161_MHZ = 0b0000_0000_0000_0001;
        const BAND_216_MHZ = 0b0000_0000_0000_0010;
        const BAND_217_MHZ = 0b0000_0000_0000_0100;
        const BAND_220_MHZ = 0b0000_0000_0000_1000;
        const BAND_450_MHZ = 0b0000_0000_0001_0000;
        const BAND_779_MHZ = 0b0000_0000_0010_0000;
        const BAND_800_MHZ = 0b0000_0000_0100_0000;
        const BAND_806_MHZ = 0b0000_0000_1000_0000;
        const BAND_896_MHZ = 0b0000_0001_0000_0000;
        const BAND_915_MHZ = 0b0000_0010_0000_0000;
        const BAND_928_MHZ = 0b0000_0100_0000_0000;
        const BAND_2450_MHZ = 0b0000_1000_0000_0000;
        const BAND_4965_MHZ = 0b0001_0000_0000_0000;
        const BAND_5800_MHZ = 0b0010_0000_0000_0000;
        const BAND_RESERVED = 0b1100_0000_0000_0000;
    }
}

bitflags::bitflags! {
    pub struct SupportedRccPhyAndModulation: u16 {
        const GMSK_9_6_KBPS = 0b0000_0000_0000_0001;
        const GMSK_19_2_KBPS = 0b0000_0000_0000_0010;
        const FOUR_FSK_9_6_KBPS = 0b0000_0000_0000_0100;
        const FOUR_FSK_19_2_KBPS = 0b0000_0000_0000_1000;
        const FOUR_FSK_38_4_KBPS = 0b0000_0000_0001_0000;
        const QPSK_16_KBPS = 0b0000_0000_0010_0000;
        const QPSK_32_KBPS = 0b0000_0000_0100_0000;
        const PI_4_DQPSK_16_KBPS = 0b0000_0000_1000_0000;
        const PI_4_DQPSK_32_KBPS = 0b0000_0001_0000_0000;
        const PI_4_DQPSK_64_KBPS = 0b0000_0010_0000_0000;
        const DSSS_DPSK = 0b0000_0100_0000_0000;
        const DSSS_BPSK = 0b0000_1000_0000_0000;
        const RESERVED = 0b1111_0000_0000_0000;
    }
}

bitflags::bitflags! {
    pub struct SupportedDsssDpskModulation: u16 {
        const RATE_300_KCPS = 0b0000_0000_0000_0001;
        const RATE_600_KCPS = 0b0000_0000_0000_0010;
        const RATE_800_KCPS = 0b0000_0000_0000_0100;
        const RATE_1_MCPS = 0b0000_0000_0000_1000;
        const RATE_1_6_MCPS = 0b0000_0000_0001_0000;
        const RATE_2_MCPS = 0b0000_0000_0010_0000;
        const RATE_3_MCPS = 0b0000_0000_0100_0000;
        const RATE_4_MCPS = 0b0000_0000_1000_0000;
        const SPREADING_11_CHIP = 0b0000_0001_0000_0000;
        const SPREADING_15_CHIP = 0b0000_0010_0000_0000;
        const SPREADING_20_CHIP = 0b0000_0100_0000_0000;
        const SPREADING_40_CHIP = 0b0000_1000_0000_0000;
        const DSSS_DBPSK = 0b0001_0000_0000_0000;
        const DSSS_DQPSK = 0b0010_0000_0000_0000;
        const RESERVED = 0b1100_0000_0000_0000;
    }
}

/// A high-level representation of a Header Information Element.
#[derive(Debug)]
pub enum HeaderInformationElementRepr {
    TimeCorrection(TimeCorrectionRepr),
    HeaderTermination1,
    HeaderTermination2,
}

impl HeaderInformationElementRepr {
    pub fn parse(ie: HeaderInformationElement<&[u8]>) -> Self {
        match ie.element_id() {
            HeaderElementId::TimeCorrection => Self::TimeCorrection(TimeCorrectionRepr {
                time_correction: TimeCorrection::new(ie.content()).time_correction(),
                nack: TimeCorrection::new(ie.content()).nack(),
            }),
            HeaderElementId::HeaderTermination1 => Self::HeaderTermination1,
            HeaderElementId::HeaderTermination2 => Self::HeaderTermination2,
            element => todo!("Received {element:?}"),
        }
    }
}

/// A high-level representation of a Time Correction Header Information Element.
#[derive(Debug)]
pub struct TimeCorrectionRepr {
    /// The time correction value in microseconds.
    pub time_correction: Duration,
    /// The negative acknowledgment flag.
    pub nack: bool,
}

/// A high-level representation of a Channel Hopping Header Information Element.
#[derive(Debug)]
pub struct ChannelHoppingRepr {
    /// The hopping sequence ID.
    pub hopping_sequence_id: u8,
}
