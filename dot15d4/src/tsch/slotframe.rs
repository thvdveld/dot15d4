use crate::frame::*;
use crate::time::*;

use bitflags::bitflags;
use heapless::Vec;

#[derive(Debug, Copy, Clone)]
pub(super) enum TschSlotType {
    Normal,
    Advertising,
    AdvertisingOnly,
}

#[derive(Debug, Clone)]
pub(super) struct TschSlot {
    slot_handle: u8,
    addr: Address,
    channel_offset: u16,
    link_options: TschLinkOption,
    link_type: TschSlotType,
}

pub(super) struct TschSlotFrame<const MAX_SLOTS: usize> {
    pub slots: Vec<Option<TschSlot>, MAX_SLOTS>,

    pub timings: TschTimeslotTimings,
}

impl<const MAX_SLOTS: usize> Default for TschSlotFrame<MAX_SLOTS> {
    fn default() -> Self {
        let mut slots = Vec::new();
        for _ in 0..MAX_SLOTS {
            slots.push(None).unwrap();
        }

        Self {
            slots,
            timings: TschTimeslotTimings::default(),
        }
    }
}

impl<const MAX_SLOTS: usize> TschSlotFrame<MAX_SLOTS> {
    pub fn minimal() -> Self {
        let mut f = Self::default();
        f.add_slot_at(
            0,
            TschSlot {
                slot_handle: 0,
                addr: Address::BROADCAST,
                channel_offset: 0,
                link_options: TschLinkOption::Tx
                    | TschLinkOption::Rx
                    | TschLinkOption::Shared
                    | TschLinkOption::TimeKeeping,
                link_type: TschSlotType::Advertising,
            },
        );

        f
    }

    pub fn add_slot_at(&mut self, offset: usize, slot: TschSlot) {
        assert!(offset < MAX_SLOTS);
        self.slots[offset] = Some(slot);
    }

    pub fn add_tx_frame(&mut self) {}
}

impl<const MAX_SLOTS: usize> core::fmt::Debug for TschSlotFrame<MAX_SLOTS> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "TSCH slot frame:")?;

        for slot in self.slots.iter().flatten() {
            writeln!(
                f,
                "  Link Options {:?}, type {:?}, offset {}, address {}",
                slot.link_options, slot.link_type, slot.channel_offset, slot.addr
            )?;
        }

        Ok(())
    }
}

pub(super) struct HoppingSequence<const MAX_HOPS: usize> {
    sequence: Vec<u8, MAX_HOPS>,
}

impl<const MAX_HOPS: usize> HoppingSequence<MAX_HOPS> {
    /// Return the default hopping sequences (16 channels, 16 hops):
    /// ```txt
    /// 16, 17, 23, 18, 26, 15, 25, 22, 19, 11, 12, 13, 24, 14, 20, 21
    /// ```
    pub fn sequence_16_16() -> Self {
        Self::new(&[16, 17, 23, 18, 26, 15, 25, 22, 19, 11, 12, 13, 24, 14, 20, 21])
    }

    /// Return the default hopping sequences (16 channels, 4 hops):
    /// ```txt
    /// 20, 26, 25, 26, 15, 15, 25, 20, 26, 15, 26, 25, 20, 15, 20, 25
    /// ```
    pub fn sequence_4_16() -> Self {
        Self::new(&[20, 26, 25, 26, 15, 15, 25, 20, 26, 15, 26, 25, 20, 15, 20, 25])
    }

    /// Return the default hopping sequences (4 channels, 4 hops):
    /// ```txt
    /// 15, 25, 26, 20
    /// ```
    pub fn sequence_4_4() -> Self {
        Self::new(&[15, 25, 26, 20])
    }

    /// Return the default hopping sequences (2 channels, 2 hops):
    /// ```txt
    /// 20, 25
    /// ```
    pub fn sequence_2_2() -> Self {
        Self::new(&[20, 25])
    }

    /// Return the default hopping sequences (1 channel, 1 hop):
    /// ```txt
    /// 20
    /// ```
    pub fn sequence_1_1() -> Self {
        Self::new(&[20])
    }

    /// Create a new hopping sequence from a slice of channels.
    pub fn new(s: &[u8]) -> Self {
        let mut sequence = Vec::new();
        sequence.extend_from_slice(s).unwrap();
        Self { sequence }
    }

    /// Return the channel for a given channel offset and ASN.
    pub fn get_channel(&self, offset: u8, asn: u64) -> u8 {
        ((self.sequence[offset as usize] as u64 + asn) % (self.sequence.len() as u64)) as u8
    }
}
