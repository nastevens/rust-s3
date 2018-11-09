//! Simple access to Amazon Web Service's (AWS) Simple Storage Service (S3)
extern crate chrono;
#[macro_use]
extern crate error_chain;
extern crate hex;
extern crate hmac;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_xml_rs as serde_xml;
extern crate sha2;
extern crate url;
extern crate ini;
extern crate dirs;


mod bucket;
mod command;
mod credentials;
mod deserializer;
mod error;
mod region;
mod request;
mod serde_types;
mod signing;

pub use bucket::Bucket;
pub use credentials::Credentials;
pub use error::S3Error;
pub use region::Region;
pub use request::{Headers, Query};
pub use serde_types::ListBucketResult;

const LONG_DATE: &str = "%Y%m%dT%H%M%SZ";
const EMPTY_PAYLOAD_SHA: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
