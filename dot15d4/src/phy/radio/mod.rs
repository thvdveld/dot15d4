pub mod futures;

use core::task::{Context, Poll};

use super::config::{RxConfig, TxConfig};

pub trait Radio {
    type RadioFrame<T>: RadioFrame<T>
    where
        T: AsRef<[u8]>;
    type RxToken<'a>: RxToken;
    type TxToken<'b>: TxToken;

    /// Request the radio to idle to a low-power sleep mode.
    fn off(&mut self, ctx: &mut Context<'_>) -> Poll<()>;

    /// Request the radio to go in receive mode and try to receive a packet into the supplied
    /// buffer.
    ///
    /// # Safety
    /// The supplied buffer must remain writable until either
    /// successful reception, or the radio state changed.
    unsafe fn prepare_receive(
        &mut self,
        ctx: &mut Context<'_>,
        cfg: &RxConfig,
        bytes: &mut [u8; 128],
    ) -> Poll<()>;

    /// Request the radio to go in receive mode and try to receive a packet.
    fn receive(&mut self, ctx: &mut Context<'_>) -> Poll<bool>;

    /// Request the radio to go in transmit mode and try to send a packet.
    ///
    /// # Safety
    /// The supplied buffer must remain valid until either
    /// successful reception, or the radio state changed.
    unsafe fn prepare_transmit(
        &mut self,
        ctx: &mut Context<'_>,
        cfg: &TxConfig,
        bytes: &[u8],
    ) -> Poll<()>;

    /// When working with futures, it is not always guaranteed that a future
    /// will complete. This method must be seen as a notification to the radio
    /// where it can prepare for this cancelation. This method may not use any
    /// async behavior, as dropping in Rust (when a future is cancelled) can
    /// not be async.
    fn cancel_current_opperation(&mut self);

    /// Request the radio to transmit the queued packet.
    ///
    /// Returns whether a transmission was successful.
    fn transmit(&mut self, ctx: &mut Context<'_>) -> Poll<bool>;

    /// Returns the IEEE802.15.4 8-octet MAC address of the radio device.
    fn ieee802154_address(&self) -> [u8; 8];
}

pub trait RadioFrame<T: AsRef<[u8]>>: Sized {
    #[cfg(not(feature = "defmt"))]
    type Error: Sized;
    #[cfg(feature = "defmt")]
    type Error: Sized + defmt::Format;

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
    use std::{cell::RefCell, collections::VecDeque, ptr::NonNull, rc::Rc, task::Poll};

    use crate::phy::{
        config::{RxConfig, TxConfig},
        driver::PacketBuffer,
    };

    use super::{Radio, RadioFrame, RadioFrameMut, RxToken, TxToken};

    #[derive(Debug, Clone, PartialEq)]
    pub enum TestRadioEvent {
        Off,
        PrepareReceive,
        Receive,
        PrepareTransmit,
        CancelCurrentOperation,
        Transmit,
    }

    pub struct TestRadioInner {
        pub ieee802154_address: [u8; 8],
        pub should_receive: Option<[u8; 128]>,
        pub receive_buffer: Option<NonNull<[u8]>>,
        pub events: Vec<TestRadioEvent>,
        pub cca_fail: bool,
        pub assert_nxt: VecDeque<TestRadioEvent>,
        pub total_event_count: usize,
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

            inner.total_event_count += 1;
            if let Some(assert_nxt) = inner.assert_nxt.pop_front() {
                assert_eq!(
                    assert_nxt, evnt,
                    "Check if the next event is the expected event in the radio [{}]",
                    inner.total_event_count,
                );
            }
            inner.events.push(evnt);
        }
    }

    impl Default for TestRadio {
        fn default() -> Self {
            Self::new([0; 8])
        }
    }

    impl Radio for TestRadio {
        type RadioFrame<T> = TestRadioFrame<T> where T: AsRef<[u8]>;
        type RxToken<'a> = TestRxToken<'a>;
        type TxToken<'b> = TestTxToken<'b>;

        fn off(&mut self, ctx: &mut core::task::Context<'_>) -> core::task::Poll<()> {
            self.new_event(TestRadioEvent::Off);
            Poll::Ready(())
        }

        unsafe fn prepare_receive(
            &mut self,
            ctx: &mut core::task::Context<'_>,
            cfg: &crate::phy::config::RxConfig,
            bytes: &mut [u8; 128],
        ) -> core::task::Poll<()> {
            self.new_event(TestRadioEvent::PrepareReceive);
            // Safety: Rust references are always valid and never dangling
            // Reference is also owned by the caller which will stay alive for the entire duration this part of the api is used.
            self.inner.borrow_mut().receive_buffer = Some(unsafe { NonNull::new_unchecked(bytes) });
            Poll::Ready(())
        }

        /// # Safety:
        /// This API should only be used during tests where the caller of the radio API is the MAC protocol under test. Otherwise there are invalid pointer dereferences, making the tests UB.
        fn receive(&mut self, ctx: &mut core::task::Context<'_>) -> core::task::Poll<bool> {
            ctx.waker().wake_by_ref(); // Always wake immediatly again
            self.new_event(TestRadioEvent::Receive);

            let mut inner = self.inner.borrow_mut();

            if let Some(mut receive_buffer) = inner.receive_buffer {
                if let Some(should_receive) = inner.should_receive {
                    // Safety: The user of this API should also be the one that owns the receive_buffer
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
        }

        unsafe fn prepare_transmit(
            &mut self,
            ctx: &mut core::task::Context<'_>,
            cfg: &crate::phy::config::TxConfig,
            bytes: &[u8],
        ) -> core::task::Poll<()> {
            self.new_event(dbg!(TestRadioEvent::PrepareTransmit));
            Poll::Ready(())
        }

        fn cancel_current_opperation(&mut self) {
            self.new_event(TestRadioEvent::CancelCurrentOperation);
        }

        fn transmit(&mut self, ctx: &mut core::task::Context<'_>) -> core::task::Poll<bool> {
            self.new_event(dbg!(TestRadioEvent::Transmit));
            Poll::Ready(!self.inner.borrow().cca_fail)
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
            self.buffer.as_ref()
        }
    }
    impl<T: AsRef<[u8]> + AsMut<[u8]>> RadioFrameMut<T> for TestRadioFrame<T> {
        fn data_mut(&mut self) -> &mut [u8] {
            self.buffer.as_mut()
        }
    }

    pub struct TestRxToken<'a> {
        buffer: &'a mut PacketBuffer,
    }
    impl<'a> RxToken for TestRxToken<'a> {
        fn consume<F, R>(self, f: F) -> R
        where
            F: FnOnce(&mut [u8]) -> R,
        {
            f(&mut self.buffer.buffer[..])
        }
    }
    impl<'a> From<&'a mut PacketBuffer> for TestRxToken<'a> {
        fn from(mut value: &'a mut PacketBuffer) -> Self {
            Self { buffer: value }
        }
    }
    pub struct TestTxToken<'a> {
        buffer: &'a mut PacketBuffer,
    }
    impl<'a> TxToken for TestTxToken<'a> {
        fn consume<F, R>(self, len: usize, f: F) -> R
        where
            F: FnOnce(&mut [u8]) -> R,
        {
            f(&mut self.buffer.buffer[..len])
        }
    }
    impl<'a> From<&'a mut PacketBuffer> for TestTxToken<'a> {
        fn from(value: &'a mut PacketBuffer) -> Self {
            Self { buffer: value }
        }
    }
}
