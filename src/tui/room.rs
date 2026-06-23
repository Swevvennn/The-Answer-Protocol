use crate::tui::ToListItem;

pub struct RoomPage {
    pub players: crate::tui::List<String>,
    pub npcs: crate::tui::List<crate::game::Npc>,
    pub items: crate::tui::List<crate::game::Item>,
}

impl RoomPage {
    pub fn init(&mut self, knowledge: &crate::tui::Knowledge) {
        self.players.items = String::items_from_iter(&knowledge.room.players);
        self.npcs.items = crate::game::Npc::items_from_iter(&knowledge.room.npcs);
        self.items.items = crate::game::Item::items_from_iter(&knowledge.room.items);
    }
}

impl Default for RoomPage {
    fn default() -> Self {
        Self {
            players: crate::tui::List::new("Players"),
            npcs: crate::tui::List::new("NPCs"),
            items: crate::tui::List::new("Items"),
        }
    }
}

impl crate::tui::NotebookPage for RoomPage {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn title(&self) -> &str {
        "ROOM"
    }
}

impl crate::tui::Widget for RoomPage {
    fn render_with_data(&mut self, knowledge: &crate::tui::Knowledge, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        let [top, bottom] = ratatui::layout::Layout::vertical([
            ratatui::layout::Constraint::Fill(1),
            ratatui::layout::Constraint::Fill(1),
        ])
            .areas(area);
        let [left, middle, right] = ratatui::layout::Layout::horizontal([
            ratatui::layout::Constraint::Fill(1),
            ratatui::layout::Constraint::Fill(1),
            ratatui::layout::Constraint::Fill(1),
        ])
            .areas(bottom);
        self.players.render_with_data(knowledge, left, buf);
        self.npcs.render_with_data(knowledge, middle, buf);
        self.items.render_with_data(knowledge, right, buf);
    }
}
