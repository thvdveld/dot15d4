use embedded_hal_async::delay::DelayNs;

use super::yield_now;

/// Implementation of a timer that can be used in tests. Delays are yield at
/// least once, but then afterwards immediatly resolve.
#[derive(Default, Clone)]
pub struct Delay {}

impl DelayNs for Delay {
    async fn delay_ns(&mut self, ns: u32) {
        yield_now::yield_now().await
    }
}
