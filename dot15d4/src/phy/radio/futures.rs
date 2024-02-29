use core::future::Future;
use core::pin::Pin;
use core::task::{ready, Poll};

use super::Radio;
use crate::phy::config::{RxConfig, TxConfig};

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
pub fn transmit<'task, T, R: Radio>(
    radio: &'task mut R,
    data: &'task T,
    config: TxConfig,
) -> TransmitTask<'task, T, R> {
    TransmitTask {
        radio,
        data,
        state: TransmissionTaskState::Preparing,
        config,
    }
}

impl<'task, T, R> Future for TransmitTask<'task, T, R>
where
    R: Radio,
    T: AsRef<[u8]>,
{
    type Output = bool;

    fn poll(self: Pin<&mut Self>, cx: &mut core::task::Context<'_>) -> Poll<Self::Output> {
        match self.state {
            TransmissionTaskState::Preparing => {
                let this = self.get_mut();
                unsafe {
                    ready!(this
                        .radio
                        .prepare_transmit(cx, &this.config, this.data.as_ref()))
                };
                this.state = TransmissionTaskState::Transmitting;

                Poll::Pending
            }
            TransmissionTaskState::Transmitting => {
                let this = self.get_mut();
                let result = ready!(this.radio.transmit(cx));

                Poll::Ready(result)
            }
        }
    }
}

impl<T, R: Radio> Drop for TransmitTask<'_, T, R> {
    fn drop(&mut self) {
        self.radio.cancel_current_opperation()
    }
}

enum ReceiveTaskState {
    Preparing,
    Receiving,
}

/// Future around receiving through a radio. Use the `receive` function when you want to
/// use this future.
pub struct ReceiveTask<'task, R: Radio> {
    data: &'task mut [u8; 128],
    radio: &'task mut R,
    state: ReceiveTaskState,
    config: RxConfig,
}

/// Convenience Future around receiving through the radio. This future first prepares the radio,
/// then receives before succeeding. This future, upon canceling, stops the radio from receiving
/// and puts the radio in an IDLE state.
pub fn receive<'task, R: Radio>(
    radio: &'task mut R,
    data: &'task mut [u8; 128],
    config: RxConfig,
) -> ReceiveTask<'task, R> {
    ReceiveTask {
        radio,
        data,
        state: ReceiveTaskState::Preparing,
        config,
    }
}

impl<'task, R> Future for ReceiveTask<'task, R>
where
    R: Radio,
{
    type Output = bool;

    fn poll(self: Pin<&mut Self>, cx: &mut core::task::Context<'_>) -> Poll<Self::Output> {
        match self.state {
            ReceiveTaskState::Preparing => {
                let this = self.get_mut();
                unsafe { ready!(this.radio.prepare_receive(cx, &this.config, this.data)) };
                this.state = ReceiveTaskState::Receiving;

                Poll::Pending
            }
            ReceiveTaskState::Receiving => {
                let this = self.get_mut();
                let result = ready!(this.radio.receive(cx));

                Poll::Ready(result)
            }
        }
    }
}

impl<R> Drop for ReceiveTask<'_, R>
where
    R: Radio,
{
    fn drop(&mut self) {
        self.radio.cancel_current_opperation()
    }
}
