//! An `async` wrapper for `ureq::Request` that runs all blocking IO on the
//! `blocking` thread pool.

use bytes::Bytes;
use futures_lite::{future::block_on, AsyncRead, AsyncReadExt};
use std::io;

pub trait SendToPool: 'static + Send {}
impl<T> SendToPool for T where T: 'static + Send {}

/// Extension trait that gives [`ureq::Request`] async wrappers for all methods
/// that perform blocking IO.
#[async_trait::async_trait]
pub trait AsyncRequest {
    async fn call_async(self) -> Result<ureq::Response, ureq::Error>;

    async fn send_async(
        self,
        reader: impl AsyncRead + SendToPool + Unpin,
    ) -> Result<ureq::Response, ureq::Error>;

    #[cfg(feature = "json")]
    async fn send_json_async(
        self,
        data: impl serde::Serialize + SendToPool,
    ) -> Result<ureq::Response, ureq::Error>;

    async fn send_bytes_async(self, bytes: Bytes) -> Result<ureq::Response, ureq::Error>;

    async fn send_string_async(self, data: String) -> Result<ureq::Response, ureq::Error>;

    async fn send_form_async<S: AsRef<str> + SendToPool>(
        self,
        form: Vec<(S, S)>,
    ) -> Result<ureq::Response, ureq::Error>;
}

#[async_trait::async_trait]
impl AsyncRequest for ureq::Request {
    async fn call_async(self) -> Result<ureq::Response, ureq::Error> {
        blocking::unblock(move || self.call()).await
    }

    async fn send_async(
        self,
        reader: impl AsyncReadExt + SendToPool + Unpin,
    ) -> Result<ureq::Response, ureq::Error> {
        blocking::unblock(move || self.send(ReadProxy { reader })).await
    }

    #[cfg(feature = "json")]
    async fn send_json_async(
        self,
        data: impl serde::Serialize + SendToPool,
    ) -> Result<ureq::Response, ureq::Error> {
        blocking::unblock(move || self.send_json(data)).await
    }

    async fn send_bytes_async(self, bytes: Bytes) -> Result<ureq::Response, ureq::Error> {
        blocking::unblock(move || self.send_bytes(&bytes)).await
    }

    async fn send_string_async(self, data: String) -> Result<ureq::Response, ureq::Error> {
        blocking::unblock(move || self.send_string(&data)).await
    }

    async fn send_form_async<S: AsRef<str> + SendToPool>(
        self,
        form: Vec<(S, S)>,
    ) -> Result<ureq::Response, ureq::Error> {
        blocking::unblock(move || {
            let form_refs: Vec<_> = form.iter().map(|(k, v)| (k.as_ref(), v.as_ref())).collect();
            self.send_form(&form_refs)
        })
        .await
    }
}

struct ReadProxy<R> {
    reader: R,
}

impl<R: AsyncRead + Unpin> io::Read for ReadProxy<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        block_on(self.reader.read(buf))
    }
}
