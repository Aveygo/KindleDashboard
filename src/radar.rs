use reqwest;
use image;
use serde::Deserialize;
use serde_json::from_reader;
use std::fs::File;

use image::{DynamicImage, GenericImageView, GenericImage, imageops};

use regex::Regex;
use reqwest::header::USER_AGENT;

use log::{info, warn};
use std::time::Instant;


#[derive(Deserialize, Debug)]
struct Bom {
    station: String,
}

pub async fn get_image(url:String) -> Result<image::DynamicImage, String> {
    let client = reqwest::Client::new();

    let img_bytes = match client
        .get(url)
        .header(USER_AGENT, "Mozilla/5.0 (Android 4.4; Mobile; rv:41.0) Gecko/41.0 Firefox/41.0")
        .send()
        .await {
            Ok(response) => {
                match response.bytes().await {
                    Ok(img_bytes) => img_bytes,
                    Err(err) => return Err(format!("Failed to read response bytes: {}", err)),
                }
            },
            Err(err) => return Err(format!("Failed to fetch image: {}", err)),
    };

    let image = match image::load_from_memory(&img_bytes) {
        Ok(img) => img,
        Err(err) => return Err(format!("Failed to load image: {}", err)),
    };

    Ok(image)
}

pub async fn get_radar_id(station:String) -> Result<String, Box<dyn std::error::Error>> {
    let url = format!("https://reg.bom.gov.au/products/{station}.loop.shtml");
    
    let client = reqwest::Client::new();

    let response = match client
        .get(url)
        .header(USER_AGENT, "Mozilla/5.0 (Android 4.4; Mobile; rv:41.0) Gecko/41.0 Firefox/41.0")
        .send()
        .await {
            Ok(response) => {
                match response.text().await {
                    Ok(img_bytes) => img_bytes,
                    Err(err) => return Err(format!("Failed to read response bytes: {}", err).into()),
                }
            },
            Err(err) => return Err(format!("Failed to fetch image: {}", err).into()),
    };

    let re = Regex::new(format!(r#"/radar/{station}\.T\.\d+\.png"#).as_str())?;
    let matches: Vec<_> = re.find_iter(&response).collect();
    
    if let Some(last_match) = matches.last() {
        let url: &_ = &response[last_match.start()..last_match.end()];
        return Ok(url.to_string());
    }

    return Err("No images?".to_string().into())
}

fn hide_banner(image: &DynamicImage) -> DynamicImage {
    // BOM adds a copyright flag at the top. Here we are just making those pixels transparent.
    // This will cause issues if the radar map is busy - empty "bar" at the top. 

    let mut img = image.clone();
    
    for x in 0..512 {
        for y in 0..16 {
            let mut pixel = img.get_pixel(x, y);
            pixel.0[3] = 0;
            img.put_pixel(x, y, pixel);
        }
    }
    img
}

pub async fn fetch_radar() -> Result<DynamicImage, String> {
    info!("Fetching radar...");
    let now = Instant::now();

    let file = File::open("sensitive/bom.json").expect("Unable to open bom.json");
    let json: Bom = from_reader(file).expect("Unable to parse bom.json");
    let station = json.station.clone();

    if let Ok(radar_id) = get_radar_id(station.clone()).await {
        if let Ok(image1) = get_image(format!("https://reg.bom.gov.au{}", radar_id)).await {
            let image1 = hide_banner(&image1);

            if let Ok(mut image2) = get_image(format!("https://reg.bom.gov.au/products/radar_transparencies/{station}.background.png")).await {
                imageops::overlay(&mut image2, &image1, 0, 0);
                info!("Radar took {:.2?}", now.elapsed());
                return Ok(image2);
            } else {
                warn!("Could not load background image for station {}", station);
                return Err(format!("Could not load background image for station {}", station));
            }
        } else {
            warn!("Could not load rain data for radar ID {}", radar_id);
            return Err(format!("Could not load rain data for radar ID {}", radar_id));
        }
    } else {
        warn!("Could not get radar ID for station {}", station);
        return Err(format!("Could not get radar ID for station {}", station));
    }
}

