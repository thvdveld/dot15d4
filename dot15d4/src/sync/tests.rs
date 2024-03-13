use core::future::Future;
use core::task::Poll;
use core::task::Waker;
use std::sync::mpsc::RecvTimeoutError;

use embedded_hal_async::delay::DelayNs;

use super::yield_now;

/// Implementation of a timer that can be used in tests. Delays delay for 10
/// iterations
#[derive(Default, Clone)]
pub struct Delay {}

impl DelayNs for Delay {
    async fn delay_ns(&mut self, _ns: u32) {
        for _ in 0..10 {
            yield_now::yield_now().await
        }
    }
}

#[cfg(feature = "std")]
#[derive(Default, Clone)]
pub struct StdDelay {}

#[cfg(feature = "std")]
impl DelayNs for StdDelay {
    async fn delay_ns(&mut self, ns: u32) {
        StdDelayFuture::new(std::time::Duration::from_nanos(ns as u64)).await
    }
}

#[cfg(feature = "std")]
pub enum StdDelayFuture {
    Init {
        wake_at: std::time::Instant,
    },
    Waiting {
        waker: std::sync::Arc<std::sync::Mutex<Waker>>,
        wake_at: std::time::Instant,
        shutdown_signal: std::sync::mpsc::Sender<()>,
    },
    Finished,
}

impl StdDelayFuture {
    pub fn new(duration: std::time::Duration) -> Self {
        let now = std::time::Instant::now();
        let wake_at = now + duration;

        Self::Init { wake_at }
    }
}

#[cfg(feature = "std")]
impl Future for StdDelayFuture {
    type Output = ();

    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        let this = self.get_mut();
        loop {
            match this {
                StdDelayFuture::Init { wake_at } => {
                    let (tx, rx) = std::sync::mpsc::channel();
                    let waker = std::sync::Arc::new(std::sync::Mutex::new(cx.waker().clone()));
                    std::thread::spawn({
                        let wake_at = *wake_at;
                        let waker = waker.clone();
                        move || {
                            let now = std::time::Instant::now();
                            if now < wake_at {
                                // We use the channel here as a timer, has the advantage that we do
                                // not have to manage the sleeping logic ourselves
                                match rx.recv_timeout(wake_at - now) {
                                    // We got a kill signal
                                    Ok(_) => return,
                                    // Continue to wake
                                    Err(RecvTimeoutError::Timeout) => (),
                                    // Channel is dead, stop thread
                                    Err(RecvTimeoutError::Disconnected) => return,
                                }
                            }

                            // Only wake if the future is still alive/has not yet sent a kill signal
                            if let Ok(waker) = waker.lock() {
                                waker.wake_by_ref()
                            }
                        }
                    });

                    *this = Self::Waiting {
                        waker,
                        wake_at: *wake_at,
                        shutdown_signal: tx,
                    };

                    return Poll::Pending;
                }
                StdDelayFuture::Waiting {
                    waker,
                    wake_at,
                    shutdown_signal,
                } => {
                    let now = std::time::Instant::now();

                    if *wake_at < now {
                        // Make the thread terminate without calling the waker
                        // We do not care if the thread has already terminated, just notfying it
                        let _ = shutdown_signal.send(());

                        *this = Self::Finished;
                        continue; // Delegate to other state
                    }

                    // Not yet time -> replace waker
                    if let Ok(mut waker) = waker.lock() {
                        if waker.will_wake(cx.waker()) {
                            waker.wake_by_ref();
                            waker.clone_from(cx.waker());
                        } else {
                            waker.clone_from(cx.waker());
                        }
                    };

                    return Poll::Pending;
                }
                StdDelayFuture::Finished => return Poll::Ready(()),
            }
        }
    }
}

#[cfg(feature = "std")]
impl Drop for StdDelayFuture {
    fn drop(&mut self) {
        if let Self::Waiting {
            shutdown_signal, ..
        } = self
        {
            // We do not care if the thread has already terminated
            let _ = shutdown_signal.send(());
        }
    }
}

#[cfg(test)]
mod inner_tests {
    use std::time::{Duration, Instant};

    use embedded_hal_async::delay::DelayNs;

    use super::StdDelay;

    #[pollster::test]
    pub async fn test_std_delay_future() {
        let mut delay = StdDelay::default();
        let start = Instant::now();
        delay.delay_ms(100).await;
        let end = Instant::now();
        assert!(
            end - start > Duration::from_millis(100),
            "Difference in time should be greater than 100ms: {start:?} -> {end:?}"
        )
    }
}
