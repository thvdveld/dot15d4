pub mod constants;
pub mod user_configurable_constants;

use std::process::Output;

use constants::*;
use embedded_hal_async::delay::DelayNs;
use rand_core::RngCore;
use user_configurable_constants::*;

use crate::{
    frame::{Address, Buffer, Frame, FrameBuilder, FrameRepr, FrameType},
    phy::{
        config::{RxConfig, TxConfig},
        driver::{self, Driver, PacketBuffer},
        radio::{
            futures::{receive, transmit},
            Radio, RadioFrame, RadioFrameMut, TxToken,
        },
    },
    sync::{
        channel::{Channel, Receiver, Sender},
        join,
        mutex::Mutex,
        select,
        yield_now::yield_now,
        Either,
    },
    time::Duration,
};

enum TransmissionTaskError<D> {
    InvalidIEEEFrame,
    InvalidDeviceFrame(D),
}

/// Structure that setups the CSMA futures
pub struct CsmaDevice<R: Radio, Rng, D: Driver, TIMER> {
    radio: Mutex<R>,
    rng: Mutex<Rng>,
    driver: D,
    timer: TIMER,
    hardware_address: [u8; 8],
}

impl<R, Rng, D, TIMER> CsmaDevice<R, Rng, D, TIMER>
where
    R: Radio,
    for<'a> R::RadioFrame<&'a mut [u8]>: RadioFrameMut<&'a mut [u8]>,
    for<'a> R::TxToken<'a>: TxToken<'a, Buffer = &'a mut [u8]> + From<&'a mut PacketBuffer>,
    Rng: RngCore,
    D: Driver,
    TIMER: DelayNs + Clone,
{
    /// Creates a new CSMA object that is ready to be run
    pub fn new(radio: R, rng: Rng, driver: D, timer: TIMER) -> Self {
        let hardware_address = radio.ieee802154_address();
        CsmaDevice {
            radio: Mutex::new(radio),
            rng: Mutex::new(rng),
            driver,
            timer,
            hardware_address,
        }
    }

    // /// Retrieve the Ieee802.15.4 driver needed for use with the `embassy-net` stack.
    // /// This method is async because it temporarily needs to take a lock on the radio.
    // /// If the CSMA module is not yet running, the async-ness should be almost a no-op.
    // pub fn driver(&self) -> Ieee802154Driver<R> {
    //     Ieee802154Driver {
    //         tx: PacketBuffer::default(),
    //         rx: None,
    //         tx_channel: self.tx_channel.sender(),
    //         rx_channel: self.rx_channel.receiver(),

    //         address: self.hardware_address,

    //         _r: Default::default(),
    //     }
    // }

    /// Run the CSMA module. This should be run in its own task and polled seperately.
    pub async fn run(&self) -> ! {
        let mut wants_to_transmit_signal = Channel::new();
        let (sender, receiver) = wants_to_transmit_signal.split();
        match select::select(
            self.transmit_package_task(sender),
            self.receive_package_task(receiver),
        )
        .await
        {
            Either::First(_) => panic!("Tasks should never terminate, csma transmission just did"),
            Either::Second(_) => panic!("Tasks should never terminate, csma receiving just did"),
        }
    }

    /// Checks if the current frame is intended for us. For the hardware address, the full 64bit
    /// address should be provided.
    fn is_package_for_us<BUF: AsRef<[u8]>>(hardware_address: &[u8; 8], frame: &Frame<BUF>) -> bool {
        let Some(addr) = frame
            .addressing()
            .and_then(|fields| fields.dst_address(&frame.frame_control()))
        else {
            return false;
        };

        match &addr {
            _ if addr.is_broadcast() => true,
            Address::Absent => false,
            Address::Short(addr) => hardware_address[6..] == addr[..2],
            Address::Extended(addr) => hardware_address == addr,
        }
    }

    async fn receive_package_task(&self, mut wants_to_transmit_signal: Receiver<'_, ()>) -> ! {
        let mut rx = PacketBuffer::default();
        let mut radio_guard = None;
        let mut timer = self.timer.clone();

        // Allocate tx buffer for ACK messages
        let mut tx_ack = PacketBuffer::default();

        'outer: loop {
            yield_now().await;

            // try to receive something
            let receive_result = {
                radio_guard = match radio_guard {
                    Some(_) => radio_guard,
                    None => Some(self.radio.lock().await),
                };
                match select::select(
                    receive(
                        &mut **radio_guard.as_mut().unwrap(),
                        &mut rx.buffer,
                        RxConfig::default(),
                    ),
                    wants_to_transmit_signal.receive(),
                )
                .await
                {
                    Either::First(receive_result) => receive_result,
                    Either::Second(_) => false,
                }
            };
            // wants_to_transmit_signal.reset();

            // Check if something went wrong
            if !receive_result {
                rx.dirty = false;
                radio_guard = None;
                continue 'outer;
            }

            let (should_ack, sequence_number) = {
                // Check if package is valid IEEE and not an ACK
                let Ok(frame) = R::RadioFrame::new_checked(&mut rx.buffer) else {
                    rx.dirty = false;
                    continue 'outer;
                };
                let Ok(frame) = Frame::new(frame.data()) else {
                    rx.dirty = false;
                    continue 'outer;
                };

                // Check if package is meant for us
                if !Self::is_package_for_us(&self.hardware_address, &frame) {
                    // Package is not for us to handle, ignore
                    rx.dirty = false;
                    continue 'outer;
                }

                if frame.frame_control().frame_type() == FrameType::Ack {
                    // Ignore this ACK as it is not at an expected time, or not for us
                    rx.dirty = false;
                    continue 'outer;
                }

                (frame.frame_control().ack_request(), frame.sequence_number())
            };

            // Concurrently send the received message to the upper layers, and if we need to ACK, we ACK
            rx.dirty = true;
            join::join(self.driver.received(core::mem::take(&mut rx)), async {
                if should_ack {
                    // Set correct sequence number and send an ACK only if valid sequence number
                    if let Some(sequence_number) = sequence_number {
                        let ieee_repr = FrameBuilder::new_imm_ack(sequence_number)
                            .finalize()
                            .expect("A simple imm-ACK should always be possible to build");
                        let ack_token = R::TxToken::from(&mut tx_ack);
                        ack_token.consume(ieee_repr.buffer_len(), |buffer| {
                            let mut frame = Frame::new_unchecked(buffer);
                            ieee_repr.emit(&mut frame);
                        });

                        // Wait before sending the ACK (AIFS)
                        let delay = ACKNOWLEDGEMENT_INTERYFRAME_SPACING;
                        timer.delay_us(delay.as_us() as u32).await;

                        // We already have the lock on the radio, so start transmitting and do not have to check anymore
                        transmit(
                            &mut **radio_guard.as_mut().unwrap(),
                            &tx_ack.buffer,
                            TxConfig::default(),
                        )
                        .await;
                    }
                } else {
                    radio_guard = None; // Immediatly drop gruard if we do not longer need it to ACK
                }
            })
            .await;
            rx.dirty = false; // Reset for the following iteration
        }
    }

    fn set_ack_request_if_possible<'a, RadioFrame>(
        buffer: &'a mut [u8],
    ) -> Result<Option<u8>, TransmissionTaskError<RadioFrame::Error>>
    where
        RadioFrame: RadioFrameMut<&'a mut [u8]>,
    {
        let mut frame =
            RadioFrame::new_checked(buffer).map_err(TransmissionTaskError::InvalidDeviceFrame)?;
        let mut frame =
            Frame::new(frame.data_mut()).map_err(|_err| TransmissionTaskError::InvalidIEEEFrame)?;
        frame.frame_control_mut().set_ack_request(true);
        Ok(frame.sequence_number())
    }

    async fn wait_for_valid_ack(radio: &mut R, sequence_number: u8, ack_rx: &mut [u8; 128]) {
        loop {
            let result = receive(radio, ack_rx, RxConfig::default()).await;
            if !result {
                continue;
            } // No succesful receive, try again

            // Check if we received a valid ACK
            let Ok(frame) = R::RadioFrame::new_checked(ack_rx) else {
                continue;
            };
            let Ok(frame) = Frame::new(frame.data()) else {
                continue;
            };

            if frame.frame_control().frame_type() == FrameType::Ack
                && frame.sequence_number() == Some(sequence_number)
            {
                return;
            }
        }
    }

    async fn transmit_package_task(&self, mut wants_to_transmit_signal: Sender<'_, ()>) -> !
    where
        R: Radio,
        for<'a> R::RadioFrame<&'a mut [u8]>: RadioFrameMut<&'a mut [u8]>,
        Rng: RngCore,
        D: Driver,
    {
        let mut ack_rx = PacketBuffer::default();
        let mut timer = self.timer.clone();

        'outer: loop {
            let mut tx = self.driver.transmit().await; // Wait until we have a frame to send

            yield_now().await;

            // Enable ACK in frame coming from higher layers
            let mut sequence_number = None;
            match Self::set_ack_request_if_possible::<R::RadioFrame<_>>(&mut tx.buffer) {
                Ok(seq_number) => sequence_number = Some(seq_number).flatten(),
                Err(TransmissionTaskError::InvalidIEEEFrame) => {
                    // Invalid IEEE frame encountered
                }
                Err(TransmissionTaskError::InvalidDeviceFrame(err)) => {
                    // Invalid device frame encountered
                }
            }

            let mut radio_guard = None;
            'ack: for i_ack in 0..MAC_MAX_FRAME_RETIES + 1 {
                // Set vars for CCA
                let mut backoff_exponent = MAC_MIN_BE;
                'cca: for number_of_backoffs in 1..MAC_MAX_CSMA_BACKOFFS + 1 {
                    // try to transmit
                    let transmission_result = {
                        radio_guard = match radio_guard {
                            Some(_) => radio_guard,
                            None => {
                                'inner: loop {
                                    // repeatably ask for the lock, as this might need a few tries to prevent deadlocks
                                    match self.radio.try_lock() {
                                        Some(guard) => {
                                            // wants_to_transmit_signal.reset(); // reset signal, such that the receiving end may continue the next time it acquires the lock
                                            break 'inner Some(guard);
                                        }
                                        None => {
                                            wants_to_transmit_signal.send(()); // Ask the receiving loop to let go of the radio
                                            yield_now().await; // Give the receiving end time to react
                                            continue;
                                        }
                                    }
                                }
                            }
                        };
                        transmit(
                            &mut **radio_guard.as_mut().unwrap(),
                            &tx.buffer,
                            TxConfig::default_with_cca(),
                        )
                        .await
                    };
                    if transmission_result {
                        break 'cca; // Send succesfully, now wait for ack
                    }

                    // As we are now going to wait a number of periods, release the
                    // mutex on the radio
                    radio_guard = None;

                    // CCA did not go succesfully
                    // Was this the last attempt?
                    if number_of_backoffs == MAC_MAX_CSMA_BACKOFFS {
                        break 'ack; // Fail transmission
                    }

                    // Wait now for a random number of periods, before retrying
                    backoff_exponent = core::cmp::min(backoff_exponent + 1, MAC_MAX_BE);

                    // delay periods = random(2^{BE} - 1) periods
                    // Page 63 IEEE 802.15.4 2015 edition
                    let max_backoff = (1u32 << backoff_exponent) - 1;
                    // The +1 in (max_backoff + 1) comes from the interpretation that the random() function
                    // used in the specification includes max_backoff as a possible value. The possible
                    // values periods now can take are: [0, max_backoff].
                    let periods = self.rng.lock().await.next_u32() % (max_backoff + 1);
                    let delay = MAC_UNIT_BACKOFF_DURATION * periods as usize;
                    timer.delay_us(delay.as_us() as u32).await;
                }

                // We now want to try and receive an ACK
                if let Some(sequence_number) = sequence_number {
                    radio_guard = match radio_guard {
                        Some(_) => radio_guard,
                        None => {
                            'inner: loop {
                                // repeatably ask for the lock, as this might need a few tries to prevent deadlocks
                                match self.radio.try_lock() {
                                    Some(guard) => {
                                        // wants_to_transmit_signal.reset(); // reset signal, such that the receiving end may continue the next time it acquires the lock
                                        break 'inner Some(guard);
                                    }
                                    None => {
                                        wants_to_transmit_signal.send(()); // Ask the receiving loop to let go of the radio
                                        yield_now().await; // Give the receiving end time to react
                                        continue;
                                    }
                                }
                            }
                        }
                    };

                    let delay = ACKNOWLEDGEMENT_INTERYFRAME_SPACING
                        + MAC_SIFT_PERIOD.max(Duration::from_us(A_TURNAROUND_TIME as i64));
                    match select::select(
                        Self::wait_for_valid_ack(
                            &mut *radio_guard.unwrap(),
                            sequence_number,
                            &mut ack_rx.buffer,
                        ),
                        timer.delay_us(delay.as_us() as u32), // Timeout for waiting on an ACK
                    )
                    .await
                    {
                        Either::First(()) => {
                            // ACK succesful, transmission succesful
                            continue 'outer; // This releases the radio_gaurd too
                        }
                        Either::Second(()) => (), // Timout, retry logic if following part of the code
                    }
                } else {
                    // We do not have a sequence number, so do not wait for an ACK
                    continue 'outer; // Transmission is considered a success
                }

                // Whether we succeeded or not, we no longer need sole access to the radio module, so we can
                // release the lock
                radio_guard = None;

                // Wait for SIFS here
                let delay = MAC_SIFT_PERIOD.max(Duration::from_us(A_TURNAROUND_TIME as i64));
                timer.delay_us(delay.as_us() as u32).await;

                // Was this the last attempt?
                if i_ack == MAC_MAX_FRAME_RETIES {
                    break 'ack; // Fail transmission
                }
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    use pollster::FutureExt;

    use crate::{phy::radio::tests::TestRadio, sync::test::Delay};

    use self::driver::tests::TestDriver;

    use super::*;

    #[test]
    pub fn test_happy_path_transmit() {
        async {
            let mut radio = TestRadio {
                ieee802154_address: [1, 2, 3, 4, 5, 6, 7, 8],
                should_receive: None,
                events: vec![],
                cca_fail: false,
            };
            let mut tx = Channel::new();
            let (mut tx_send, tx_recv) = tx.split();
            let mut rx = Channel::new();
            let (mut rx_send, rx_recv) = rx.split();
            let mut driver = TestDriver::new(tx_recv, rx_send);
            // let csma = CsmaDevice::new(radio, rand::thread_rng(), driver, Delay::default());
        }
        .block_on()
    }
}
