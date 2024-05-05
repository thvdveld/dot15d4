use core::future::Future;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Error {
    /// Cca failed, resulting in a backoff (nth try)
    CcaBackoff(u16),
    /// Cca failed after to many fallbacks
    CcaFailed,
    /// Ack failed, resulting in a retry later (nth try)
    AckBackoff(u16),
    /// Ack failed, after to many retransmissions
    AckFailed,
    /// The buffer did not follow the correct device structure
    InvalidDeviceStructure,
    /// Invalid IEEE frame
    InvalidIEEEStructure,
    /// Something went wrong in the radio
    RadioError,
}

/// Should be given as an argument to the task that will run the network
/// protocol. This trait allows to abstract over channels in async executors.
pub trait Driver {
    /// Waits until there is something to be transmitted
    fn transmit(&self) -> impl Future<Output = FrameBuffer>;
    /// Hold until the buffer is received successfully
    fn received(&self, buffer: FrameBuffer) -> impl Future<Output = ()>;
    /// Hold until the buffer is received successfully
    fn error(&self, error: Error) -> impl Future<Output = ()>;
}

/// A buffer that is used to store 1 frame.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, PartialEq)]
pub struct FrameBuffer {
    /// The data of the frame that should be transmitted over the radio.
    /// Normally, a MAC layer frame is 127 bytes long.
    /// Some radios, like the one on the nRF52840, have the possibility to not
    /// having to include the checksum at the end, but require one extra byte to
    /// specify the length of the frame. This results in this case in
    /// needing 126 bytes in total. However, some radios like the one on the
    /// Zolertia Zoul, add extra information about the link quality. In that
    /// case, the total buffer length for receiving data comes on 128 bytes.
    /// If you would like to support a radio that needs more than 128 bytes,
    /// please file a PR.
    pub buffer: [u8; 128],
    /// Whether or not the buffer is ready to be read from
    pub dirty: bool,
}

impl Default for FrameBuffer {
    fn default() -> Self {
        Self {
            buffer: [0u8; 128],
            dirty: false,
        }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::sync::channel::{Channel, Receiver, Sender};

    use super::*;

    pub enum TestDriverEvent {
        TxProcessed,
        RxAvailable,
        NewError,
    }

    #[derive(Default)]
    pub struct TestDriverChannel {
        pub tx: Channel<FrameBuffer>,
        pub rx: Channel<FrameBuffer>,
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
        pub tx: Sender<'a, FrameBuffer>,
        pub rx: Receiver<'a, FrameBuffer>,
        pub errors: Receiver<'a, Error>,
    }

    pub struct TestDriver<'a> {
        tx: Receiver<'a, FrameBuffer>,
        rx: Sender<'a, FrameBuffer>,
        errors: Sender<'a, Error>,
    }

    impl Driver for TestDriver<'_> {
        async fn transmit(&self) -> FrameBuffer {
            self.tx.receive().await
        }

        async fn received(&self, buffer: FrameBuffer) {
            self.rx.send(buffer);
        }

        async fn error(&self, error: Error) {
            self.errors.send(error);
        }
    }
}
