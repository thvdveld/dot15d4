pub mod futures;

use core::task::{Context, Poll};

use super::config::{RxConfig, TxConfig};

pub trait Radio {
    type RadioFrame<T>: RadioFrame<T>
    where
        T: AsRef<[u8]>;

    /// Request the radio to idle to a low-power sleep mode.
    fn off(&mut self, ctx: &mut Context<'_>) -> Poll<()>;

    /// Request the radio to go in receive mode and try to receive a packet into the supplied
    /// buffer.
    ///
    /// # Safety
    /// The supplied buffer must remain writable until either
    /// successful reception, or the radio state changed.
    unsafe fn prepare_receive(
        &mut self,
        ctx: &mut Context<'_>,
        cfg: &RxConfig,
        bytes: &mut [u8; 128],
    ) -> Poll<()>;

    /// Request the radio to go in receive mode and try to receive a packet.
    fn receive(&mut self, ctx: &mut Context<'_>) -> Poll<bool>;

    /// Request the radio to go in transmit mode and try to send a packet.
    ///
    /// # Safety
    /// The supplied buffer must remain valid until either
    /// successful reception, or the radio state changed.
    unsafe fn prepare_transmit(
        &mut self,
        ctx: &mut Context<'_>,
        cfg: &TxConfig,
        bytes: &[u8],
    ) -> Poll<()>;

    /// When working with futures, it is not always guaranteed that a future
    /// will complete. This method must be seen as a notification to the radio
    /// where it can prepare for this cancelation. This method may not use any
    /// async behavior, as dropping in Rust (when a future is cancelled) can
    /// not be async.
    fn cancel_current_opperation(&mut self);

    /// Request the radio to transmit the queued packet.
    ///
    /// Returns whether a transmission was successful.
    fn transmit(&mut self, ctx: &mut Context<'_>) -> Poll<bool>;

    /// Returns the IEEE802.15.4 8-octet MAC address of the radio device.
    fn ieee802154_address(&self) -> [u8; 8];
}

pub trait RadioFrame<T: AsRef<[u8]>>: Sized {
    #[cfg(not(feature = "defmt"))]
    type Error: Sized;
    #[cfg(feature = "defmt")]
    type Error: Sized + defmt::Format;

    fn new_unchecked(buffer: T) -> Self;
    fn new_checked(buffer: T) -> Result<Self, Self::Error>;
    fn data(&self) -> &[u8];
}

pub trait RadioFrameMut<T: AsRef<[u8]> + AsMut<[u8]>>: RadioFrame<T> {
    fn data_mut(&mut self) -> &mut [u8];
}
