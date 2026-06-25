pub struct QuestsPage {
    pub quests: crate::tui::List<crate::game::Quest>,
}

impl QuestsPage {
    pub fn update(&mut self, knowledge: &crate::tui::Knowledge) {
        self.quests.items = knowledge.player.quests
            .values()
            .map(|i| crate::tui::ListItem::Unknown(i.quest.clone()))
            .collect();
    }
}

impl Default for QuestsPage {
    fn default() -> Self {
        Self {
            quests: crate::tui::List::new("Quests"),
        }
    }
}

impl crate::tui::NotebookPage for QuestsPage {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn title(&self) -> &str {
        "QUESTS"
    }
}

impl crate::tui::Widget for QuestsPage {
    fn render_with_data(&mut self, knowledge: &mut crate::tui::Knowledge, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        self.quests.render_with_data(knowledge, area, buf);
    }
}
