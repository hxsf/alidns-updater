use std::net::IpAddr;
use std::sync::Arc;

use clap::Parser;
use tide::{Body, Request};
use tide::prelude::*;
use tide_tracing::TraceMiddleware;
use tracing_subscriber::fmt::format;

use crate::alidns::AliDNS;

mod alidns;

#[derive(Debug, Clone)]
struct Config {
    domain: String,
}

#[derive(Debug, Clone)]
struct State {
    aliyun: Arc<AliDNS>,
    config: Config,
}

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    domain: String,

    #[clap(short, long)]
    key: String,
    #[clap(short, long)]
    secret: String,
    #[clap(short, long)]
    port: u16,
}


#[async_std::main]
async fn main() -> tide::Result<()> {
    let args: Args = Args::parse();

    tracing_subscriber::fmt::init();
    let mut app = tide::with_state(State {
        aliyun: Arc::new(AliDNS::new(args.key, args.secret)),
        config: Config { domain: args.domain },
    });
    app.with(TraceMiddleware::new()).with(tide::log::LogMiddleware::new());
    app.at("/dns")
        .get(get_all_dns);
    app.at("/dns/:rr")
        .get(get_dns)
        .post(update_dns);
    app.listen(format!("0.0.0.0:{}", args.port)).await?;
    Ok(())
}

#[derive(Debug, Deserialize, Serialize)]
struct DomainName {
    ip: IpAddr,
}

async fn get_all_dns(req: Request<State>) -> tide::http::Result<Body> {
    let records = req.state().aliyun.get_all(req.state().config.domain.as_str()).await?;
    Body::from_json(&records)
}

async fn get_dns(req: Request<State>) -> tide::http::Result<Body> {
    let domain = req.state().config.domain.as_str();
    let name = req.param("rr")?;
    let full_name = format!("{}.{}", name, domain);
    println!("will get {} ip list.", full_name);
    let records = req.state().aliyun.get(full_name.as_str(), domain).await?;
    Body::from_json(&records)
}

async fn update_dns(mut req: Request<State>) -> tide::Result {
    let DomainName { ip } = req.body_json().await.or_else(|_| req.query())?;
    let domain = req.state().config.domain.as_str();
    let name = req.param("rr")?;
    println!("will update {}.{} with {}.", name, domain, ip);
    let aliyun = req.state().aliyun.clone();
    let full_name = format!("{}.{}", name, domain);
    if !aliyun.get(full_name.as_str(), domain).await?.records.is_empty() {
        aliyun.remove(name, domain).await?;
    }
    aliyun.append(name, domain, ip).await?;
    Ok(format!("{}.{} -> {}", name, domain, ip).into())
}
