use ratatui::widgets::Widget;

#[derive(Default)]
pub struct GroupPage {
    scrollbar: crate::tui::Scrollbar,
}

impl crate::tui::NotebookPage for GroupPage {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn title(&self) -> &str {
        "GROUP"
    }
}

impl crate::tui::Widget for GroupPage {
    fn render_with_data(&mut self, knowledge: &mut crate::tui::Knowledge, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        let block = ratatui::widgets::Block::default()
            .padding(ratatui::widgets::Padding::uniform(1));
        self.scrollbar.content = 3;
        if knowledge.player.group.is_empty() {
            ratatui::widgets::Paragraph::new("You are not in a group")
                .render(block.inner(area), buf);
        } else {
            let [top, _bottom] = ratatui::layout::Layout::vertical([
                ratatui::layout::Constraint::Length(2),
                ratatui::layout::Constraint::Fill(1),
            ])
                .areas(block.inner(area));
            ratatui::widgets::Paragraph::new(ratatui::text::Line::from(vec![
                ratatui::text::Span::styled("In group: ", ratatui::style::Style::default().fg(ratatui::style::Color::White)),
                ratatui::text::Span::styled(&knowledge.player.group, ratatui::style::Style::default().fg(ratatui::style::Color::LightMagenta)),
            ]))
                .render(top, buf);
            self.scrollbar.content += 1;
        }
        block.render(area, buf);
        self.scrollbar.viewport = area.height as usize;
        self.scrollbar.render(area, buf);
    }
}
