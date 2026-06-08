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
