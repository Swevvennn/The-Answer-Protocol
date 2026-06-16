use rand::seq::IndexedRandom;

#[derive(
    serde::Deserialize,
    serde::Serialize,
)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub enum NPCKind {
    Enemy {
        hp: u32,
    },
    Neutral {
        #[serde(default)]
        #[serde(skip_serializing_if = "Vec::is_empty")]
        dialogues: Vec<String>,

        #[serde(default)]
        #[serde(skip_serializing_if = "Vec::is_empty")]
        quests: Vec<String>,

        #[serde(default)]
        #[serde(skip_serializing_if = "Vec::is_empty")]
        trades: Vec<crate::game::Trade>,
    },
}

#[derive(
    serde::Deserialize,
    serde::Serialize,
)]
#[serde(deny_unknown_fields)]
pub struct NPC {
    pub id: String,
    pub name: String,
    pub data: NPCKind,
}

impl NPC {
    pub fn talk(game: &crate::game::GameState, npc: &String) -> crate::messages::Message {
        if let Some(npc) = game.npcs.get(npc) {
            if let NPCKind::Neutral {
                dialogues,
                quests: _,
                trades: _,
            } = &npc.data {
                let mut dialogue =  "...".to_string();
                let mut rng = rand::rng();
                if let Some(choosed) = dialogues.choose(&mut rng) {
                    dialogue = choosed.clone();
                }
                crate::messages::Message::Response(crate::messages::Response {
                    payload: crate::messages::Payload::new(&[
                        crate::messages::PayloadKind::String(dialogue)
                    ]),
                })
            } else {
                crate::messages::Message::Error(crate::messages::Error::NPCNotNeutral)
            }
        } else {
            crate::messages::Message::Error(crate::messages::Error::NPCNotFound)
        }
    }
}
