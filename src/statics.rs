use crate::structures::beatmap::Beatmap;
use reqwest::Client;
use std::collections::HashMap;

lazy_static::lazy_static! {
    // id: Beatmap
    pub static ref BEATMAP_CACHE: HashMap<u32, Beatmap> = HashMap::new();
    pub static ref HTTPCLIENT: Client = Client::new();
}
