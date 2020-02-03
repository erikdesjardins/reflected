use std::convert::Infallible;
use std::io::Cursor;
use std::sync::Arc;
use std::task::Context;

use http_body::SizeHint;
use hyper::body::{Buf, HttpBody};
use hyper::header::HeaderValue;
use hyper::HeaderMap;
use tokio::macros::support::{Pin, Poll};

pub struct ArcBody {
    cursor: Option<Cursor<ArcInnerAsRef<[u8]>>>,
}

impl ArcBody {
    pub fn new(bytes: impl AsRef<[u8]> + Sync + Send + 'static) -> Self {
        Self {
            cursor: Some(Cursor::new(ArcInnerAsRef(Arc::new(bytes)))),
        }
    }

    pub fn from_arc(arc: Arc<dyn AsRef<[u8]> + Sync + Send>) -> Self {
        Self {
            cursor: Some(Cursor::new(ArcInnerAsRef(arc))),
        }
    }

    pub fn empty() -> Self {
        Self { cursor: None }
    }
}

pub struct ArcInnerAsRef<T: ?Sized>(Arc<dyn AsRef<T> + Sync + Send>);

impl<T: ?Sized> AsRef<T> for ArcInnerAsRef<T> {
    fn as_ref(&self) -> &T {
        AsRef::as_ref(&*self.0)
    }
}

impl HttpBody for ArcBody {
    type Data = Cursor<ArcInnerAsRef<[u8]>>;
    type Error = Infallible;

    fn poll_data(
        mut self: Pin<&mut Self>,
        _: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        Poll::Ready(self.cursor.take().map(Ok))
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
    ) -> Poll<Result<Option<HeaderMap<HeaderValue>>, Self::Error>> {
        Poll::Ready(Ok(None))
    }

    fn is_end_stream(&self) -> bool {
        self.cursor.is_none()
    }

    fn size_hint(&self) -> SizeHint {
        let len = match &self.cursor {
            Some(cursor) => cursor.remaining(),
            None => 0,
        };
        SizeHint::with_exact(len as u64)
    }
}
