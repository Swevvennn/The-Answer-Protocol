use ratatui::widgets::Widget;

impl crate::tui::ToListItem for crate::game::Npc {
    fn update_item(knowledge: &crate::tui::Knowledge, s: &str) -> Option<Self> {
        knowledge.npcs.get(s).cloned()
    }

    fn to_item(&self, _: &crate::tui::Knowledge) -> ratatui::widgets::ListItem<'static> {
        ratatui::widgets::ListItem::new(self.name.clone())
    }
}

impl crate::tui::Widget for crate::game::Npc {
    fn width(&self) -> u16 {
        self.description.len() as u16
    }

    fn height(&self) -> u16 {
        1
    }

    fn render(&mut self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        ratatui::widgets::Paragraph::new(self.description.as_str())
            .render(area, buf);
    }
}

impl crate::tui::PopupDescribeInfos for crate::game::Npc {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn title(&self) -> &str {
        &self.name
    }
}

impl crate::tui::ToListItem for crate::game::Item {
    fn update_item(knowledge: &crate::tui::Knowledge, s: &str) -> Option<Self> {
        knowledge.items.get(s).cloned()
    }

    fn to_item(&self, _: &crate::tui::Knowledge) -> ratatui::widgets::ListItem<'static> {
        ratatui::widgets::ListItem::new(self.name.clone())
    }
}

impl crate::tui::Widget for crate::game::Item {
    fn width(&self) -> u16 {
        self.description.len() as u16
    }

    fn height(&self) -> u16 {
        1
    }

    fn render(&mut self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        ratatui::widgets::Paragraph::new(self.description.as_str())
            .render(area, buf);
    }
}

impl crate::tui::PopupDescribeInfos for crate::game::Item {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn title(&self) -> &str {
        &self.name
    }
}

impl crate::tui::ToListItem for crate::game::Quest {
    fn update_item(knowledge: &crate::tui::Knowledge, s: &str) -> Option<Self> {
        knowledge.quests.get(s).cloned()
    }

    fn to_item(&self, knowledge: &crate::tui::Knowledge) -> ratatui::widgets::ListItem<'static> {
        ratatui::widgets::ListItem::new(ratatui::text::Line::from(vec![
            ratatui::text::Span::styled("(", ratatui::style::Style::default().fg(ratatui::style::Color::White)),
            ratatui::text::Span::styled(
                knowledge.player.quests[&self.id].status.to_string(),
                ratatui::style::Style::default().fg(match knowledge.player.quests[&self.id].status {
                    crate::game::QuestStatus::Abandoned => ratatui::style::Color::Red,
                    crate::game::QuestStatus::Active => ratatui::style::Color::Yellow,
                    crate::game::QuestStatus::Completed => ratatui::style::Color::LightGreen,
                })
            ),
            ratatui::text::Span::styled(") ", ratatui::style::Style::default().fg(ratatui::style::Color::White)),
            ratatui::text::Span::styled(self.name.clone(), ratatui::style::Style::default().fg(ratatui::style::Color::White)),
        ]))
    }
}
