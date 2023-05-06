pub struct Beatmap {
    pub id: u32,
    pub set_id: u32,
    pub md5: String,
    pub od: f32,
    pub ar: f32,
    pub cs: f32,
    pub hp: f32,
    pub bpm: f32,
    pub diff: f32,
    pub mode: u8,
    pub status: u8,
    pub max_combo: u32,
    pub total_length: u32,
    pub diffname: String,
}

pub struct BeatmapSet {
    pub set_id: u32,
    pub artist: String,
    pub title: String,
    pub creator: String,
    pub status: u8,
    pub maps: Vec<Beatmap>,
}

pub struct BeatmapCache {
    pub beatmaps: Vec<Beatmap>,
    pub beatmap_sets: Vec<BeatmapSet>,
}
