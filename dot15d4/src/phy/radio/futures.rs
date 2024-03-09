use core::future::Future;
use core::pin::Pin;
use core::task::{ready, Poll};
use std::cell::RefCell;
use std::mem::MaybeUninit;

use super::Radio;
use crate::phy::config::{RxConfig, TxConfig};
use crate::sync::mutex::Mutex;

/// Helper structure to have cleanup logic when dropping a future
struct OnDrop<F: FnOnce()> {
    f: MaybeUninit<F>, // needed such that we can have an FnOnce in drop
}

impl<F: FnOnce()> OnDrop<F> {
    pub fn new(f: F) -> Self {
        Self {
            f: MaybeUninit::new(f),
        }
    }
}

impl<F: FnOnce()> Drop for OnDrop<F> {
    fn drop(&mut self) {
        // Safety: Drop is only called once and we can only construct OnDrop from initialized memory
        unsafe { self.f.as_ptr().read()() };
    }
}

enum TransmissionTaskState {
    Preparing,
    Transmitting,
}

/// Future around transmitting through a radio. Use the `transmit` function when you want to
/// use this future.
pub struct TransmitTask<'task, T, R: Radio> {
    data: &'task T,
    radio: &'task mut R,
    state: TransmissionTaskState,
    config: TxConfig,
}

/// Convenience Future around transmitting through the radio. This future first prepares the radio,
/// then transmits before succeeding. This future, upon canceling, stops the radio from transmitting
/// and puts the radio in an IDLE state.
#[allow(clippy::await_holding_refcell_ref)]
pub async fn transmit<'task, T: AsRef<[u8]>, R: Radio>(
    radio: &'task mut R,
    data: &'task T,
    config: TxConfig,
) -> bool {
    let radio = RefCell::new(radio);
    // Should just work as a drop is handled at the end, after the other radio uses
    let _on_drop = OnDrop::new(|| unsafe { radio.borrow_mut().cancel_current_opperation() });

    let mut radio = radio.borrow_mut();
    unsafe {
        radio.prepare_transmit(&config, data.as_ref()).await;
    }
    radio.transmit().await
}

// impl<'task, T, R> Future for TransmitTask<'task, T, R>
// where
//     R: Radio,
//     T: AsRef<[u8]>,
// {
//     type Output = bool;

//     fn poll(self: Pin<&mut Self>, cx: &mut core::task::Context<'_>) -> Poll<Self::Output> {
//         let this = self.get_mut();
//         'outer: loop {
//             match this.state {
//                 TransmissionTaskState::Preparing => {
//                     unsafe {
//                         ready!(this
//                             .radio
//                             .prepare_transmit(cx, &this.config, this.data.as_ref()))
//                     };
//                     this.state = TransmissionTaskState::Transmitting;

//                     continue 'outer; // We can make more progress
//                 }
//                 TransmissionTaskState::Transmitting => {
//                     let result = ready!(this.radio.transmit(cx));
//                     break 'outer Poll::Ready(result);
//                 }
//             }
//         }
//     }
// }

// impl<T, R: Radio> Drop for TransmitTask<'_, T, R> {
//     fn drop(&mut self) {
//         self.radio.cancel_current_opperation()
//     }
// }

// enum ReceiveTaskState {
//     Preparing,
//     Receiving,
// }

/// Future around receiving through a radio. Use the `receive` function when you want to
/// use this future.
// pub struct ReceiveTask<'task, R: Radio> {
//     data: &'task mut [u8; 128],
//     radio: &'task mut R,
//     state: ReceiveTaskState,
//     config: RxConfig,
// }

/// Convenience Future around receiving through the radio. This future first prepares the radio,
/// then receives before succeeding. This future, upon canceling, stops the radio from receiving
/// and puts the radio in an IDLE state.
#[allow(clippy::await_holding_refcell_ref)]
pub async fn receive<'task, R: Radio>(
    radio: &'task mut R,
    data: &'task mut [u8; 128],
    config: RxConfig,
) -> bool {
    let radio = RefCell::new(radio);
    // Should just work as a drop is handled at the end, after the other radio uses
    let _on_drop = OnDrop::new(|| unsafe { radio.borrow_mut().cancel_current_opperation() });

    let mut radio = radio.borrow_mut();
    unsafe {
        radio.prepare_receive(&config, data).await;
    }
    radio.receive().await
}

// impl<'task, R> Future for ReceiveTask<'task, R>
// where
//     R: Radio,
// {
//     type Output = bool;

//     fn poll(self: Pin<&mut Self>, cx: &mut core::task::Context<'_>) -> Poll<Self::Output> {
//         let this = self.get_mut();
//         'outer: loop {
//             match this.state {
//                 ReceiveTaskState::Preparing => {
//                     unsafe { ready!(this.radio.prepare_receive(cx, &this.config, this.data)) };
//                     this.state = ReceiveTaskState::Receiving;

//                     continue 'outer;
//                 }
//                 ReceiveTaskState::Receiving => {
//                     let result = ready!(this.radio.receive(cx));
//                     break 'outer Poll::Ready(result);
//                 }
//             }
//         }
//     }
// }

// impl<R> Drop for ReceiveTask<'_, R>
// where
//     R: Radio,
// {
//     fn drop(&mut self) {
//         self.radio.cancel_current_opperation()
//     }
// }
