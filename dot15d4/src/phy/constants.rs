#![allow(dead_code)]

// Constants from section 11.3, Table 11-1, PHY constants
/// The maximum PSDU size (in octets) the PHY shall be able to receive.
pub const MAX_PHY_PACKET_SIZE: u32 = 127;
/// RX-to-TX or TX-to-RX turnaround time (in symbol periods), as defined in
/// 10.2.2 and 10.2.3.
pub const TURNAROUND_TIME: u32 = 12;
/// The time required to perform CCA detection in symbol periods.
pub const CCA_TIME: u32 = 8;
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
        assert_eq!(SYMBOL_RATE_INV_US, 16);
    }
}
