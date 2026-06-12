#[derive(
    Default,
    serde::Deserialize,
    serde::Serialize,
)]
#[serde(deny_unknown_fields)]
pub struct World {
    pub start: String,
    pub rooms: Vec<crate::game::Room>,
}

impl World {
    pub fn new(path: &str) -> Result<Self, std::io::Error> {
        Ok(
            serde_json::from_str(
                &std::fs::read_to_string(path)?
            )?
        )
    }
}
