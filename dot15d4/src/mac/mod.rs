pub mod acknowledgment;
pub mod command;
pub mod constants;
pub mod mcps;
pub mod mlme;
pub mod neighbors;
pub mod pib;
pub mod tsch;
pub mod utils;

use crate::{
    phy::{
        radio::{Radio, RadioFrame, RadioFrameMut},
        FrameBuffer,
    },
    sync::{
        channel::{Receiver, Sender},
        join,
        mutex::Mutex,
        select,
        yield_now::yield_now,
        Either,
    },
    upper::UpperLayer,
};
use dot15d4_frame::{Frame, FrameType};
use embedded_hal_async::delay::DelayNs;
use rand_core::RngCore;

pub use command::{MacIndication, MacRequest};

/// MAC-related error propagated to higher layer
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Error {
    /// Cca failed, resulting in a backoff (nth try)
    CcaBackoff(u8),
    /// Cca failed after to many fallbacks
    CcaFailed,
    /// Ack failed, resulting in a retry later (nth try)
    AckRetry(u8),
    /// Ack failed, after to many retransmissions
    AckFailed,
    /// The buffer did not follow the correct device structure
    InvalidDeviceStructure,
    /// Invalid IEEE frame
    InvalidIEEEStructure,
    /// Something went wrong
    Error,
}

#[allow(dead_code)]
/// Structure handling MAC sublayer services such as MLME and MCPS. This runs the main event loop
/// that handles interactions between an upper layer and the PHY sublayer. It uses signals to
/// communicate with the upper layer and with the PHY sublayer.
pub struct MacService<'a, Rng, U: UpperLayer, TIMER, R> {
    /// Pseudo-random number generator
    rng: &'a mut Mutex<Rng>,
    /// Timer enabling delays operation
    timer: TIMER,
    /// Upper layer handler from which MAC commands are received and to which
    /// frames and responses are passed.
    upper_layer: &'a mut U,
    /// Signal for receiving command from the upper layer
    rx_recv: Receiver<'a, FrameBuffer>,
    /// Signal for sending frame to the PHY sublayer
    tx_send: Sender<'a, FrameBuffer>,
    /// Signal used for end of transmission of a frame submitted to the PHY
    /// sublayer.
    tx_done: Receiver<'a, ()>,
    /// PAN Information Base
    pub pib: pib::Pib,
    /// Phantom data used for associating Radio type in order to extract the
    /// data from a radio buffer
    _phantom: core::marker::PhantomData<R>,
}

impl<'a, Rng, U, TIMER, R> MacService<'a, Rng, U, TIMER, R>
where
    Rng: RngCore,
    U: UpperLayer,
    R: Radio,
    for<'b> R::RadioFrame<&'b mut [u8]>: RadioFrameMut<&'b mut [u8]>,
    for<'b> R::TxToken<'b>: From<&'b mut [u8]>,
{
    /// Creates a new [`MacService<Rng, U, TIMER, R>`].
    pub fn new(
        rng: &'a mut Mutex<Rng>,
        upper_layer: &'a mut U,
        timer: TIMER,
        rx_recv: Receiver<'a, FrameBuffer>,
        tx_send: Sender<'a, FrameBuffer>,
        tx_done: Receiver<'a, ()>,
    ) -> Self {
        Self {
            rng,
            upper_layer,
            timer,
            rx_recv,
            tx_send,
            tx_done,
            pib: pib::Pib::default(),
            _phantom: Default::default(),
        }
    }
}

#[allow(dead_code)]
impl<Rng, U, TIMER, R> MacService<'_, Rng, U, TIMER, R>
where
    Rng: RngCore,
    U: UpperLayer,
    TIMER: DelayNs + Clone,
    R: Radio,
    for<'b> R::RadioFrame<&'b mut [u8]>: RadioFrameMut<&'b mut [u8]>,
    for<'b> R::TxToken<'b>: From<&'b mut [u8]>,
{
    /// Run the main event loop used by the MAC sublayer for its operation. For
    /// now, the loop waits for either receiving a command from
    /// upper layer or receiving a frame/indication from PHY sublayer.
    pub async fn run(&mut self) -> ! {
        let mut indication = None;

        loop {
            yield_now().await;
            // Wait until we have a command to process from upper layer or we
            // receive an indication from PHY sublayer
            match select::select(
                self.upper_layer.mac_request(),
                self.receive_indication(&mut indication),
            )
            .await
            {
                Either::First(command) => self.handle_command(command).await,
                Either::Second(_) => self.handle_indication(&mut indication).await,
            };
        }
    }

    /// Submit a buffer to the PHY sublayer via a signal that is received by
    /// the PHY task. Wait for the frame to be fully transmitted before
    /// returning.
    async fn phy_send(&self, tx: FrameBuffer) {
        self.tx_send.send_async(tx).await;
        self.tx_done.receive().await;
    }

    /// Waits for a frame to be received from the PHY sublayer's task via a
    /// signal.
    async fn phy_receive(&self) -> FrameBuffer {
        self.rx_recv.receive().await
    }

    async fn receive_indication(&self, indication: &mut Option<MacIndication>) {
        let mut rx_frame = self.phy_receive().await;
        // TODO: remove this artifact from the old CSMA implementation
        rx_frame.dirty = true;

        // Optional ack frame that is used if required
        let mut ack_frame = None;
        self.prepare_ack(&mut rx_frame, &mut ack_frame);

        // Acknowledgment is sent while the indication is processed
        join::join(self.transmit_ack(&mut ack_frame), async {
            let frame_type = {
                let frame = R::RadioFrame::new_checked(&mut rx_frame.buffer[..]).unwrap();
                let frame = Frame::new(frame.data()).unwrap();
                frame.frame_control().frame_type()
            };
            // TODO: support timestamp
            let timestamp = 0;
            *indication = match frame_type {
                FrameType::Data => Some(MacIndication::McpsData(mcps::data::DataIndication {
                    buffer: rx_frame,
                    timestamp,
                })),
                FrameType::Beacon => Some(MacIndication::MlmeBeaconNotify(
                    mlme::beacon::BeaconNotifyIndication {
                        buffer: rx_frame,
                        timestamp,
                    },
                )),
                _ => None,
            }
        })
        .await;
    }

    async fn handle_indication(&self, indication: &mut Option<MacIndication>) {
        if let Some(indication) = indication {
            match indication {
                MacIndication::McpsData(data_indication) => {
                    self.mcps_data_indication(data_indication).await;
                }
                MacIndication::MlmeBeaconNotify(beacon_notify_indication) => {
                    self.mlme_beacon_notify_indication(beacon_notify_indication)
                        .await;
                }
            }
        }
    }

    async fn handle_command(&mut self, command: MacRequest) {
        match command {
            MacRequest::McpsDataRequest(mut request) => {
                // TODO: handle errors with upper layer
                let _ = self.mcps_data_request(&mut request.buffer).await;
            }
            MacRequest::MlmeBeaconRequest(beacon_request) => {
                // TODO: handle errors with upper layer
                let _ = self.mlme_beacon_request(&beacon_request).await;
            }
            MacRequest::MlmeSetRequest(set_request_attribute) => {
                // TODO: handle errors with upper layer
                let _ = self.mlme_set_request(set_request_attribute).await;
            }
            MacRequest::EmptyRequest => {}
        }
    }
}
