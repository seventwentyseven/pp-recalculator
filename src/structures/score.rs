#[derive(Debug, Clone)]
pub struct Score {
    // Score metadata
    pub id: u32,
    pub map_id: u32,
    pub set_id: u32,
    pub map_md5: String,
    pub status: u8,
    pub mode: u8,
    // Score stats
    pub pp: f32,
    pub acc: f32,
    pub mods: u32,
    pub max_combo: u16,
    pub score: i32,
    pub n300: u16,
    pub n100: u16,
    pub n50: u16,
    pub n_misses: u16,
    pub n_katu: u16,
    pub n_geki: u16,
    pub perfect: bool,
    pub grade: String,
}
