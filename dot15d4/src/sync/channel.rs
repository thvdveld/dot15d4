//! Simple oneshot Channel implementation based on the one described in the book
//! of Mara Bos, but adapted to work as a signaling mechanism. Sending will
//! remain non-blocking and just overwrite the previous message.
use core::cell::UnsafeCell;
use core::future::poll_fn;
use core::mem::MaybeUninit;
use core::task::Poll;
use core::task::Waker;

struct ChannelState {
    is_ready: bool, // We always stay in the same task/thread -> no atomic needed here
    waker_recv: Option<Waker>,
    waker_send: Option<Waker>,
}

pub struct Channel<T> {
    message: UnsafeCell<MaybeUninit<T>>,
    state: UnsafeCell<ChannelState>,
}

impl<T> Channel<T> {
    pub fn new() -> Self {
        Self {
            message: UnsafeCell::new(MaybeUninit::uninit()),
            state: UnsafeCell::new(ChannelState {
                is_ready: false,
                waker_recv: None,
                waker_send: None,
            }),
        }
    }

    pub fn split(&mut self) -> (Sender<'_, T>, Receiver<'_, T>) {
        *self = Self::new(); // Drop previous channel to reset state. We have exclusive access here
        (Sender { channel: self }, Receiver { channel: self })
    }
}

impl<T> Default for Channel<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Sender<'a, T> {
    channel: &'a Channel<T>,
}

impl<T> Sender<'_, T> {
    /// Sends a message across the channel. Sending multiple messages before the
    /// Receiver can read them, results in overwriting the previous messages.
    /// Only the last one will be actually sent. This method returns whether or
    /// not the previous message was overwritten
    pub fn send(&self, message: T) -> bool {
        // If the channel is ready, make the message drop
        // Safety: The state is only accessed inside a function body and never across an
        // await point. No concurrent access here (same task)
        let state = unsafe { &mut *self.channel.state.get() };
        let did_replace = if state.is_ready {
            unsafe {
                // Drop previous message
                // Safety: This is ok, as we are the only ones with access to the
                // Sender and there is no concurrent access from the reader
                let maybe_uninit = &mut *self.channel.message.get();
                core::ptr::drop_in_place(maybe_uninit.as_mut_ptr());

                // Store the new message
                maybe_uninit.as_mut_ptr().write(message);
                // The channel is already set to be ready -> keep it this way
            }

            // Wake the Receiver task
            if let Some(waker) = state.waker_recv.take() {
                waker.wake()
            }

            // Signal that the channel has replaced something
            true
        } else {
            // The channel is not yet ready -> store the message and make it ready
            // Safety: We are the only one with access to the Sender and no concurrent
            // access with the Receiver possible
            unsafe {
                let maybe_uninit = &mut *self.channel.message.get();
                maybe_uninit.as_mut_ptr().write(message);
            }

            // Signal that the channel was empty before
            false
        };
        // Wake the Receiver task
        state.is_ready = true;
        if let Some(waker) = state.waker_recv.take() {
            waker.wake()
        }

        // Did we replace the inner message or not
        did_replace
    }

    /// Check if there is an item in the channel
    pub fn has_item(&self) -> bool {
        let state = unsafe { &mut *self.channel.state.get() };
        state.is_ready
    }

    /// Wait before sending
    pub async fn send_async(&self, message: T) {
        poll_fn(|cx| {
            if self.has_item() {
                // Replace waker if necessary
                // Safety: we are the only ones that have access to the state at this moment
                let state = unsafe { &mut *self.channel.state.get() };

                let new_waker = cx.waker();
                state.waker_send = match state.waker_send.take() {
                    Some(mut waker) => {
                        if new_waker.will_wake(&waker) {
                            waker.clone_from(new_waker);
                            Some(waker)
                        } else {
                            // We have a different waker now, wake the previous one before replacing
                            // it
                            waker.wake();
                            Some(new_waker.clone())
                        }
                    }
                    None => Some(new_waker.clone()),
                };

                Poll::Pending
            } else {
                Poll::Ready(())
            }
        })
        .await;

        self.send(message);
    }
}

pub struct Receiver<'a, T> {
    channel: &'a Channel<T>,
}

impl<T> Receiver<'_, T> {
    pub async fn receive(&self) -> T {
        poll_fn(|cx| {
            // Safety: We only access the state in the bounds of this call and never across
            // an await point
            let state = unsafe { &mut *self.channel.state.get() };

            if !state.is_ready {
                // Not yet ready, store/replace the context
                match &mut state.waker_recv {
                    Some(waker) => waker.clone_from(cx.waker()),
                    waker @ None => *waker = Some(cx.waker().clone()),
                }

                Poll::Pending
            } else {
                // Safety: We have a message, and exclusive access to the channel as there is no
                // concurrent access possible
                let message = unsafe {
                    let maybe_uninit = &mut *self.channel.message.get();
                    maybe_uninit.assume_init_read()
                };
                // Reset the state, such that we can send again
                state.is_ready = false;

                // Wake if possible the waker for sending
                if let Some(waker) = state.waker_send.take() {
                    waker.wake();
                }

                Poll::Ready(message)
            }
        })
        .await
    }

    /// Check if there is an item in the channel
    #[allow(dead_code)]
    pub fn has_item(&self) -> bool {
        let state = unsafe { &mut *self.channel.state.get() };
        state.is_ready
    }
}

#[cfg(test)]
mod tests {
    use pollster::FutureExt as _;

    use crate::sync::yield_now;
    use crate::sync::{join::join, yield_now::yield_now};

    use super::Channel;

    #[test]
    pub fn test_channel_no_concurrency() {
        async {
            let mut channel = Channel::new();
            let (send, recv) = channel.split();
            send.send(1);
            assert_eq!(recv.receive().await, 1);
        }
        .block_on();
    }

    #[test]
    pub fn test_channel_join_concurrency() {
        async {
            let mut channel = Channel::new();
            let (send, recv) = channel.split();

            join(
                async {
                    for i in 0..10 {
                        send.send(i);
                        yield_now().await;
                    }
                },
                async {
                    for i in 0..10 {
                        assert_eq!(recv.receive().await, i);
                    }
                },
            )
            .await;
        }
        .block_on();
    }

    #[test]
    /// Check with Miri whether or not drop is called correctly. If true, then
    /// all heap allocation should be deallocated correctly
    pub fn test_drop_by_leaking() {
        async {
            let mut channel = Channel::new();
            let (send, recv) = channel.split();
            send.send(Box::new(0));
            send.send(Box::new(1));
            send.send(Box::new(2));
            assert_eq!(*recv.receive().await, 2);
        }
        .block_on()
    }

    #[test]
    pub fn test_multiple_receivers() {
        async {
            let mut channel = Channel::new();
            let (send, recv) = channel.split();

            for _ in 0..10 {
                join(
                    join(
                        async {
                            assert_eq!(recv.receive().await, 0);
                        },
                        async {
                            assert_eq!(recv.receive().await, 1);
                        },
                    ),
                    async {
                        for _ in 0..10 {
                            yield_now::yield_now().await;
                        }
                        send.send(0);
                        yield_now::yield_now().await;
                        send.send(1);
                    },
                )
                .await;
            }
        }
        .block_on()
    }

    #[test]
    pub fn test_multiple_channels_at_once() {
        async {
            let mut channel1 = Channel::new();
            let (tx1, rx1) = channel1.split();
            let mut channel2 = Channel::new();
            let (tx2, rx2) = channel2.split();

            join(
                async {
                    tx1.send_async(0).await;
                    tx2.send_async(0).await;
                    tx1.send_async(1).await;
                    tx2.send_async(1).await;
                },
                async {
                    for _ in 0..10 {
                        yield_now::yield_now().await;
                    }

                    join(
                        async {
                            assert_eq!(rx1.receive().await, 0);
                            assert_eq!(rx2.receive().await, 0);
                        },
                        async {
                            assert_eq!(rx1.receive().await, 1);
                            assert_eq!(rx2.receive().await, 1);
                        },
                    )
                    .await;
                },
            )
            .await;
        }
        .block_on()
    }
}
