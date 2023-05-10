mod config;
mod structures;

use akatsuki_pp::{Beatmap, BeatmapExt, Mods};
use config::CONFIG;
use sqlx::{
    mysql::{MySqlPool, MySqlPoolOptions},
    Executor,
};
use std::collections::HashMap;
use structures::user::User;
use tokio::time::Instant;

lazy_static::lazy_static! {
    pub static ref BMAPCACHE: HashMap<u32, Beatmap> = HashMap::new();
}

async fn init_sql() -> Result<MySqlPool, sqlx::Error> {
    let pool = MySqlPoolOptions::new()
        .max_connections(10)
        .connect(
            &format!(
                "mysql://{}:{}@{}:{}/{}",
                CONFIG.MySQL.username,
                CONFIG.MySQL.password,
                CONFIG.MySQL.host,
                CONFIG.MySQL.port,
                CONFIG.MySQL.database
            )
            .as_str(),
        )
        .await?;

    Ok(pool)
}

#[tokio::main]
async fn main() {
    // Acquire database connection
    // Create database url from config
    let pool = init_sql().await.expect("Failed to connect to database");

    // Fetch all users from the database
    let start = Instant::now();
    let users = User::fetch_all(&pool).await;
    let end = Instant::now();
    println!("Fetched {} users in {:?}", users.len(), end - start);
}

// fn recalculate(score: &structures::score::Score, bmap: &Beatmap) {
//     if (score.mode < 4) {} else if (score.mode in 4..7) {} else if (score.mode == 8) {}

//     }
// }
