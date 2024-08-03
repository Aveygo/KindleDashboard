mod linear_rg;

mod spx;
mod btc;
mod yield_spread;
mod linux;
mod halving;
mod linux_version;

use chrono::{DateTime, Utc};

use log::info;
use std::time::Instant;

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

    let a = spx::fetch();
    let d = linux::fetch();
    let e = halving::fetch();
    let f = linux_version::fetch();

    let (a, d, e, f) = (
        a.await, 
        d.await, 
        e.await, 
        f.await
    );

    // staggered fetching to spread out fred.com requests 
    let b = btc::fetch().await;
    let c = yield_spread::fetch().await;

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