extern crate s3;

use std::str;

use s3::{Bucket, Credentials};

const BUCKET: &str = "example-bucket";
const REGION: &str = "us-east-1";
const MESSAGE: &str = "I want to go to S3";

pub fn main() -> Result<(), Box<std::error::Error>> {
    // Create Bucket in REGION for BUCKET
    let credentials = Credentials::default();
    let region = REGION.parse()?;
    let bucket = Bucket::new(BUCKET, region, credentials)?;

    // List out contents of directory
    let results = bucket.list("", None)?;
    for (list, code) in results {
        assert_eq!(200, code);
        println!("{:?}", list.contents.len());
    }

    // Make sure that our "test_file" doesn't exist, delete it if it does. Note
    // that the s3 library returns the HTTP code even if it indicates a failure
    // (i.e. 404) since we can't predict desired usage. For example, you may
    // expect a 404 to make sure a file doesn't exist.
    let (_, code) = bucket.delete("test_file")?;
    assert_eq!(204, code);

    // Put a "test_file" with the contents of MESSAGE at the root of the
    // bucket.
    let (_, code) = bucket.put("test_file", MESSAGE.as_bytes(), "text/plain")?;
    assert_eq!(200, code);

    // Get the "test_file" contents and make sure that the returned message
    // matches what we sent.
    let (data, code) = bucket.get("test_file")?;
    let string = str::from_utf8(&data)?;
    assert_eq!(200, code);
    assert_eq!(MESSAGE, string);

    Ok(())
}
