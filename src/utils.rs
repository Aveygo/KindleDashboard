
use std::process::Command;
use log::info;
use std::path::Path;
use reqwest::blocking::get;

pub fn check_xrandr() -> Result<(), String> {
    let output = Command::new("xrandr").output();

    match output {
        Ok(_r) => {
            info!("Found xrandr!");
            Ok(())
        },
        Err(e) => {
            Err(format!("Could not find xrandr: {e}"))
        }
    }
}

pub fn check_eips() -> Result<(), String> {
    // eips MUST have at least one argument or it "fails"
    let output = Command::new("eips").arg("-c").output();

    match output {
        Ok(_r) => {
            info!("Found eips!");
            Ok(())
        },
        Err(e) => {
            Err(format!("Could not find eips: {e}"))
        }
    }
}

pub fn check_internet() -> Result<(), String> {
    match get("http://www.google.com") {
        Ok(_) => Ok(()),
        Err(_) => Err("Could not connect to the internet".to_string()),
    }
}

pub fn check_sensitives() -> Result<(), String> {
    let calendar = Path::new("sensitive/creds.json").exists();
    let weather = Path::new("sensitive/openweatherkey.json").exists();
    let bom = Path::new("sensitive/bom.json").exists();

    if calendar {
        if weather {
            if bom {
                Ok(())
            } else {
                Err("No sensitive/bom.json".to_string())
            }
        } else {
            Err("No sensitive/openweatherkey.json".to_string())
        }
    } else {
        Err("No sensitive/creds.json".to_string())
    }
}