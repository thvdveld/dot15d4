use crate::phy::radio::{Radio, RadioFrameMut};
use crate::upper::UpperLayer;
use embedded_hal_async::delay::DelayNs;
use rand_core::RngCore;

use super::MacService;

pub enum SetError {
    InvalidParameter,
}

/// Attributes that may be written by an upper layer
pub enum SetRequestAttribute {
    PanId(u16),
    ShortAddress(u16),
    ExtendedAddress([u8; 8]),
    AssociationPermit(bool),
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
    /// Used by the next higher layer to attempt to write the given value to
    /// the indicated MAC PIB attribute.
    ///
    /// * `attribute` - Attribute to write
    pub(crate) async fn mlme_set_request(
        &mut self,
        attribute: SetRequestAttribute,
    ) -> Result<(), SetError> {
        match attribute {
            SetRequestAttribute::PanId(pan_id) => self.pib.pan_id = pan_id,
            SetRequestAttribute::ShortAddress(short_address) => {
                self.pib.short_address = short_address
            }
            SetRequestAttribute::ExtendedAddress(extended_address) => {
                self.pib.extended_address = Some(extended_address)
            }
            SetRequestAttribute::AssociationPermit(association_permit) => {
                self.pib.association_permit = association_permit
            }
        }
        Ok(())
    }
}
