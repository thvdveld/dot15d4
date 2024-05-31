/// Export all user definable constants
pub use constants::*;

#[cfg(test)]
mod constants {
    #![allow(dead_code)]
    use crate::csma::{SYMBOL_RATE_INV_US, UNIT_BACKOFF_PERIOD};
    use crate::time::Duration;

    // XXX These are just random numbers I picked by fair dice roll; what should
    // they be?
    pub const MAC_MIN_BE: u16 = 0;
    pub const MAC_MAX_BE: u16 = 8;
    pub const MAC_MAX_CSMA_BACKOFFS: u16 = 16;
    pub const MAC_UNIT_BACKOFF_DURATION: Duration =
        Duration::from_us((UNIT_BACKOFF_PERIOD * SYMBOL_RATE_INV_US) as i64);
    pub const MAC_MAX_FRAME_RETIES: u16 = 3; // 0-7
    pub const MAC_INTER_FRAME_TIME: Duration = Duration::from_us(1000); // TODO: XXX
    /// AIFS=1ms, for SUN PHY, LECIM PHY, TVWS PHY
    pub const MAC_AIFS_PERIOD: Duration = Duration::from_us(1000);
    pub const MAC_SIFS_PERIOD: Duration = Duration::from_us(1000); // TODO: SIFS=XXX
    pub const MAC_LIFS_PERIOD: Duration = Duration::from_us(10_000); // TODO: LIFS=XXX
}

#[cfg(not(test))]
mod constants {
    #![allow(unused)]
    include!(concat!(env!("OUT_DIR"), "/config.rs"));
}
