use crate::tui::ToListItem;

pub struct StatsPage {
    pub inventory: crate::tui::List<crate::game::Item>,
}

impl StatsPage {
    pub fn update(&mut self, knowledge: &crate::tui::Knowledge) {
        self.inventory.items = crate::game::Item::items_from_iter(&knowledge.player.items);
    }
}

impl Default for StatsPage {
    fn default() -> Self {
        Self {
            inventory: crate::tui::List::new("Inventory"),
        }
    }
}

impl crate::tui::NotebookPage for StatsPage {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn title(&self) -> &str {
        "STATS"
    }
}

impl crate::tui::Widget for StatsPage {
    fn render_with_data(&mut self, knowledge: &mut crate::tui::Knowledge, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        self.inventory.render_with_data(knowledge, area, buf);
    }
}
