use std::fmt::Debug;
use std::net::IpAddr;

use anyhow::{bail, Error};
use serde::{Deserialize, Deserializer, Serialize};
use serde::de::DeserializeOwned;

use alidns_updater::RpcClient;

#[derive(Debug, Serialize)]
pub struct AliDNS {
    key: String,
    secret: String,
    #[serde(skip)]
    client: RpcClient,
}

impl AliDNS {
    #[allow(dead_code)]
    pub fn new(key: String, secret: String) -> Self {
        let client = RpcClient::new(
            key.clone(),
            secret.clone(),
            "https://alidns.cn-shanghai.aliyuncs.com".into(),
            "2015-01-09".into(),
        );
        AliDNS { key, secret, client }
    }
    async fn call<T: Debug + DeserializeOwned>(&self, cmd: &str, args: &[(&str, &str)]) -> Result<T, Error> {
        let data = self.client.get(cmd).query(args).send().await;
        if let Err(e) = data {
            bail!(e)
        }
        data
    }
    pub async fn get_all(&self, domain: &str) -> Result<DomainList, Error> {
        self.call("DescribeDomainRecords", &[
            ("DomainName", domain),
            ("PageSize", "500"),
        ]).await
    }
    pub async fn get(&self, full_name: &str, domain: &str) -> Result<DomainList, Error> {
        self.call("DescribeSubDomainRecords", &[
            ("SubDomain", full_name),
            ("DomainName", domain),
        ]).await
    }
    pub async fn remove(&self, sub_name: &str, domain: &str) -> Result<usize, Error> {
        let res: RemoveResponse = self.call("DeleteSubDomainRecords", &[
            ("RR", sub_name),
            ("DomainName", domain),
        ]).await?;
        Ok(res.total_count)
    }
    pub async fn append(&self, sub_name: &str, domain: &str, value: IpAddr) -> Result<String, Error> {
        let res: AppendResponse = self.call("AddDomainRecord", &[
            ("DomainName", domain),
            ("RR", sub_name),
            ("Type", "A"),
            ("Value", value.to_string().as_str()),
        ]).await?;
        Ok(res.id)
    }
    // pub async fn set(&self, name: String) -> Result<Vec<Record>, Error> {
    //     self.call("DescribeSubDomainRecords", &[]).await
    // }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all(deserialize = "PascalCase", serialize = "snake_case"))]
struct AppendResponse {
    #[serde(rename(deserialize = "RecordId"))]
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all(deserialize = "PascalCase", serialize = "snake_case"))]
struct RemoveResponse {
    pub total_count: usize,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all(deserialize = "PascalCase", serialize = "snake_case"))]
pub struct DomainList {
    #[serde(flatten)]
    pub pagination: Pagination,
    #[serde(rename(deserialize = "DomainRecords"), deserialize_with = "Wrapper::deserialize")]
    pub records: Vec<Record>,
}

#[derive(Deserialize)]
#[serde(rename_all(deserialize = "PascalCase", serialize = "snake_case"))]
struct Wrapper<T> {
    #[serde(rename(deserialize = "Record"))]
    inner: T,
}

impl<T> Wrapper<T> {
    fn deserialize<'de, D>(deserializer: D) -> Result<T, D::Error>
        where
            T: Deserialize<'de>,
            D: Deserializer<'de>,
    {
        let wrapper = <Self as Deserialize>::deserialize(deserializer)?;
        Ok(wrapper.inner)
    }
}


#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "PascalCase", serialize = "snake_case"))]
pub struct Pagination {
    pub total_count: usize,
    pub page_number: usize,
    pub page_size: usize,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all(deserialize = "PascalCase", serialize = "snake_case"))]
pub struct Record {
    #[serde(rename(deserialize = "RecordId"))]
    pub id: String,

    #[serde(rename(deserialize = "Type", serialize = "type"))]
    pub typ: DNSType,
    #[serde(rename(deserialize = "RR"))]
    pub rr: String,
    pub domain_name: String,
    pub value: String,

    // unit: seconds
    #[serde(rename(deserialize = "TTL"))]
    pub ttl: u64,
    pub weight: Option<usize>,

    pub status: String,
    pub line: String,
    pub locked: bool,
}


#[derive(Serialize, Deserialize, Debug)]
pub enum DNSType {
    A,
    AAAA,
    CNAME,
    TXT,
    NS,
    MX,
    SRV,
    CAA,
    #[serde(deserialize_with = "deserialize_ignore_any")]
    Unknown,
}

pub fn deserialize_ignore_any<'de, D: Deserializer<'de>, T: Default>(
    deserializer: D,
) -> Result<T, D::Error> {
    serde::de::IgnoredAny::deserialize(deserializer).map(|_| T::default())
}
