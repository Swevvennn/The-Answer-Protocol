#[derive(
    serde::Deserialize,
    serde::Serialize,
)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub enum QuestKind {
    Bring {
        item: String,
        count: u32,
    },
    Goto {
        room: String,
    },
    Kill {
        enemy: String,
        count: u32,
    },
    Talk {
        npc: String,
    }
}

#[derive(
    serde::Deserialize,
    serde::Serialize,
)]
#[serde(deny_unknown_fields)]
pub struct Quest {
    pub id: String,
    pub name: String,
    pub description: String,

    #[serde(default)]
    pub autocomplete: bool,

    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub requirements: Vec<String>,

    pub task: QuestKind,
    pub reward: Vec<String>,
}

#[derive(
    Clone,
    serde::Deserialize,
    serde::Serialize,
)]
#[serde(rename_all = "lowercase")]
pub enum QuestStatus {
    Abandoned,
    Active,
    Completed,
}

#[derive(
    serde::Deserialize,
    serde::Serialize,
)]
pub struct QuestProgress {
    pub quest: String,
    pub status: QuestStatus,
    pub progress: u32,
}

impl QuestProgress {
    pub fn new(quest: &Quest) -> Self {
        Self {
            quest: quest.id.clone(),
            status: QuestStatus::Active,
            progress: 0,
        }
    }
}
