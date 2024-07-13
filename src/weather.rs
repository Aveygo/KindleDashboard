use reqwest;
use serde::Deserialize;
use chrono::prelude::*;

use log::info;
use std::time::Instant;


#[derive(Deserialize, Debug)]
struct OpenWeatherMapKey {
    key: String,
}

#[derive(Deserialize, Debug)]
struct WeatherData {
    list: Vec<Data>,
}

#[derive(Deserialize, Debug)]
struct Data {
    dt: i64,
    main: Main,
    rain: Option<Rain>,
    cloud: Option<Cloud>
}

#[derive(Deserialize, Debug)]
struct Main {
    temp_min: f64,
    temp_max: f64,
}

#[derive(Deserialize, Debug)]
struct Rain {
    #[serde(rename = "3h")]
    three_h: f64,
}

#[derive(Deserialize, Debug)]
struct Cloud {
    all: f64,
}

#[derive(Default, Debug)]
pub struct DayData {
    pub data_points: i8,
    pub date: u32,
    pub day: String,
    pub rain_sum: f64,
    pub cloud_sum: f64,
    pub max_c: f64,
    pub min_c: f64,
}

// Async function to fetch weather data
pub async fn fetch_weather() -> Result<Vec<DayData>, Box<dyn std::error::Error>> {

    info!("Fetching weather...");
    let now = Instant::now();

    let file = std::fs::File::open("sensitive/openweatherkey.json")?;
    let json_key:OpenWeatherMapKey = serde_json::from_reader(file)?;
    let key = json_key.key;
    let url = format!("http://api.openweathermap.org/data/2.5/forecast?lat=-33.8679&lon=151.2073&units=metric&appid={key}");
    
    let response = reqwest::get(&url).await?;
    let response = response.error_for_status()?;
    let weather_data: WeatherData = response.json().await?;
    let mut result = vec![];

    let mut current_day = chrono::offset::Utc::now().day();

    for point in weather_data.list {

        let point_day = DateTime::from_timestamp(point.dt, 0).ok_or("Invalid datetime")?;

        if result.len() == 0 {
            let mut data = DayData::default();
            current_day = point_day.day();

            data.date = current_day;
            data.day = point_day.weekday().to_string();
            data.min_c = point.main.temp_min;
            data.max_c = point.main.temp_max;
            result.push(data);
        }

        if point_day.day() == current_day {
            let current = result.pop();

            match current {
                Some(mut current) => {
                    current.data_points += 1;
                    if point.main.temp_min < current.min_c {current.min_c = point.main.temp_min}
                    

                    current.max_c = f64::max(current.max_c, point.main.temp_max);
                    current.rain_sum += match point.rain { Some(rain) => rain.three_h, None => 0.0 };
                    current.cloud_sum += match point.cloud { Some(cloud) => cloud.all, None => 0.0 };
                    result.push(current);
                },
                None => {
                    panic!("No weather data - This should be impossible to reach...")
                }
            }
        } else {
            let mut data = DayData::default();
            data.date = point_day.day();
            data.day = point_day.weekday().to_string();
            data.min_c = f64::INFINITY;
            data.max_c = f64::NEG_INFINITY;
            result.push(data);
            current_day += 1;
        }
    }

    let elapsed = format!("{:.2?}", now.elapsed());
    info!("Weather took {elapsed}");

    Ok(result)    
}