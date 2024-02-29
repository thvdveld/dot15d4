pub mod constants;
pub mod user_configurable_constants;

use constants::*;
use rand_core::RngCore;
use user_configurable_constants::*;

use crate::{
    frame::{Address, Frame, FrameBuilder, FrameRepr, FrameType},
    phy::{
        config::{RxConfig, TxConfig},
        driver::{self, Driver, PacketBuffer},
        radio::{
            futures::{receive, transmit},
            Radio, RadioFrame, RadioFrameMut,
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
pub struct CsmaDevice<R: Radio, Rng, D: Driver> {
    radio: Mutex<R>,
    rng: Mutex<Rng>,
    driver: D,
    hardware_address: [u8; 8],
}

impl<R, Rng, D> CsmaDevice<R, Rng, D>
where
    R: Radio,
    Rng: RngCore,
    D: Driver,
{
    /// Creates a new CSMA object that is ready to be run
    pub fn new(radio: R, rng: Rng, driver: D) -> Self {
        let hardware_address = radio.ieee802154_address();
        CsmaDevice {
            radio: Mutex::new(radio),
            rng: Mutex::new(rng),
            driver,
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
}

impl<R, Rng, D> CsmaDevice<R, Rng, D>
where
    R: Radio,
    for<'a> R::RadioFrame<&'a mut [u8]>: RadioFrameMut<&'a mut [u8]>,
    Rng: RngCore,
    D: Driver,
{
    /// Run the CSMA module. This should be run in its own task and polled seperately.
    pub async fn run(&self) -> ! {
        let mut wants_to_transmit_signal = Channel::new();
        let (sender, receiver) = wants_to_transmit_signal.split();
        match select::select(
            transmit_package_task(&self.radio, &self.rng, &self.driver, sender),
            receive_package_task(&self.radio, &self.driver, receiver, &self.hardware_address),
        )
        .await
        {
            Either::First(_) => panic!("Tasks should never terminate, csma transmission just did"),
            Either::Second(_) => panic!("Tasks should never terminate, csma receiving just did"),
        }
    }
}

/// Checks if the current frame is intended for us. For the hardware address, the full 64bit
/// address should be provided.
fn is_package_for_us(hardware_address: &[u8; 8], frame: &FrameRepr<'_>) -> bool {
    let Some(addr) = frame
        .addressing_fields
        .and_then(|fields| fields.dst_address)
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

async fn receive_package_task<R, D: Driver>(
    radio: &Mutex<R>,
    driver: &D,
    wants_to_transmit_signal: Receiver<'_, ()>,
    hardware_address: &[u8; 8],
) -> !
where
    R: Radio,
    for<'a> R::RadioFrame<&'a mut [u8]>: RadioFrameMut<&'a mut [u8]>,
{
    let mut rx = PacketBuffer::default();
    let mut radio_guard = None;

    // Allocate tx buffer for ACK messages
    let mut tx_ack = PacketBuffer::default();

    'outer: loop {
        yield_now().await;

        // try to receive something
        let receive_result = {
            radio_guard = match radio_guard {
                Some(_) => radio_guard,
                None => Some(radio.lock().await),
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
            let Ok(frame) = FrameRepr::new_checked(frame.data()) else {
                rx.dirty = false;
                continue 'outer;
            };

            // Check if package is meant for us
            if !is_package_for_us(hardware_address, &frame) {
                // Package is not for us to handle, ignore
                rx.dirty = false;
                continue 'outer;
            }

            if frame.frame_control.frame_type == FrameType::Ack {
                // Ignore this ACK as it is not at an expected time, or not for us
                rx.dirty = false;
                continue 'outer;
            }

            (frame.frame_control.ack_request, frame.sequence_number)
        };

        // Concurrently send the received message to the upper layers, and if we need to ACK, we ACK
        rx.dirty = true;
        join::join(driver.received(core::mem::take(&mut rx)), async {
            if should_ack {
                // Set correct sequence number and send an ACK only if valid sequence number
                if let Some(sequence_number) = sequence_number {
                    let ieee_repr = FrameBuilder::new_imm_ack(sequence_number)
                        .finalize()
                        .expect("A simple imm-ACK should always be possible to build");
                    let ack_token = DriverTxToken::from(&mut tx_ack);
                    ack_token.consume(ieee_repr.buffer_len(), |buffer| {
                        let mut frame = FrameRepr::new_unchecked(buffer);
                        ieee_repr.emit(&mut frame);
                    });

                    // Wait before sending the ACK (AIFS)
                    let delay = ACKNOWLEDGEMENT_INTERYFRAME_SPACING;
                    Timer::after(delay).await;

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

fn set_ack_request_if_possible<'a, Frame>(
    buffer: &'a mut [u8],
) -> Result<Option<u8>, TransmissionTaskError<Frame::Error>>
where
    Frame: RadioFrameMut<&'a mut [u8]>,
{
    let mut frame =
        Frame::new_checked(buffer).map_err(TransmissionTaskError::InvalidDeviceFrame)?;
    let mut frame = FrameRepr::new_checked(frame.data_mut())
        .map_err(|_err| TransmissionTaskError::InvalidIEEEFrame)?;
    frame.set_ack_request(true);
    Ok(frame.sequence_number())
}

async fn wait_for_valid_ack<R: Radio>(radio: &mut R, sequence_number: u8, ack_rx: &mut [u8; 128]) {
    loop {
        let result = receive(radio, ack_rx, RxConfig::default()).await;
        if !result {
            continue;
        } // No succesful receive, try again

        // Check if we received a valid ACK
        let Ok(frame) = R::RadioFrame::new_checked(&*ack_rx) else {
            continue;
        };
        let Ok(frame) = FrameRepr::new_checked(frame.data()) else {
            continue;
        };

        if frame.frame_type() == FrameType::Ack && frame.sequence_number() == Some(sequence_number)
        {
            return;
        }
    }
}

async fn transmit_package_task<'s, R, Rng, D>(
    radio: &Mutex<R>,
    rng: &Mutex<Rng>,
    driver: &D,
    wants_to_transmit_signal: Sender<'s, ()>,
) -> !
where
    R: Radio,
    for<'a> R::RadioFrame<&'a mut [u8]>: RadioFrameMut<&'a mut [u8]>,
    Rng: RngCore,
    D: Driver,
{
    let mut ack_rx = PacketBuffer::default();

    'outer: loop {
        let mut tx = driver.transmit().await; // Wait until we have a frame to send

        yield_now().await;

        // Enable ACK in frame coming from higher layers
        let mut sequence_number = None;
        match set_ack_request_if_possible::<R::RadioFrame<_>>(&mut tx.buffer) {
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
                                match radio.try_lock() {
                                    Ok(guard) => {
                                        // wants_to_transmit_signal.reset(); // reset signal, such that the receiving end may continue the next time it acquires the lock
                                        break 'inner Some(guard);
                                    }
                                    Err(_) => {
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
                let periods = rng.lock().await.next_u32() % (max_backoff + 1);
                let delay = MAC_UNIT_BACKOFF_DURATION * periods;
                Timer::after(delay).await;
            }

            // We now want to try and receive an ACK
            if let Some(sequence_number) = sequence_number {
                radio_guard = match radio_guard {
                    Some(_) => radio_guard,
                    None => {
                        'inner: loop {
                            // repeatably ask for the lock, as this might need a few tries to prevent deadlocks
                            match radio.try_lock() {
                                Ok(guard) => {
                                    // wants_to_transmit_signal.reset(); // reset signal, such that the receiving end may continue the next time it acquires the lock
                                    break 'inner Some(guard);
                                }
                                Err(_) => {
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
                    wait_for_valid_ack(
                        &mut *radio_guard.unwrap(),
                        sequence_number,
                        &mut ack_rx.buffer,
                    ),
                    Timer::after(delay), // Timeout for waiting on an ACK
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
            Timer::after(delay).await;

            // Was this the last attempt?
            if i_ack == MAC_MAX_FRAME_RETIES {
                break 'ack; // Fail transmission
            }
        }
    }
}
