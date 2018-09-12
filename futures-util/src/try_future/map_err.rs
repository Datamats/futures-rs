use core::marker::Unpin;
use core::pin::PinMut;
use futures_core::future::{Future, FusedFuture, TryFuture};
use futures_core::task::{self, Poll};
use pin_utils::{unsafe_pinned, unsafe_unpinned};

/// Future for the [`map_err`](super::TryFutureExt::map_err) combinator.
#[derive(Debug)]
#[must_use = "futures do nothing unless polled"]
pub struct MapErr<Fut, F> {
    future: Fut,
    f: Option<F>,
}

impl<Fut, F> MapErr<Fut, F> {
    unsafe_pinned!(future: Fut);
    unsafe_unpinned!(f: Option<F>);

    /// Creates a new MapErr.
    pub(super) fn new(future: Fut, f: F) -> MapErr<Fut, F> {
        MapErr { future, f: Some(f) }
    }
}

impl<Fut: Unpin, F> Unpin for MapErr<Fut, F> {}

impl<Fut, F> FusedFuture for MapErr<Fut, F> {
    fn can_poll(&self) -> bool { self.f.is_some() }
}

impl<Fut, F, E> Future for MapErr<Fut, F>
    where Fut: TryFuture,
          F: FnOnce(Fut::Error) -> E,
{
    type Output = Result<Fut::Ok, E>;

    fn poll(
        mut self: PinMut<Self>,
        cx: &mut task::Context,
    ) -> Poll<Self::Output> {
        match self.future().try_poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(result) => {
                let f = self.f().take()
                    .expect("MapErr must not be polled after it returned `Poll::Ready`");
                Poll::Ready(result.map_err(f))
            }
        }
    }
}
