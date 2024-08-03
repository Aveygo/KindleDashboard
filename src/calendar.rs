use chrono::{NaiveDateTime, TimeZone};

use std::io::BufReader;
use ical::IcalParser;
use reqwest::header::{HeaderMap, USER_AGENT};
use reqwest::Client;
use chrono::{DateTime, Utc};
use serde::Deserialize;

use tokio::fs::File as tFile;
use tokio::io::AsyncReadExt;
use serde_json;
use futures::future::join_all;
use log::info;
use std::time::Instant;

#[derive(Debug)]
pub struct CalendarEvent {
    pub start_time: DateTime<chrono::Utc>,
    pub name: String,
}

#[derive(Debug, Deserialize)]
struct CalendarUrls {
    urls: Vec<String>,
}

async fn fetch_ics(url: &str) -> Result<String, reqwest::Error> {
    let custom_user_agent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/79.0.3945.79 Safari/537.36";
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, custom_user_agent.parse().unwrap());
    
    let response = Client::new().get(url).headers(headers).send().await?;
    response.text().await
}

fn parse_datetime(s: &str) -> Result<DateTime<Utc>, String> {
    let utc_formats = [
        "%Y%m%dT%H%M%SZ",
        "%Y-%m-%dT%H:%M:%S%z",
        "%Y-%m-%dT%H:%M:%S%:z",
    ];

    let local_formats = [
        "%Y-%m-%d",
        "%Y%m%d",
        "%Y-%m-%d (%a)",
        "%Y-W%W-%u",
        "%H:%M:%S",
        "%H:%M:%S%.f",
        "%I:%M:%S %p",
        
        "%Y%m%dT%H%M%S",
        "%Y-%m-%dT%H:%M:%S%.f",
        "%Y-W%W-%uT%H:%M:%S",
        "%a, %d %b %Y %H:%M:%S",
        "%d %B %Y %H:%M:%S",     
        "%Y/%m/%d %H-%M-%S",
        "%Y-%j %H:%M:%S",
    ];

    for format in local_formats.iter() {

        let naive_date = chrono::NaiveDate::parse_from_str(s, format);
        
        if let Ok(naive_date) = naive_date {
            let local_date_time = chrono::Local.from_local_datetime(&naive_date.and_hms_opt(0, 0, 0).unwrap())
                .single()
                .expect("Failed to convert to local DateTime");
            return Ok(local_date_time.into());
        }
    }


    for format in utc_formats.iter() {
        let naive_date = chrono::NaiveDate::parse_from_str(s, format);
        if let Ok(naive_date) = naive_date {
            let naive_date_time: NaiveDateTime = naive_date.into();
            let time = Utc.from_utc_datetime(&naive_date_time);
            return Ok(time);
        }
    }
    
    Err("Failed to parse datetime".to_string())
}

fn parse_ics(data: &str) -> Vec<CalendarEvent> {
    let buf = BufReader::new(data.as_bytes());
    let reader = IcalParser::new(buf);
    let mut events = Vec::new();
    let now = Utc::now();

    for line in reader {
        if let Ok(line) = line {
            for cal in line.events {
                let mut start_time = None;
                let mut name = None;

                for prop in cal.properties {
                    match prop.name.as_str() {
                        
                        // Get the start of the event
                        // Note! We assume that if a "z" is not present, then it is in local time

                        "DTSTART" => {
                            if let Some(value) = prop.value {

                                let utc_time = parse_datetime(&value);
                                
                                if let Ok(utc_time) = utc_time {
                                    start_time = Some(utc_time)
                                }
                            }
                        }
                        "SUMMARY" => {
                            name = prop.value.clone();
                        }
                        _ => {}
                    }
                }
                
                if let (Some(start_time), Some(name)) = (start_time, name) {

                    if start_time > now {
                        events.push(CalendarEvent { start_time, name });
                    }
                }

                
            }

        }
    }

    events
}




pub async fn fetch_event() -> Result<Option<CalendarEvent>, String> {
    info!("Fetching calendar..");
    let now = Instant::now();

    // Asynchronously read the file
    let mut file = tFile::open("sensitive/calendars.json").await
        .map_err(|e| e.to_string())?;
    
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)
        .await
        .map_err(|e| e.to_string())?;
    
    let calendar_urls: CalendarUrls = serde_json::from_slice(&contents)
        .map_err(|e| e.to_string())?;
    
    // Fetch all ICS data concurrently
    let fetch_futures: Vec<_> = calendar_urls.urls
        .iter()
        .map(|url| fetch_ics(url))
        .collect();
    
    let fetched_ics_data = join_all(fetch_futures)
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    
    // Parse all ICS data concurrently
    let parse_futures: Vec<_> = fetched_ics_data
        .into_iter()
        .map(|ics_data| async move { parse_ics(&ics_data) })
        .collect();
    
    let mut all_events: Vec<_> = join_all(parse_futures)
        .await
        .into_iter()
        .flatten()
        .collect();
    
    all_events.sort_by_key(|d| d.start_time);
    all_events.reverse();
        
    let next_event = all_events.pop();

    let elapsed = format!("{:.2?}", now.elapsed());
    info!("Calendar took {elapsed}");

    Ok(next_event)
}
