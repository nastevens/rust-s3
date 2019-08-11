use std::mem;
use std::thread;

use futures::{self, future, Async, Future, Poll, Stream};
use reqwest::StatusCode;
use reqwest::async::Decoder;
use serde_xml;

use error::{ErrorKind, S3Error, S3Result};
use serde_types::AwsError;

/// Re-export for use by async response types
pub use reqwest::async::Chunk;

pub struct Response {
    response: ::reqwest::async::Response,
}

impl Response {
    pub fn stream(self) -> Decoder {
        self.response.into_body()
    }

    pub fn block_on_result(self) -> S3Result<(Vec<u8>, u32)> {
        let status = self.response.status().as_u16() as u32;
        // let body = self.response.into_body().concat2().wait()?;
        // Ok((body.to_vec(), status))
        Ok((Vec::new(), status))
    }

    pub fn status(&self) -> u32 {
        self.response.status().as_u16() as u32
    }
}

pub enum ResponseFuture {
    // This is boxed because of https://github.com/seanmonstar/reqwest/issues/205
    Pending(Box<Future<Item = ::reqwest::async::Response, Error = ::reqwest::Error> + Send>),
    ParseError(Vec<u8>, ::reqwest::async::Decoder, u32),
    Done,
    None,
}

impl Future for ResponseFuture {
    type Item = Response;
    type Error = S3Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            let state = mem::replace(self, ResponseFuture::None);
            match state {
                ResponseFuture::Pending(mut pending) => {
                    match pending.poll() {
                        Ok(Async::Ready(response)) => {
                            let resp_code = response.status().as_u16() as u32;
                            if resp_code < 300 {
                                *self = ResponseFuture::Done;
                                return Ok(Async::Ready(Response { response }));
                            } else {
                                *self = ResponseFuture::ParseError(Vec::new(), response.into_body(), resp_code);
                            }
                        }
                        Ok(Async::NotReady) => {
                            *self = ResponseFuture::Pending(pending);
                            return Ok(Async::NotReady);
                        }
                        Err(e) => return Err(e.into()),
                    }
                }
                ResponseFuture::ParseError(mut buffer, mut decoder, resp_code) => {
                    match decoder.poll() {
                        Ok(Async::Ready(Some(chunk))) => {
                            buffer.extend(chunk);
                            *self = ResponseFuture::ParseError(buffer, decoder, resp_code);
                        }
                        Ok(Async::Ready(None)) => {
                            let deserialized: AwsError = serde_xml::deserialize(buffer.as_slice())?;
                            let err = ErrorKind::AwsError {
                                info: deserialized,
                                status: resp_code,
                                body: String::from_utf8_lossy(buffer.as_slice()).into_owned(),
                            };
                            *self = ResponseFuture::Done;
                            return Err(err.into());
                        }
                        Ok(Async::NotReady) => {
                            *self = ResponseFuture::ParseError(buffer, decoder, resp_code);
                            return Ok(Async::NotReady);
                        }
                        Err(e) => return Err(e.into()),
                    }
                }
                ResponseFuture::Done => panic!("ResponseFuture used after Ready"),
                ResponseFuture::None => panic!("ResponseFuture is None"),
            }
        }
    }
}


