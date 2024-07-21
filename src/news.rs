use reqwest;
use serde::Deserialize;
use reqwest::header::USER_AGENT;

use log::info;
use std::time::Instant;

#[derive(Deserialize, Debug)]
struct RedditResponse {
    data: RedditData,
}

#[derive(Deserialize, Debug)]
struct RedditData {
    children: Vec<Children>,
}

#[derive(Deserialize, Debug)]
struct Children {
    data: Data,
}

#[derive(Deserialize, Debug)]
struct Data {
    title: String,
    pinned: bool,
    stickied: bool,
}


pub async fn fetch_news() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    info!("Fetching news..");
    let now = Instant::now();

    let url = "https://www.reddit.com/r/worldnews/top/.json".to_string();
    let client = reqwest::Client::new();

    let response = match client
        .get(&url)
        .header(USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36")
        .send()
        .await
    {
        Ok(response) => response,
        Err(err) => return Err(format!("Failed to send request: {}", err).into()),
    };

    if !response.status().is_success() {
        return Err(format!("Request failed with status: {}", response.status()).into());
    }

    let news_data: RedditResponse = match response.json().await {
        Ok(data) => data,
        Err(err) => return Err(format!("Failed to read response json: {}", err).into()),
    };

    let mut result = vec![];
    for child in news_data.data.children {
        if ! (child.data.pinned || child.data.stickied) {
            result.push(child.data.title)
        }
    }

    let elapsed = format!("{:.2?}", now.elapsed());
    info!("News took {elapsed}");
   
    Ok(result)    
}