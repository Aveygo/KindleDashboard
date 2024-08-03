mod linear_rg;

mod spx;
mod btc;
mod yield_spread;
mod linux;
mod halving;
mod linux_version;

use chrono::{DateTime, Utc};

use log::{info, warn};
use std::time::Instant;

use async_std::future;
use std::time::Duration as stdDuration;

#[derive(Debug)]
pub struct Stats {
    /* Uses a sneaky way to scrap financial data from FRED */
    pub d_spx500: Option<f64>,
    pub d_btc: Option<f64>,
    pub yield_spread: Option<f64>,

    pub linux_share: Option<f64>,
    pub btc_halving: Option<DateTime<Utc>>, /* Estimated future date of event */
    pub kernel_version: Option<String>
}

pub async fn fetch_stats() -> Result<Stats, Box<dyn std::error::Error>> {
    
    info!("Fetching statistics...");
    let now = Instant::now();
    
    let timeout = stdDuration::from_secs(25);

    //let a = spx::fetch();
    //let d = linux::fetch();
    //let e = halving::fetch();
    //let f = linux_version::fetch();

    let a = future::timeout(timeout, spx::fetch());
    let d = future::timeout(timeout, linux::fetch());
    let e = future::timeout(timeout, halving::fetch());
    let f = future::timeout(timeout, linux_version::fetch());

    let (a, d, e, f) = (
        a.await, 
        d.await, 
        e.await, 
        f.await
    );

    let a = match a {Ok(r) => {r}, Err(e) => Err(format!("Timeout: {e}").into())};
    let d = match d {Ok(r) => {r}, Err(e) => Err(format!("Timeout: {e}").into())};
    let e = match e {Ok(r) => {r}, Err(e) => Err(format!("Timeout: {e}").into())};
    let f = match f {Ok(r) => {r}, Err(e) => Err(format!("Timeout: {e}").into())};

    match &a {Ok(_) => {}, Err(e) => warn!("SPX stats failed: {e}")}
    match &d {Ok(_) => {}, Err(e) => warn!("Linux Share stats failed: {e}")}
    match &e {Ok(_) => {}, Err(e) => warn!("Halving stats failed: {e}")}
    match &f {Ok(_) => {}, Err(e) => warn!("Linux Version stats failed: {e}")}

    // staggered fetching to spread out fred.com requests 
    let b = future::timeout(timeout, btc::fetch());
    let c = future::timeout(timeout, yield_spread::fetch());

    let (b, c) = (
        b.await, 
        c.await, 
    );

    let b = match b {Ok(r) => {r}, Err(e) => Err(format!("Timeout: {e}").into())};
    let c = match c {Ok(r) => {r}, Err(e) => Err(format!("Timeout: {e}").into())};

    match &b {Ok(_) => {}, Err(e) => warn!("BTC stats failed: {e}")}
    match &c {Ok(_) => {}, Err(e) => warn!("Yield Spread Version stats failed: {e}")}

    let elapsed = format!("{:.2?}", now.elapsed());
    info!("Statistics took {elapsed}");
    
    Ok(
        Stats{
            d_spx500: a.ok(),
            d_btc: b.ok(),
            yield_spread: c.ok(),
            linux_share: d.ok(),
            btc_halving: e.ok(),
            kernel_version: f.ok()
        }
    )
}