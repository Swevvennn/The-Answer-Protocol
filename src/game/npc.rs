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

impl NPCKind {
    pub fn is_enemy(&self) -> bool {
        matches!(self, Self::Enemy { hp: _ })
    }
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
    pub fn is_enemy(&self) -> bool {
        self.data.is_enemy()
    }

    pub fn talk(game: &mut crate::game::GameState, player: &String, npc: &String) -> crate::messages::Message {
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
                if let Some(player) = game.players.get_mut(player) {
                    crate::game::Player::update_quests(&game.quests, player, "talk", &npc.id);
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

    pub fn quest(game: &mut crate::game::GameState, player: &String, npc: &String) -> crate::messages::Message {
        if let Some(npc) = game.npcs.get(npc) {
            if let NPCKind::Neutral {
                dialogues: _,
                quests,
                trades: _,
            } = &npc.data {
                if let Some(player) = game.players.get_mut(player) {
                    for quest in quests {
                        let mut ok = true;
                        if let Some(quest) = player.quests.get_mut(quest) {
                            if matches!(quest.status, crate::game::QuestStatus::Abandoned) {
                                quest.status = crate::game::QuestStatus::Active;
                            } else {
                                ok = false;
                            }
                        } else {
                            let quest = &game.quests[quest];
                            for require in &quest.requirements {
                                if !player.quests.contains_key(require) || !matches!(player.quests[require].status, crate::game::QuestStatus::Completed) {
                                    ok = false;
                                    break;
                                }
                            }
                            if ok {
                                player.quests.insert(
                                    quest.id.clone(),
                                    crate::game::QuestProgress::new(quest),
                                );
                            }
                        }
                        if ok {
                            crate::game::Player::update_quests(&game.quests, player, "", "");
                            return crate::messages::Message::Response(crate::messages::Response {
                                payload: crate::messages::Payload::new(&[
                                    crate::messages::PayloadKind::new_json(&quest),
                                ]),
                            });
                        }
                    }
                    crate::messages::Message::Error(crate::messages::Error::NoQuestAvailable)
                } else {
                    crate::messages::Message::Error(crate::messages::Error::ServerError)
                }
            } else {
                crate::messages::Message::Error(crate::messages::Error::NPCNotNeutral)
            }
        } else {
            crate::messages::Message::Error(crate::messages::Error::NPCNotFound)
        }
    }
}
