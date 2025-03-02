#![allow(dead_code)]
pub use customizable::*;

use crate::phy::constants::{CCA_TIME, TURNAROUND_TIME};
pub const BROADCAST_PAN_ID: u16 = 0xffff;
// Constants of section 8.4.2, Table 8-93, MAC constants
/// The number of symbols forming a superframe slot when the superframe order is
/// equal to zero, as described in 6.2.1.
pub const BASE_SLOT_DURATION: u32 = 60;
/// The number of symbols forming a superframe when the superframe order is
/// equal to zero.
pub const BASE_SUPERFRAME_DURATION: u32 = BASE_SLOT_DURATION * NUM_SUPERFRAME_SLOTS;
/// The number of superframes in which a GTS descriptor exists in the beacon
/// frame of the PAN coordinator.
pub const GTS_DESC_PERSISTENCE_TIME: u32 = 4;
/// The number of consecutive lost beacons that will cause the MAC sublayer of a
/// receiving device to declare a loss of synchronization.
pub const MAX_LOST_BEACONS: u32 = 4;
/// The maximum size of an MPDU, in octets, that can be followed by a SIFS
/// period.
pub const MAX_SIFS_FRAME_SIZE: u32 = 18;
/// The minimum number of symbols forming the CAP. This ensures that MAC
/// commands can still be transferred to devices when GTSs are being used.
///
/// An exception to this minimum shall be allowed for the accommodation of the
/// temporary increase in the beacon frame length needed to perform GTS
/// maintenance, as described in 7.3.1.5. Additional restrictions apply when PCA
/// is enabled, as described in 6.2.5.4.
pub const MIN_CAP_LENGTH: u32 = 440;
/// The number of slots contained in any superframe.
pub const NUM_SUPERFRAME_SLOTS: u32 = 440;
/// The number of symbols forming the basic time period used by the CSMA-CA
/// algorithm.
pub const UNIT_BACKOFF_PERIOD: u32 = TURNAROUND_TIME + CCA_TIME;
/// The number of symbols forming an RCCN superframe slot.
pub const RCCN_BASE_SLOT_DURATION: u32 = 60;

#[cfg(test)]
mod customizable {
    #![allow(dead_code)]
    use crate::{phy::constants::SYMBOL_RATE_INV_US, time::Duration};

    // XXX These are just random numbers I picked by fair dice roll; what should
    // they be?
    pub const MAC_MIN_BE: u8 = 0;
    pub const MAC_MAX_BE: u8 = 8;
    pub const MAC_MAX_CSMA_BACKOFFS: u8 = 16;
    pub const MAC_UNIT_BACKOFF_DURATION: Duration =
        Duration::from_us((super::UNIT_BACKOFF_PERIOD * SYMBOL_RATE_INV_US) as i64);
    pub const MAC_MAX_FRAME_RETIES: u8 = 3; // 0-7
    pub const MAC_INTER_FRAME_TIME: Duration = Duration::from_us(1000); // TODO: XXX
    /// AIFS=1ms, for SUN PHY, LECIM PHY, TVWS PHY
    pub const MAC_AIFS_PERIOD: Duration = Duration::from_us(1000);
    pub const MAC_SIFS_PERIOD: Duration = Duration::from_us(1000); // TODO: SIFS=XXX
    pub const MAC_LIFS_PERIOD: Duration = Duration::from_us(10_000); // TODO: LIFS=XXX
                                                                     // PAN Id
    pub const MAC_PAN_ID: u16 = 0xffff;
    pub const MAC_IMPLICIT_BROADCAST: bool = false;
}

#[cfg(not(test))]
mod customizable {
    #![allow(unused)]
    include!(concat!(env!("OUT_DIR"), "/config.rs"));
}
