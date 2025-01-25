use embedded_hal_async::delay::DelayNs;
use rand_core::RngCore;

use super::user_configurable_constants::*;
use super::utils;

use crate::{
    mac::Error,
    phy::{
        config::{self, TxConfig},
        radio::{futures::transmit, Radio},
        FrameBuffer,
    },
    sync::{
        channel::Sender,
        join::join,
        mutex::{Mutex, MutexGuard},
    },
    upper::UpperLayer,
};

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TransmissionError {
    CcaError,
}

#[allow(clippy::too_many_arguments)]
pub async fn transmit_cca<'m, R, TIMER, Rng, U>(
    radio: &'m Mutex<R>,
    radio_guard: &mut Option<MutexGuard<'m, R>>,
    channel: config::Channel,
    wants_to_transmit_signal: &Sender<'_, ()>,
    tx_frame: &mut FrameBuffer,
    timer: &mut TIMER,
    mut backoff_strategy: CCABackoffStrategy<'_, Rng>,
    upper_layer: &U,
) -> Result<(), TransmissionError>
where
    R: Radio,
    TIMER: DelayNs,
    Rng: RngCore,
    U: UpperLayer,
{
    'cca: for number_of_backoffs in 1..MAC_MAX_CSMA_BACKOFFS + 1 {
        // try to transmit
        let transmission_result = {
            utils::acquire_lock(radio, wants_to_transmit_signal, radio_guard).await;
            transmit(
                &mut **radio_guard.as_mut().unwrap(),
                &mut tx_frame.buffer,
                TxConfig {
                    channel,
                    ..TxConfig::default_with_cca()
                },
            )
            .await
        };
        if transmission_result {
            break 'cca; // Send succesfully, now wait for ack
        }

        // As we are now going to wait a number of periods, release the
        // mutex on the radio
        *radio_guard = None;

        // CCA did not go succesfully
        // Was this the last attempt?
        if number_of_backoffs == MAC_MAX_CSMA_BACKOFFS {
            return Err(TransmissionError::CcaError); // Fail transmission
        } else {
            // Perform backoff and report current status to upper_layer
            join(
                backoff_strategy.perform_backoff(timer),
                upper_layer.error(Error::CcaBackoff(number_of_backoffs)),
            )
            .await;
        }
    }

    Ok(())
}

pub enum CCABackoffStrategy<'r, Rng: RngCore> {
    None,
    ExponentialBackoff {
        backoff_exponent: u16,
        rng: &'r Mutex<Rng>,
    },
}

impl<'r, Rng: RngCore> CCABackoffStrategy<'r, Rng> {
    pub fn new_none() -> Self {
        Self::None
    }

    pub fn new_exponential_backoff(rng: &'r Mutex<Rng>) -> Self {
        Self::ExponentialBackoff {
            backoff_exponent: MAC_MIN_BE,
            rng,
        }
    }

    pub async fn perform_backoff<TIMER: DelayNs>(&mut self, timer: &mut TIMER) {
        match self {
            Self::None => {}
            Self::ExponentialBackoff {
                backoff_exponent,
                rng,
            } => {
                // Wait now for a random number of periods, before retrying
                *backoff_exponent = core::cmp::min(*backoff_exponent + 1, MAC_MAX_BE);

                // delay periods = random(2^{BE} - 1) periods
                // Page 63 IEEE 802.15.4 2015 edition
                let max_backoff = (1u32 << *backoff_exponent) - 1;
                // The +1 in (max_backoff + 1) comes from the interpretation that the random()
                // function used in the specification includes max_backoff as a
                // possible value. The possible values periods now can take are:
                // [0, max_backoff].
                let periods = rng.lock().await.next_u32() % (max_backoff + 1);
                let delay = MAC_UNIT_BACKOFF_DURATION * periods as usize;
                timer.delay_us(delay.as_us() as u32).await;
            }
        }
    }
}
