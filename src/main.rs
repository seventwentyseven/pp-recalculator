mod config;
mod statics;
mod structures;

use akatsuki_pp::{Beatmap as AkatBmap, BeatmapExt};
use config::CONFIG;
use console::Emoji;
use indicatif::{HumanDuration, ProgressBar, ProgressStyle};
use sqlx::{mysql::MySqlPoolOptions, MySql, MySqlPool, Pool};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::{spawn, time::Instant};

use crate::statics::HttpClient;
use crate::structures::beatmap::Beatmap;

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
    let mut missing: Vec<u32> = Vec::new();
    for map in maps {
        // Try to set map.bmap_obj with AkatBmap::from_path(&path + map.id);
        // Construct path to the beatmap
        pb.inc(1);
        let map_path = format!("{}/osu/{}.osu", &CONFIG.Main.path, map.id);
        // let mut bmap_obj = match AkatBmap::from_path(&map_path) {
        //     Ok(bmap) => bmap,
        //     Err(error) => {
        //         // TODO: Implement force fetching from osu then kitsu, if none skip and leave log
        //         // pb.println(format!(
        //         //     "{}Failed to fetch beatmap with id {}",
        //         //     FAILED, map.id
        //         // ));

        //         missing += 1;
        //         continue;
        //     }
        // };
        // Check if file exists @ map_path
        if !std::path::Path::new(&map_path).exists() {
            missing.push(map.id);
            continue;
        }
    }
    println!("Missing: {}", missing.len());
    pb.reset();
    pb.set_prefix("[2/5]");
    pb.set_message(format!("{}Downloading Missing Maps...", FETCHING));
    pb.set_length(missing.len() as u64);

    let mut tasks = Vec::new();

    for map in missing {
        pb.inc(1);

        // Spawn a new Tokio task to download the map in a separate thread
        let task = spawn(async move {
            // Get el map
            match Beatmap::get_osu_file(map).await {
                Ok(bmap) => bmap,
                // Err do nothing
                Err(_) => return,
            };
        });

        tasks.push(task);

        // Limit the number of active tasks to 4
        if tasks.len() >= 32 {
            // Await the completion of one of the tasks
            let _ = tasks.pop().unwrap().await;
        }
    }
}

async fn get_missing(pool: &Pool<MySql>) {
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
    let mut missing: Vec<u32> = Vec::new();
    for map in maps {
        // Try to set map.bmap_obj with AkatBmap::from_path(&path + map.id);
        // Construct path to the beatmap
        pb.inc(1);
        let map_path = format!("{}/osu/{}.osu", &CONFIG.Main.path, map.id);

        // Check if file exists @ map_path
        if !std::path::Path::new(&map_path).exists() {
            missing.push(map.id);
            continue;
        }
    }
    println!("Missing: {}", missing.len());
    pb.reset();
    pb.set_prefix("[2/5]");
    pb.set_message(format!("{}Downloading Missing Maps...", FETCHING));
    pb.set_length(missing.len() as u64);

    let mut tasks = Vec::new();

    for map in missing {
        pb.inc(1);

        // Spawn a new Tokio task to download the map in a separate thread
        let task = spawn(async move {
            // Get el map
            let response = HttpClient
                .get(format!("https://catboy.best/osu/{}", map))
                .send()
                .await
                .unwrap();

            // println!("{:?}", response);

            // If failed to get the file or the response is 404, return an empty vector
            if response.status().is_success() {
                // Since the response is text/plain, just return the string (ignore the error)
                let text = response.text().await.unwrap();

                // Save the file to disk
                let file_path = format!("/opt/pp-recalculator/testfiles/downloads/{}.osu", map);
                let mut file = File::create(file_path).await?;
                file.write_all(text.as_bytes()).await?;
                Ok(())
            } else {
                // There was an error or the response was 404
                Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Failed to fetch map file",
                )))
            }
        });

        tasks.push(task);

        // Limit the number of active tasks to 4
        if tasks.len() >= 32 {
            // Await the completion of one of the tasks
            let _ = tasks.pop().unwrap().await;
        }
    }

    // Await the completion of the remaining tasks
    for task in tasks {
        let _ = task.await;
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

    // Connect to the database
    let pool = init_sql().await.unwrap();

    // Recalc maps
    // ! Turned off until we find reliable fix for missing .osu files that won't spam someone's API
    get_missing(&pool).await;

    // let res = Beatmap::get_osu_file(75 as u32).await;
    // println!("{:?}", res)
}
