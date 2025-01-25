use core::future::Future;

use crate::mac::Error;
use crate::phy::FrameBuffer;

/// This traits provides interactions with upper layer. It allows to abstract
/// over channels in async executors. Should be given as an argument to the
/// task that will run the network protocol.
pub trait UpperLayer {
    /// Waits for upper layer to provide a frame to transmit.
    fn frame_to_transmit(&self) -> impl Future<Output = FrameBuffer>;
    /// Notifies upper layer a frame has been received. Holds until the buffer
    /// is received successfully by upper layer.
    fn received_frame(&self, buffer: FrameBuffer) -> impl Future<Output = ()>;
    /// Notifies upper layer of an error while transmitting a frame.
    fn error(&self, error: Error) -> impl Future<Output = ()>;
}

#[cfg(test)]
pub mod tests {
    use crate::sync::channel::{Channel, Receiver, Sender};

    use super::*;

    pub enum TestUpperLayerEvent {
        TxProcessed,
        RxAvailable,
        NewError,
    }

    #[derive(Default)]
    pub struct TestUpperLayerChannel {
        pub tx: Channel<FrameBuffer>,
        pub rx: Channel<FrameBuffer>,
        pub errors: Channel<Error>,
    }

    impl TestUpperLayerChannel {
        pub fn new() -> Self {
            Self {
                tx: Channel::new(),
                rx: Channel::new(),
                errors: Channel::new(),
            }
        }

        pub fn split(&mut self) -> (TestUpperLayer<'_>, TestUpperLayerMonitor<'_>) {
            let (tx_send, tx_recv) = self.tx.split();
            let (rx_send, rx_recv) = self.rx.split();
            let (errors_send, errors_recv) = self.errors.split();
            (
                TestUpperLayer {
                    tx: tx_recv,
                    rx: rx_send,
                    errors: errors_send,
                },
                TestUpperLayerMonitor {
                    tx: tx_send,
                    rx: rx_recv,
                    errors: errors_recv,
                },
            )
        }
    }

    pub struct TestUpperLayerMonitor<'a> {
        pub tx: Sender<'a, FrameBuffer>,
        pub rx: Receiver<'a, FrameBuffer>,
        pub errors: Receiver<'a, Error>,
    }

    pub struct TestUpperLayer<'a> {
        tx: Receiver<'a, FrameBuffer>,
        rx: Sender<'a, FrameBuffer>,
        errors: Sender<'a, Error>,
    }

    impl UpperLayer for TestUpperLayer<'_> {
        async fn frame_to_transmit(&self) -> FrameBuffer {
            self.tx.receive().await
        }

        async fn received_frame(&self, buffer: FrameBuffer) {
            self.rx.send(buffer);
        }

        async fn error(&self, error: Error) {
            self.errors.send(error);
        }
    }
}
