//! Time-Slotted Channel Hopping (TSCH) implementation.
//!
//! This module implements the Time-Slotted Channel Hopping (TSCH) mode of IEEE 802.15.4-2015. It
//! is a medium access control (MAC) protocol that uses time division multiple access (TDMA) with
//! channel hopping (CH).
//!
//! ## Time synchronization
//! TSCH requires time synchronization between nodes. This is done by sending Enhanced Beacons
//! periodically. The Enhanced Beacon contains the current ASN and the current hopping sequence.
//! The Enhanced Beacon is sent in the first slot of the slotframe. Upon receiving an Enhanced
//! Beacon, the node synchronizes its slotframe to the ASN and hopping sequence of the Enhanced
//! Beacon. The node then schedules the next radio wake-up at the start of the next slot.
//! The Enhanced Beacon also contains a join priority metric. This metric is used to determine
//! which node should be the time source. The node with the lowest join priority metric is
//! selected as the time source.
//!
//! ## Slotframe
//! The slotframe is a list of slots. There are different types of slots:
//! - Transmit slot: the node transmits a frame in this slot.
//! - Receive slot: the node listens for a frame in this slot.
//! - Shared slot: the slot is shared between multiple nodes.
//! - Time keeping slot: the node wakes up in this slot to synchronize its slotframe.
//! - Priority slot: the node wakes up in this slot to transmit a frame with a higher
//!  priority.
//!
//! The time source is the node with the lowest

mod frame;
mod frame_buffer;
mod neighbor;
mod slotframe;

use frame::*;
use frame_buffer::*;
use neighbor::*;
use slotframe::*;

use crate::frame::*;
use crate::phy::Device;
use crate::time::*;

#[macro_use]
use crate::utils::log;
use bitflags::bitflags;
use heapless::Vec;

pub struct TschDevice<'d, D: Device> {
    device: &'d mut D,
    inner: TschDeviceInner,
}

impl<'d, D: Device> TschDevice<'d, D> {
    pub fn new(device: &'d mut D) -> Self {
        device.disable();
        Self {
            device,
            inner: TschDeviceInner::new(),
        }
    }

    pub fn poll(&mut self, now: Instant) {
        self.device.receive(|frame, timestamp| {
            self.inner.process_frame(frame, timestamp);
        });
    }

    pub fn poll_at(&self) -> u64 {
        todo!();
    }
}

struct TschDeviceInner {
    associated: bool,
    asn: u64,
    neighbors: NeighborTable<16>,
    time_source: Option<Neighbor>,
    slot_frame: TschSlotFrame<7>,
    hopping_sequence_id: u8,
    current_slot_start: Instant,
    last_sync_asn: u64,
    last_sync_timestamp: Instant,
    tx_frame_queue: FrameBuffer<TschFrame, 8>,
    rx_frame_queue: FrameBuffer<TschFrame, 8>,
}

impl TschDeviceInner {
    fn new() -> Self {
        Self {
            neighbors: NeighborTable::default(),
            asn: 0,
            time_source: None,
            slot_frame: TschSlotFrame::minimal(),
            hopping_sequence_id: 0,
            associated: false,
            current_slot_start: Instant::now(),
            last_sync_asn: 0,
            last_sync_timestamp: Instant::now(),
            tx_frame_queue: FrameBuffer::new(),
            rx_frame_queue: FrameBuffer::new(),
        }
    }

    /// Return the next slot start.
    fn next_slot_start(&self, now: Instant, asn: u64) -> Instant {
        // Calculate the next slot where we need to do something. This is indicated by the
        // TschSlotOptions.

        let current_i = (asn % self.slot_frame.slots.len() as u64) as usize;

        for i in 0..self.slot_frame.slots.len() {
            let j = (i + current_i) % self.slot_frame.slots.len();

            if self.slot_frame.slots[j].is_some() {
                let i = if i == 0 {
                    i + self.slot_frame.slots.len()
                } else {
                    i
                };

                return now + self.slot_frame.timings.time_slot_length() * i;
            }
        }

        now + self.slot_frame.timings.time_slot_length()
    }

    fn process_frame(&mut self, frame: &[u8], timestamp: Instant) {
        let frame = Frame::new(frame).unwrap();

        match frame.frame_control().frame_type() {
            FrameType::Beacon if self.associated => self.process_enhanced_beacon(frame, timestamp),
            FrameType::Beacon => self.associate(frame, timestamp),
            FrameType::Ack => self.process_ack(frame, timestamp),
            _ => {}
        }
    }

    fn process_enhanced_beacon(&mut self, frame: Frame<&[u8]>, timestamp: Instant) {
        let Some(ie) = frame.information_elements() else {
            return;
        };

        let Some(eb) = EnhancedBeacon::parse(&frame) else {
            return;
        };
    }

    fn process_ack(&mut self, frame: Frame<&[u8]>, timestamp: Instant) {
        todo!();
    }

    fn associate(&mut self, frame: Frame<&[u8]>, timestamp: Instant) {
        let Some(ie) = frame.information_elements() else {
            return;
        };

        let Some(eb) = EnhancedBeacon::parse(&frame) else {
            return;
        };

        trace!(
            "TSCH associated ASN {}, jp {}, timeslot ID {}",
            eb.asn,
            eb.join_metric,
            eb.hopping_sequence_id
        );

        // Synchronize time frame with the time source.
        let slot_start = timestamp - self.slot_frame.timings.tx_offset();

        trace!("TSCH synchronized slotframe to ASN {}", eb.asn);
        critical_section::with(|_| {
            // TODO: synchronize time and ASN.
            self.current_slot_start = slot_start;
            self.last_sync_asn = eb.asn;
            self.last_sync_timestamp = Instant::now();
        });

        let next_radio_wake = self.next_slot_start(timestamp, eb.asn);
        self.asn = eb.asn
            + next_radio_wake.as_us() as u64
                / self.slot_frame.timings.time_slot_length().as_us() as u64;
        trace!("TSCH next radio wake at {}", next_radio_wake);
        trace!(" --> waking up in {}", next_radio_wake - timestamp);
        trace!(" --> Next ASN {}", self.asn);
    }
}

impl Default for TschDeviceInner {
    fn default() -> Self {
        Self::new()
    }
}
