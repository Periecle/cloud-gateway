use bytes::Bytes;
use hyper::body::{Body, Frame};
use std::pin::Pin;
use std::task::{Context, Poll};

/// SingleChunkBody holds one optional chunk of bytes.
#[derive(Debug)]
pub struct SingleChunkBody {
    data: Option<Bytes>,
}

impl SingleChunkBody {
    pub fn new(data: Bytes) -> Self {
        Self { data: Some(data) }
    }
}

impl Body for SingleChunkBody {
    type Data = Bytes;
    /// We unify all body errors as `hyper::Error`, so the gateway code
    /// can handle or propagate them. Internally, we never actually produce
    /// an error except by artificially creating a `hyper::Error`.
    type Error = hyper::Error;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        // If we still have data, produce one chunk and then end the stream.
        if let Some(bytes) = self.data.take() {
            let frame = Frame::data(bytes);
            Poll::Ready(Some(Ok(frame)))
        } else {
            // No more frames.
            Poll::Ready(None)
        }
    }

    fn is_end_stream(&self) -> bool {
        // If `data` is None, we've emitted our chunk and are done.
        self.data.is_none()
    }
}
