use crate::structures::beatmap::Beatmap;
use lazy_static::lazy_static;
use reqwest::Client;
use std::collections::HashMap;

lazy_static::lazy_static! {
    // id: Beatmap
    pub static ref BMAPCACHE: HashMap<u32, Beatmap> = HashMap::new();
    pub static ref HttpClient: Client = Client::new();
}
