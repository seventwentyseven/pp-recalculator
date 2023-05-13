use lazy_static::lazy_static;
use serde::Deserialize;
use std::fs::File;
use std::io::prelude::*;

#[derive(Deserialize)]
pub struct MySQL {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
pub struct Main {
    pub path: String,
    pub ppSystem: i8,
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
pub struct Config {
    pub MySQL: MySQL,
    pub Main: Main,
}

// Read yaml config file
fn init_config() -> Config {
    // Open the file in read-only mode (ignoring errors).
    let file = File::open("/opt/pp-recalculator/config.yml");

    // Read the file contents into a string, returns `io::Result<usize>`
    let mut contents = String::new();
    match file {
        Ok(mut f) => f.read_to_string(&mut contents).unwrap(),
        Err(e) => panic!("Error reading config file: {}", e),
    };

    // Parse the string of data into serde_yaml::Value.
    let config: Config = serde_yaml::from_str(&contents).unwrap();

    return config;
}

lazy_static! {
    pub static ref CONFIG: Config = init_config();
}
