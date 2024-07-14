extern crate reqwest;
use reqwest::header;
use serde::Deserialize;

use crate::stats::linear_rg;

use chrono::{Duration, Utc};

#[derive(Deserialize, Debug)]
struct FredResponse {
    observations: Vec<Vec<Vec<Option<f64>>>>,
}

fn get_start_date() -> String {
    let today = Utc::now().naive_utc();
    let one_week_ago = today - Duration::weeks(1);
    one_week_ago.format("%Y-%m-%d").to_string()
}

fn get_end_date() -> String {
    let today = Utc::now().naive_utc();
    today.format("%Y-%m-%d").to_string()
}

pub async fn fetch() -> Result<f64, Box<dyn std::error::Error>> {
    let mut headers = header::HeaderMap::new();
    headers.insert("User-Agent", "Mozilla/5.0 (X11; Linux x86_64; rv:126.0) Gecko/20100101 Firefox/126.0".parse()?);

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;

    let start = get_start_date();
    let end = get_end_date();
    
    let response = client.post("https://fred.stlouisfed.org/graph/api/series/?obs=true&sid=CBBTCUSD")
        .headers(headers)
        .body(format!("{{\"seriesObjects\":[{{\"series_objects\":{{\"a\":{{\"series_id\":\"CBBTCUSD\",\"min_obs_start_date\":\"{start}\",\"max_obs_start_date\":\"{end}\"}}}}}}]}}"))
        .send()
        .await?;

    let response = response.error_for_status()?;
    let mut data:FredResponse = response.json().await?;

    let mut x:Vec<f64> = vec![];
    let mut y:Vec<f64> = vec![];

    for mut point in data.observations.pop().ok_or("Invalid data")? {

        // We always expect at least a pair of data in each point
        let time = point.pop().ok_or("Invalid data")?;
        let value = point.pop().ok_or("Invalid data")?;

        // Data can be null...
        if let Some(time) = time {
            if let Some(value) = value {
                x.push(time);
                y.push(value);
            }
        }
    }

    let x_tail = x.as_slice()[x.len()-14..].to_vec();
    let y_tail = (0..14).map(|x| x as f64).collect();

    let mut algo = linear_rg::LinearRegression::new();
    algo.fit(x_tail, y_tail);
    Ok(algo.slope * 100.0)
}