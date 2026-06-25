use ratatui::prelude::Widget;

trait PopupWidget: crate::tui::Widget {
    fn title(&self) -> &str;

    fn content_width(&self) -> u16;

    fn content_height(&self) -> u16;

    fn render_content(&mut self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer);
}

impl<T: PopupWidget> crate::tui::Widget for T {
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
        ratatui::widgets::Clear
            .render(area, buf);
        let block = ratatui::widgets::Block::bordered()
            .title(format!(" {} ", self.title()))
            .padding(ratatui::widgets::Padding::symmetric(2, 1));
        self.render_content(block.inner(area), buf);
        block.render(area, buf);
    }
}

pub struct PopupAction {
    pub id: &'static str,
    pub title: String,
    pub buttons: crate::tui::Buttons,
}

impl PopupAction {
    pub fn new(id: &'static str, title: &str, actions: Vec<String>) -> Self {
        Self {
            id,
            title: title.to_string(),
            buttons: crate::tui::Buttons::new(
                actions,
                crate::tui::ButtonsKind::Vertical,
            ),
        }
    }
}

impl PopupWidget for PopupAction {
    fn title(&self) -> &str {
        &self.title
    }

    fn content_width(&self) -> u16 {
        use crate::tui::Widget;
        std::cmp::max(
            self.buttons.width(),
            2,
        )
    }

    fn content_height(&self) -> u16 {
        use crate::tui::Widget;
        self.buttons.height()
    }

    fn render_content(&mut self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        use crate::tui::Widget;
        self.buttons.render(area, buf);
    }
}

impl crate::cli::HandleEvent for PopupAction {
    async fn handle_event(&mut self) -> Option<crate::cli::Event> {
        self.buttons.handle_event().await
    }
}

pub trait PopupDescribeInfos: crate::tui::Widget {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;

    fn title(&self) -> &str;
}

pub struct PopupDescribe {
    pub id: &'static str,
    pub buttons: crate::tui::Buttons,
    b_return: crate::tui::Button,
    data: Box<dyn PopupDescribeInfos>,
    handler: crate::cli::Handler,
}

impl PopupDescribe {
    pub fn new(id: &'static str, data: Box<dyn PopupDescribeInfos>, actions: Vec<String>) -> Self {
        Self {
            id,
            buttons: crate::tui::Buttons::new(
                actions,
                crate::tui::ButtonsKind::Horizontal,
            ),
            b_return: crate::tui::Button::new("Return"),
            data,
            handler: crate::cli::Handler::default(),
        }
    }

    pub fn data<T: 'static>(&mut self) -> &mut T {
        self.data
            .as_any_mut()
            .downcast_mut::<T>()
            .unwrap()
    }
}

impl PopupWidget for PopupDescribe {
    fn title(&self) -> &str {
        self.data.title()
    }

    fn content_width(&self) -> u16 {
        use crate::tui::Widget;
        std::cmp::max(
            std::cmp::max(
                self.data.width(),
                self.buttons.width(),
            ),
            8,
        )
    }

    fn content_height(&self) -> u16 {
        self.data.height() + 4
    }

    fn render_content(&mut self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        let [top, _, middle, _, bottom] = ratatui::layout::Layout::vertical([
            ratatui::layout::Constraint::Length(1),
            ratatui::layout::Constraint::Length(1),
            ratatui::layout::Constraint::Fill(1),
            ratatui::layout::Constraint::Length(1),
            ratatui::layout::Constraint::Length(1),
        ])
            .areas(area);
        use crate::tui::Widget;
        self.b_return.render(top, buf);
        self.data.render(middle, buf);
        self.buttons.render(bottom, buf);
    }
}

impl crate::cli::HandleEvent for PopupDescribe {
    async fn handle_event(&mut self) -> Option<crate::cli::Event> {
        let event = if self.b_return.focus {
            self.handler.handle_event().await
        } else {
            self.buttons.handle_event().await
        };
        if let Some(crate::cli::Event::Key {
            code: crate::cli::KeyCode::Up | crate::cli::KeyCode::Down,
            modifiers: crate::cli::KeyModifiers::NONE
        }) = event {
            self.b_return.focus = !self.b_return.focus;
            if self.b_return.focus {
                self.buttons.unfocus();
            } else {
                self.buttons.focus(0);
            }
            None
        } else {
            event
        }
    }
}

pub struct PopupError {
    title: String,
    error: String,
    button: crate::tui::Button,
    handler: crate::cli::Handler,
}

impl PopupError {
    pub fn new(error: crate::messages::Error) -> Self {
        Self {
            title: format!("ERR {}", error.code()),
            error: error.message().to_string(),
            button: crate::tui::Button::new("Ok"),
            handler: crate::cli::Handler::default(),
        }
    }
}

impl PopupWidget for PopupError {
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
        use crate::tui::Widget;
        self.button.focus = true;
        self.button.render(bottom, buf);
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

pub struct PopupInfo {
    title: String,
    info: String,
    handler: crate::cli::Handler,
}

impl PopupInfo {
    pub fn new(title: &str, info: &str) -> Self {
        Self {
            title: title.to_string(),
            info: info.to_string(),
            handler: crate::cli::Handler::default(),
        }
    }
}

impl PopupWidget for PopupInfo {
    fn title(&self) -> &str {
        &self.title
    }

    fn content_width(&self) -> u16 {
        std::cmp::max(
            std::cmp::max(
                self.title.len() as u16,
                self.info.len() as u16,
            ),
            2,
        )
    }

    fn content_height(&self) -> u16 {
        1
    }

    fn render_content(&mut self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        ratatui::widgets::Paragraph::new(self.info.as_str())
            .centered()
            .render(area, buf);
    }
}

impl crate::cli::HandleEvent for PopupInfo {
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

impl PopupWidget for PopupInput {
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
    Action(PopupAction),
    Describe(PopupDescribe),
    Error(PopupError),
    Info(PopupInfo),
    Input(PopupInput),
}

impl Popup {
    pub fn action(id: &'static str, title: &str, actions: Vec<String>) -> Self {
        Self::Action(
            PopupAction::new(id, title, actions),
        )
    }

    pub fn describe(id: &'static str, data: Box<dyn PopupDescribeInfos>, actions: Vec<String>) -> Self {
        Self::Describe(
            PopupDescribe::new(id, data, actions),
        )
    }

    pub fn error(error: crate::messages::Error) -> Self {
        Self::Error(
            PopupError::new(error),
        )
    }

    pub fn info(title: &str, info: &str) -> Self {
        Self::Info(
            PopupInfo::new(title, info),
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
            Popup::Action(popup) => popup.handle_event().await,
            Popup::Describe(popup) => popup.handle_event().await,
            Popup::Error(popup) => popup.handle_event().await,
            Popup::Info(popup) => popup.handle_event().await,
            Popup::Input(popup) => popup.handle_event().await,
        }
    }
}

impl crate::tui::Widget for Popup {
    fn render(&mut self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        match self {
            Popup::Action(popup) => popup.render(area, buf),
            Popup::Describe(popup) => popup.render(area, buf),
            Popup::Error(popup) => popup.render(area, buf),
            Popup::Info(popup) => popup.render(area, buf),
            Popup::Input(popup) => popup.render(area, buf),
        }
    }
}
