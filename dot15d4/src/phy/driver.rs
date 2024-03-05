use core::future::Future;
use core::task::Context;
use core::task::Poll;

#[cfg_attr(feature = "std", derive(Debug))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone, Copy)]
pub enum Error {
    CCAFailed,
    ACKFailed,
    InvalidStructure,
    RadioError,
}

/// Should be given as an argument to the task that will run the network protocol.
/// This trait allows to abstract over channels in async executors.
pub trait Driver {
    /// Waits until there is something to be transmitted
    fn transmit(&self) -> impl Future<Output = PacketBuffer>;
    /// Hold until the buffer is received successfully
    fn received(&self, buffer: PacketBuffer) -> impl Future<Output = ()>;
    /// Hold until the buffer is received successfully
    fn error(&self, error: Error) -> impl Future<Output = ()>;
}

/// A buffer that is used to store 1 frame.
#[cfg_attr(feature = "std", derive(Debug))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, PartialEq)]
pub struct PacketBuffer {
    /// The data of the frame that should be transmitted over the radio
    pub buffer: [u8; 128],
    /// Whether or not the buffer is ready to be read from
    pub dirty: bool,
}

impl Default for PacketBuffer {
    fn default() -> Self {
        Self {
            buffer: [0u8; 128],
            dirty: false,
        }
    }
}

#[cfg(test)]
pub mod tests {
    use std::future::poll_fn;

    use crate::sync::{
        channel::{Channel, Receiver, Sender},
        mutex::Mutex,
    };

    use super::*;

    pub enum TestDriverEvent {
        TxProcessed,
        RxAvailable,
        NewError,
    }

    #[derive(Default)]
    pub struct TestDriverChannel {
        pub tx: Channel<PacketBuffer>,
        pub rx: Channel<PacketBuffer>,
        pub errors: Channel<Error>,
    }

    impl TestDriverChannel {
        pub fn new() -> Self {
            Self {
                tx: Channel::new(),
                rx: Channel::new(),
                errors: Channel::new(),
            }
        }

        pub fn split(&mut self) -> (TestDriver<'_>, TestDriverMonitor<'_>) {
            let (tx_send, tx_recv) = self.tx.split();
            let (rx_send, rx_recv) = self.rx.split();
            let (errors_send, errors_recv) = self.errors.split();
            (
                TestDriver {
                    tx: tx_recv,
                    rx: rx_send,
                    errors: errors_send,
                },
                TestDriverMonitor {
                    tx: tx_send,
                    rx: rx_recv,
                    errors: errors_recv,
                },
            )
        }
    }

    pub struct TestDriverMonitor<'a> {
        pub tx: Sender<'a, PacketBuffer>,
        pub rx: Receiver<'a, PacketBuffer>,
        pub errors: Receiver<'a, Error>,
    }

    pub struct TestDriver<'a> {
        tx: Receiver<'a, PacketBuffer>,
        rx: Sender<'a, PacketBuffer>,
        errors: Sender<'a, Error>,
    }

    impl Driver for TestDriver<'_> {
        async fn transmit(&self) -> PacketBuffer {
            self.tx.receive().await
        }

        async fn received(&self, buffer: PacketBuffer) {
            self.rx.send(buffer);
        }

        async fn error(&self, error: Error) {
            self.errors.send(error);
        }
    }
}
