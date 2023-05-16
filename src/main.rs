mod config;
mod statics;
mod structures;

use std::println;
use std::sync::Arc;
use std::time::Instant;

use crate::structures::beatmap::Beatmap;
use akatsuki_pp::{Beatmap as AkatBmap, BeatmapExt};
use config::CONFIG;
use console::Emoji;
use futures::stream::{FuturesUnordered, StreamExt};
use indicatif::{HumanDuration, ProgressBar, ProgressStyle};
use sqlx::{mysql::MySqlPoolOptions, MySql, MySqlPool, Pool};
use tokio::sync::Semaphore;

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

async fn relac_maps(pool: &Pool<MySql>) {
    let start = Instant::now();
    let maps = Beatmap::fetch_all(&pool).await;
    let pb = ProgressBar::new(maps.len() as u64);
    let style = ProgressStyle::default_bar()
        .progress_chars("█▓▒░ ")
        .template(
        "{prefix:.bold.dim} 🎵 Getting Missing Maps | {bar:40.white} | {pos}/{len} | ETA: {eta}",
    );
    pb.set_style(style.unwrap());
    pb.set_prefix("[2/5]");
    pb.set_message(format!("{}Getting missing maps...", BEATMAP));

    // Recalc code
    for mut map in maps {
        // Try to set map.bmap_obj with AkatBmap::from_path(&path + map.id);
        // Construct path to the beatmap
        pb.inc(1);
        let bmap_obj =
            match AkatBmap::from_path(format!("{}/osu/{}.osu", &CONFIG.Main.path, map.id).as_str())
            {
                Ok(bmap) => bmap,
                Err(_) => {
                    continue;
                }
            };

        // Set map.bmap_obj
        map.bmap_obj = Some(bmap_obj);

        // Check if file exists @ map_path
    }
    println!("Done in {}", HumanDuration(start.elapsed()));
}

async fn download_missing_maps(pool: &Pool<MySql>) {
    // Get maps from db
    let maps: Vec<Beatmap> = Beatmap::fetch_all(&pool).await;
    let mut missing: Vec<u32> = Vec::new();

    for map in maps {
        // Check if file exists @ map_path
        if !std::path::Path::new(&format!("{}/osu/{}.osu", &CONFIG.Main.path, map.id)).exists() {
            missing.push(map.id);
            continue;
        }
    }

    let pb = ProgressBar::new(missing.len() as u64);
    let style = ProgressStyle::default_bar()
        .progress_chars("█▓▒░ ")
        .template(
        "{prefix:.bold.dim} 🎵 Downloading Missing Maps| {bar:40.white} | {pos}/{len} | ETA: {eta}",
    );
    pb.set_style(style.unwrap());
    pb.set_prefix("[2/5]");

    // Concurrent downloads to speed up the process
    let concurrency_limit = 4;
    let sem = Arc::new(Semaphore::new(concurrency_limit));
    let mut futures = FuturesUnordered::new();

    for id in missing {
        let sem_clone = Arc::clone(&sem);
        let fut = async move {
            let _permit = sem_clone.acquire().await.unwrap();
            match Beatmap::get_osu_file(id).await {
                Ok(_) => {}
                Err(err) => {
                    println!("Failed to download map with id {}, {}", id, err)
                }
            };
            // tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        };
        futures.push(fut);
    }

    while let Some(_) = futures.next().await {
        pb.inc(1);
    }

    pb.finish();
}

#[tokio::main]
async fn main() {
    // NOTE: That ref passing to unrelated functions is dumb as fuck, change it in future
    // TODO: Change exception message style to match rest of the code
    let started = std::time::Instant::now();

    // Connect to the database
    let pool = init_sql().await.unwrap();

    // Recalc Maps
    download_missing_maps(&pool).await;

    // Relcalc Maps

    // let res = Beatmap::get_osu_file(75 as u32).await;
    // println!("{:?}", res)
}
