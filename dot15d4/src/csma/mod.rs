pub mod constants;
pub mod transmission;
pub mod user_configurable_constants;
mod utils;

use constants::*;
use embedded_hal_async::delay::DelayNs;
use rand_core::RngCore;
use user_configurable_constants::*;

use crate::{
    phy::{
        config::{RxConfig, TxConfig},
        driver::{self, Driver, FrameBuffer},
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
use dot15d4_frame::{Address, Frame, FrameBuilder, FrameType};

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy)]
enum TransmissionTaskError<D: core::fmt::Debug> {
    InvalidIEEEFrame,
    InvalidDeviceFrame(D),
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct CsmaConfig {
    /// All to be transmitted frames will get the ack_request flag set if they
    /// are unicast and a data frame
    ack_unicast: bool,
    /// All to be transmitted frames will get the ack_request flag set if they
    /// are broadcast and a data frame
    ack_broadcast: bool,
    /// If false, all incoming packets will be sent up the layer stack, useful
    /// if making a sniffer. This does not include MAC layer control traffic.
    /// The default is true, meaning that packets not ment for us (non-broadcast
    /// or packets with a different destination address) will be filtered out.
    ignore_not_for_us: bool,
    /// Even if there is no ack_request flag set, ack it anyway
    ack_everything: bool,
}

impl Default for CsmaConfig {
    fn default() -> Self {
        Self {
            ack_unicast: true,
            ack_broadcast: false,
            ignore_not_for_us: true,
            ack_everything: false,
        }
    }
}

/// Structure that setups the CSMA futures
pub struct CsmaDevice<R: Radio, Rng, D: Driver, TIMER> {
    radio: Mutex<R>,
    rng: Mutex<Rng>,
    driver: D,
    timer: TIMER,
    hardware_address: [u8; 8],
    config: CsmaConfig,
}

impl<R, Rng, D, TIMER> CsmaDevice<R, Rng, D, TIMER>
where
    R: Radio,
    Rng: RngCore,
    D: Driver,
{
    /// Creates a new CSMA object that is ready to be run
    pub fn new(radio: R, rng: Rng, driver: D, timer: TIMER, config: CsmaConfig) -> Self {
        let hardware_address = radio.ieee802154_address();
        CsmaDevice {
            radio: Mutex::new(radio),
            rng: Mutex::new(rng),
            driver,
            timer,
            hardware_address,
            config,
        }
    }
}

impl<R, Rng, D, TIMER> CsmaDevice<R, Rng, D, TIMER>
where
    R: Radio,
    for<'a> R::RadioFrame<&'a mut [u8]>: RadioFrameMut<&'a mut [u8]>,
    for<'a> R::TxToken<'a>: From<&'a mut [u8]>,
    Rng: RngCore,
    D: Driver,
    TIMER: DelayNs + Clone,
{
    /// Run the CSMA module. This should be run in its own task and polled
    /// seperately.
    pub async fn run(&mut self) -> ! {
        let mut wants_to_transmit_signal = Channel::new();
        let (sender, receiver) = wants_to_transmit_signal.split();
        self.radio.get_mut().enable().await; // Wake up radio
        match select::select(
            self.transmit_package_task(sender),
            self.receive_frame_task(receiver),
        )
        .await
        {
            Either::First(_) => panic!("Tasks should never terminate, csma transmission just did"),
            Either::Second(_) => panic!("Tasks should never terminate, csma receiving just did"),
        }
    }

    /// Checks if the current frame is intended for us. For the hardware
    /// address, the full 64-bit address should be provided.
    fn is_package_for_us(hardware_address: &[u8; 8], frame: &Frame<&'_ [u8]>) -> bool {
        let Some(addr) = frame.addressing().and_then(|fields| fields.dst_address()) else {
            return false;
        };

        match &addr {
            _ if addr.is_broadcast() => true,
            Address::Absent => false,
            Address::Short(addr) => hardware_address[6..] == addr[..2],
            Address::Extended(addr) => hardware_address == addr,
        }
    }

    async fn receive_frame_task(&self, wants_to_transmit_signal: Receiver<'_, ()>) -> ! {
        let mut rx = FrameBuffer::default();
        let mut radio_guard = None;
        let mut timer = self.timer.clone();

        // Allocate tx buffer for ACK messages
        let mut tx_ack = FrameBuffer::default();

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
                if !Self::is_package_for_us(&self.hardware_address, &frame)
                    && !self.config.ignore_not_for_us
                {
                    // Package is not for us to handle, ignore
                    rx.dirty = false;
                    continue 'outer;
                }

                if frame.frame_control().frame_type() == FrameType::Ack {
                    // Ignore this ACK as it is not at an expected time, or not for us
                    rx.dirty = false;
                    continue 'outer;
                }

                let should_ack = match frame.addressing().and_then(|addr| addr.dst_address()) {
                    // Overwrite in config
                    _ if self.config.ack_everything => true,

                    // If we do not want an ACK, don't ack
                    _ if !frame.frame_control().ack_request() => false,

                    // We want ACK on broadcast -> check config
                    Some(addr) if addr.is_broadcast() => self.config.ack_broadcast,

                    // We want ACK on unicast -> check config
                    Some(_) => self.config.ack_unicast,

                    // All other scenarios -> don't ack
                    None => false,
                };
                (should_ack, frame.sequence_number())
            };

            // Concurrently send the received message to the upper layers, and if we need to
            // ACK, we ACK
            rx.dirty = true;
            join::join(self.driver.received(core::mem::take(&mut rx)), async {
                if should_ack {
                    // Set correct sequence number and send an ACK only if valid sequence number
                    if let Some(sequence_number) = sequence_number {
                        let ieee_repr = FrameBuilder::new_imm_ack(sequence_number)
                            .finalize()
                            .expect("A simple imm-ACK should always be possible to build");
                        let ack_token = R::TxToken::from(&mut tx_ack.buffer);
                        ack_token.consume(ieee_repr.buffer_len(), |buffer| {
                            let mut frame = Frame::new_unchecked(buffer);
                            ieee_repr.emit(&mut frame);
                        });

                        // Wait before sending the ACK (AIFS)
                        let delay = ACKNOWLEDGEMENT_INTERFRAME_SPACING;
                        timer.delay_us(delay.as_us() as u32).await;

                        // We already have the lock on the radio, so start transmitting and do not
                        // have to check anymore
                        transmit(
                            &mut **radio_guard.as_mut().unwrap(),
                            &mut tx_ack.buffer,
                            TxConfig::default(),
                        )
                        .await;
                    }
                } else {
                    // Immediatly drop gruard if we do not longer need it to ACK
                    radio_guard = None;
                }
            })
            .await;
            rx.dirty = false; // Reset for the following iteration
        }
    }

    /// If the frame is malformed/invalid -> parsing error will be returned.
    /// If the frame is no ack'able -> the sequence number will be None.
    /// Second argument in the option is the frame length -> useful to find out
    /// how long we should wait for an ACK
    fn set_ack_request_if_possible<'a, RadioFrame>(
        &self,
        buffer: &'a mut [u8],
    ) -> Result<Option<(u8, u8)>, TransmissionTaskError<RadioFrame::Error>>
    where
        RadioFrame: RadioFrameMut<&'a mut [u8]>,
    {
        let mut frame =
            RadioFrame::new_checked(buffer).map_err(TransmissionTaskError::InvalidDeviceFrame)?;
        let frame_len = frame.data().len() as u8;
        let mut frame =
            Frame::new(frame.data_mut()).map_err(|_err| TransmissionTaskError::InvalidIEEEFrame)?;

        // Only Data and MAC Commands should be able to get an ACK
        let frame_type = frame.frame_control().frame_type();
        if frame_type == FrameType::Data || frame_type == FrameType::MacCommand {
            match frame.addressing().and_then(|addr| addr.dst_address()) {
                Some(addr) if addr.is_unicast() && self.config.ack_unicast => {
                    frame.frame_control_mut().set_ack_request(true);
                    Ok(frame.sequence_number().map(|seq| (seq, frame_len)))
                }
                Some(addr) if addr.is_broadcast() && self.config.ack_broadcast => {
                    frame.frame_control_mut().set_ack_request(true);
                    Ok(frame.sequence_number().map(|seq| (seq, frame_len)))
                }
                Some(_) | None => {
                    // Make sure that the ack_request field is set to false independent on how the
                    // frame was actually created
                    frame.frame_control_mut().set_ack_request(false);
                    Ok(None)
                }
            }
        } else {
            // We want an ACK, and here is the sequence number
            frame.frame_control_mut().set_ack_request(false);
            Ok(None)
        }
    }

    async fn wait_for_valid_ack(radio: &mut R, sequence_number: u8, ack_rx: &mut [u8; 128]) {
        loop {
            let result = receive(radio, ack_rx, RxConfig::default()).await;
            if !result {
                // No succesful receive, try again
                continue;
            }

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

    async fn transmit_package_task(&self, wants_to_transmit_signal: Sender<'_, ()>) -> !
    where
        R: Radio,
        for<'a> R::RadioFrame<&'a mut [u8]>: RadioFrameMut<&'a mut [u8]>,
        Rng: RngCore,
        D: Driver,
    {
        let mut ack_rx = FrameBuffer::default();
        let mut timer = self.timer.clone();

        'outer: loop {
            // Wait until we have a frame to send
            let mut tx = self.driver.transmit().await;

            yield_now().await;

            // Enable ACK in frame coming from higher layers
            let mut sequence_number = None;
            match self.set_ack_request_if_possible::<R::RadioFrame<_>>(&mut tx.buffer) {
                Ok(seq_number) => sequence_number = seq_number,
                Err(TransmissionTaskError::InvalidIEEEFrame) => {
                    // Invalid IEEE frame encountered
                    self.driver.error(driver::Error::InvalidStructure).await;
                }
                Err(TransmissionTaskError::InvalidDeviceFrame(_err)) => {
                    // Invalid device frame encountered
                    self.driver.error(driver::Error::InvalidStructure).await;
                }
            }

            let mut radio_guard = None;
            'ack: for i_ack in 0..MAC_MAX_FRAME_RETIES + 1 {
                // Set vars for CCA
                let backoff_strategy =
                    transmission::CCABackoffStrategy::new_exponential_backoff(&self.rng);
                // Perform CCA
                match transmission::transmit_cca(
                    &self.radio,
                    &mut radio_guard,
                    &wants_to_transmit_signal,
                    &mut tx,
                    &mut timer,
                    backoff_strategy,
                )
                .await
                {
                    Ok(()) => {}
                    Err(_err) => {
                        // Transmission failed
                        self.driver.error(driver::Error::CcaFailed).await;
                        break 'ack;
                    }
                }

                // We now want to try and receive an ACK
                if let Some((sequence_number, frame_length)) = sequence_number {
                    utils::acquire_lock(&self.radio, &wants_to_transmit_signal, &mut radio_guard)
                        .await;

                    let delay = ACKNOWLEDGEMENT_INTERFRAME_SPACING
                        + (MAC_SIFT_PERIOD.max(Duration::from_us(
                            (TURNAROUND_TIME * SYMBOL_RATE_INV_US * frame_length as u32) as i64,
                        )));
                    match select::select(
                        Self::wait_for_valid_ack(
                            &mut *radio_guard.unwrap(),
                            sequence_number,
                            &mut ack_rx.buffer,
                        ),
                        // Timeout for waiting on an ACK
                        timer.delay_us(delay.as_us() as u32),
                    )
                    .await
                    {
                        Either::First(()) => {
                            // ACK succesful, transmission succesful
                            // This releases the radio_gaurd too
                            continue 'outer;
                        }
                        Either::Second(()) => {
                            // Timout, retry logic if following part of the code
                        }
                    }
                } else {
                    // We do not have a sequence number, so do not wait for an ACK
                    // Transmission is considered a success
                    continue 'outer;
                }

                // Whether we succeeded or not, we no longer need sole access to the radio
                // module, so we can release the lock
                radio_guard = None;

                // Wait for SIFS here
                let delay = MAC_SIFT_PERIOD.max(Duration::from_us(
                    (TURNAROUND_TIME * SYMBOL_RATE_INV_US) as i64,
                ));
                timer.delay_us(delay.as_us() as u32).await;

                // Was this the last attempt?
                if i_ack == MAC_MAX_FRAME_RETIES {
                    // Fail transmission
                    self.driver.error(driver::Error::AckFailed).await;
                    break 'ack;
                }
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    use self::driver::tests::*;
    use crate::{phy::radio::tests::*, phy::radio::*, sync::tests::*, sync::*};

    use super::*;

    #[pollster::test]
    pub async fn test_happy_path_transmit_no_ack() {
        let radio = TestRadio::default();
        let mut channel = TestDriverChannel::new();
        let (driver, monitor) = channel.split();
        let mut csma = CsmaDevice::new(
            radio.clone(),
            rand::thread_rng(),
            driver,
            Delay::default(),
            CsmaConfig::default(),
        );

        // Select here, such that everything ends when the test is over
        select::select(csma.run(), async {
            let packet = FrameBuffer::default();
            radio.inner(|inner| {
                inner.assert_nxt.append(
                    &mut [
                        TestRadioEvent::PrepareReceive,
                        // By default we receive
                        TestRadioEvent::Receive,
                        TestRadioEvent::CancelCurrentOperation,
                        TestRadioEvent::PrepareTransmit,
                        // Then we get the request to transmit
                        TestRadioEvent::Transmit,
                        // After which we go back to receiving normal traffic
                        TestRadioEvent::PrepareReceive,
                        TestRadioEvent::Receive,
                    ]
                    .into(),
                );
            });
            monitor.tx.send_async(packet.clone()).await;
            radio.wait_until_asserts_are_consumed().await;
        })
        .await;
    }

    #[pollster::test]
    pub async fn test_happy_path_transmit_with_ack() {
        let radio = TestRadio::default();
        let mut channel = TestDriverChannel::new();
        let (driver, monitor) = channel.split();
        let mut csma = CsmaDevice::new(
            radio.clone(),
            rand::thread_rng(),
            driver,
            Delay::default(),
            CsmaConfig::default(),
        );

        select::select(csma.run(), async {
            let sequence_number = 123;
            let mut packet = FrameBuffer::default();
            let mut frame_repr = FrameBuilder::new_data(&[1, 2, 3, 4])
                .set_sequence_number(sequence_number)
                .set_dst_address(Address::Extended([1, 2, 3, 4, 5, 6, 7, 8]))
                .set_src_address(Address::Extended([1, 2, 3, 4, 9, 8, 7, 6]))
                .set_dst_pan_id(0xfff)
                .set_src_pan_id(0xfff)
                .finalize()
                .unwrap();
            // Set ACK to false, such that we can test if it acks
            frame_repr.frame_control.ack_request = false;

            let token = TestTxToken::from(&mut packet.buffer[..]);
            token.consume(frame_repr.buffer_len(), |buf| {
                let mut frame = Frame::new_unchecked(buf);
                frame_repr.emit(&mut frame);
            });

            // Check if frame is correct
            let frame = TestRadioFrame::new_checked(&packet.buffer).unwrap();
            let _frame = Frame::new(frame.data()).unwrap();

            monitor.tx.send_async(packet.clone()).await;
            radio.inner(|inner| {
                inner.assert_nxt.clear();
                inner.assert_nxt.append(
                    &mut [
                        TestRadioEvent::PrepareReceive,
                        // By default we listen
                        TestRadioEvent::Receive,
                        TestRadioEvent::CancelCurrentOperation,
                        TestRadioEvent::PrepareTransmit,
                        // Then we get the request to transmit
                        TestRadioEvent::Transmit,
                        TestRadioEvent::PrepareReceive,
                        // After which we wait for an ACK
                        TestRadioEvent::Receive,
                    ]
                    .into(),
                );
                inner.total_event_count = 0;
            });
            radio.wait_until_asserts_are_consumed().await;
            radio.inner(|inner| {
                // Assert that we have the correct transmitted packet
                let mut frame = Frame::new_unchecked(&mut packet.buffer);
                frame.frame_control_mut().set_ack_request(true);
                assert_eq!(
                    inner.last_transmitted,
                    Some(packet.buffer),
                    "The transmitted packet should have the ack_request set by default"
                );

                let mut ack_frame = FrameBuffer::default();
                let token = TestTxToken::from(&mut ack_frame.buffer[..]);
                let ack_repr = FrameBuilder::new_imm_ack(sequence_number)
                    .finalize()
                    .unwrap();
                token.consume(ack_repr.buffer_len(), |buf| {
                    let mut frame = Frame::new_unchecked(buf);
                    ack_repr.emit(&mut frame);
                });
                inner.should_receive = Some(ack_frame.buffer);

                inner.assert_nxt.append(
                    &mut [
                        // At the end, we receive again
                        TestRadioEvent::PrepareReceive,
                        TestRadioEvent::Receive,
                    ]
                    .into(),
                )
            });
            radio.wait_until_asserts_are_consumed().await;
            assert!(!monitor.errors.has_item(), "No errors should have occurred");
        })
        .await;
    }

    #[pollster::test]
    pub async fn test_happy_path_receive() {
        let radio = TestRadio::default();

        radio.inner(|inner| {
            inner.assert_nxt.append(
                &mut [
                    TestRadioEvent::Enable,
                    TestRadioEvent::PrepareReceive,
                    TestRadioEvent::Receive,
                ]
                .into(),
            )
        });

        let mut channel = TestDriverChannel::new();
        let (driver, monitor) = channel.split();
        let mut csma = CsmaDevice::new(
            radio.clone(),
            rand::thread_rng(),
            driver,
            Delay::default(),
            CsmaConfig::default(),
        );

        select::select(csma.run(), async {
            let sequence_number = 123;
            let mut packet = FrameBuffer::default();
            let mut frame_repr = FrameBuilder::new_data(&[1, 2, 3, 4])
                .set_sequence_number(sequence_number)
                .set_dst_address(Address::Extended([1, 2, 3, 4, 5, 6, 7, 8]))
                .set_src_address(Address::Extended([1, 2, 3, 4, 9, 8, 7, 6]))
                .set_dst_pan_id(0xfff)
                .set_src_pan_id(0xfff)
                .finalize()
                .unwrap();
            frame_repr.frame_control.ack_request = true;

            let token = TestTxToken::from(&mut packet.buffer[..]);
            token.consume(frame_repr.buffer_len(), |buf| {
                let mut frame = Frame::new_unchecked(buf);
                frame_repr.emit(&mut frame);
            });
            radio.wait_until_asserts_are_consumed().await;
            radio.inner(|inner| {
                inner.should_receive = Some(packet.buffer);
                inner
                    .assert_nxt
                    .append(&mut [TestRadioEvent::PrepareTransmit, TestRadioEvent::Transmit].into())
            });
            assert_eq!(monitor.rx.receive().await.buffer, packet.buffer);
            radio.wait_until_asserts_are_consumed().await;
            radio.inner(|inner| {
                assert_eq!(
                    inner.last_transmitted.map(|frame| {
                        let frame = TestRadioFrame::new_checked(frame)
                            .expect("The frame should be a valid TestTxFrame");
                        let frame = Frame::new(frame.data()).expect("Should be a valid IEEE frame");

                        frame.frame_control().frame_type()
                    }),
                    Some(FrameType::Ack),
                    "An ACK request should return an ACK"
                );
            })
        })
        .await;
    }

    #[pollster::test]
    pub async fn test_receive_no_ack() {
        let radio = TestRadio::default();

        radio.inner(|inner| {
            inner.assert_nxt.append(
                &mut [
                    TestRadioEvent::Enable,
                    TestRadioEvent::PrepareReceive,
                    TestRadioEvent::Receive,
                ]
                .into(),
            )
        });

        let mut channel = TestDriverChannel::new();
        let (driver, monitor) = channel.split();
        let mut csma = CsmaDevice::new(
            radio.clone(),
            rand::thread_rng(),
            driver,
            Delay::default(),
            CsmaConfig::default(),
        );

        select::select(csma.run(), async {
            let sequence_number = 123;
            let mut packet = FrameBuffer::default();
            let mut frame_repr = FrameBuilder::new_data(&[1, 2, 3, 4])
                .set_sequence_number(sequence_number)
                .set_dst_address(Address::Extended([1, 2, 3, 4, 5, 6, 7, 8]))
                .set_src_address(Address::Extended([1, 2, 3, 4, 9, 8, 7, 6]))
                .set_dst_pan_id(0xfff)
                .set_src_pan_id(0xfff)
                .finalize()
                .unwrap();
            frame_repr.frame_control.ack_request = false;

            let token = TestTxToken::from(&mut packet.buffer[..]);
            token.consume(frame_repr.buffer_len(), |buf| {
                let mut frame = Frame::new_unchecked(buf);
                frame_repr.emit(&mut frame);
            });
            radio.wait_until_asserts_are_consumed().await;
            radio.inner(|inner| {
                inner.should_receive = Some(packet.buffer);
                inner
                    .assert_nxt
                    .append(&mut [TestRadioEvent::PrepareReceive, TestRadioEvent::Receive].into())
            });
            assert_eq!(monitor.rx.receive().await.buffer, packet.buffer);
            radio.wait_until_asserts_are_consumed().await;
            radio.inner(|inner| {
                assert!(
                    inner.last_transmitted.is_none(),
                    "If there is not an ACK request, we should not ACK (by default)"
                );
            })
        })
        .await;
    }

    #[pollster::test]
    pub async fn test_wait_for_ack_but_receive_garbage_and_cca_issues() {
        let radio = TestRadio::default();
        let mut channel = TestDriverChannel::new();
        let (driver, monitor) = channel.split();
        let mut csma = CsmaDevice::new(
            radio.clone(),
            rand::thread_rng(),
            driver,
            Delay::default(),
            CsmaConfig::default(),
        );

        select::select(csma.run(), async {
            let sequence_number = 123;
            let mut packet = FrameBuffer::default();
            let mut frame_repr = FrameBuilder::new_data(&[1, 2, 3, 4])
                .set_sequence_number(sequence_number)
                .set_dst_address(Address::Extended([1, 2, 3, 4, 5, 6, 7, 8]))
                .set_src_address(Address::Extended([1, 2, 3, 4, 9, 8, 7, 6]))
                .set_dst_pan_id(0xfff)
                .set_src_pan_id(0xfff)
                .finalize()
                .unwrap();
            // Set ACK to false, such that we can test if it acks
            frame_repr.frame_control.ack_request = false;

            let token = TestTxToken::from(&mut packet.buffer[..]);
            token.consume(frame_repr.buffer_len(), |buf| {
                let mut frame = Frame::new_unchecked(buf);
                frame_repr.emit(&mut frame);
            });

            // Check if frame is correct
            let frame = TestRadioFrame::new_checked(&packet.buffer).unwrap();
            let _frame = Frame::new(frame.data()).unwrap();

            monitor.tx.send_async(packet.clone()).await;
            radio.inner(|inner| {
                inner.assert_nxt.clear();
                inner.assert_nxt.append(
                    &mut [
                        TestRadioEvent::PrepareReceive,
                        // By default we receive
                        TestRadioEvent::Receive,
                        TestRadioEvent::CancelCurrentOperation,
                        TestRadioEvent::PrepareTransmit,
                        // Then we get a request to transmit
                        TestRadioEvent::Transmit,
                        // After which we wait for an ACK
                        TestRadioEvent::PrepareReceive,
                        TestRadioEvent::Receive,
                    ]
                    .into(),
                );
                inner.total_event_count = 0;
            });
            radio.wait_until_asserts_are_consumed().await;
            radio.inner(|inner| {
                // Assert that we have the correct transmitted packet
                let mut frame = Frame::new_unchecked(&mut packet.buffer);
                frame.frame_control_mut().set_ack_request(true);
                assert_eq!(
                    inner.last_transmitted,
                    Some(packet.buffer),
                    "The transmitted packet should have the ack_request set by default"
                );

                let mut ack_frame = FrameBuffer::default();
                ack_frame.buffer[0] = 42;
                ack_frame.buffer[1] = 42;
                ack_frame.buffer[2] = 42;
                ack_frame.buffer[3] = 42;
                inner.should_receive = Some(ack_frame.buffer);

                inner.cca_fail = true;
                inner.assert_nxt.append(
                    &mut [
                        TestRadioEvent::PrepareReceive,
                        // We receive garbage, timer is not yet done
                        TestRadioEvent::Receive,
                        TestRadioEvent::CancelCurrentOperation,
                    ]
                    .repeat(3) // magic number corresponds to delay
                    .into(),
                );
                inner.assert_nxt.append(
                    &mut [
                        // CCA should have failed here
                        TestRadioEvent::PrepareTransmit,
                        TestRadioEvent::Transmit,
                        // We go back to receive to process other messages, until delay
                        TestRadioEvent::PrepareReceive,
                        TestRadioEvent::Receive,
                        TestRadioEvent::CancelCurrentOperation,
                        // We go back to receive to process other messages, until delay
                        TestRadioEvent::PrepareReceive,
                        TestRadioEvent::Receive,
                        TestRadioEvent::CancelCurrentOperation,
                    ]
                    .repeat(MAC_MAX_CSMA_BACKOFFS as usize - 1)
                    .into(),
                );
            });
            radio.wait_until_asserts_are_consumed().await;
            assert_eq!(
                monitor.errors.receive().await,
                driver::Error::CcaFailed, // CCA has failed, so we propagate an error up
                "Packet transmission should fail due to CCA"
            );
        })
        .await;
    }

    #[pollster::test]
    pub async fn test_transmit_no_ack_received() {
        let radio = TestRadio::default();
        let mut channel = TestDriverChannel::new();
        let (driver, monitor) = channel.split();
        let mut csma = CsmaDevice::new(
            radio.clone(),
            rand::thread_rng(),
            driver,
            Delay::default(),
            CsmaConfig::default(),
        );

        select::select(csma.run(), async {
            let sequence_number = 123;
            let mut packet = FrameBuffer::default();
            let mut frame_repr = FrameBuilder::new_data(&[1, 2, 3, 4])
                .set_sequence_number(sequence_number)
                .set_dst_address(Address::Extended([1, 2, 3, 4, 5, 6, 7, 8]))
                .set_src_address(Address::Extended([1, 2, 3, 4, 9, 8, 7, 6]))
                .set_dst_pan_id(0xfff)
                .set_src_pan_id(0xfff)
                .finalize()
                .unwrap();
            // Set ACK to false, such that we can test if it acks
            frame_repr.frame_control.ack_request = false;

            let token = TestTxToken::from(&mut packet.buffer[..]);
            token.consume(frame_repr.buffer_len(), |buf| {
                let mut frame = Frame::new_unchecked(buf);
                frame_repr.emit(&mut frame);
            });

            // Check if frame is correct
            let frame = TestRadioFrame::new_checked(&packet.buffer).unwrap();
            let _frame = Frame::new(frame.data()).unwrap();

            monitor.tx.send_async(packet.clone()).await;
            radio.inner(|inner| {
                inner.assert_nxt.clear();
                inner.assert_nxt.append(
                    &mut [
                        TestRadioEvent::PrepareReceive,
                        // By default we receive
                        TestRadioEvent::Receive,
                        TestRadioEvent::CancelCurrentOperation,
                        TestRadioEvent::PrepareTransmit,
                        // Then we get a request to transmit
                        TestRadioEvent::Transmit,
                    ]
                    .into(),
                );
                inner.assert_nxt.append(
                    &mut [
                        // After which we wait for an ACK, which does not come
                        TestRadioEvent::PrepareReceive,
                        TestRadioEvent::Receive,
                        TestRadioEvent::CancelCurrentOperation,
                    ]
                    .repeat(3)
                    .into(),
                );
                inner.total_event_count = 0;
            });
            radio.wait_until_asserts_are_consumed().await;
            assert_eq!(
                monitor.errors.receive().await,
                driver::Error::AckFailed, // ACK has failed, so we propagate an error up
                "Packet transmission should fail due to ACK not received after to many times"
            );
        })
        .await;
    }

    #[pollster::test]
    pub async fn test_do_not_ack_by_default_on_broadcast() {
        let radio = TestRadio::default();

        radio.inner(|inner| {
            inner.assert_nxt.append(
                &mut [
                    TestRadioEvent::Enable,
                    TestRadioEvent::PrepareReceive,
                    TestRadioEvent::Receive,
                ]
                .into(),
            )
        });

        let mut channel = TestDriverChannel::new();
        let (driver, monitor) = channel.split();
        let mut csma = CsmaDevice::new(
            radio.clone(),
            rand::thread_rng(),
            driver,
            Delay::default(),
            CsmaConfig::default(),
        );

        select::select(csma.run(), async {
            let sequence_number = 123;
            let mut packet = FrameBuffer::default();
            let mut frame_repr = FrameBuilder::new_data(&[1, 2, 3, 4])
                .set_sequence_number(sequence_number)
                .set_dst_address(Address::BROADCAST)
                .set_src_address(Address::Extended([1, 2, 3, 4, 9, 8, 7, 6]))
                .set_dst_pan_id(0xfff)
                .set_src_pan_id(0xfff)
                .finalize()
                .unwrap();
            frame_repr.frame_control.ack_request = true; // This should be ignored

            let token = TestTxToken::from(&mut packet.buffer[..]);
            token.consume(frame_repr.buffer_len(), |buf| {
                let mut frame = Frame::new_unchecked(buf);
                frame_repr.emit(&mut frame);
            });
            radio.wait_until_asserts_are_consumed().await;
            radio.inner(|inner| {
                inner.should_receive = Some(packet.buffer);
                inner
                    .assert_nxt
                    .append(&mut [TestRadioEvent::PrepareReceive, TestRadioEvent::Receive].into())
            });
            assert_eq!(monitor.rx.receive().await.buffer, packet.buffer);
            radio.wait_until_asserts_are_consumed().await;
            radio.inner(|inner| {
                assert_eq!(
                    inner.last_transmitted, None,
                    "No ACK should have been transmitted on a broadcast"
                );
            })
        })
        .await;
    }
}
