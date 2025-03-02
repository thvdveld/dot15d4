use crate::phy::radio::{Radio, RadioFrameMut};
use crate::upper::UpperLayer;
use embedded_hal_async::delay::DelayNs;
use rand_core::RngCore;

use super::MacService;

struct StartConfirm {}

#[allow(dead_code)]
impl<Rng, U, TIMER, R> MacService<'_, Rng, U, TIMER, R>
where
    Rng: RngCore,
    U: UpperLayer,
    TIMER: DelayNs + Clone,
    R: Radio,
    for<'a> R::RadioFrame<&'a mut [u8]>: RadioFrameMut<&'a mut [u8]>,
    for<'a> R::TxToken<'a>: From<&'a mut [u8]>,
{
    /// Used by PAN coordinator to initiate a new PAN or to begin using a new
    /// configuration. Also used by a device already associated with an
    /// existing PAN to begin using a new configuration.
    async fn mlme_start_request(&mut self) -> Result<StartConfirm, ()> {
        Err(())
    }
}
