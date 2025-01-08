use std::pin::Pin;

use bytes::Bytes;
use hyper::body::Body;
use pin_project_lite::pin_project;

/// A pinned type alias for a Body whose data is `Bytes` and error is `hyper::Error`.
pub type BoxBody = Pin<Box<dyn Body<Data = Bytes, Error = hyper::Error> + Send>>;

/// A pinned adapter that wraps any `B: Body<Data=Bytes, Error=hyper::Error>`
/// so that it can be stored in a `Pin<Box<...>>`.
pub fn box_pinned_body<B>(body: B) -> BoxBody
where
    B: Body<Data = Bytes, Error = hyper::Error> + Send + 'static,
{
    Box::pin(PinnedBody::new(body))
}

pin_project! {
    #[derive(Debug)]
    pub struct PinnedBody<B> {
        #[pin]
        inner: B,
    }
}

impl<B> PinnedBody<B> {
    pub fn new(inner: B) -> Self {
        Self { inner }
    }
}

impl<B> Body for PinnedBody<B>
where
    B: Body<Data = Bytes, Error = hyper::Error>,
{
    type Data = Bytes;
    type Error = hyper::Error;

    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Result<hyper::body::Frame<Self::Data>, Self::Error>>> {
        let this = self.project();
        this.inner.poll_frame(cx)
    }
}
