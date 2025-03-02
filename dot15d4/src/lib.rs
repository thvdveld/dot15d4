#![no_std]

#[cfg(any(feature = "std", test))]
#[macro_use]
extern crate std;

#[macro_use]
pub(crate) mod utils;

pub use dot15d4_frame as frame;

pub mod mac;
pub mod phy;
pub mod sync;
pub mod time;
pub mod upper;

use crate::{
    phy::radio::{Radio, RadioFrameMut},
    sync::{channel::Channel, mutex::Mutex, select, Either},
    upper::UpperLayer,
};
use embedded_hal_async::delay::DelayNs;
use rand_core::RngCore;

use self::{mac::MacService, phy::PhyService};

pub struct Device<R: Radio, Rng, U: UpperLayer, TIMER> {
    radio: Mutex<R>,
    rng: Mutex<Rng>,
    upper_layer: U,
    timer: TIMER,
}

impl<R, Rng, U, TIMER> Device<R, Rng, U, TIMER>
where
    R: Radio,
    Rng: RngCore,
    U: UpperLayer,
{
    pub fn new(radio: R, rng: Rng, upper_layer: U, timer: TIMER) -> Self {
        Self {
            radio: Mutex::new(radio),
            rng: Mutex::new(rng),
            upper_layer,
            timer,
        }
    }
}

impl<R, Rng, U, TIMER> Device<R, Rng, U, TIMER>
where
    R: Radio,
    for<'a> R::RadioFrame<&'a mut [u8]>: RadioFrameMut<&'a mut [u8]>,
    for<'a> R::TxToken<'a>: From<&'a mut [u8]>,
    Rng: RngCore,
    U: UpperLayer,
    TIMER: DelayNs + Clone,
{
    pub async fn run(&mut self) -> ! {
        let (mut tx, mut rx) = (Channel::new(), Channel::new());
        let (tx_send, tx_recv) = tx.split();
        let (rx_send, rx_recv) = rx.split();

        let mut tx_done = Channel::new();
        let (tx_done_send, tx_done_recv) = tx_done.split();

        let mut phy_service = PhyService::new(&mut self.radio, tx_recv, rx_send, tx_done_send);
        let mut mac_service = MacService::<'_, Rng, U, TIMER, R>::new(
            &mut self.rng,
            &mut self.upper_layer,
            self.timer.clone(),
            rx_recv,
            tx_send,
            tx_done_recv,
        );

        match select::select(mac_service.run(), phy_service.run()).await {
            Either::First(_) => panic!("Tasks should never terminate, MAC MCPS just did"),
            Either::Second(_) => panic!("Tasks should never terminate, MAC MLME just did"),
        }
    }

    // pub async fn start_as_coordinator(&mut self) {
    //     self.scan_energy().await;
    // }

    // pub async fn start(&mut self) {
    //     //
    //     self.scan_channels().await;
    // }

    // async fn receive_beacon_request<'a>(
    //     &self,
    //     buffer: &mut [u8; 128],
    //     radio_guard: &mut Option<MutexGuard<'a, R>>,
    // ) {
    //     receive(
    //         &mut **radio_guard.as_mut().unwrap(),
    //         buffer,
    //         RxConfig {
    //             channel: crate::phy::config::Channel::_26,
    //         },
    //     )
    //     .await;
    // }
}
