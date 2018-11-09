extern crate futures;
extern crate s3;
extern crate tokio;

use std::str;

use futures::{future, Future, Stream};
use s3::{Bucket, Credentials, S3Response};

const BUCKET: &str = "bitcurry-s3-test";
const REGION: &str = "us-east-1";
const MESSAGE: &str = "I want to go to S3";

pub fn main() {
    // Create Bucket in REGION for BUCKET
    let credentials = Credentials::new(None, None, None, Some("personal".into()));
    let region = REGION.parse().unwrap();
    let bucket = Bucket::new(BUCKET, region, credentials);

    // List out contents of directory
    // let results = bucket.list("", None).unwrap();
    // for (list, code) in results {
    //     assert_eq!(200, code);
    //     println!("{:?}", list.contents.len());
    // }

    // // Make sure that our "test_file" doesn't exist, delete it if it does. Note
    // // that the s3 library returns the HTTP code even if it indicates a failure
    // // (i.e. 404) since we can't predict desired usage. For example, you may
    // // expect a 404 to make sure a file doesn't exist.
    // let (_, code) = bucket.delete("test_file").unwrap();
    // assert_eq!(204, code);

    // // Put a "test_file" with the contents of MESSAGE at the root of the
    // // bucket.
    // let (_, code) = bucket.put("test_file", MESSAGE.as_bytes(), "text/plain").unwrap();
    // assert_eq!(200, code);

    // Get the "test_file" contents and make sure that the returned message
    // matches what we sent.
    let f = bucket.get_async("test_file").unwrap();
    tokio::run(
        f.and_then(|response: S3Response| {
            assert_eq!(200, response.response.status().as_u16());
            response.response.into_body().concat2().and_then(|buffer| {
                let string = str::from_utf8(&buffer).unwrap();
                println!("File contents: {:?}", string);
                assert_eq!(MESSAGE, string);
                future::ok(())
            }).map_err(|e| panic!("{:?}", e))
        }).map_err(|e| println!("{:?}", e)),
    );
}
