use core::future::Future;
use core::task::Context;
use core::task::Poll;

/// Should be given as an argument to the task that will run the network protocol.
/// This trait allows to abstract over channels in async executors.
pub trait Driver {
    /// Waits until there is something to be transmitted
    fn transmit(&mut self) -> impl Future<Output = PacketBuffer>;
    /// Hold until the buffer is received successfully
    fn received(&mut self, buffer: PacketBuffer) -> impl Future<Output = ()>;
}

/// A buffer that is used to store 1 frame.
#[derive(Clone)]
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
