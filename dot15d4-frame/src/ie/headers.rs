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
    /// Return the beacon order field value.
    #[bits(4)]
    #[into(BeaconOrder)]
    beacon_order: u8,
    /// Return the superframe order field value.
    #[bits(4)]
    #[into(SuperframeOrder)]
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

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
#[repr(u8)]
/// Indicates the frequency at which the beacon is transmitted.
pub enum BeaconOrder {
    /// The beacon is transmitted at an interval:
    /// `base_super_frame_duration * 2^{beacon_order}`.
    Order(u8),
    /// The beacon is transmitted on demand.
    OnDemand,
}

impl From<u8> for BeaconOrder {
    fn from(value: u8) -> Self {
        match value {
            value @ 0..=14 => Self::Order(value),
            _ => Self::OnDemand,
        }
    }
}
impl From<BeaconOrder> for u8 {
    fn from(value: BeaconOrder) -> Self {
        match value {
            BeaconOrder::Order(value) => value,
            BeaconOrder::OnDemand => 15,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
#[repr(u8)]
///  The lenght of the active portion of the superframe.
pub enum SuperframeOrder {
    /// The superframe duration is calculated with:
    /// `base_super_frame_duration * 2^{superframe_order}`
    Order(u8),
    /// The superframe is inactive after the the beacon.
    Inactive,
}

impl From<u8> for SuperframeOrder {
    fn from(value: u8) -> Self {
        match value {
            value @ 0..=14 => Self::Order(value),
            _ => Self::Inactive,
        }
    }
}
impl From<SuperframeOrder> for u8 {
    fn from(value: SuperframeOrder) -> Self {
        match value {
            SuperframeOrder::Order(value) => value,
            SuperframeOrder::Inactive => 15,
        }
    }
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

#[frame]
#[derive(Debug)]
/// Guaranteed Time Slot specification.
pub struct GtsSpecification {
    #[bits(3)]
    /// GTS descriptor count.
    descriptor_count: u8,
    #[bits(4)]
    _reserved: u8,
    /// GTS is permitted.
    #[bits(1)]
    gts_permit: bool,
}

/// Guaranteed Timeslot Descriptor
#[frame(no_constructor)]
pub struct GtsSlot {
    /// Short address of the intended device.
    #[bytes(2)]
    #[into(crate::Address)]
    short_address: &[u8],
    /// Superframe slot at which the GTS is to begin.
    #[bits(4)]
    starting_slot: u8,
    /// Number of contiguous superframe slots over which the GTS is active.
    #[bits(4)]
    length: u8,

    /// The GTS slot direction.
    #[field]
    direction: GtsDirection,
}

impl<T: AsRef<[u8]>> GtsSlot<T> {
    /// Create a new [`#name`] reader/writer from a given buffer.
    pub fn new(buffer: T, direction: GtsDirection) -> Result<Self> {
        let s = Self::new_unchecked(buffer, direction);

        if !s.check_len() {
            return Err(Error);
        }

        Ok(s)
    }

    /// Returns `false` if the buffer is too short to contain this structure.
    fn check_len(&self) -> bool {
        self.buffer.as_ref().len() >= Self::size()
    }

    /// Create a new [`#name`] reader/writer from a given buffer without length
    /// checking.
    pub fn new_unchecked(buffer: T, direction: GtsDirection) -> Self {
        Self { buffer, direction }
    }
}

impl<T: AsRef<[u8]>> GtsSpecification<T> {
    /// Return a [`GtsSlotIterator`].
    pub fn slots(&self) -> GtsSlotIterator {
        if self.descriptor_count() == 0 {
            GtsSlotIterator {
                data: &[],
                count: 0,
                terminated: true,
            }
        } else {
            GtsSlotIterator {
                data: &self.buffer.as_ref()[1..]
                    [..1 + self.descriptor_count() as usize * GtsSlot::<T>::size()],
                count: 0,
                terminated: false,
            }
        }
    }
}

/// An [`Iterator`] over GTS slots.
pub struct GtsSlotIterator<'f> {
    data: &'f [u8],
    count: usize,
    terminated: bool,
}

impl<'f> Iterator for GtsSlotIterator<'f> {
    type Item = GtsSlot<&'f [u8]>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.terminated {
            None
        } else {
            const L: usize = GtsSlot::<&[u8]>::size();
            if 1 + self.count * L >= self.data.len() {
                return None;
            }

            let direction = GtsDirection::from((self.data[0] >> self.count) & 0b1);
            let descriptor = GtsSlot::new(&self.data[1 + self.count * L..], direction).ok()?;

            self.count += 1;
            if 1 + self.count * L >= self.data.len() {
                self.terminated = true;
            }

            Some(descriptor)
        }
    }
}

/// GTS direciton.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum GtsDirection {
    /// GTS Receive direction.
    Receive,
    /// GTS Transmit direction.
    Transmit,
}

impl From<u8> for GtsDirection {
    fn from(value: u8) -> Self {
        match value {
            0b0 => Self::Receive,
            _ => Self::Transmit,
        }
    }
}

impl From<GtsDirection> for u8 {
    fn from(value: GtsDirection) -> Self {
        match value {
            GtsDirection::Receive => 0b0,
            GtsDirection::Transmit => 0b1,
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn superframe_specification() {
        let data = [0xff, 0x0f];
        let ie = SuperframeSpecification::new(&data).unwrap();
        assert_eq!(ie.beacon_order(), BeaconOrder::OnDemand);
        assert_eq!(ie.superframe_order(), SuperframeOrder::Inactive);
        assert_eq!(ie.final_cap_slot(), 0x0f);
        assert!(!ie.battery_life_extension());
        assert!(!ie.pan_coordinator());
        assert!(!ie.association_permit());
    }

    #[test]
    fn gts_specification() {
        use crate::Address;

        let data = [0b0000_0000];
        let gts = GtsSpecification::new(&data).unwrap();
        assert_eq!(gts.descriptor_count(), 0);
        assert!(!gts.gts_permit());

        let data = [0b1000_0000];
        let gts = GtsSpecification::new(&data).unwrap();
        assert_eq!(gts.descriptor_count(), 0);
        assert!(gts.gts_permit());

        assert!(gts.slots().next().is_none());

        let data = [0x82, 0x01, 0x34, 0x12, 0x11, 0x78, 0x56, 0x14];
        let gts = GtsSpecification::new(&data).unwrap();

        assert!(gts.gts_permit());
        assert_eq!(gts.descriptor_count(), 2);

        let mut slots = gts.slots();
        let s0 = slots.next().unwrap();
        assert_eq!(s0.short_address(), Address::Short([0x34, 0x12]));
        assert_eq!(s0.starting_slot(), 1);
        assert_eq!(s0.length(), 1);
        assert_eq!(s0.direction(), GtsDirection::Transmit);

        let s1 = slots.next().unwrap();
        assert_eq!(s1.short_address(), Address::Short([0x78, 0x56]));
        assert_eq!(s1.starting_slot(), 4);
        assert_eq!(s1.length(), 1);
        assert_eq!(s1.direction(), GtsDirection::Receive);

        assert!(slots.next().is_none());
    }

    #[test]
    fn gts_slot() {
        use crate::Address;
        let data = [0xab, 0xcd, 0b0101_1010];
        let slot = GtsSlot::new(&data[..], GtsDirection::Transmit).unwrap();
        assert_eq!(slot.short_address(), Address::Short([0xab, 0xcd]));
        assert_eq!(slot.starting_slot(), 0b1010);
        assert_eq!(slot.length(), 0b0101);
        assert_eq!(slot.direction(), GtsDirection::Transmit);
    }

    #[test]
    fn header_iformation_element_id() {
        assert_eq!(
            HeaderElementId::from(0x00),
            HeaderElementId::VendorSpecificHeader
        );
        assert_eq!(HeaderElementId::from(0x1a), HeaderElementId::Csl);
        assert_eq!(HeaderElementId::from(0x1b), HeaderElementId::Rit);
        assert_eq!(
            HeaderElementId::from(0x1c),
            HeaderElementId::DsmePanDescriptor
        );
        assert_eq!(HeaderElementId::from(0x1d), HeaderElementId::RendezvousTime);
        assert_eq!(HeaderElementId::from(0x1e), HeaderElementId::TimeCorrection);
        assert_eq!(
            HeaderElementId::from(0x21),
            HeaderElementId::ExtendedDsmePanDescriptor
        );
        assert_eq!(
            HeaderElementId::from(0x22),
            HeaderElementId::FragmentSequenceContextDescription
        );
        assert_eq!(
            HeaderElementId::from(0x23),
            HeaderElementId::SimplifiedSuperframeSpecification
        );
        assert_eq!(
            HeaderElementId::from(0x24),
            HeaderElementId::SimplifiedGtsSpecification
        );
        assert_eq!(
            HeaderElementId::from(0x25),
            HeaderElementId::LecimCapabilities
        );
        assert_eq!(HeaderElementId::from(0x26), HeaderElementId::TrleDescriptor);
        assert_eq!(
            HeaderElementId::from(0x27),
            HeaderElementId::RccCapabilities
        );
        assert_eq!(HeaderElementId::from(0x28), HeaderElementId::RccnDescriptor);
        assert_eq!(HeaderElementId::from(0x29), HeaderElementId::GlobalTime);
        assert_eq!(HeaderElementId::from(0x2b), HeaderElementId::Da);
        assert_eq!(
            HeaderElementId::from(0x7e),
            HeaderElementId::HeaderTermination1
        );
        assert_eq!(
            HeaderElementId::from(0x7f),
            HeaderElementId::HeaderTermination2
        );
        assert_eq!(HeaderElementId::from(0x80), HeaderElementId::Unkown);
    }
}
