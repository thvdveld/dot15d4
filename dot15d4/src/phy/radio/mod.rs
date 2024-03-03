pub mod futures;

use core::task::{Context, Poll};

use super::config::{RxConfig, TxConfig};

pub trait Radio {
    type RadioFrame<T>: RadioFrame<T>
    where
        T: AsRef<[u8]>;
    type RxToken<'a>: RxToken<'a>;
    type TxToken<'b>: TxToken<'b>;

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

pub trait RxToken<'a> {
    type Buffer: 'a;

    fn consume<F, R>(self, f: F) -> R
    where
        F: FnOnce(Self::Buffer) -> R;
}

pub trait TxToken<'a> {
    type Buffer: 'a;

    fn consume<F, R>(self, len: usize, f: F) -> R
    where
        F: FnOnce(Self::Buffer) -> R;
}

// #[cfg(test)]
pub mod tests {
    use std::task::Poll;

    use crate::phy::{
        config::{RxConfig, TxConfig},
        driver::PacketBuffer,
    };

    use super::{Radio, RadioFrame, RadioFrameMut, RxToken, TxToken};

    pub enum TestRadioEvent {
        Off,
        PrepareReceive(RxConfig),
        Receive,
        PrepareTransmit(TxConfig, Vec<u8>),
        CancelCurrentOperation,
        Transmit,
    }

    pub struct TestRadio {
        pub ieee802154_address: [u8; 8],
        pub should_receive: Option<[u8; 128]>,
        pub events: Vec<TestRadioEvent>,
        pub cca_fail: bool,
    }
    impl Radio for TestRadio {
        type RadioFrame<T> = TestRadioFrame<T> where T: AsRef<[u8]>;
        type RxToken<'a> = TestRxToken<'a>;
        type TxToken<'b> = TestTxToken<'b>;

        fn off(&mut self, ctx: &mut std::task::Context<'_>) -> std::task::Poll<()> {
            self.events.push(TestRadioEvent::Off);
            Poll::Ready(())
        }

        unsafe fn prepare_receive(
            &mut self,
            ctx: &mut core::task::Context<'_>,
            cfg: &crate::phy::config::RxConfig,
            bytes: &mut [u8; 128],
        ) -> std::task::Poll<()> {
            self.events
                .push(TestRadioEvent::PrepareReceive(cfg.clone()));
            Poll::Ready(())
        }

        fn receive(&mut self, ctx: &mut std::task::Context<'_>) -> std::task::Poll<bool> {
            self.events.push(TestRadioEvent::Receive);
            Poll::Ready(true)
        }

        unsafe fn prepare_transmit(
            &mut self,
            ctx: &mut std::task::Context<'_>,
            cfg: &crate::phy::config::TxConfig,
            bytes: &[u8],
        ) -> std::task::Poll<()> {
            self.events.push(TestRadioEvent::PrepareTransmit(
                cfg.clone(),
                Vec::from(bytes),
            ));
            Poll::Ready(())
        }

        fn cancel_current_opperation(&mut self) {
            self.events.push(TestRadioEvent::CancelCurrentOperation);
        }

        fn transmit(&mut self, ctx: &mut std::task::Context<'_>) -> std::task::Poll<bool> {
            self.events.push(TestRadioEvent::Transmit);
            Poll::Ready(self.cca_fail)
        }

        fn ieee802154_address(&self) -> [u8; 8] {
            self.ieee802154_address
        }
    }

    #[derive(Debug, Clone)]
    pub struct TestRadioFrame<T: AsRef<[u8]>> {
        buffer: T,
    }
    impl<T: AsRef<[u8]>> RadioFrame<T> for TestRadioFrame<T> {
        type Error = ();

        fn new_unchecked(buffer: T) -> Self {
            Self { buffer }
        }

        fn new_checked(buffer: T) -> Result<Self, Self::Error> {
            Ok(Self { buffer })
        }

        fn data(&self) -> &[u8] {
            self.buffer.as_ref()
        }
    }
    impl<T: AsRef<[u8]> + AsMut<[u8]>> RadioFrameMut<T> for TestRadioFrame<T> {
        fn data_mut(&mut self) -> &mut [u8] {
            self.buffer.as_mut()
        }
    }

    pub struct TestRxToken<'a> {
        buffer: &'a mut PacketBuffer,
    }
    impl<'a> RxToken<'a> for TestRxToken<'a> {
        type Buffer = &'a mut [u8];

        fn consume<F, R>(self, f: F) -> R
        where
            F: FnOnce(Self::Buffer) -> R,
        {
            f(&mut self.buffer.buffer[..])
        }
    }
    impl<'a> From<&'a mut PacketBuffer> for TestRxToken<'a> {
        fn from(mut value: &'a mut PacketBuffer) -> Self {
            Self { buffer: value }
        }
    }
    pub struct TestTxToken<'a> {
        buffer: &'a mut PacketBuffer,
    }
    impl<'a> TxToken<'a> for TestTxToken<'a> {
        type Buffer = &'a mut [u8];

        fn consume<F, R>(self, len: usize, f: F) -> R
        where
            F: FnOnce(Self::Buffer) -> R,
        {
            f(&mut self.buffer.buffer[..len])
        }
    }
    impl<'a> From<&'a mut PacketBuffer> for TestTxToken<'a> {
        fn from(value: &'a mut PacketBuffer) -> Self {
            Self { buffer: value }
        }
    }
}
