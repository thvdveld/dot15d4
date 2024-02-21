//! Access to IEEE 802.15.4 devices.
//!
//! This module provides access to IEEE 802.15.4 devices. It provides a trait for transmitting and
//! recieving frames, [Device].

use crate::time::Instant;

/// An IEEE 802.15.4 device.
pub trait Device {
    fn disable(&mut self);
    fn enable(&mut self);

    fn receive<RX>(&mut self, rx: RX)
    where
        RX: FnMut(&[u8], Instant);

    fn transmit<TX>(&mut self, tx: TX)
    where
        TX: for<'b> Fn(&'b mut [u8]) -> Option<&'b [u8]>;
}
