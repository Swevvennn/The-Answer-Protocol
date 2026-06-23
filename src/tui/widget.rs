pub trait Widget {
    fn width(&self) -> u16 {
        0
    }

    fn height(&self) -> u16 {
        0
    }

    fn render(&mut self, _: ratatui::layout::Rect, _: &mut ratatui::buffer::Buffer) {}

    fn render_with_data(&mut self, _: &crate::tui::Knowledge, _: ratatui::layout::Rect, _: &mut ratatui::buffer::Buffer) {}
}
