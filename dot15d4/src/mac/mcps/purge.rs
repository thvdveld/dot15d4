use embedded_hal_async::delay::DelayNs;

use crate::phy::radio::{Radio, RadioFrameMut};
use crate::{mac::MacService, upper::UpperLayer};
use rand_core::RngCore;

#[allow(dead_code)]
pub struct PurgeConfirm {
    msdu_handle: u8,
}

pub enum PurgeError {
    InvalidHandle,
}

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
    /// Allows a higher layer to purge an MSDU from the transaction
    /// queue.
    async fn purge_request(&mut self) -> Result<PurgeConfirm, PurgeError> {
        // TODO: not supported
        Err(PurgeError::InvalidHandle)
    }
}
