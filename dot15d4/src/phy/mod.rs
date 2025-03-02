//! Access to IEEE 802.15.4 devices.
//!
//! This module provides access to IEEE 802.15.4 devices. It provides a trait
//! for transmitting and recieving frames, [Device].

pub mod config;
pub mod constants;
pub mod pib;
pub mod radio;

use radio::{Radio, RadioFrameMut};

use crate::sync::{
    channel::{Receiver, Sender},
    mutex::{Mutex, MutexGuard},
    select,
    yield_now::yield_now,
    Either,
};

use self::{
    config::{RxConfig, TxConfig},
    radio::futures::{receive, transmit},
};

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Error {
    /// Ack failed, after to many retransmissions
    AckFailed,
    /// The buffer did not follow the correct device structure
    InvalidDeviceStructure,
    /// Something went wrong in the radio
    RadioError,
}

/// A buffer that is used to store 1 frame.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, PartialEq)]
pub struct FrameBuffer {
    /// The data of the frame that should be transmitted over the radio.
    /// Normally, a MAC layer frame is 127 bytes long.
    /// Some radios, like the one on the nRF52840, have the possibility to not
    /// having to include the checksum at the end, but require one extra byte to
    /// specify the length of the frame. This results in this case in
    /// needing 126 bytes in total. However, some radios like the one on the
    /// Zolertia Zoul, add extra information about the link quality. In that
    /// case, the total buffer length for receiving data comes on 128 bytes.
    /// If you would like to support a radio that needs more than 128 bytes,
    /// please file a PR.
    pub buffer: [u8; 128],
    /// Whether or not the buffer is ready to be read from
    pub dirty: bool,
}

impl Default for FrameBuffer {
    fn default() -> Self {
        Self {
            buffer: [0u8; 128],
            dirty: false,
        }
    }
}

/// Structure handling PHY sublayer services. This runs the main event loop
/// that handles interactions with the MAC sublayer (sending/receiving frames).
/// It uses signals to communicate with the MAC sublayer.
pub struct PhyService<'radio, R: Radio> {
    radio: &'radio mut Mutex<R>,
    tx_recv: Receiver<'radio, FrameBuffer>,
    rx_send: Sender<'radio, FrameBuffer>,
    tx_done: Sender<'radio, ()>,
    /// PAN Information Base
    pub pib: pib::Pib,
}

impl<'radio, R> PhyService<'radio, R>
where
    R: Radio,
{
    /// Creates a new [`PhyService<R>`].
    pub fn new(
        radio: &'radio mut Mutex<R>,
        tx_recv: Receiver<'radio, FrameBuffer>,
        rx_send: Sender<'radio, FrameBuffer>,
        tx_done: Sender<'radio, ()>,
    ) -> Self {
        Self {
            radio,
            tx_recv,
            rx_send,
            tx_done,
            pib: pib::Pib::default(),
        }
    }
}

impl<R> PhyService<'_, R>
where
    R: Radio,
    for<'a> R::RadioFrame<&'a mut [u8]>: RadioFrameMut<&'a mut [u8]>,
    for<'a> R::TxToken<'a>: From<&'a mut [u8]>,
{
    /// Run the main event loop used by the PHY sublayer for its operation. For
    /// now, the loop waits for either receiving a frame from the MAC sublayer
    ///  or receiving a frame from the radio.
    pub async fn run(&mut self) {
        self.radio.get_mut().enable().await; // Wake up radio
        let mut rx_frame = FrameBuffer::default();
        let mut radio_guard = self.radio.lock().await;

        loop {
            yield_now().await;

            match select::select(
                self.listening(&mut rx_frame, &mut radio_guard),
                self.mac_recv(),
            )
            .await
            {
                Either::First(_) => {
                    self.mac_send(core::mem::take(&mut rx_frame)).await;
                }
                Either::Second(mut tx_frame) => {
                    self.transmit_frame(&mut tx_frame, &mut radio_guard).await;
                    self.tx_done.send_async(()).await;
                }
            };
        }
        //
    }

    /// Send a frame back to the MAC sublayer.
    async fn mac_send(&self, tx: FrameBuffer) {
        self.rx_send.send_async(tx).await;
    }

    /// Wait for a frame from the MAC sublayer to be transmitted.
    async fn mac_recv(&self) -> FrameBuffer {
        self.tx_recv.receive().await
    }

    /// Listen for a frame on the radio
    async fn listening(&self, frame: &mut FrameBuffer, radio_guard: &mut MutexGuard<'_, R>) {
        receive(
            &mut **radio_guard,
            &mut frame.buffer,
            RxConfig {
                channel: self.pib.current_channel.try_into().unwrap(),
            },
        )
        .await;
    }

    /// Transmit the given frame to the radio
    async fn transmit_frame(&self, frame: &mut FrameBuffer, radio_guard: &mut MutexGuard<'_, R>) {
        transmit(
            &mut **radio_guard,
            &mut frame.buffer,
            TxConfig {
                channel: self.pib.current_channel.try_into().unwrap(),
                ..Default::default()
            },
        )
        .await;
    }
}
