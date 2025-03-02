use embedded_hal_async::delay::DelayNs;
use rand_core::RngCore;

use crate::mac::MacService;
use crate::phy::radio::{Radio, RadioFrameMut};
use crate::upper::UpperLayer;

pub struct AssociateConfirm;

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
    /// Requests the association with a coordinator.
    async fn mlme_associate_request(&self) -> Result<AssociateConfirm, ()> {
        // TODO: support association
        Err(())
    }
}
