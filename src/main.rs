mod config;
mod statics;
mod structures;

use std::sync::Arc;
use std::time::Instant;

use crate::structures::beatmap::Beatmap;
use akatsuki_pp::{Beatmap as AkatBmap, BeatmapExt};
use config::CONFIG;
use console::Emoji;
use futures::stream::{FuturesUnordered, StreamExt};
use indicatif::{HumanDuration, ProgressBar, ProgressStyle};
use sqlx::{mysql::MySqlPoolOptions, MySql, MySqlPool, Pool};
use statics::BEATMAP_CACHE;
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
    if missing.len() == 0 {
        println!("No missing maps found!");
        return;
    }

    let pb = ProgressBar::new(missing.len() as u64);
    let style = ProgressStyle::default_bar()
        .progress_chars("█▓▒░ ")
        .template(
        "{prefix:.bold.dim} 🎵 Downloading Missing Maps | {bar:40.white} | {pos}/{len} | ETA: {eta}",
    );
    pb.set_style(style.unwrap());
    pb.set_prefix("[2/8]");

    // Concurrent downloads to speed up the process
    let concurrency_limit = 8;
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

async fn recalc_maps(pool: &Pool<MySql>) {
    let start = Instant::now();
    let maps = Beatmap::fetch_all(&pool).await;
    let pb = ProgressBar::new(maps.len() as u64);
    let style = ProgressStyle::default_bar()
        .progress_chars("█▓▒░ ")
        .template(
        "{prefix:.bold.dim} 🎵 Preforming Beatmap Recalculation | {bar:40.white} | {pos}/{len} | ETA: {eta}",
    );
    pb.set_style(style.unwrap());
    pb.set_prefix("[3/8]");

    // Recalc code
    let concurrency_limit = 24;
    let sem = Arc::new(Semaphore::new(concurrency_limit));
    let mut futures = FuturesUnordered::new();
    for mut map in maps {
        let sem_clone = Arc::clone(&sem);
        let fut = async move {
            let _permit = sem_clone.acquire().await.unwrap();

            let mut bmap_obj = match AkatBmap::from_path(
                format!("{}/osu/{}.osu", &CONFIG.Main.path, map.id).as_str(),
            ) {
                Ok(bmap) => bmap,
                Err(_) => {
                    //TODO: Try to get that missing map instead of skipping
                    return;
                }
            };

            // Calculate new stars, round to 3 decimal places
            let stars_new = (bmap_obj.stars().calculate().stars() * 1000.0).round() / 1000.0;

            // Execute query
            let query = format!("UPDATE maps SET diff = {} WHERE id = {}", stars_new, map.id);
            match sqlx::query(&query).execute(pool).await {
                Ok(_) => {}
                Err(err) => {
                    println!("Failed to update map with id {}, {}", map.id, err);
                    return;
                }
            };
        };
        // Cache map in static cache
        // map.diff = stars_new as f32; // is it safe?
        // map.bmap_obj = Some(bmap_obj);
        // BEATMAP_CACHE.insert(map.id.clone(), map.clone());

        futures.push(fut);

        // Cache map in static cache
    }

    while let Some(_) = futures.next().await {
        pb.inc(1);
    }

    println!("Done in {}", HumanDuration(start.elapsed()));
}

#[tokio::main]
async fn main() {
    // Connect to the database
    let pool = init_sql().await.unwrap();

    // Recalc Maps
    download_missing_maps(&pool).await;

    // Relcalc Maps
    recalc_maps(&pool).await;
}
