
use crate::calendar;
use crate::stats;
use crate::weather;
use crate::news;
use crate::radar;

use crate::calendar::CalendarEvent;
use crate::stats::Stats;
use crate::weather::DayData;

use resvg;
use tiny_skia::{Transform, PixmapMut, BYTES_PER_PIXEL};
use usvg::Tree;
use image::{ImageBuffer, Rgba, DynamicImage};

use std::process::Command;

use base64::prelude::*;
use std::io::Cursor;
use regex::Regex;

use chrono::{DateTime, Duration, Timelike, Utc};
use std::time::Instant;

use log::{info, warn};
use futures::join;

use async_std::future;
use std::time::Duration as stdDuration;

#[derive(Debug)]
struct KindleDisplayData {
    short_stats: Option<stats::Stats>,
    weather: Option<Vec<weather::DayData>>,
    news: Option<Vec<String>>,
    calendar_event: Option<Option<calendar::CalendarEvent>>,
    image: Option<DynamicImage>
}

async fn build_all_data() -> KindleDisplayData {
    info!("Fetching all data...");
    let now = Instant::now();

    let timeout = stdDuration::from_secs(30);

    let short_stats = future::timeout(timeout, stats::fetch_stats());
    let weather = future::timeout(timeout, weather::fetch_weather());
    let news = future::timeout(timeout, news::fetch_news());
    let calendar_event = future::timeout(timeout, calendar::fetch_event());
    let image = future::timeout(timeout, radar::fetch_radar());

    let (
        short_stats, 
        weather, 
        news, 
        calendar_event, 
        image
    )  = join!(short_stats, weather, news, calendar_event, image);

    
    let elapsed = format!("{:.2?}", now.elapsed());
    info!("Fetched all kindle data in {elapsed}");

    // Checking timeout messages
    let short_stats = match short_stats {Ok(r) => {r}, Err(e) => Err(format!("Timeout: {e}").into())};
    let weather = match weather {Ok(r) => {r}, Err(e) => Err(format!("Timeout: {e}").into())};
    let news = match news {Ok(r) => {r}, Err(e) => Err(format!("Timeout: {e}").into())};
    let calendar_event = match calendar_event {Ok(r) => {r}, Err(e) => Err(format!("Timeout: {e}").into())};
    let image = match image {Ok(r) => {r}, Err(e) => Err(format!("Timeout: {e}").into())};

    // Warning on error
    match &short_stats {Ok(_) => {}, Err(e) => warn!("Short stats failed: {e}")}
    match &weather {Ok(_) => {}, Err(e) => warn!("Weather failed: {e}")}
    match &news {Ok(_) => {}, Err(e) => warn!("News failed: {e}")}
    match &calendar_event {Ok(_) => {}, Err(e) => warn!("Calendar failed: {e}")}
    match &image {Ok(_) => {}, Err(e) => warn!("Radar failed: {e}")}

    KindleDisplayData {
        short_stats: short_stats.ok(),
        weather: weather.ok(),
        news: news.ok(),
        calendar_event: calendar_event.ok(),
        image: image.ok()
    }
}

async fn _build_some_data() -> KindleDisplayData {
    // Used for testing

    let news = vec![
        "Russia loses more than 70,000 soldiers in 2 months".to_string(),
        "UAE deports graduate who yelled 'Free Palestine' as he received his diploma".to_string(),
        "Move by some NATO members to let Kyiv strike Russia with their arms is a dangerous escalation, Kremlin says".to_string(),
        "'After PM Modi went back, I am being asked to go to frontline': Punjab man in Russian army".to_string(),
        "Biden: There’s a lot I wish I’d been able to convince the Israelis to do".to_string(),
        "Germany says it won't be cowed by Russia after reported plot to kill Rheinmetall CEO".to_string(),
        "Russian Missile Strike Targets Likely F-16 Airfield in Starokostyantyniv".to_string(),
        "Ukraine will likely have to wait a year before it's able to launch another counteroffensive, NATO official says".to_string(),
    ];

    let calendar_event = CalendarEvent {
        start_time: Utc::now() + Duration::days(1),
        name: "ASSESSMENT 3 (Part G) - Oral Defense (Points - 25), DUE DATE: Starting from Monday, May 27, 2024".to_string()
    };
    
    let short_stats = Stats {
        d_spx500: Some(0.0),
        d_btc: Some(0.0),
        yield_spread: Some(0.0),
        linux_share: Some(0.0),
        btc_halving: Some(Utc::now()),
        kernel_version: Some("6.10".to_string())
    };


    let weather = vec![
        DayData {
            data_points: 10,
            date: 10,
            day: "FRI".to_string(),
            rain_sum: 1.0,
            cloud_sum: 1.0,
            max_c: 10.0,
            min_c: 20.0,
        },
        DayData {
            data_points: 10,
            date: 11,
            day: "SAT".to_string(),
            rain_sum: 10.0,
            cloud_sum: 10.0,
            max_c: 10.0,
            min_c: 20.0,
        },
        DayData {
            data_points: 10,
            date: 12,
            day: "SUN".to_string(),
            rain_sum: 100.0,
            cloud_sum: 10.0,
            max_c: 10.0,
            min_c: 20.0,
        }
    ];
    
    // let image = radar::fetch_radar().await.unwrap(); // too slow for testing

    KindleDisplayData {
        short_stats: Some(short_stats),
        weather: Some(weather),
        news: Some(news),
        calendar_event: Some(Some(calendar_event)),
        image: None
    }
}

fn generate_svg_text(text: Vec<String>, max_lines:usize, max_width: f64, x:i32, y:i32, font_size:i32, line_height:f64) -> String {
    // SVGs don't have a way to automate line wrapping. Instead, we have to do it ourselves.

    let mut svg_text = String::new();
    let mut current_lines = 0;

    let line_height = font_size as f64 * line_height;
    let mut y_new = y.clone() as f64;

    for news in text {

        let lines = textwrap::wrap(&news, max_width as usize);

        // Check if we exceed max, but not on the first line (we have to show *some* info at least).
        if (current_lines + lines.len() >= max_lines) && (current_lines != 0) {
            return svg_text
        }

        // Push the current lines
        for (i, line) in lines.iter().enumerate() {
            svg_text.push_str(&format!(
                r#"<tspan x="{}" y="{}" font-family="FreeSans" font-weight="bold" font-size="{}px">"#,
                x, y_new, font_size
            ));
            svg_text.push_str(&line);
            svg_text.push_str("</tspan>");
            current_lines = current_lines + 1;
            y_new += line_height;

            /* If we are "sitting" on the end but there are more lines to go, then just show ... and return */
            if (current_lines >= max_lines - 1) && (i != line.len()-1) {
                svg_text.push_str(&format!(
                    r#"<tspan x="{}" y="{}" font-family="FreeSans" font-weight="bold" font-size="{}px">"#,
                    x, y_new, font_size
                ));
                svg_text.push_str("* * *");
                svg_text.push_str("</tspan>");
                return svg_text
            }
        }
        
        y_new += line_height;
        current_lines  = current_lines + 1;

    }

    svg_text
}

fn time_remaining(target: DateTime<Utc>) -> String {
    let now = Utc::now();
    let duration = target - now;

    if duration.num_days() >= 365 {
        let years = duration.num_days() / 365;
        format!("{} years", years)
    } else if duration.num_days() >= 30 {
        let months = duration.num_days() / 30;
        format!("{} months", months)
    } else if duration.num_days() >= 7 {
        let weeks = duration.num_days() / 7;
        format!("{} weeks", weeks)
    } else if duration.num_hours() >= 24 {
        let days = duration.num_days();
        format!("{} days", days)
    } else if duration.num_minutes() >= 60 {
        let hours = duration.num_hours();
        format!("{} hours", hours)
    } else if duration.num_seconds() >= 60 {
        let minutes = duration.num_minutes();
        format!("{} mins", minutes)
    } else {
        let seconds = duration.num_seconds();
        format!("{} secs", seconds)
    }
}

fn format_news(template: String, data: &KindleDisplayData) -> String {
    let new_template = match &data.news {
        Some(news) => {            
            template.replace("#N1", &generate_svg_text(news.clone(), 18, 35.0, 2267, 878, 120, 1.2))
        },
        None => {
            template.replace("#N1", "ERR")
        }
    };

    new_template
}

fn escape_xml(s: &str) -> String {
    s.replace("&", "&amp;")
     .replace("<", "&lt;")
     .replace(">", "&gt;")
     .replace("\"", "&quot;")
     .replace("'", "&apos;")
}

fn format_calendar(template: String, data: &KindleDisplayData) -> String {
    let mut template = template.clone();
    match &data.calendar_event {
        Some(possible_calendar_event) => {
            match possible_calendar_event {
                Some(calendar_event) => {
                    let name = escape_xml(&calendar_event.name);
                    let time = calendar_event.start_time.clone();
                    let remaining = escape_xml(&time_remaining(time));

                    template = template.replace("#G2", &format!("in {remaining}"));
                    template = template.replace("#G1", &generate_svg_text(vec![name], 5, 33.0, 3080, 180, 100, 1.2))
                }
                None => {
                    template = template.replace("#G2", "");
                    template = template.replace("#G1", "No upcoming events");
                }
            }
        },
        None => {
            template = template.replace("#G2", "ERR!");
            template = template.replace("#G1", "Could not fetch any events");
        }
    };

    template
}

fn format_stats(template: String, data: &KindleDisplayData) -> String {
    let mut template = template.clone();
    match &data.short_stats {
        Some(short_stats) => {

            template = template.replace("#I1", &match short_stats.d_spx500 {
                Some(v) => format!("{:.2}%", v),
                None => "NA".to_string()
            });

            template = template.replace("#I2", &match short_stats.yield_spread {
                Some(v) => format!("{:.2}", v),
                None => "NA".to_string()
            });

            template = template.replace("#I3", &match short_stats.d_btc {
                Some(v) => format!("{:.2}%", v),
                None => "NA".to_string()
            });

            template = template.replace("#I4", &match short_stats.linux_share {
                Some(v) => format!("{:.2}%", v),
                None => "NA".to_string()
            });

            template = template.replace("#I5", &match short_stats.btc_halving {
                Some(v) => time_remaining(v),
                None => "NA".to_string()
            });

            template = template.replace("#I6", &match short_stats.kernel_version.clone() {
                Some(v) => v,
                None => "NA".to_string()
            });
            
        },
        None => {
            template = template.replace("#I1", "ERR");
            template = template.replace("#I2", "ERR");
            template = template.replace("#I3", "ERR");
            template = template.replace("#I4", "ERR");
            template = template.replace("#I5", "ERR");
            template = template.replace("#I6", "ERR");
        }
    };

    return template;
}

fn format_time(template: String, _data: &KindleDisplayData) -> String {
    // We assume that making the primary requests take less than a minute to create the nice "every 15 minute" effect.

    let mut template = template.clone();
    
    let now = chrono::offset::Utc::now() + Duration::hours(10);
    let hour = now.hour();
    let minute = now.minute();

    template = template.replace("#1", &format!("{:0>2}", hour));
    template = template.replace("#2", &format!("{:0>2}", minute));
    return template;
}

fn weather_to_icon(day: &DayData) -> String {
    let icon1 = include_str!("icons/1.svg").to_string();
    let icon2 = include_str!("icons/2.svg").to_string();
    let icon3 = include_str!("icons/3.svg").to_string();
    let icon4 = include_str!("icons/4.svg").to_string();
    let icon5 = include_str!("icons/5.svg").to_string();
    let icon6 = include_str!("icons/6.svg").to_string();
    let icon7 = include_str!("icons/7.svg").to_string();
    let icon8 = include_str!("icons/8.svg").to_string();

    let avg_rain = day.rain_sum / day.data_points as f64;
    let avg_cloud = day.cloud_sum / day.data_points as f64;    

    let mut result = icon1;

    if avg_cloud > 20.0 {result = icon2} 
    if avg_cloud > 50.0 {result = icon3}
    if avg_cloud > 80.0 {result = icon4}

    if avg_rain > 0.1 {result = icon5}
    if avg_rain > 0.5 {result = icon6} 
    if avg_rain > 1.0 {result = icon7} 
    if avg_rain > 5.0 {result = icon8}

    result

}

fn format_weather(template: String, data: &KindleDisplayData) -> String {
    let mut template = template.clone();

    match &data.weather {
        Some(weather) => {
            
            // Trust me, I'm not happy with this code either

            template = match weather.get(0) {
                Some(day) => {
                    template = template.replace("#D1", &format!("{:0>2} {}", day.date, day.day));
                    template = template.replace("#T1", &format!("{:.1}", day.max_c));
                    template = template.replace("#T2", &format!("{:.1}", day.min_c));
                    template = template.replace("<path\n         id=\"WEATHERICON1\" />", &weather_to_icon(day));
                    template
                },
                None => {
                    template = template.replace("#D1", "NA");
                    template = template.replace("#T1", "NA");
                    template = template.replace("#T2", "NA");
                    template
                }
            };

            template = match weather.get(1) {
                Some(day) => {
                    template = template.replace("#D2", &format!("{:0>2} {}", day.date, day.day));
                    template = template.replace("#T3", &format!("{:.1}", day.max_c));
                    template = template.replace("#T4", &format!("{:.1}", day.min_c));
                    template = template.replace("<path\n         id=\"WEATHERICON2\" />", &weather_to_icon(day));
                    template
                },
                None => {
                    template = template.replace("#D2", "NA");
                    template = template.replace("#T3", "NA");
                    template = template.replace("#T4", "NA");
                    template
                }
            };

            template = match weather.get(2) {
                Some(day) => {
                    template = template.replace("#D3", &format!("{:0>2} {}", day.date, day.day));
                    template = template.replace("#T5", &format!("{:.1}", day.max_c));
                    template = template.replace("#T6", &format!("{:.1}", day.min_c));
                    template = template.replace("<path\n         id=\"WEATHERICON3\" />", &weather_to_icon(day));
                    template
                },
                None => {
                    template = template.replace("#D3", "NA");
                    template = template.replace("#T5", "NA");
                    template = template.replace("#T6", "NA");
                    template
                }
            };
            
        },
        None => {
            template = template.replace("#D1", "ERR");
            template = template.replace("#D2", "ERR");
            template = template.replace("#D3", "ERR");
            template = template.replace("#T1", "ERR");
            template = template.replace("#T2", "ERR");
            template = template.replace("#T3", "ERR");
            template = template.replace("#T4", "ERR");
            template = template.replace("#T5", "ERR");
            template = template.replace("#T6", "ERR");
        }
    };

    return template;
}

fn format_radar(template: String, data: &KindleDisplayData) -> String {
    let mut template = template.clone();
    match &data.image {
        Some(image) => {
            
            let mut buffer = Cursor::new(Vec::new());
            
            let r = image.write_to(&mut buffer, image::ImageFormat::Png);
            match r {
                Ok(_r) => {
                    let encoded_image = BASE64_STANDARD.encode(buffer.get_ref());
                    template = template.replace("BASE64RADAR", &encoded_image);
                },
                Err(e) => {
                    warn!("Could not write to buffer: {e}")
                }
            }
        },
        None => {}
    };

    return template;
}

struct Screen {
    width: u32,
    height: u32
}

fn get_screen_dim() -> Option<Screen> {
    // Run xrandr to get screen data
    let output = Command::new("xrandr").output();
    
    match output {
        Ok(output) => {
            let output_str = String::from_utf8_lossy(&output.stdout);

            // regex to get the current screen size
            let re = Regex::new(r"current (\d+) x (\d+)")
                .expect("Failed to compile regex");

            if let Some(caps) = re.captures(&output_str) {
                let width = caps.get(1).map_or("", |m| m.as_str()).parse::<u32>();
                let height = caps.get(2).map_or("", |m| m.as_str()).parse::<u32>();

                match width {
                    Ok(width) => {
                        match height {
                            Ok(height) =>{
                                Some(Screen {
                                    width: width,
                                    height: height
                                })
                            },
                            Err(e) => {
                                warn!("Could not determine screen size (height) from: {output_str} due to {e}");
                                None
                            }
                        }
                    },
                    Err(e) => {
                        warn!("Could not determine screen size (width) from: {output_str} due to {e}");
                        None
                    } 
                }

            } else {
                None
            }
        },

        Err(e) => {
            warn!("Could not run xrandr to get screen size: {e}");
            None
        }
    }
}

async fn create_output_svg() -> String {
    let mut template = include_str!("template.svg").to_string();
    
    let data = build_all_data().await;

    template = format_news(template, &data);
    template = format_calendar(template, &data);
    template = format_stats(template, &data);
    template = format_time(template, &data);
    template = format_weather(template, &data);
    template = format_radar(template, &data);

    template
}

async fn render_svg(template: String) -> DynamicImage {
    let mut fontdb = usvg::fontdb::Database::new();
    fontdb.load_font_data(include_bytes!("fonts/FreeSans.ttf").to_vec());
    fontdb.load_font_data(include_bytes!("fonts/FreeSansBold.ttf").to_vec());

    let mut options = usvg::Options::default();
    options.fontdb = std::sync::Arc::new(fontdb);

    let svg_tree = Tree::from_str(&template, &options).unwrap();

    let size = svg_tree.size();
    let (width, height) = (size.width() as usize, size.height() as usize);

    let mut image:Vec<u8> = vec![0; width * height * BYTES_PER_PIXEL];    

    info!("Rendering the svg...");
    let now = Instant::now();
    resvg::render(&svg_tree, Transform::identity(), &mut PixmapMut::from_bytes(&mut image, size.width() as u32, size.height() as u32).unwrap());
    let elapsed = format!("{:.2?}", now.elapsed());
    info!("Rendering took {elapsed}");

    let image_vec = image.to_vec();
    let img_buffer: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_raw(width as u32, height as u32, image_vec).unwrap();
    let result = DynamicImage::ImageRgba8(img_buffer);
    return result;
    
}

async fn clear_screen() {
    Command::new("eips").arg("-d").arg("l=0,w=9999,h=9999").output().ok();
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    Command::new("eips").arg("-c").output().ok();
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    Command::new("eips").arg("-d").arg("l=0,w=9999,h=9999").output().ok();
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
}

pub async fn update_screen(path: String) -> Result<std::process::Output, std::io::Error> {
    clear_screen().await;
    Command::new("eips").arg("-g").arg(path).output()
}


pub async fn show_panic(panic: &String) -> Result<(), Box<dyn std::error::Error>> {
    // As minimal as possible to avoid any "dangerous" code 
    if std::env::var("NOT_KINDLE").is_err() {
        tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
        Command::new("eips").arg("-c").output().ok();
        tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
        let output = Command::new("eips").arg("2").arg("1").arg(format!("\"{panic}\"")).output();
        match output {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Could not show error: {e}").into())
        }
    } else {
        info!("Skipping showing the panic due to env NOT_KINDLE");
        Ok(())
    } 
    
}


pub fn save(mut image: DynamicImage) -> String {
    let output_path = "output.png".to_string();

    info!("Saving the rendering...");
    let now = Instant::now();
    
    let screen = get_screen_dim().unwrap_or_else(|| {
        warn!("Could not determine screen size, switching to 1200x900");
        Screen{
            width: 1200,
            height: 900
        }
    });

    image = image.resize_exact(screen.height, screen.width, image::imageops::FilterType::Lanczos3);
    image = image.rotate90();
    let result: image::GrayImage = DynamicImage::ImageRgb8(image.into()).into_luma8();
    result.save(output_path.clone()).unwrap();
    let elapsed = format!("{:.2?}", now.elapsed());
    info!("Saving took {elapsed}");
    output_path
}

pub async fn render_png() {
    let start = Instant::now();

    let template = create_output_svg().await;
    let image = render_svg(template).await;
    let output_pth = save(image.clone());
    let eips_result = update_screen(output_pth).await;

    match eips_result {
        Ok(_r) => {info!("Success! Now showing the result!")},
        Err(e) => warn!("Could not show result! Is eips available? {e}") // Mainly for testing
    }

    let elapsed = format!("{:.2?}", start.elapsed());
    info!("Finished in {elapsed}");

}
