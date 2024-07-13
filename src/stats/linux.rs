use reqwest;
use reqwest::header::USER_AGENT;
use regex::Regex;

pub async fn fetch() -> Result<f64, Box<dyn std::error::Error>> {
    let url = "https://gs.statcounter.com/os-market-share/desktop/worldwide".to_string();
    
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .header(USER_AGENT, "Mozilla/5.0 (Android 4.4; Mobile; rv:41.0) Gecko/41.0 Firefox/41.0")
        .send()
        .await?;

    let response = response.error_for_status()?;
    let site_html = response.text().await?;

    let re = Regex::new(r#"<th>\s*Linux\s*</th>\s*<td><span class="count">([\d\.]+)</span>%"#)?;
    if let Some(captures) = re.captures(&site_html) {
        if let Some(count) = captures.get(1) {
            let result: f64 = count.as_str().parse()?;
            return Ok(result);
        }
    }

    Ok(0.0)    
}