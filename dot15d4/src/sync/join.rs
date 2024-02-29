#![no_std]

use core::{
    future::Future,
    pin::pin,
    pin::Pin,
    task::{Context, Poll},
};

use super::Either;

/// Combine 2 futures and return if both are ready.
pub fn join<F1: Future, F2: Future>(
    f1: F1,
    f2: F2,
) -> impl Future<Output = (F1::Output, F2::Output)> {
    JoinFuture {
        f1: Either::First(f1),
        f2: Either::First(f2),
    }
}

pub struct JoinFuture<F1: Future, F2: Future> {
    f1: Either<F1, Option<F1::Output>>,
    f2: Either<F2, Option<F2::Output>>,
}

impl<F1, F2> Future for JoinFuture<F1, F2>
where
    F1: Future,
    F2: Future,
{
    type Output = (F1::Output, F2::Output);

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Safety: self is not used after this point, so it is not moved
        let this = unsafe { self.get_unchecked_mut() };

        // Make progress if the task is not yet finished
        // Safety: this is pinned, so f1 will not be moved either
        if let Some(res) = unsafe { poll_future(&mut this.f1, cx) } {
            this.f1 = Either::Second(Some(res));
        }
        if let Some(res) = unsafe { poll_future(&mut this.f2, cx) } {
            this.f2 = Either::Second(Some(res));
        }

        match (&mut this.f1, &mut this.f2) {
            (Either::Second(f1 @ Some(_)), Either::Second(f2 @ Some(_))) => {
                Poll::Ready((f1.take().unwrap(), f2.take().unwrap()))
            }
            _ => Poll::Pending,
        }
    }
}

/// # Safety
/// The future behind the f may not be moved while this function is being executed
unsafe fn poll_future<F: Future>(
    f: &mut Either<F, Option<F::Output>>,
    cx: &mut Context,
) -> Option<F::Output> {
    match f {
        Either::First(f) => match Pin::new_unchecked(f).poll(cx) {
            Poll::Ready(res) => Some(res),
            Poll::Pending => None,
        },
        Either::Second(rus) => None,
    }
}

#[cfg(test)]
mod tests {
    use core::future::poll_fn;
    use core::task::Poll;

    use pollster::FutureExt as _;

    use crate::sync::Either;

    use super::join;

    #[test]
    pub fn test_select_immediate_ready_first() {
        async {
            let f1 = poll_fn(|_| Poll::Ready(1));
            let f2 = poll_fn(|_| Poll::Ready(2));

            assert_eq!(join(f1, f2).await, (1, 2));
        }
        .block_on();
    }

    #[test]
    pub fn test_select_wait_until_one_finished() {
        async {
            let mut counter = 10;
            let f1 = poll_fn(move |cx| {
                if counter == 0 {
                    Poll::Ready(())
                } else {
                    counter -= 1;
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
            });
            let f2 = poll_fn(|_| Poll::Ready(()));

            assert_eq!(join(f1, f2).await, ((), ()));
        }
        .block_on();
    }

    #[test]
    pub fn test_select_wait_until_second_finished() {
        async {
            let mut counter = 10;
            let f1 = poll_fn(|_| Poll::Ready(()));
            let f2 = poll_fn(move |cx| {
                if counter == 0 {
                    Poll::Ready(())
                } else {
                    counter -= 1;
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
            });

            assert_eq!(join(f1, f2).await, ((), ()));
        }
        .block_on();
    }
}
