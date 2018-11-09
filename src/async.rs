use std::mem;

#[cfg(feature = "async")]
use futures::{self, future, Async, Future, Poll, Stream};

#[cfg(feature = "async")]
use reqwest::async::Decoder;

/// Re-export for use by async response types
pub use reqwest::async::Chunk;

pub struct Response {
    data: Vec<u8>,
    status: u32,
}

pub struct AsyncResponse {
    response: ::reqwest::async::Response,
}

pub enum S3ResponseFuture {
    // This is boxed for now because of https://github.com/seanmonstar/reqwest/issues/205
    Pending(Box<Future<Item = ::reqwest::async::Response, Error = ::reqwest::Error> + Send>),
    ParseError(Vec<u8>, ::reqwest::async::Decoder, u32),
    Done,
    None,
}

impl Future for S3ResponseFuture {
    type Item = S3Response;
    type Error = S3Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            let state = mem::replace(self, S3ResponseFuture::None);
            match state {
                S3ResponseFuture::Pending(mut pending) => {
                    match pending.poll() {
                        Ok(Async::Ready(response)) => {
                            let resp_code = response.status().as_u16() as u32;
                            if resp_code < 300 {
                                *self = S3ResponseFuture::Done;
                                return Ok(Async::Ready(S3Response { response }));
                            } else {
                                *self = S3ResponseFuture::ParseError(Vec::new(), response.into_body(), resp_code);
                            }
                        }
                        Ok(Async::NotReady) => {
                            *self = S3ResponseFuture::Pending(pending);
                            return Ok(Async::NotReady);
                        }
                        Err(e) => return Err(e.into()),
                    }
                }
                S3ResponseFuture::ParseError(mut buffer, mut decoder, resp_code) => {
                    match decoder.poll() {
                        Ok(Async::Ready(Some(chunk))) => {
                            buffer.extend(chunk);
                            *self = S3ResponseFuture::ParseError(buffer, decoder, resp_code);
                        }
                        Ok(Async::Ready(None)) => {
                            let deserialized: AwsError = serde_xml::deserialize(buffer.as_slice())?;
                            let err = ErrorKind::AwsError {
                                info: deserialized,
                                status: resp_code,
                                body: String::from_utf8_lossy(buffer.as_slice()).into_owned(),
                            };
                            *self = S3ResponseFuture::Done;
                            return Err(err.into());
                        }
                        Ok(Async::NotReady) => {
                            *self = S3ResponseFuture::ParseError(buffer, decoder, resp_code);
                            return Ok(Async::NotReady);
                        }
                        Err(e) => return Err(e.into()),
                    }
                }
                S3ResponseFuture::Done => panic!("S3ResponseFuture used after Ready"),
                S3ResponseFuture::None => panic!("S3ResponseFuture is None"),
            }
        }
    }
}


    pub fn get_async(&self, path: &str) -> S3Result<S3ResponseFuture> {
        let command = Command::Get;
        let request = Request::new(self, path, command);
        request.execute_async()
    }

    #[cfg(feature = "async")]
    pub fn execute_async(&self) -> S3Result<S3ResponseFuture> {
        let client = if cfg!(feature = "no-verify-ssl") {
            reqwest::async::Client::builder()
                .danger_accept_invalid_certs(true)
                .danger_accept_invalid_hostnames(true)
                .build()?
        } else {
            reqwest::async::Client::new()
        };

        // Build headers
        let headers = self.headers()?;

        // Get owned content to pass to reqwest
        let content = if let Command::Put { content, .. } = self.command {
            Vec::from(content)
        } else {
            Vec::new()
        };

        // Build and send HTTP request
        let request = client
            .request(self.command.http_verb(), self.url())
            .headers(headers)
            .body(content);

        Ok(S3ResponseFuture::Pending(Box::new(request.send())))
    }
