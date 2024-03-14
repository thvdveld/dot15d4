#![allow(dead_code)]

// Constants from section 11.3, Table 11-1, PHY constants
/// The maximum PSDU size (in octets) the PHY shall be able to receive.
pub const MAX_PHY_PACKET_SIZE: u32 = 127;
/// RX-to-TX or TX-to-RX turnaround time (in symbol periods), as defined in
/// 10.2.2 and 10.2.3.
pub const TURNAROUND_TIME: u32 = 12;
/// The time required to perform CCA detection in symbol periods.
pub const CCA_TIME: u32 = 8;

// /// The delay between the start of the SFD and the LEIP, as described in
// /// 18.6.
// const A_LEIP_DELAY_TIME: u32 = 0.815 ms

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

/// The symbol rate of IEEE 802.15.4 on 2.5 Ghz (symbols/s)
// pub const SYMBOL_RATE: u32 = 250_000;
pub const SYMBOL_RATE: u32 = 62_500;
/// The symbol rate of IEEE 802.15.4 on 2.5 Ghz (µs/symbol)
pub const SYMBOL_RATE_INV_US: u32 = 1_000_000 / SYMBOL_RATE;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inv_symbol_rate() {
        assert_eq!(SYMBOL_RATE_INV_US, 4);
    }
}
