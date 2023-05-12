use sqlx::{Executor, MySql, Pool, Row};

#[derive(Debug, Clone)]
pub struct Beatmap {
    // Meta
    pub set_id: i32,
    pub id: i32,
    pub md5: String,
    pub status: i32,
    pub frozen: bool,

    // Attributes
    pub total_length: i32,
    pub max_combo: i32,
    pub mode: i32,
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
        // Using pool to fetch all maps
        let start = std::time::Instant::now();
        let rows = pool
            .fetch_all("SELECT set_id, id, md5, status, frozen, total_length, max_combo, mode, bpm, ar, od, cs, hp, diff FROM maps")
            .await
            .unwrap()
            .into_iter();
        println!("Query took {:?}", std::time::Instant::now() - start);

        let maps = rows
            .map(|row| Beatmap {
                set_id: row.get("set_id"),
                id: row.get("id"),
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
}
