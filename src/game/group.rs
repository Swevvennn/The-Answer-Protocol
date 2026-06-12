#[derive(Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Group {
    pub name: String,
    pub players: std::collections::HashSet<String>,
    pub invited: std::collections::HashSet<String>,
}

impl Group {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            players: std::collections::HashSet::new(),
            invited: std::collections::HashSet::new(),
        }
    }
}
