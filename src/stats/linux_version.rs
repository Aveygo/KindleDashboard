use reqwest;
use serde::Deserialize;
use reqwest::header::USER_AGENT;

#[derive(Deserialize, Debug)]
struct GithubResponse {
    #[serde(rename = "ref")]
    tag_ref: String,
}

pub async fn fetch() -> Result<String, Box<dyn std::error::Error>> {
    let url = "https://api.github.com/repos/torvalds/linux/git/refs/tags".to_string();
    
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .header(USER_AGENT, "Mozilla/5.0 (Android 4.4; Mobile; rv:41.0) Gecko/41.0 Firefox/41.0")
        .send()
        .await?;

    let response = response.error_for_status()?;
    let mut response:Vec<GithubResponse> = response.json().await?;

    let current_version = response.pop().ok_or("Invalid data")?.tag_ref;
    let current_version = current_version.split('/').collect::<Vec<&str>>().pop().ok_or("Invalid data")?;

    Ok(current_version.to_string())
}