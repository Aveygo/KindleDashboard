// RUSTFLAGS="-C target-feature=+crt-static" cross build --target arm-unknown-linux-musleabi --release

mod calendar;
mod weather;
mod news;
mod stats;
mod radar;
mod renderer;

mod utils;

use chrono::Timelike;
use env_logger;
use log;
use std::{env, panic::AssertUnwindSafe};
use futures::FutureExt;

use log::info;

fn get_duration_until_next_interval() -> u64 {
    let now = chrono::Local::now();
    let minutes = now.minute();
    let seconds = now.second();
    let next_interval_minutes = 15 - (minutes % 15);
    (next_interval_minutes * 60 - seconds) as u64
}

async fn panic_wrapper() -> Result<(), String> {
    /*
    
        The only time a panic should happen is if we cannot allocate memory, write to disk, or create a valid svg.
        This SHOULD require user attention, as most likely the kindle has run out of space.

        In the case that eips can not even be used to show the panic message, only then does
        the code "promote" the panic and stop the program permanently.
     */

    let may_panic = async {
        utils::check_internet().unwrap();
        renderer::render_png().await
    };

    let panic_result = AssertUnwindSafe(may_panic).catch_unwind().await;

    match panic_result {
        Ok(_r) => {return Ok(())},
        Err(e) => {

            let panic_message;
            if let Some(s) = e.downcast_ref::<&str>() {
                panic_message = s.to_string();
            } else if let Some(s) = e.downcast_ref::<String>() {
                panic_message = s.to_string();
            } else {
                panic_message = "Panic occurred but could not be downcast to a string".to_string();
            }
            
            // Minimal render to show panic message incase it an svg based fail  
            let r = renderer::show_panic(panic_message.clone()).await;

            match r {
                // We showed the panic message successfully, but we still panicked...
                Ok(_r) => Err(panic_message), 

                // We cant even show the panic message. Now we REALLY panic. 
                Err(e) => panic!("Could not show panic message: \"{panic_message}\", due to: {e}")
            }
        }
    }
}

#[tokio::main]
async fn main() {
    if env::var("RUST_LOG").is_err() {env::set_var("RUST_LOG", "info")}
    env_logger::init();

    if env::var("NOT_KINDLE").is_err() {
        utils::check_xrandr().unwrap();
        utils::check_eips().unwrap();
        utils::check_sensitives().unwrap();
    }

    panic_wrapper().await.ok();

    loop {
        let wait = get_duration_until_next_interval();
        info!("Sleeping for {wait} seconds...");
        tokio::time::sleep(std::time::Duration::from_secs(wait)).await;
        panic_wrapper().await.ok();
    }
}