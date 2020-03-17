use std::convert::Infallible;
use std::io::Cursor;
use std::ops::Deref;
use std::sync::Arc;
use std::task::Context;
use std::u32;

use http_body::SizeHint;
use hyper::body::HttpBody;
use hyper::header::HeaderValue;
use hyper::HeaderMap;
use tokio::macros::support::{Pin, Poll};

use crate::as_ref::{ForwardAsRef, ReindexAsRef};

type ArcAsRefBytes = Arc<dyn AsRef<[u8]> + Sync + Send>;

pub struct ArcBody {
    data: Option<ArcAsRefBytes>,
    pos: usize,
}

impl ArcBody {
    pub fn new(bytes: impl AsRef<[u8]> + Sync + Send + 'static) -> Self {
        Self::from_arc(Arc::new(bytes))
    }

    pub fn from_arc(arc: ArcAsRefBytes) -> Self {
        Self {
            data: Some(arc),
            pos: 0,
        }
    }

    pub fn empty() -> Self {
        Self { data: None, pos: 0 }
    }

    fn remaining_size(&self) -> usize {
        match &self.data {
            Some(data) => Arc::deref(data).as_ref().len().saturating_sub(self.pos),
            None => 0,
        }
    }
}

impl HttpBody for ArcBody {
    type Data = Cursor<ForwardAsRef<ArcAsRefBytes, [u8]>>;
    type Error = Infallible;

    fn poll_data(
        mut self: Pin<&mut Self>,
        _: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        let Self { data, pos } = &mut *self;

        // tokio or std::sys::windows will panic if we try to send a slice bigger than this
        let chunk_size = u32::MAX as usize;

        let chunk = match data {
            Some(data) if Arc::deref(data).as_ref().len().saturating_sub(*pos) > chunk_size => {
                let chunk = Arc::new(ReindexAsRef::new(
                    ForwardAsRef::new(Arc::clone(data)),
                    *pos..*pos + chunk_size,
                ));
                *pos += chunk_size;
                chunk
            }
            data @ Some(_) => data.take().unwrap(),
            None => return Poll::Ready(None),
        };

        Poll::Ready(Some(Ok(Cursor::new(ForwardAsRef::new(chunk)))))
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
    ) -> Poll<Result<Option<HeaderMap<HeaderValue>>, Self::Error>> {
        Poll::Ready(Ok(None))
    }

    fn is_end_stream(&self) -> bool {
        self.remaining_size() == 0
    }

    fn size_hint(&self) -> SizeHint {
        SizeHint::with_exact(self.remaining_size() as u64)
    }
}
