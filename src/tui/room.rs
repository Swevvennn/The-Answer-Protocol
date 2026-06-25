use ratatui::widgets::Widget;

use crate::tui::ToListItem;

pub struct RoomPage {
    pub reload: crate::tui::Button,
    pub move_to: crate::tui::Button,
    pub players: crate::tui::List<String>,
    pub npcs: crate::tui::List<crate::game::Npc>,
    pub items: crate::tui::List<crate::game::Item>,
}

impl RoomPage {
    pub fn update(&mut self, knowledge: &crate::tui::Knowledge) {
        self.players.items = String::items_from_iter(&knowledge.room.players);
        self.npcs.items = crate::game::Npc::items_from_iter(&knowledge.room.npcs);
        self.items.items = crate::game::Item::items_from_iter(&knowledge.room.items);
    }
}

impl Default for RoomPage {
    fn default() -> Self {
        Self {
            reload: crate::tui::Button::new("Reload"),
            move_to: crate::tui::Button::new("Move"),
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
    fn render_with_data(&mut self, knowledge: &mut crate::tui::Knowledge, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        let [c0, top, c1, desc, c2, middle, c3, move_to, c4, bottom] = ratatui::layout::Layout::vertical([
            ratatui::layout::Constraint::Length(1),
            ratatui::layout::Constraint::Length(1),
            ratatui::layout::Constraint::Length(1),
            ratatui::layout::Constraint::Length(1),
            ratatui::layout::Constraint::Length(1),
            ratatui::layout::Constraint::Fill(1),
            ratatui::layout::Constraint::Length(1),
            ratatui::layout::Constraint::Length(1),
            ratatui::layout::Constraint::Length(1),
            ratatui::layout::Constraint::Fill(1),
        ])
            .areas(area);
        Map.render_with_data(knowledge, middle, buf);
        ratatui::widgets::Clear.render(c0, buf);
        ratatui::widgets::Clear.render(top, buf);
        ratatui::widgets::Clear.render(c1, buf);
        ratatui::widgets::Clear.render(desc, buf);
        ratatui::widgets::Clear.render(c2, buf);
        ratatui::widgets::Clear.render(c3, buf);
        ratatui::widgets::Clear.render(move_to, buf);
        ratatui::widgets::Clear.render(c4, buf);
        ratatui::widgets::Clear.render(bottom, buf);
        self.reload.render(top, buf);
        ratatui::widgets::Paragraph::new(format!(
            "{}: {}",
            knowledge.room.room.name.as_str(),
            knowledge.room.room.description.as_str(),
        ))
            .centered()
            .render(desc, buf);
        self.move_to.centered = true;
        self.move_to.render(move_to, buf);
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

struct Map;

impl crate::tui::Widget for Map {
    fn render_with_data(&mut self, knowledge: &mut crate::tui::Knowledge, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        if !knowledge.positions.contains_key(&knowledge.room.room.id) {
            return;
        }
        if area.width < 6 || area.height < 5 {
            return;
        }
        let n = (
            (area.width / 6).clamp(1, 5),
            (area.height / 5).clamp(1, 5),
        );
        let cell = (
            std::cmp::max(area.width / n.0, 6),
            std::cmp::max(area.height / n.1, 5),
        );
        let space = (
            std::cmp::max(cell.0 / 6, 1),
            std::cmp::max(cell.1 / 6, 1),
        );
        let begin = (
            (area.x + area.width / 2).saturating_sub(cell.0 * n.0 / 2 + cell.0 / 2),
            (area.y + area.height / 2).saturating_sub(cell.1 * n.1 / 2 + cell.1 / 2),
        );
        let mut pos = knowledge.positions[&knowledge.room.room.id];
        pos.0 -= n.0 as i32 / 2;
        pos.1 -= n.1 as i32 / 2;
        for y in 0..n.1 {
            for x in 0..n.0 {
                let current = (pos.0 + x as i32, pos.1 + y as i32);
                if let Some((_, name)) = knowledge.rooms.get(&current) {
                    ratatui::widgets::Paragraph::new(format!(
                        "{}{}",
                        "\n".repeat(((cell.1 - space.1 * 2 - 3) / 2) as usize),
                        name.as_str()
                    ))
                        .block(ratatui::widgets::Block::bordered()
                            .padding(ratatui::widgets::Padding::horizontal(1))
                        )
                        .centered()
                        .render(
                            ratatui::layout::Rect::new(
                                begin.0 + cell.0 * x + space.0,
                                begin.1 + cell.1 * y + space.1,
                                cell.0 - space.0 * 2,
                                cell.1 - space.1 * 2,
                            ),
                            buf,
                        );
                }
                if x == 0 && knowledge.connections.contains(&((current.0 - 1, current.1), current)) {
                    ratatui::widgets::Paragraph::new("─".repeat(space.0 as usize * 2))
                        .render(
                            ratatui::layout::Rect::new(
                                begin.0 + cell.0 * x - space.0 * 2,
                                begin.1 + cell.1 * y + cell.1 / 2,
                                space.0 * 2,
                                1,
                            ),
                            buf,
                        );
                }
                if y == 0 && knowledge.connections.contains(&((current.0, current.1 - 1), current)) {
                    ratatui::widgets::Paragraph::new("│\n".repeat(space.1 as usize * 2))
                        .render(
                            ratatui::layout::Rect::new(
                                begin.0 + cell.0 * x + cell.0 / 2,
                                begin.1 + cell.1 * y - space.1 * 2,
                                1,
                                space.1 * 2,
                            ),
                            buf,
                        );
                }
                if knowledge.connections.contains(&(current, (current.0 + 1, current.1))) {
                    ratatui::widgets::Paragraph::new("─".repeat(space.0 as usize * 2))
                        .render(
                            ratatui::layout::Rect::new(
                                begin.0 + cell.0 * (x + 1) - space.0,
                                begin.1 + cell.1 * y + cell.1 / 2,
                                space.0 * 2,
                                1,
                            ),
                            buf,
                        );
                }
                if knowledge.connections.contains(&(current, (current.0, current.1 + 1))) {
                    ratatui::widgets::Paragraph::new("│\n".repeat(space.1 as usize * 2))
                        .render(
                            ratatui::layout::Rect::new(
                                begin.0 + cell.0 * x + cell.0 / 2,
                                begin.1 + cell.1 * (y + 1) - space.1,
                                1,
                                space.1 * 2,
                            ),
                            buf,
                        );
                }
            }
        }
    }
}
