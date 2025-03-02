use crate::{
    mac::{
        constants::{MAC_AIFS_PERIOD, MAC_SIFS_PERIOD},
        MacService,
    },
    phy::{
        radio::{Radio, RadioFrame, RadioFrameMut, TxToken},
        FrameBuffer,
    },
    sync::{select, Either},
    time::Duration,
    upper::UpperLayer,
};
use dot15d4_frame::{DataFrame, Frame, FrameBuilder};
use embedded_hal_async::delay::DelayNs;
use rand_core::RngCore;

impl<Rng, U, TIMER, R> MacService<'_, Rng, U, TIMER, R>
where
    Rng: RngCore,
    U: UpperLayer,
    TIMER: DelayNs + Clone,
    R: Radio,
    for<'a> R::RadioFrame<&'a mut [u8]>: RadioFrameMut<&'a mut [u8]>,
    for<'a> R::TxToken<'a>: From<&'a mut [u8]>,
{
    /// Transmit acknowledgment for a frame that has been received.
    /// Returns when acknowledgment has been transmitted by the radio.
    ///
    /// * `rx_frame` - Received Frame to acknowledge
    /// * `ack_frame` - Frame buffer to use for sending acknowledgment
    pub(crate) async fn transmit_ack(&self, ack_frame: &mut Option<FrameBuffer>) {
        if let Some(ack_frame) = ack_frame {
            self.phy_send(core::mem::take(ack_frame)).await;
        }
    }

    /// Prepare an acknowledgment frame if one has to be sent for the given
    /// received frame.
    ///
    /// * `rx_frame` - Received frame to potentially acknowledge
    /// * `ack_frame` - Mutable reference to the potential ack frame to
    ///    generate.
    pub(crate) fn prepare_ack(
        &self,
        rx_frame: &mut FrameBuffer,
        ack_frame: &mut Option<FrameBuffer>,
    ) {
        let rx_frame = R::RadioFrame::new_checked(&mut rx_frame.buffer).unwrap();
        let rx_frame = Frame::new(rx_frame.data()).unwrap();
        if let Frame::Data(data_frame) = rx_frame {
            if !data_frame.frame_control().ack_request() {
                return;
            }
            // If frame has a sequence number, we send an ack
            if let Some(sequence_number) = rx_frame.sequence_number() {
                let ieee_repr = FrameBuilder::new_imm_ack(sequence_number)
                    .finalize()
                    .expect("A simple ACK should always be possible to build");
                *ack_frame = Some(FrameBuffer::default());
                let buffer = &mut ack_frame.as_mut().unwrap().buffer;
                let ack_token = R::TxToken::from(buffer);
                ack_token.consume(ieee_repr.buffer_len(), |buffer| {
                    let mut frame = DataFrame::new_unchecked(&mut buffer[..ieee_repr.buffer_len()]);
                    ieee_repr.emit(&mut frame);
                });
            }
        }
    }

    /// Wait for the reception of an acknowledgment for a specific sequence
    /// number. Time out if ack is not received within a specific delay.
    /// Return `true` if such an ack is received, return `else` otherwise (or
    /// if timed out).
    ///
    /// * `sequence_number` - Sequence number of the frame waiting for ack
    pub(crate) async fn wait_for_ack(&self, sequence_number: u8) -> bool {
        let mut timer = self.timer.clone();
        // We expect an ACK to come back AIFS + time for an ACK to travel + SIFS (guard)
        // An ACK is 3 bytes + 6 bytes (PHY header) long
        // and should take around 288us at 250kbps to get back
        let delay = MAC_AIFS_PERIOD + MAC_SIFS_PERIOD + Duration::from_us(288);

        match select::select(
            async {
                // We may receive multiple frame during that period of time.
                // non-matching frames are dropped.
                loop {
                    let mut ack_frame = self.phy_receive().await;
                    let ack_frame = R::RadioFrame::new_checked(&mut ack_frame.buffer).unwrap();
                    if let Frame::Ack(ack_frame) = Frame::new(ack_frame.data()).unwrap() {
                        if sequence_number == ack_frame.sequence_number() {
                            break;
                        }
                    }
                }
            },
            // Timeout for waiting on an ACK
            async {
                timer.delay_us(delay.as_us() as u32).await;
                info!("Expired !");
            },
        )
        .await
        {
            Either::First(_) => true,
            Either::Second(_) => false,
        }
    }

    /// Check if the given frame needs to be acknowledged, based on current
    /// buffer content and frame addressing. If so, acknowledgment request is
    /// set in the frame.
    ///
    /// * `frame` - Frame buffer to check and update, if necessary.
    pub(crate) fn set_ack(frame: &mut FrameBuffer) -> Option<u8> {
        let mut frame = R::RadioFrame::new_checked(&mut frame.buffer).unwrap();
        if let Frame::Data(mut data_frame) = Frame::new(frame.data_mut()).unwrap() {
            match data_frame.addressing().and_then(|addr| addr.dst_address()) {
                Some(addr) if addr.is_unicast() => {
                    data_frame.frame_control_mut().set_ack_request(true);
                    data_frame.sequence_number()
                }
                _ => None,
            }
        } else {
            None
        }
    }
}
