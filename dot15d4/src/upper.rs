use core::future::Future;

use crate::mac::command::MacIndication;
use crate::mac::{self, MacRequest};

/// This traits provides interactions with upper layer. It allows to abstract
/// over channels in async executors. Should be given as an argument to the
/// task that will run the network protocol.
pub trait UpperLayer {
    /// Waits for upper layer to provide a MAC request to handle.
    fn mac_request(&self) -> impl Future<Output = MacRequest>;
    /// Notifies upper layer a MAC indication has been received. Holds until
    /// the indication is received successfully by upper layer.
    fn received_mac_indication(&self, indication: MacIndication) -> impl Future<Output = ()>;
    /// Notifies upper layer of an error while handling a MAC request.
    fn error(&self, error: mac::Error) -> impl Future<Output = ()>;
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
        pub tx: Channel<MacRequest>,
        pub rx: Channel<MacIndication>,
        pub errors: Channel<mac::Error>,
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
        pub tx: Sender<'a, MacRequest>,
        pub rx: Receiver<'a, MacIndication>,
        pub errors: Receiver<'a, mac::Error>,
    }

    pub struct TestUpperLayer<'a> {
        tx: Receiver<'a, MacRequest>,
        rx: Sender<'a, MacIndication>,
        errors: Sender<'a, mac::Error>,
    }

    impl UpperLayer for TestUpperLayer<'_> {
        async fn mac_request(&self) -> MacRequest {
            self.tx.receive().await
        }

        async fn received_mac_indication(&self, indication: MacIndication) {
            self.rx.send(indication);
        }

        async fn error(&self, error: mac::Error) {
            self.errors.send(error);
        }
    }
}
