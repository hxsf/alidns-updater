use std::borrow::Borrow;

use anyhow::{bail, Error};
use chrono::{Local, Utc};
use crypto::{hmac::Hmac, mac::Mac, sha1::Sha1};
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use url::form_urlencoded::byte_serialize;

#[derive(Debug, Clone, Eq, PartialOrd, PartialEq, Ord, Serialize, Deserialize)]
pub struct Record {
    name: String,
}

/// Default const param.
const DEFAULT_PARAM: &[(&str, &str)] = &[
    ("Format", "JSON"),
    ("SignatureMethod", "HMAC-SHA1"),
    ("SignatureVersion", "1.0"),
];

/// Config for request.
struct Config {
    action: String,
    query: Vec<(String, String)>,
}

/// The rpc style api client.
#[derive(Clone, Debug)]
pub struct RpcClient {
    /// The access key id of aliyun developer account.
    access_key_id: String,
    /// The access key secret of aliyun developer account.
    access_key_secret: String,
    /// The api endpoint of aliyun api service (need start with http:// or https://).
    endpoint: String,
    /// The api version of aliyun api service.
    version: String,
    /// The http client used to send request.
    http: surf::Client,
}

impl RpcClient {
    /// Create a rpc style api client.
    pub fn new(
        access_key_id: String,
        access_key_secret: String,
        endpoint: String,
        version: String,
    ) -> Self {
        RpcClient {
            access_key_id,
            access_key_secret,
            endpoint,
            version,
            http: surf::Client::new(),
        }
    }
    /// Create a get request, and set action of api.
    ///
    /// This function return a `Request` struct for send request.
    pub fn get(&self, action: &str) -> Call {
        Call::new(
            &self.access_key_id,
            &self.access_key_secret,
            &self.endpoint,
            &self.version,
            &self.http,
        )
            .get(action)
    }
}


/// The request.
pub struct Call<'a> {
    /// The access key id of aliyun developer account.
    access_key_id: &'a str,
    /// The access key secret of aliyun developer account.
    access_key_secret: &'a str,
    /// The api endpoint of aliyun api service (need start with http:// or https://).
    endpoint: &'a str,
    /// The api version of aliyun api service.
    version: &'a str,
    /// The config of http request.
    config: Config,
    /// The http client used to send request.
    http: &'a surf::Client,
}

impl<'a> Call<'a> {
    /// Create a request object.
    pub fn new(
        access_key_id: &'a str,
        access_key_secret: &'a str,
        endpoint: &'a str,
        version: &'a str,
        http: &'a surf::Client,
    ) -> Self {
        Call {
            access_key_id,
            access_key_secret,
            endpoint,
            version,
            config: Config {
                action: String::new(),
                query: Vec::new(),
            },
            http,
        }
    }
    /// Create a get request, and set action of api.
    pub fn get(mut self, action: &str) -> Self {
        self.config.action = action.to_string();
        self
    }

    /// Set queries for request.
    pub fn query<I>(mut self, iter: I) -> Self
        where
            I: IntoIterator,
            I::Item: Borrow<(&'a str, &'a str)>,
    {
        for i in iter.into_iter() {
            let b = i.borrow();
            self.config.query.push((b.0.to_string(), b.1.to_string()));
        }
        self
    }


    /// Send a request to api service.
    pub async fn send<T: DeserializeOwned>(&self) -> Result<T, Error> {
        // gen timestamp.
        let nonce = Local::now().timestamp_subsec_nanos().to_string();
        let ts = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

        // build params.
        let mut params = Vec::from(DEFAULT_PARAM);
        params.push(("Action", &self.config.action));
        params.push(("AccessKeyId", self.access_key_id));
        params.push(("SignatureNonce", &nonce));
        params.push(("Timestamp", &ts));
        params.push(("Version", self.version));
        params.extend(
            self.config
                .query
                .iter()
                .map(|(k, v)| (k.as_ref(), v.as_ref())),
        );
        params.sort_by_key(|item| item.0);

        // encode params.
        let params: Vec<String> = params
            .into_iter()
            .map(|(k, v)| format!("{}={}", url_encode(k), url_encode(v)))
            .collect();
        let sorted_query_string = params.join("&");
        let string_to_sign = format!(
            "GET&{}&{}",
            url_encode("/"),
            url_encode(&sorted_query_string)
        );

        // sign params, get finnal request url.
        let sign = sign(format!("{}&", self.access_key_secret).as_str(), string_to_sign.as_str());
        let signature = url_encode(sign.as_str());
        let final_url = format!(
            "{}?Signature={}&{}",
            self.endpoint, signature, sorted_query_string
        );

        // send request.
        let mut response = match self.http.get(final_url).send().await {
            Ok(resp) => resp,
            Err(e) => bail!(e),
        };
        let body = response.body_string().await.unwrap();

        // return response.
        match serde_json::from_str(body.as_str()) {
            Ok(v) => Ok(v),
            Err(e) => {
                println!("body: {}", body);
                bail!(e)
            }
        }
    }
}


fn url_encode(s: &str) -> String {
    let s: String = byte_serialize(s.as_bytes()).collect();
    s.replace('+', "%20")
        .replace('*', "%2A")
        .replace("%7E", "~")
}

fn sign(key: &str, body: &str) -> String {
    let mut mac = Hmac::new(Sha1::new(), key.as_bytes());
    mac.input(body.as_bytes());
    let result = mac.result();
    let code = result.code();
    base64::encode(code)
}
