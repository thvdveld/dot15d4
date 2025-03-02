#![allow(dead_code)]

/// PAN Information Base (PIB) specified by PHY sublayer
pub struct Pib {
    /// The RF channel to use for all following
    /// transmissions and receptions
    pub current_channel: u8,
    /// his is the current PHY channel page. This is used in conjunction with
    /// `current_channel` to uniquely identify the channel currently being used.
    pub current_page: u8,
    /// The transmit power of the device in dBm.
    pub tx_power: i32,
}

impl Default for Pib {
    fn default() -> Self {
        Self {
            current_channel: 26,
            current_page: 0,
            // TODO: handling tx power in PIB
            tx_power: 0,
        }
    }
}
