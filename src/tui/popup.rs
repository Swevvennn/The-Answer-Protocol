use ratatui::prelude::Widget;

trait PopupBehavior: crate::tui::Widget {
    fn title(&self) -> &str;

    fn content_width(&self) -> u16;

    fn content_height(&self) -> u16;

    fn render_content(&mut self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer);
}

impl<T: PopupBehavior> crate::tui::Widget for T {
    fn render(&mut self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        let [_, area, _] = ratatui::layout::Layout::vertical([
            ratatui::layout::Constraint::Fill(1),
            ratatui::layout::Constraint::Length(4 + self.content_height()),
            ratatui::layout::Constraint::Fill(1),
        ])
            .areas(area);
        let [_, area, _] = ratatui::layout::Layout::horizontal([
            ratatui::layout::Constraint::Fill(1),
            ratatui::layout::Constraint::Length(6 + self.content_width()),
            ratatui::layout::Constraint::Fill(1),
        ])
            .areas(area);
        ratatui::widgets::Clear::default()
            .render(area, buf);
        let block = ratatui::widgets::Block::bordered()
            .title(format!(" {} ", self.title()))
            .padding(ratatui::widgets::Padding::symmetric(2, 1));
        self.render_content(block.inner(area), buf);
        block.render(area, buf);
    }
}

pub struct PopupError {
    pub id: &'static str,
    title: String,
    error: String,
    handler: crate::cli::Handler,
}

impl PopupError {
    pub fn new(id: &'static str, error: crate::messages::Error) -> Self {
        Self {
            id,
            title: format!("ERR {}", error.code()),
            error: error.message().to_string(),
            handler: crate::cli::Handler::default(),
        }
    }
}

impl PopupBehavior for PopupError {
    fn title(&self) -> &str {
        &self.title
    }

    fn content_width(&self) -> u16 {
        std::cmp::max(
            std::cmp::max(
                self.title.len() as u16,
                self.error.len() as u16,
            ),
            2,
        )
    }

    fn content_height(&self) -> u16 {
        3
    }

    fn render_content(&mut self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        let [top, _, bottom] = ratatui::layout::Layout::vertical([
            ratatui::layout::Constraint::Length(1),
            ratatui::layout::Constraint::Length(1),
            ratatui::layout::Constraint::Length(1),
        ])
            .areas(area);
        ratatui::widgets::Paragraph::new(self.error.as_str())
            .centered()
            .render(top, buf);
        ratatui::widgets::Paragraph::new(ratatui::text::Span::styled(
            "OK",
            ratatui::style::Style::default()
                .fg(ratatui::style::Color::Black)
                .bg(ratatui::style::Color::White)
        ))
            .centered()
            .render(bottom, buf);
    }
}

impl crate::cli::HandleEvent for PopupError {
    async fn handle_event(&mut self) -> Option<crate::cli::Event> {
        let event = self.handler.handle_event().await;
        if let Some(crate::cli::Event::Validate) = event {
            event
        } else {
            None
        }
    }
}

pub struct PopupInput {
    pub id: &'static str,
    pub input: crate::tui::Input,
    title: String,
}

impl PopupInput {
    pub fn new(id: &'static str, title: &str) -> Self {
        Self {
            id,
            title: title.to_string(),
            input: crate::tui::Input::new(30, 30),
        }
    }
}

impl PopupBehavior for PopupInput {
    fn title(&self) -> &str {
        &self.title
    }

    fn content_width(&self) -> u16 {
        use crate::tui::Widget;
        std::cmp::max(
            self.title.len() as u16,
            self.input.width(),
        )
    }

    fn content_height(&self) -> u16 {
        use crate::tui::Widget;
        self.input.height()
    }

    fn render_content(&mut self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        use crate::tui::widget::Widget;
        self.input.render(area, buf);
    }
}

impl crate::cli::HandleEvent for PopupInput {
    async fn handle_event(&mut self) -> Option<crate::cli::Event> {
        self.input.handle_event().await
    }
}

pub enum Popup {
    Error(PopupError),
    Input(PopupInput),
}

impl Popup {
    pub fn error(id: &'static str, error: crate::messages::Error) -> Self {
        Self::Error(
            PopupError::new(id, error),
        )
    }

    pub fn input(id: &'static str, title: &str) -> Self {
        Self::Input(
            PopupInput::new(id, title),
        )
    }
}

impl crate::cli::HandleEvent for Popup {
    async fn handle_event(&mut self) -> Option<crate::cli::Event> {
        match self {
            Popup::Error(popup) => popup.handle_event().await,
            Popup::Input(popup) => popup.handle_event().await,
        }
    }
}

impl crate::tui::Widget for Popup {
    fn render(&mut self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        match self {
            Popup::Error(popup) => popup.render(area, buf),
            Popup::Input(popup) => popup.render(area, buf),
        }
    }
}
