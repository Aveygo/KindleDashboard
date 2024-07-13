use reqwest;
use serde::Deserialize;
use reqwest::header::USER_AGENT;
use chrono::prelude::*;

#[derive(Deserialize, Debug)]
struct BlockchainResponse {
    block_index: i64,
}

pub async fn fetch() -> Result<DateTime<Utc>, Box<dyn std::error::Error>> {
    let url = "https://blockchain.info/latestblock".to_string();
    
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .header(USER_AGENT, "Mozilla/5.0 (Android 4.4; Mobile; rv:41.0) Gecko/41.0 Firefox/41.0")
        .send()
        .await?;

    let response = response.error_for_status()?;
    let response:BlockchainResponse = response.json().await?;

    let remaining_blocks = 210_000 - (response.block_index % 210_000); // BTC halves every 210_000 blocks
    let seconds_remaining = remaining_blocks * 10 * 60; // * 10 minutes per block, in seconds
    let now = Utc::now().timestamp();
    let halving = now + seconds_remaining;
    
    Ok(DateTime::from_timestamp(halving, 0).ok_or("Invalid timestamp")?)
}