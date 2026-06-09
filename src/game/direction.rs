#[derive(
    Clone,
    Eq,
    Hash,
    PartialEq,
    serde::Deserialize,
    serde::Serialize,
)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    East,
    North,
    South,
    West,
}

impl Direction {
    pub fn from_str(s: &str) -> Result<Self, std::io::Error> {
        match s {
            "east" => Ok(Self::East),
            "north" => Ok(Self::North),
            "south" => Ok(Self::South),
            "west" => Ok(Self::West),
            _ => Err(std::io::Error::other(format!("unknown direction '{s}'"))),
        }
    }
}
