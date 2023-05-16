use crate::statics::HTTPCLIENT;
use sqlx::{Executor, MySql, Pool, Row};
use tokio::{
    fs::{File, OpenOptions},
    io::AsyncWriteExt,
};

#[derive(Debug, Clone)]
pub struct Beatmap {
    // Meta
    pub set_id: u32,
    pub id: u32,
    pub md5: String,
    pub status: u32,
    pub frozen: bool,

    // Attributes
    pub total_length: u32,
    pub max_combo: u32,
    pub mode: u8,
    pub bpm: f32,
    pub ar: f32,
    pub od: f32,
    pub cs: f32,
    pub hp: f32,
    pub diff: f32,

    pub bmap_obj: Option<akatsuki_pp::Beatmap>,
}

impl Beatmap {
    pub async fn fetch_all(pool: &Pool<MySql>) -> Vec<Beatmap> {
        let rows = pool
            .fetch_all("SELECT id, set_id, md5, status, frozen, total_length, max_combo, mode, bpm, ar, od, cs, hp, diff FROM maps")
            .await
            .unwrap()
            .into_iter();

        let maps = rows
            .map(|row| Beatmap {
                id: row.get("id"),
                set_id: row.get("set_id"),
                md5: row.get("md5"),
                status: row.get("status"),
                frozen: row.get("frozen"),
                total_length: row.get("total_length"),
                max_combo: row.get("max_combo"),
                mode: row.get("mode"),
                bpm: row.get("bpm"),
                ar: row.get("ar"),
                od: row.get("od"),
                cs: row.get("cs"),
                hp: row.get("hp"),
                diff: row.get("diff"),
                bmap_obj: None,
            })
            .collect();

        return maps;
    }

    pub async fn get_osu_file(map_id: u32) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: Add kitsu.moe mirror and if it fails, use osu api

        let response = HTTPCLIENT
            .get(format!("https://catboy.best/osu/{}", map_id))
            .send()
            .await?;

        // println!("{:?}", response);

        // If failed to get the file or the response is 404, return an empty vector
        if response.status().is_success() {
            // Since the response is text/plain, just return the string (ignore the error)
            let text = response.text().await.unwrap();

            // Save the file to disk
            let file_path = format!("/opt/gug/.data/osu/{}.osu", map_id);
            let mut file = File::create(file_path).await?;
            file.write_all(text.as_bytes()).await?;
            Ok(())
        } else {
            // Save log to /opt/err.log
            // TODO: Allow changing the log path in config
            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open("/opt/err.log")
                .await?;
            file.write_all(
                format!(
                    "Failed to fetch map file for map ID ;{};, Error: {}\n",
                    map_id,
                    response.text().await.unwrap()
                )
                .as_bytes(),
            )
            .await?;

            // Return error
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Failed to fetch map file",
            )))
        }
    }
}
