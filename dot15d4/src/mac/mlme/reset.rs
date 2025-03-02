use crate::phy::radio::{Radio, RadioFrameMut};
use crate::{mac::pib, upper::UpperLayer};
use embedded_hal_async::delay::DelayNs;
use rand_core::RngCore;

use super::MacService;

#[allow(dead_code)]
pub struct ResetConfirm {
    status: bool,
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
    /// Used by the next higher layer to request a reset operation that
    /// involves resetting the PAN Information Base
    async fn mlme_reset_request(&mut self, set_default_pib: bool) -> Result<ResetConfirm, ()> {
        if set_default_pib {
            self.pib = pib::Pib::default();
        }
        Ok(ResetConfirm { status: true })
    }
}
