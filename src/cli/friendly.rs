use std::str::FromStr;

use crate::cli::HandleEvent;
use crate::tui::Widget;

#[derive(Default)]
enum Focus {
    #[default]
    Chat,

    RoomPlayers,
    RoomNPCs,
    RoomItems,
    StatsInventory,
    QuestsQuests,
}

#[derive(Default)]
pub struct FriendlyCli {
    waiter: crate::utils::Waiter,
    client: crate::network::Client,
    command: Option<crate::messages::Command>,
    focused: Focus,
    popup: Option<crate::tui::Popup>,
    notebook: crate::tui::Notebook,
    knowledge: crate::tui::Knowledge,
}

impl FriendlyCli {
    pub async fn start(&mut self, client: crate::network::Client) -> Option<crate::messages::Error> {
        self.waiter.begin(3);
        self.client = client;
        let mut terminal = match crate::tui::Terminal::new() {
            Ok(v) => v,
            Err(_) => return Some(crate::messages::Error::UnknownError),
        };
        self.notebook = crate::tui::Notebook::new(vec![
            Box::new(crate::tui::RoomPage::default()),
            Box::new(crate::tui::StatsPage::default()),
            Box::new(crate::tui::QuestsPage::default()),
            Box::new(crate::tui::ChatPage::default()),
        ]);
        self.notebook.page::<crate::tui::ChatPage>(3).chat.focus = true;
        crate::cli::Handler::init();
        let r = self.run(&mut terminal).await;
        crate::cli::Handler::cleanup();
        let _ = terminal.close();
        r
    }

    async fn run(&mut self, terminal: &mut crate::tui::Terminal) -> Option<crate::messages::Error> {
        self.popup = Some(crate::tui::Popup::input("auth", "CHOOSE YOUR USERNAME"));
        loop {
            terminal.update(&mut *self, Self::render);
            if !self.client.is_open() {
                return None;
            }
            if let Some(e) = tokio::select! {
                _ = self.waiter.wait() => Some(crate::messages::Error::ServerTimeOut),
                event = async {
                    if self.command.is_some() {
                        crate::utils::Waiter::block().await;
                    }
                    if let Some(popup) = &mut self.popup {
                        popup.handle_event().await
                    } else {
                        match &self.focused {
                            Focus::RoomPlayers => self.notebook.page::<crate::tui::RoomPage>(0).players.handle_event().await,
                            Focus::RoomNPCs => self.notebook.page::<crate::tui::RoomPage>(0).npcs.handle_event().await,
                            Focus::RoomItems => self.notebook.page::<crate::tui::RoomPage>(0).items.handle_event().await,
                            Focus::StatsInventory => self.notebook.page::<crate::tui::StatsPage>(1).inventory.handle_event().await,
                            Focus::QuestsQuests => self.notebook.page::<crate::tui::QuestsPage>(2).quests.handle_event().await,
                            Focus::Chat => self.notebook.page::<crate::tui::ChatPage>(3).chat.handle_event().await,
                        }
                    }
                } => {
                    if let Some(event) = event {
                        match event {
                            crate::cli::Event::Interrupted => return None,
                            crate::cli::Event::Key {
                                code: crate::cli::KeyCode::Tab,
                                modifiers: crate::cli::KeyModifiers::NONE,
                            } => {
                                let (_, next, focused) = self.focus_info();
                                *focused = false;
                                self.focused = next;
                                let (_, _, focused) = self.focus_info();
                                *focused = true;
                                None
                            }
                            crate::cli::Event::Key {
                                code: crate::cli::KeyCode::Tab,
                                modifiers: crate::cli::KeyModifiers::SHIFT,
                            } => {
                                let (previous, _, focused) = self.focus_info();
                                *focused = false;
                                self.focused = previous;
                                let (_, _, focused) = self.focus_info();
                                *focused = true;
                                None
                            }
                            event => if let Some(command) = self.process_input(event) {
                                self.send_command(command).await
                            } else {
                                None
                            },
                        }
                    } else {
                        None
                    }
                }
                message = self.client.reader.read() => {
                    match message {
                        Ok(Some(message)) => match crate::messages::Message::from_str(&message) {
                            Ok(message) => {
                                self.waiter.end();
                                match message {
                                    crate::messages::Message::Error(error) => {
                                        self.command = None;
                                        Some(error)
                                    },
                                    crate::messages::Message::Response(response) => self.process_response(response).await,
                                    crate::messages::Message::Event(event) => self.process_event(event),
                                    crate::messages::Message::Command(_) => Some(crate::messages::Error::UnexpectedServerResponse),
                                }
                            }
                            Err(_) => Some(crate::messages::Error::UnexpectedServerResponse),
                        }
                        _ => Some(crate::messages::Error::ConnectionClosed),
                    }
                }
            } {
                if e.is_fatal() {
                    return Some(e);
                } else {
                    self.popup = Some(crate::tui::Popup::error("auth", e));
                }
            }
        }
    }

    fn focus_info(&mut self) -> (Focus, Focus, &mut bool) {
        match &self.focused {
            Focus::RoomPlayers => (Focus::Chat, Focus::RoomNPCs, &mut self.notebook.page::<crate::tui::RoomPage>(0).players.focus),
            Focus::RoomNPCs => (Focus::RoomPlayers, Focus::RoomItems, &mut self.notebook.page::<crate::tui::RoomPage>(0).npcs.focus),
            Focus::RoomItems => (Focus::RoomNPCs, Focus::StatsInventory, &mut self.notebook.page::<crate::tui::RoomPage>(0).items.focus),
            Focus::StatsInventory => (Focus::RoomItems, Focus::QuestsQuests, &mut self.notebook.page::<crate::tui::StatsPage>(1).inventory.focus),
            Focus::QuestsQuests => (Focus::StatsInventory, Focus::Chat, &mut self.notebook.page::<crate::tui::QuestsPage>(2).quests.focus),
            Focus::Chat => (Focus::QuestsQuests, Focus::RoomPlayers, &mut self.notebook.page::<crate::tui::ChatPage>(3).chat.focus),
        }
    }

    async fn send_command(&mut self, command: crate::messages::Command) -> Option<crate::messages::Error> {
        if let Some(writer) = &self.client.writer {
            self.command = Some(command.clone());
            if writer.write_message(&crate::messages::Message::Command(command)).await.is_err() {
                return Some(crate::messages::Error::SendFailed);
            }
            self.waiter.begin(3);
        }
        None
    }

    fn process_input(&mut self, event: crate::cli::Event) -> Option<crate::messages::Command> {
        if self.popup.is_some() {
            if matches!(event, crate::cli::Event::Validate) {
                let mut popup = self.popup.take().unwrap();
                match &mut popup {
                    crate::tui::Popup::Error(popup) => match popup.id {
                        "auth" => self.popup = Some(crate::tui::Popup::input(
                            "auth",
                            "CHOOSE YOUR USERNAME",
                        )),
                        _ => (),
                    }
                    crate::tui::Popup::Input(popup) => match popup.id {
                        "auth" => return Some(crate::messages::Command {
                            kind: crate::messages::CommandKind::Connect,
                            payload: crate::messages::Payload::new(&[
                                crate::messages::PayloadKind::String(popup.input.input.consume()),
                            ]),
                        }),
                        _ => (),
                    }
                }
            }
        } else {
            match &self.focused {
                Focus::Chat if matches!(event, crate::cli::Event::Validate) => {
                    return Some(crate::messages::Command {
                        kind: crate::messages::CommandKind::Chat,
                        payload: crate::messages::Payload::new(&[
                            crate::messages::PayloadKind::String(self.notebook.page::<crate::tui::ChatPage>(3).chat.scope.to_string()),
                            crate::messages::PayloadKind::String(self.notebook.page::<crate::tui::ChatPage>(3).chat.input.input.consume()),
                        ]),
                    });
                }
                _ => (),
            }
        }
        None
    }

    async fn process_response(&mut self, response: crate::messages::Response) -> Option<crate::messages::Error> {
        let command = self.command.take();
        match command {
            Some(command) => match command.kind {
                crate::messages::CommandKind::Connect => if response.payload.extract(&mut [
                    crate::messages::PayloadExtractor::String(&mut "connected".to_string()),
                ]).is_err() {
                    return Some(crate::messages::Error::UnexpectedServerResponse);
                } else {
                    self.client.state = crate::network::ClientState::Authenticated;
                    self.knowledge.player.username.clear();
                    if command.payload.extract(&mut [
                        crate::messages::PayloadExtractor::String(&mut self.knowledge.player.username),
                    ]).is_err() {
                        return Some(crate::messages::Error::UnexpectedServerResponse);
                    }
                    self.send_command(crate::messages::Command {
                        kind: crate::messages::CommandKind::Look,
                        payload: crate::messages::Payload::default(),
                    }).await;
                }
                crate::messages::CommandKind::Describe => {
                    let mut data = crate::game::WorldData::default();
                    if response.payload.extract(&mut [
                        crate::messages::PayloadExtractor::Json(&mut data),
                    ]).is_err() || self.knowledge.update(data).is_err() {
                        return Some(crate::messages::Error::UnexpectedServerResponse);
                    }
                }
                crate::messages::CommandKind::Look => {
                    let mut room = crate::game::RoomState::default();
                    if response.payload.extract(&mut [
                        crate::messages::PayloadExtractor::Json(&mut room),
                    ]).is_err() {
                        return Some(crate::messages::Error::UnexpectedServerResponse);
                    }
                    self.knowledge.change_room(room);
                    self.notebook.page::<crate::tui::RoomPage>(0).init(&self.knowledge);
                }
                crate::messages::CommandKind::Quit => if response.payload.extract(&mut [
                    crate::messages::PayloadExtractor::String(&mut "bye".to_string()),
                ]).is_err() {
                    return Some(crate::messages::Error::UnexpectedServerResponse);
                } else {
                    self.client.close();
                }
                _ => (),
            }
            None => {
                self.client.proto.clear();
                if response.payload.extract(&mut [
                    crate::messages::PayloadExtractor::String(&mut "hello".to_string()),
                    crate::messages::PayloadExtractor::KeyValue {
                        key: &mut "proto".to_string(),
                        value: &mut self.client.proto,
                    }
                ]).is_err() {
                    return Some(crate::messages::Error::UnexpectedServerResponse);
                }
                self.knowledge.addr = self.client.addr.clone();
                self.knowledge.proto = self.client.proto.clone();
            }
        };
        if self.command.is_none() && let Some(id) = self.knowledge.need() {
            self.send_command(crate::messages::Command {
                kind: crate::messages::CommandKind::Describe,
                payload: crate::messages::Payload::new(&[
                    crate::messages::PayloadKind::String(id),
                ]),
            }).await;
        }
        None
    }

    fn process_event(&mut self, event: crate::messages::Event) -> Option<crate::messages::Error> {
        match event.kind {
            crate::messages::EventKind::Chat => {
                let mut message = crate::tui::ChatMessage {
                    scope: event.scope,
                    author: String::new(),
                    content: String::new(),
                };
                if matches!(message.scope, crate::messages::EventScope::Player | crate::messages::EventScope::Stats) || event.payload.extract(&mut [
                    crate::messages::PayloadExtractor::String(&mut message.author),
                    crate::messages::PayloadExtractor::String(&mut message.content),
                ]).is_err() {
                    return Some(crate::messages::Error::UnexpectedServerResponse);
                }
                self.notebook.page::<crate::tui::ChatPage>(3).chat.push(message);
            }
            crate::messages::EventKind::Players => {
                let mut n = String::new();
                if event.payload.extract(&mut [
                    crate::messages::PayloadExtractor::KeyValue {
                        key: &mut "players".to_string(),
                        value: &mut n,
                    },
                ]).is_err() {
                    return Some(crate::messages::Error::UnexpectedServerResponse);
                }
                match n.parse::<usize>() {
                    Ok(v) => self.knowledge.players = v,
                    Err(_) => return Some(crate::messages::Error::UnexpectedServerResponse),
                }
            }
            _ => (),
        }
        None
    }

    fn render(&mut self, frame: &mut ratatui::Frame) {
        let [header, body] = ratatui::layout::Layout::vertical([
            ratatui::layout::Constraint::Length(3),
            ratatui::layout::Constraint::Fill(1),
        ])
            .areas(frame.area());
        crate::tui::Header.render_with_data(&self.knowledge, header, frame.buffer_mut());
        self.notebook.current = match &self.focused {
            Focus::RoomItems | Focus::RoomNPCs | Focus::RoomPlayers => 0,
            Focus::StatsInventory => 1,
            Focus::QuestsQuests => 2,
            Focus::Chat => 3,
        };
        self.notebook.render_with_data(&self.knowledge, body, frame.buffer_mut());
        if let Some(popup) = &mut self.popup {
            popup.render(body, frame.buffer_mut());
        }
    }
}

impl crate::tui::ToListItem for crate::game::Npc {
    fn update_item(knowledge: &crate::tui::Knowledge, s: &str) -> Option<Self> {
        knowledge.npcs.get(s).cloned()
    }

    fn to_item(&self, _: &crate::tui::Knowledge) -> ratatui::widgets::ListItem<'static> {
        ratatui::widgets::ListItem::new(self.name.clone())
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
