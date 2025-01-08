use bytes::Bytes;
pub use pinned_body::BoxBody;
pub use single_chunk_body::SingleChunkBody;

pub mod pinned_body;
pub mod single_chunk_body;

/// Produces a pinned single-chunk body from a string or byte slice.
pub fn single_chunk_response_body(data: impl Into<Bytes>) -> BoxBody {
    Box::pin(SingleChunkBody::new(data.into()))
}
