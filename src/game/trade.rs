#[derive(
    serde::Deserialize,
    serde::Serialize,
)]
#[serde(deny_unknown_fields)]
pub struct Trade {
    pub require: Vec<String>,
    pub give: Vec<String>,
}
