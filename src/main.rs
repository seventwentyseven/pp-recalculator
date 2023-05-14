mod config;
mod structures;

use akatsuki_pp::{Beatmap as AkatBmap, BeatmapExt};
use config::CONFIG;
use console::Emoji;
use indicatif::{HumanDuration, ProgressBar, ProgressStyle};
use sqlx::{mysql::MySqlPoolOptions, pool, Executor, MySql, MySqlPool, Pool, Row};
use std::{collections::HashMap, thread, time::Duration};
use tokio::time::Instant;

use crate::structures::beatmap::Beatmap;

lazy_static::lazy_static! {
    // id: Beatmap
    pub static ref BMAPCACHE: HashMap<u32, Beatmap> = HashMap::new();
}

static BEATMAP: Emoji<'_, '_> = Emoji("🎵 ", "");
static USER: Emoji<'_, '_> = Emoji("👤 ", "");
static SCORE: Emoji<'_, '_> = Emoji("🏆 ", "");
static STATS: Emoji<'_, '_> = Emoji("📊 ", "");
static DONE: Emoji<'_, '_> = Emoji("✅ ", "");
static FAILED: Emoji<'_, '_> = Emoji("❌ ", "");
static CONNECTING: Emoji<'_, '_> = Emoji("🔌 ", "");
static FETCHING: Emoji<'_, '_> = Emoji("📥 ", "");

async fn init_sql() -> Result<MySqlPool, sqlx::Error> {
    let pb = ProgressBar::new(1);
    pb.set_style(ProgressStyle::with_template("{prefix:.bold.dim} {wide_msg}").unwrap());
    pb.set_prefix("[1/5]");
    pb.set_message(format!("{}Connecting to database...", CONNECTING));
    let pool_options = MySqlPoolOptions::new().max_connections(10);

    // Construct the MySQL connection URL using configuration values
    let connection_url = format!(
        "mysql://{}:{}@{}:{}/{}",
        CONFIG.MySQL.username,
        CONFIG.MySQL.password,
        CONFIG.MySQL.host,
        CONFIG.MySQL.port,
        CONFIG.MySQL.database
    );

    // Connect to the MySQL database using the connection URL
    let pool_result = pool_options.connect(connection_url.as_str()).await;

    // Handle the connection result
    let pool = match pool_result {
        Ok(pool) => pool,
        Err(error) => {
            pb.finish_with_message(format!("{}Failed to connect to the database", FAILED));
            panic!("Failed to connect to the database: {}", error)
        }
    };

    pb.finish_with_message(format!("{}Database Connected!", DONE));
    // delete pb
    drop(pb);

    Ok(pool)
}

async fn relac_maps(pool: &Pool<MySql>, spinner: &ProgressStyle) {
    let start = Instant::now();
    let maps = Beatmap::fetch_all(&pool).await;
    let pb = ProgressBar::new(maps.len() as u64);
    let style = ProgressStyle::default_bar()
        .progress_chars("█▓▒░ ")
        .template(
        "{prefix:.bold.dim} 🎵 Recalculating Beatmaps | {bar:40.white} | {pos}/{len} | ETA: {eta}",
    );
    pb.set_style(style.unwrap());
    pb.set_prefix("[2/5]");
    pb.set_message(format!("{}Recalculating Beatmaps...", BEATMAP));

    // Recalc code
    for map in maps {
        println!("{}", map.id);
        // Try to set map.bmap_obj with AkatBmap::from_path(&path + map.id);
        // Construct path to the beatmap
        let map_path = format!("{}/osu/{}.osu", &CONFIG.Main.path, map.id);
        let mut bmap_obj = match AkatBmap::from_path(&map_path) {
            Ok(bmap) => bmap,
            Err(error) => {
                // TODO: Implement force fetching from osu then kitsu, if none skip and leave log
                // pb.println(format!(
                //     "{}Failed to fetch beatmap with id {}",
                //     FAILED, map.id
                // ));
                println!("{}Failed to fetch beatmap with id {}", FAILED, map.id);
                continue;
            }
        };

        pb.inc(1);
    }

    // Continue with the rest of pb code
    pb.set_style(ProgressStyle::with_template("{prefix:.bold.dim} {wide_msg}").unwrap());
    pb.set_prefix("[2/5]");
    pb.finish_with_message(format!(
        "{}Beatmap recalculation done in {}",
        DONE,
        HumanDuration(start.elapsed())
    ));
}

#[tokio::main]
async fn main() {
    // NOTE: That ref passing to unrelated functions is dumb as fuck, change it in future
    // TODO: Change exception message style to match rest of the code
    let started = Instant::now();
    let spinner_style = ProgressStyle::with_template("{prefix:.bold.dim} {spinner} {wide_msg}")
        .unwrap()
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");

    // Connect to the database
    let pool = init_sql().await.unwrap();

    // Recalc maps
    // ! Turned off until we find reliable fix for missing .osu files that won't spam someone's API
    // relac_maps(&pool, &spinner_style).await;
}
