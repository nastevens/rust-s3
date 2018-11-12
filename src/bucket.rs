use std::collections::HashMap;
use std::mem;

use reqwest::Client;
use serde_xml;

use credentials::Credentials;
use command::Command;
use region::Region;
use request::{Request, Headers, Query};
use serde_types::ListBucketResult;
use error::S3Result;

/// Primary interface to an AWS S3 bucket.
///
/// # Example
/// ```
/// # use std::env;
/// # fn main() -> Result<(), Box<std::error::Error>> {
/// # // Fake  credentials so we don't access user's real credentials in tests
/// # env::set_var("AWS_ACCESS_KEY_ID", "AKIAIOSFODNN7EXAMPLE");
/// # env::set_var("AWS_SECRET_ACCESS_KEY", "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY");
/// #
/// use s3::{Bucket, Credentials};
///
/// let bucket_name = "rust-s3-test";
/// let region = "us-east-1".parse()?;
/// let credentials = Credentials::default();
///
/// let bucket = Bucket::new(bucket_name, region, credentials)?;
/// #
/// #     Ok(())
/// # }
/// ```
#[derive(Clone, Debug)]
pub struct Bucket {
    name: String,
    region: Region,
    credentials: Credentials,
    extra_headers: Headers,
    extra_query: Query,
    client: Client,
}

fn build_client() -> S3Result<Client> {
    if cfg!(feature = "no-verify-ssl") {
        Ok(reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .danger_accept_invalid_hostnames(true)
            .build()?)
    } else {
        Ok(reqwest::Client::builder().build()?)
    }
}

impl Bucket {
    /// Instantiate a new `Bucket`.
    ///
    /// # Example
    /// ```
    /// # use std::env;
    /// # fn main() -> Result<(), Box<std::error::Error>> {
    /// # // Fake  credentials so we don't access user's real credentials in tests
    /// # env::set_var("AWS_ACCESS_KEY_ID", "AKIAIOSFODNN7EXAMPLE");
    /// # env::set_var("AWS_SECRET_ACCESS_KEY", "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY");
    /// #
    /// use s3::{Bucket, Credentials};
    ///
    /// let bucket_name = "rust-s3-test";
    /// let region = "us-east-1".parse()?;
    /// let credentials = Credentials::default();
    ///
    /// let bucket = Bucket::new(bucket_name, region, credentials)?;
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    pub fn new(name: &str, region: Region, credentials: Credentials) -> S3Result<Bucket> {
        Ok(Bucket {
            name: name.into(),
            region,
            credentials,
            extra_headers: HashMap::new(),
            extra_query: HashMap::new(),
            client: build_client()?,
        })
    }

    /// Gets file from an S3 path.
    ///
    /// # Example:
    ///
    /// ```rust,no_run
    /// # fn main() -> Result<(), Box<std::error::Error>> {
    /// #
    /// use s3::{Bucket, Credentials};
    ///
    /// let bucket_name = "rust-s3-test";
    /// let region = "us-east-1".parse()?;
    /// let credentials = Credentials::default();
    /// let bucket = Bucket::new(bucket_name, region, credentials)?;
    ///
    /// let (data, code) = bucket.get("/test.file")?;
    /// println!("Code: {}\nData: {:?}", code, data);
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    pub fn get(&self, path: &str) -> S3Result<(Vec<u8>, u32)> {
        let command = Command::Get;
        let request = Request::new(self, path, command);
        request.execute()
    }

    /// Delete file from an S3 path.
    ///
    /// # Example:
    ///
    /// ```rust,no_run
    /// # fn main() -> Result<(), Box<std::error::Error>> {
    /// #
    /// use s3::{Bucket, Credentials};
    ///
    /// let bucket_name = &"rust-s3-test";
    /// let region = "us-east-1".parse()?;
    /// let credentials = Credentials::default();
    /// let bucket = Bucket::new(bucket_name, region, credentials)?;
    ///
    /// let (_, code) = bucket.delete("/test.file")?;
    /// assert_eq!(204, code);
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    pub fn delete(&self, path: &str) -> S3Result<(Vec<u8>, u32)> {
        let command = Command::Delete;
        let request = Request::new(self, path, command);
        request.execute()
    }

    /// Put into an S3 bucket.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # fn main() -> Result<(), Box<std::error::Error>> {
    /// #
    /// use s3::{Bucket, Credentials};
    ///
    /// let bucket_name = &"rust-s3-test";
    /// let aws_access = &"access_key";
    /// let aws_secret = &"secret_key";
    ///
    /// let bucket_name = &"rust-s3-test";
    /// let region = "us-east-1".parse()?;
    /// let credentials = Credentials::default();
    /// let bucket = Bucket::new(bucket_name, region, credentials)?;
    ///
    /// let content = "I want to go to S3".as_bytes();
    /// let (_, code) = bucket.put("/test.file", content, "text/plain")?;
    /// assert_eq!(201, code);
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    pub fn put(&self, path: &str, content: &[u8], content_type: &str) -> S3Result<(Vec<u8>, u32)> {
        let command = Command::Put {
            content,
            content_type,
        };
        let request = Request::new(self, path, command);
        request.execute()
    }

    fn _list(&self,
                 prefix: &str,
                 delimiter: Option<&str>,
                 continuation_token: Option<&str>)
                 -> S3Result<(ListBucketResult, u32)> {
        let command = Command::List {
            prefix,
            delimiter,
            continuation_token,
        };
        let request = Request::new(self, "/", command);
        let result = request.execute()?;
        let result_string = String::from_utf8_lossy(&result.0);
        let deserialized: ListBucketResult = serde_xml::deserialize(result_string.as_bytes())?;
        Ok((deserialized, result.1))
    }

    /// List the contents of an S3 bucket.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # fn main() -> Result<(), Box<std::error::Error>> {
    /// #
    /// use std::str;
    /// use s3::{Bucket, Credentials};
    ///
    /// let bucket_name = &"rust-s3-test";
    /// let aws_access = &"access_key";
    /// let aws_secret = &"secret_key";
    ///
    /// let bucket_name = &"rust-s3-test";
    /// let region = "us-east-1".parse()?;
    /// let credentials = Credentials::default();
    /// let bucket = Bucket::new(bucket_name, region, credentials)?;
    ///
    /// let results = bucket.list("/", Some("/"))?;
    /// for (list, code) in results {
    ///     assert_eq!(200, code);
    ///     println!("{:?}", list);
    /// }
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    pub fn list(&self, prefix: &str, delimiter: Option<&str>) -> S3Result<Vec<(ListBucketResult, u32)>> {
        let mut results = Vec::new();
        let mut result = self._list(prefix, delimiter, None)?;
        loop {
            results.push(result.clone());
            match result.0.next_continuation_token {
                Some(token) => result = self._list(prefix, delimiter, Some(&token))?,
                None => break,
            }
        }

        Ok(results)
    }

    /// Get a reference to the name of the S3 bucket.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get a reference to the hostname of the S3 API endpoint.
    pub fn host(&self) -> &str {
        self.region.host()
    }

    pub fn scheme(&self) -> &str {
        self.region.scheme()
    }

    /// Get the region this object will connect to.
    pub fn region(&self) -> Region {
        self.region.clone()
    }

    /// Get a reference to the AWS access key.
    pub fn access_key(&self) -> &str {
        &self.credentials.access_key
    }

    /// Get a reference to the AWS secret key.
    pub fn secret_key(&self) -> &str {
        &self.credentials.secret_key
    }

    /// Get a reference to the AWS token.
    pub fn token(&self) -> Option<&str> {
        self.credentials.token.as_ref().map(|s| s.as_str())
    }

    /// Get a reference to the full [`Credentials`](struct.Credentials.html)
    /// object used by this `Bucket`.
    pub fn credentials(&self) -> &Credentials {
        &self.credentials
    }

    /// Change the credentials used by the Bucket, returning the existing
    /// credentials.
    pub fn set_credentials(&mut self, credentials: Credentials) -> Credentials {
        mem::replace(&mut self.credentials, credentials)
    }

    /// Add an extra header to send with requests to S3.
    ///
    /// Add an extra header to send with requests. Note that the library
    /// already sets a number of headers - headers set with this method will be
    /// overridden by the library headers:
    ///   * Host
    ///   * Content-Type
    ///   * Date
    ///   * Content-Length
    ///   * Authorization
    ///   * X-Amz-Content-Sha256
    ///   * X-Amz-Date
    pub fn add_header(&mut self, key: &str, value: &str) {
        self.extra_headers.insert(key.into(), value.into());
    }

    /// Get a reference to the extra headers to be passed to the S3 API.
    pub fn extra_headers(&self) -> &Headers {
        &self.extra_headers
    }

    /// Get a mutable reference to the extra headers to be passed to the S3
    /// API.
    pub fn extra_headers_mut(&mut self) -> &mut Headers {
        &mut self.extra_headers
    }

    /// Add an extra query pair to the URL used for S3 API access.
    pub fn add_query(&mut self, key: &str, value: &str) {
        self.extra_query.insert(key.into(), value.into());
    }

    /// Get a reference to the extra query pairs to be passed to the S3 API.
    pub fn extra_query(&self) -> &Query {
        &self.extra_query
    }

    /// Get a mutable reference to the extra query pairs to be passed to the S3
    /// API.
    pub fn extra_query_mut(&mut self) -> &mut Query {
        &mut self.extra_query
    }

    pub(crate) fn client(&self) -> &Client {
        &self.client
    }
}
