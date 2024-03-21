pub mod futures;

use core::future::Future;

use super::config::{RxConfig, TxConfig};

pub trait Radio {
    type RadioFrame<T>: RadioFrame<T>
    where
        T: AsRef<[u8]>;
    type RxToken<'a>: RxToken;
    type TxToken<'b>: TxToken;

    /// Request the radio to idle to a low-power sleep mode.
    fn disable(&mut self) -> impl Future<Output = ()>;
    /// Request the radio to wake from sleep.
    fn enable(&mut self) -> impl Future<Output = ()>;

    /// Request the radio to go in receive mode and try to receive a frame into
    /// the supplied buffer.
    ///
    /// # Safety
    /// The supplied buffer must remain writable until either
    /// successful reception, or the radio state changed.
    unsafe fn prepare_receive(
        &mut self,
        cfg: &RxConfig,
        bytes: &mut [u8; 128],
    ) -> impl Future<Output = ()>;

    /// Request the radio to go in receive mode and try to receive a frame.
    fn receive(&mut self) -> impl Future<Output = bool>;

    /// Request the radio to go in transmit mode and try to send a frame.
    /// The mutability of the bytes argument is not really to modify the buffer,
    /// but rather to signify to hand over exclusive ownership. In addition this
    /// also helps with the easy_dma on the nRF family of chips as the buffer
    /// may not be in flash.
    ///
    /// # Safety
    /// The supplied buffer must remain valid until either
    /// successful reception, or the radio state changed.
    unsafe fn prepare_transmit(
        &mut self,
        cfg: &TxConfig,
        bytes: &mut [u8],
    ) -> impl Future<Output = ()>;

    /// When working with futures, it is not always guaranteed that a future
    /// will complete. This method must be seen as a notification to the radio
    /// where it can prepare for this cancelation. This method may not use any
    /// async behavior, as dropping in Rust (when a future is cancelled) can
    /// not be async.
    fn cancel_current_opperation(&mut self);

    /// Request the radio to transmit the queued frame.
    ///
    /// Returns whether a transmission was successful.
    fn transmit(&mut self) -> impl Future<Output = bool>;

    /// Returns the IEEE802.15.4 8-octet MAC address of the radio device.
    fn ieee802154_address(&self) -> [u8; 8];
}

pub trait RadioFrame<T: AsRef<[u8]>>: Sized {
    #[cfg(not(feature = "defmt"))]
    type Error: Sized + core::fmt::Debug;
    #[cfg(feature = "defmt")]
    type Error: Sized + defmt::Format + core::fmt::Debug;

    fn new_unchecked(buffer: T) -> Self;
    fn new_checked(buffer: T) -> Result<Self, Self::Error>;
    fn data(&self) -> &[u8];
}

pub trait RadioFrameMut<T: AsRef<[u8]> + AsMut<[u8]>>: RadioFrame<T> {
    fn data_mut(&mut self) -> &mut [u8];
}

pub trait RxToken {
    fn consume<F, R>(self, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R;
}

pub trait TxToken {
    fn consume<F, R>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R;
}

#[cfg(test)]
pub mod tests {
    use std::{
        cell::RefCell,
        collections::VecDeque,
        future::poll_fn,
        ptr::NonNull,
        rc::Rc,
        task::{Poll, Waker},
    };

    use embedded_hal_async::delay::DelayNs;

    use crate::sync::{select, tests::StdDelay};

    use super::{Radio, RadioFrame, RadioFrameMut, RxToken, TxToken};

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum TestRadioEvent {
        PrepareReceive,
        Receive,
        PrepareTransmit,
        CancelCurrentOperation,
        Transmit,
        Disable,
        Enable,
    }

    pub struct TestRadioInner {
        pub ieee802154_address: [u8; 8],
        pub should_receive: Option<[u8; 128]>,
        pub receive_buffer: Option<NonNull<[u8]>>,
        pub events: Vec<TestRadioEvent>,
        pub cca_fail: bool,
        pub assert_nxt: VecDeque<TestRadioEvent>,
        pub total_event_count: usize,
        pub last_transmitted: Option<[u8; 128]>,
        pub has_requested_cca: bool,
        assert_waker: Option<Waker>,
    }

    #[derive(Clone)]
    pub struct TestRadio {
        inner: Rc<RefCell<TestRadioInner>>,
    }

    impl TestRadio {
        pub fn new(ieee802154_address: [u8; 8]) -> Self {
            Self {
                inner: Rc::new(RefCell::new(TestRadioInner {
                    ieee802154_address,
                    should_receive: None,
                    events: vec![],
                    cca_fail: false,
                    assert_nxt: VecDeque::new(),
                    receive_buffer: None,
                    total_event_count: 0,
                    last_transmitted: None,
                    assert_waker: None,
                    has_requested_cca: false,
                })),
            }
        }

        pub fn inner<F, R>(&self, f: F) -> R
        where
            F: FnOnce(&mut TestRadioInner) -> R,
        {
            let mut inner = self.inner.borrow_mut();
            f(&mut inner)
        }

        pub fn new_event(&mut self, evnt: TestRadioEvent) {
            let mut inner = self.inner.borrow_mut();

            // Filter duplicates
            if Some(&evnt) == inner.events.last() {
                return;
            }

            println!(
                "New event arrived [{}]: {:?}",
                inner.total_event_count, evnt
            );

            inner.total_event_count += 1;
            if let Some(waker) = inner.assert_waker.take() {
                waker.wake();
            }
            if let Some(assert_nxt) = inner.assert_nxt.pop_front() {
                assert_eq!(
                    assert_nxt, evnt,
                    "Check if the next event is the expected event in the radio [{}](got {:?}, expected {:?})",
                    inner.total_event_count, evnt, assert_nxt
                );
            }
            inner.events.push(evnt);
        }

        /// Async wait for all radio events to have happened.
        /// This function is ment to be only used in tests an as such will panic
        /// if not all events have happened within 5s of starting
        pub async fn wait_until_asserts_are_consumed(&self) {
            let wait_for_events = poll_fn(|cx| {
                let mut inner = self.inner.borrow_mut();
                if inner.assert_nxt.is_empty() {
                    Poll::Ready(())
                } else {
                    match &mut inner.assert_waker {
                        Some(waker) if waker.will_wake(cx.waker()) => waker.clone_from(cx.waker()),
                        Some(waker) => {
                            waker.wake_by_ref();
                            waker.clone_from(cx.waker());
                        }
                        waker @ None => {
                            *waker = Some(cx.waker().clone());
                        }
                    };

                    Poll::Pending
                }
            });

            match select::select(wait_for_events, StdDelay::default().delay_ms(5000)).await {
                crate::sync::Either::First(_) => {}
                crate::sync::Either::Second(_) => {
                    panic!("Waiting timedout for events -> there is a bug in the code")
                }
            }
        }
    }

    impl Default for TestRadio {
        fn default() -> Self {
            Self::new([0xca; 8])
        }
    }

    impl Radio for TestRadio {
        type RadioFrame<T> = TestRadioFrame<T> where T: AsRef<[u8]>;
        type RxToken<'a> = TestRxToken<'a>;
        type TxToken<'b> = TestTxToken<'b>;

        async fn disable(&mut self) {
            self.new_event(TestRadioEvent::Disable);
        }

        async fn enable(&mut self) {
            self.new_event(TestRadioEvent::Enable);
        }

        async unsafe fn prepare_receive(
            &mut self,
            _cfg: &crate::phy::config::RxConfig,
            bytes: &mut [u8; 128],
        ) {
            self.new_event(TestRadioEvent::PrepareReceive);
            // Safety: Rust references are always valid and never dangling
            // Reference is also owned by the caller which will stay alive for the entire
            // duration this part of the api is used.
            self.inner.borrow_mut().receive_buffer = Some(unsafe { NonNull::new_unchecked(bytes) });
        }

        /// # Safety:
        /// This API should only be used during tests where the caller of the
        /// radio API is the MAC protocol under test. Otherwise there are
        /// invalid pointer dereferences, making the tests UB.
        async fn receive(&mut self) -> bool {
            poll_fn(|cx| {
                cx.waker().wake_by_ref(); // Always wake immediatly again
                self.new_event(TestRadioEvent::Receive);

                let mut inner = self.inner.borrow_mut();

                if let Some(mut receive_buffer) = inner.receive_buffer {
                    if let Some(should_receive) = inner.should_receive {
                        // Safety: The user of this API should also be the one that owns the
                        // receive_buffer
                        unsafe { receive_buffer.as_mut().copy_from_slice(&should_receive) }

                        // Reset pointers
                        inner.receive_buffer = None;
                        inner.should_receive = None;

                        Poll::Ready(true)
                    } else {
                        Poll::Pending
                    }
                } else {
                    Poll::Ready(false)
                }
            })
            .await
        }

        async unsafe fn prepare_transmit(
            &mut self,
            cfg: &crate::phy::config::TxConfig,
            bytes: &mut [u8],
        ) {
            self.new_event(TestRadioEvent::PrepareTransmit);
            let mut buffer = [0u8; 128];
            buffer.clone_from_slice(&bytes[..128]);
            let mut inner = self.inner.borrow_mut();
            inner.last_transmitted = Some(buffer);
            inner.has_requested_cca = cfg.cca;
        }

        fn cancel_current_opperation(&mut self) {
            self.new_event(TestRadioEvent::CancelCurrentOperation);
        }

        async fn transmit(&mut self) -> bool {
            self.new_event(TestRadioEvent::Transmit);
            let inner = self.inner.borrow();
            !(inner.has_requested_cca && inner.cca_fail)
        }

        fn ieee802154_address(&self) -> [u8; 8] {
            self.inner.borrow().ieee802154_address
        }
    }

    #[derive(Debug, Clone)]
    pub struct TestRadioFrame<T: AsRef<[u8]>> {
        buffer: T,
    }
    impl<T: AsRef<[u8]>> RadioFrame<T> for TestRadioFrame<T> {
        type Error = ();

        fn new_unchecked(buffer: T) -> Self {
            Self { buffer }
        }

        fn new_checked(buffer: T) -> Result<Self, Self::Error> {
            Ok(Self { buffer })
        }

        fn data(&self) -> &[u8] {
            &self.buffer.as_ref()[..127]
        }
    }
    impl<T: AsRef<[u8]> + AsMut<[u8]>> RadioFrameMut<T> for TestRadioFrame<T> {
        fn data_mut(&mut self) -> &mut [u8] {
            &mut self.buffer.as_mut()[..127]
        }
    }

    pub struct TestRxToken<'a> {
        buffer: &'a mut [u8],
    }
    impl<'a> RxToken for TestRxToken<'a> {
        fn consume<F, R>(self, f: F) -> R
        where
            F: FnOnce(&mut [u8]) -> R,
        {
            f(&mut self.buffer[..127])
        }
    }
    impl<'a> From<&'a mut [u8]> for TestRxToken<'a> {
        fn from(value: &'a mut [u8]) -> Self {
            Self { buffer: value }
        }
    }
    pub struct TestTxToken<'a> {
        buffer: &'a mut [u8],
    }
    impl<'a> TxToken for TestTxToken<'a> {
        fn consume<F, R>(self, len: usize, f: F) -> R
        where
            F: FnOnce(&mut [u8]) -> R,
        {
            f(&mut self.buffer[..len])
        }
    }
    impl<'a> From<&'a mut [u8]> for TestTxToken<'a> {
        fn from(value: &'a mut [u8]) -> Self {
            Self { buffer: value }
        }
    }
}
