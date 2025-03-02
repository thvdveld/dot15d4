use crate::phy::radio::{Radio, RadioFrameMut};
use crate::{phy::FrameBuffer, upper::UpperLayer};
use dot15d4_frame::{DataFrame, FrameBuilder};
use embedded_hal_async::delay::DelayNs;
use rand_core::RngCore;

use super::MacService;

pub struct BeaconRequest {}

pub struct BeaconConfirm {}

pub struct BeaconNotifyIndication {
    /// buffer containing the received frame
    pub buffer: FrameBuffer,
    /// Timestamp of frame reception
    pub timestamp: u32,
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
    /// Requests the generation of a Beacon frame or Enhanced Beacon frame.
    pub(crate) async fn mlme_beacon_request(
        &self,
        _request: &BeaconRequest,
    ) -> Result<BeaconConfirm, ()> {
        // TODO: fill with correct values
        let frame_repr = FrameBuilder::new_beacon_request()
            .finalize()
            .expect("A simple beacon request should always be possible to build");
        let mut tx = FrameBuffer::default();
        frame_repr.emit(&mut DataFrame::new_unchecked(&mut tx.buffer));
        self.phy_send(tx).await;

        Err(())
    }

    pub(crate) async fn mlme_beacon_notify_indication(
        &self,
        _indication: &mut BeaconNotifyIndication,
    ) {
        // TODO: support Beacon Notify indication
        info!("Received Beacon Notification");
    }
}
