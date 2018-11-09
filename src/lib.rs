//! Simple access to Amazon Web Service's (AWS) Simple Storage Service (S3)
//!
//! # Example
//!
//! ```no_run
//! extern crate s3;
//!
//! use std::str;
//!
//! use s3::{Bucket, Credentials};
//!
//! const BUCKET: &str = "example-bucket";
//! const REGION: &str = "us-east-1";
//! const MESSAGE: &str = "I want to go to S3";
//!
//! pub fn main() -> Result<(), Box<std::error::Error>> {
//!     // Create Bucket in REGION for BUCKET
//!     let credentials = Credentials::default();
//!     let region = REGION.parse()?;
//!     let bucket = Bucket::new(BUCKET, region, credentials)?;
//!
//!     // List out contents of directory
//!     let results = bucket.list("", None)?;
//!     for (list, code) in results {
//!         assert_eq!(200, code);
//!         println!("List of {}:", bucket.name());
//!         for obj in list.contents {
//!             println!("  {}", obj.key);
//!         }
//!     }
//!
//!     // Make sure that our "test_file" doesn't exist, delete it if it does. Note
//!     // that the s3 library returns the HTTP code even if it indicates a failure
//!     // (i.e. 404) since we can't predict desired usage. For example, you may
//!     // expect a 404 to make sure a file doesn't exist.
//!     let (_, code) = bucket.delete("test_file")?;
//!     assert_eq!(204, code);
//!
//!     // Put a "test_file" with the contents of MESSAGE at the root of the
//!     // bucket.
//!     let (_, code) = bucket.put("test_file", MESSAGE.as_bytes(), "text/plain")?;
//!     assert_eq!(200, code);
//!
//!     // Get the "test_file" contents and make sure that the returned message
//!     // matches what we sent.
//!     let (data, code) = bucket.get("test_file")?;
//!     let string = str::from_utf8(&data)?;
//!     assert_eq!(200, code);
//!     assert_eq!(MESSAGE, string);
//!
//!     Ok(())
//! }
//! ```

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
