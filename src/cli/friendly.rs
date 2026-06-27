use std::str::FromStr;

use crate::cli::HandleEvent;
use crate::tui::Widget;

enum Page {
    Welcome = 0,
    Room = 1,
    Stats = 2,
    Quests = 3,
    Group = 4,
    Chat = 5,
}

#[derive(Default)]
enum Focus {
    #[default]
    WelcomeExit,

    RoomReload,
    RoomMove,
    RoomPlayers,
    RoomNPCs,
    RoomItems,
    StatsInventory,
    QuestsQuests,
    GroupCreate,
    GroupPlayers,
    GroupLeave,
    GroupInvitations,
    Chat,
}

#[derive(Default)]
pub struct FriendlyCli {
    waiter: crate::utils::Waiter,
    client: crate::network::Client,
    commands: Vec<crate::messages::Command>,
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
            Box::new(crate::tui::WelcomePage::default()),
            Box::new(crate::tui::RoomPage::default()),
            Box::new(crate::tui::StatsPage::default()),
            Box::new(crate::tui::QuestsPage::default()),
            Box::new(crate::tui::GroupPage::default()),
            Box::new(crate::tui::ChatPage::default()),
        ]);
        self.change_focus(Focus::WelcomeExit);
        crate::cli::Handler::init();
        let r = self.run(&mut terminal).await;
        crate::cli::Handler::cleanup();
        let _ = terminal.close();
        r
    }

    async fn run(&mut self, terminal: &mut crate::tui::Terminal) -> Option<crate::messages::Error> {
        loop {
            if !self.client.is_open() {
                return None;
            }
            terminal.update(&mut *self, Self::render);
            match &self.popup {
                Some(crate::tui::Popup::Input(popup)) if popup.id == "auth" && matches!(self.client.state, crate::network::ClientState::Authenticated) => self.popup = None,
                None if !matches!(self.client.state, crate::network::ClientState::Authenticated) => self.popup = Some(crate::tui::Popup::input("auth", "Enter your username")),
                _ => (),
            }
            if let Some(e) = tokio::select! {
                _ = self.waiter.wait() => Some(crate::messages::Error::ServerTimeOut),
                event = async {
                    if !self.commands.is_empty() {
                        crate::utils::Waiter::block().await;
                    }
                    if let Some(popup) = &mut self.popup {
                        popup.handle_event().await
                    } else {
                        match &self.focused {
                            Focus::WelcomeExit => self.notebook.page::<crate::tui::WelcomePage>(Page::Welcome as usize).exit.handle_event().await,
                            Focus::RoomReload => self.notebook.page::<crate::tui::RoomPage>(Page::Room as usize).refresh.handle_event().await,
                            Focus::RoomMove => self.notebook.page::<crate::tui::RoomPage>(Page::Room as usize).move_to.handle_event().await,
                            Focus::RoomPlayers => self.notebook.page::<crate::tui::RoomPage>(Page::Room as usize).players.handle_event().await,
                            Focus::RoomNPCs => self.notebook.page::<crate::tui::RoomPage>(Page::Room as usize).npcs.handle_event().await,
                            Focus::RoomItems => self.notebook.page::<crate::tui::RoomPage>(Page::Room as usize).items.handle_event().await,
                            Focus::StatsInventory => self.notebook.page::<crate::tui::StatsPage>(Page::Stats as usize).inventory.handle_event().await,
                            Focus::QuestsQuests => self.notebook.page::<crate::tui::QuestsPage>(Page::Quests as usize).quests.handle_event().await,
                            Focus::GroupCreate => self.notebook.page::<crate::tui::GroupPage>(Page::Group as usize).create.handle_event().await,
                            Focus::GroupPlayers => self.notebook.page::<crate::tui::GroupPage>(Page::Group as usize).players.handle_event().await,
                            Focus::GroupLeave => self.notebook.page::<crate::tui::GroupPage>(Page::Group as usize).leave.handle_event().await,
                            Focus::GroupInvitations => self.notebook.page::<crate::tui::GroupPage>(Page::Group as usize).invitations.handle_event().await,
                            Focus::Chat => self.notebook.page::<crate::tui::ChatPage>(Page::Chat as usize).chat.handle_event().await,
                        }
                    }
                } => {
                    if let Some(event) = event {
                        match event {
                            crate::cli::Event::Interrupted => return None,
                            event => self.process_input(event).await,
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
                                        self.commands.remove(0);
                                        Some(error)
                                    },
                                    crate::messages::Message::Response(response) => self.process_response(response).await,
                                    crate::messages::Message::Event(event) => self.process_event(event).await,
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
                    self.popup = Some(crate::tui::Popup::error(e));
                }
            }
            if self.client.is_open() && self.commands.is_empty() && let Some(id) = self.knowledge.need() {
                self.send_command(crate::messages::Command {
                    kind: crate::messages::CommandKind::Describe,
                    payload: crate::messages::Payload::new(&[
                        crate::messages::PayloadKind::String(id),
                    ]),
                }).await;
            }
        }
    }

    fn focus_info(&mut self) -> (Focus, Focus, &mut bool) {
        match &self.focused {
            Focus::WelcomeExit => (Focus::Chat, Focus::RoomReload, &mut self.notebook.page::<crate::tui::WelcomePage>(Page::Welcome as usize).exit.focus),
            Focus::RoomReload => (Focus::WelcomeExit, Focus::RoomMove, &mut self.notebook.page::<crate::tui::RoomPage>(Page::Room as usize).refresh.focus),
            Focus::RoomMove => (Focus::RoomReload, Focus::RoomPlayers, &mut self.notebook.page::<crate::tui::RoomPage>(Page::Room as usize).move_to.focus),
            Focus::RoomPlayers => (Focus::RoomMove, Focus::RoomNPCs, &mut self.notebook.page::<crate::tui::RoomPage>(Page::Room as usize).players.focus),
            Focus::RoomNPCs => (Focus::RoomPlayers, Focus::RoomItems, &mut self.notebook.page::<crate::tui::RoomPage>(Page::Room as usize).npcs.focus),
            Focus::RoomItems => (Focus::RoomNPCs, Focus::StatsInventory, &mut self.notebook.page::<crate::tui::RoomPage>(Page::Room as usize).items.focus),
            Focus::StatsInventory => (Focus::RoomItems, Focus::QuestsQuests, &mut self.notebook.page::<crate::tui::StatsPage>(Page::Stats as usize).inventory.focus),
            Focus::QuestsQuests => (Focus::StatsInventory, if self.knowledge.player.group.is_empty() { Focus::GroupCreate } else { Focus::GroupPlayers }, &mut self.notebook.page::<crate::tui::QuestsPage>(Page::Quests as usize).quests.focus),
            Focus::GroupCreate => (Focus::QuestsQuests, Focus::GroupInvitations, &mut self.notebook.page::<crate::tui::GroupPage>(Page::Group as usize).create.focus),
            Focus::GroupPlayers => (Focus::QuestsQuests, Focus::GroupLeave, &mut self.notebook.page::<crate::tui::GroupPage>(Page::Group as usize).players.focus),
            Focus::GroupLeave => (Focus::GroupPlayers, Focus::GroupInvitations, &mut self.notebook.page::<crate::tui::GroupPage>(Page::Group as usize).leave.focus),
            Focus::GroupInvitations => (if self.knowledge.player.group.is_empty() { Focus::GroupCreate } else { Focus::GroupLeave }, Focus::Chat, &mut self.notebook.page::<crate::tui::GroupPage>(Page::Group as usize).invitations.focus),
            Focus::Chat => (Focus::GroupInvitations, Focus::WelcomeExit, &mut self.notebook.page::<crate::tui::ChatPage>(Page::Chat as usize).chat.focus),
        }
    }

    fn change_focus(&mut self, focus: Focus) {
        *self.focus_info().2 = false;
        self.focused = focus;
        *self.focus_info().2 = true;
        self.notebook.current = match &self.focused {
            Focus::WelcomeExit => Page::Welcome,
            Focus::RoomReload | Focus::RoomMove | Focus::RoomPlayers | Focus::RoomNPCs | Focus::RoomItems  => Page::Room,
            Focus::StatsInventory => Page::Stats,
            Focus::QuestsQuests => Page::Quests,
            Focus::GroupCreate | Focus::GroupPlayers | Focus::GroupLeave | Focus::GroupInvitations => Page::Group,
            Focus::Chat => Page::Chat,
        } as usize;
    }

    async fn send_command(&mut self, command: crate::messages::Command) -> Option<crate::messages::Error> {
        if let Some(writer) = &self.client.writer {
            self.commands.push(command.clone());
            if writer.write_message(&crate::messages::Message::Command(command)).await.is_err() {
                return Some(crate::messages::Error::SendFailed);
            }
            self.waiter.begin(3);
        }
        None
    }

    async fn process_input(&mut self, event: crate::cli::Event) -> Option<crate::messages::Error> {
        if self.popup.is_some() {
            match event {
                crate::cli::Event::Validate => {
                    let mut popup = self.popup.take().unwrap();
                    match &mut popup {
                        crate::tui::Popup::Ask(popup) if let Some(selected) = popup.selected() && selected => match popup.id {
                            "exit" => {
                                self.client.close();
                                None
                            }
                            "group_leave" => self.send_command(crate::messages::Command::new(crate::messages::CommandKind::GroupLeave)).await,
                            _ => None,
                        }
                        crate::tui::Popup::Action(popup) if let Some(selected) = popup.buttons.selected() => match popup.id {
                            "invite" => match selected.as_str() {
                                "Dismiss" => {
                                    self.knowledge.invitations.remove(&popup.title);
                                    self.notebook.page::<crate::tui::GroupPage>(Page::Group as usize).update(&self.knowledge);
                                    None
                                }
                                "Accept" => {
                                    self.knowledge.invitations.remove(&popup.title);
                                    self.send_command(crate::messages::Command {
                                        kind: crate::messages::CommandKind::GroupJoin,
                                        payload: crate::messages::Payload::new(&[
                                            crate::messages::PayloadKind::String(popup.title.clone()),
                                        ]),
                                    }).await
                                }
                                _ => None,
                            }
                            "move" if let Ok(direction) = crate::game::Direction::from_str(selected) => self.send_command(crate::messages::Command {
                                kind: crate::messages::CommandKind::Move,
                                payload: crate::messages::Payload::new(&[
                                    crate::messages::PayloadKind::String(direction.to_string()),
                                ]),
                            }).await,
                            "player" if selected == "Invite" => self.send_command(crate::messages::Command {
                                kind: crate::messages::CommandKind::GroupInvite,
                                payload: crate::messages::Payload::new(&[
                                    crate::messages::PayloadKind::String(popup.title.clone()),
                                ]),
                            }).await,
                            _ => None,
                        }
                        crate::tui::Popup::Describe(popup) if let Some(selected) = popup.buttons.selected() => match popup.id {
                            "item" => match selected.as_str() {
                                "Drop" => self.send_command(crate::messages::Command {
                                    kind: crate::messages::CommandKind::Drop,
                                    payload: crate::messages::Payload::new(&[
                                        crate::messages::PayloadKind::String(popup.data::<crate::game::Item>().id.clone()),
                                    ]),
                                }).await,
                                "Take" => self.send_command(crate::messages::Command {
                                    kind: crate::messages::CommandKind::Take,
                                    payload: crate::messages::Payload::new(&[
                                        crate::messages::PayloadKind::String(popup.data::<crate::game::Item>().id.clone()),
                                    ]),
                                }).await,
                                _ => None,
                            }
                            "npc" => match selected.as_str() {
                                "Talk" => self.send_command(crate::messages::Command {
                                    kind: crate::messages::CommandKind::Talk,
                                    payload: crate::messages::Payload::new(&[
                                        crate::messages::PayloadKind::String(popup.data::<crate::game::Npc>().id.clone()),
                                    ]),
                                }).await,
                                "Ask for a quest" => self.send_command(crate::messages::Command {
                                    kind: crate::messages::CommandKind::Quest,
                                    payload: crate::messages::Payload::new(&[
                                        crate::messages::PayloadKind::String(popup.data::<crate::game::Npc>().id.clone()),
                                    ]),
                                }).await,
                                _ => None,
                            }
                            "quest" if selected == "Abandon" => self.send_command(crate::messages::Command {
                                kind: crate::messages::CommandKind::AbandonQuest,
                                payload: crate::messages::Payload::new(&[
                                    crate::messages::PayloadKind::String(popup.data::<crate::game::Quest>().id.clone()),
                                ]),
                            }).await,
                            _ => None,
                        }
                        crate::tui::Popup::Input(popup) => match popup.id {
                            "auth" => self.send_command(crate::messages::Command {
                                kind: crate::messages::CommandKind::Connect,
                                payload: crate::messages::Payload::new(&[
                                    crate::messages::PayloadKind::String(popup.input.input.consume()),
                                ]),
                            }).await,
                            "group_create" => self.send_command(crate::messages::Command {
                                kind: crate::messages::CommandKind::GroupCreate,
                                payload: crate::messages::Payload::new(&[
                                    crate::messages::PayloadKind::String(popup.input.input.consume()),
                                ]),
                            }).await,
                            _ => None,
                        }
                        _ => None,
                    }
                }
                _ => None,
            }
        } else {
            match event {
                crate::cli::Event::Key {
                    code: crate::cli::KeyCode::Tab,
                    modifiers: crate::cli::KeyModifiers::NONE,
                } if matches!(self.client.state, crate::network::ClientState::Authenticated) => {
                    let focus = self.focus_info().1;
                    self.change_focus(focus);
                    None
                }
                crate::cli::Event::Key {
                    code: crate::cli::KeyCode::BackTab,
                    modifiers: crate::cli::KeyModifiers::SHIFT,
                } if matches!(self.client.state, crate::network::ClientState::Authenticated) => {
                    let focus = self.focus_info().0;
                    self.change_focus(focus);
                    None
                }
                crate::cli::Event::Validate => {
                    match &self.focused {
                        Focus::WelcomeExit => {
                            self.popup = Some(crate::tui::Popup::ask(
                                "exit",
                                "Exit",
                                "Do you really want to exit ?",
                            ));
                            None
                        }
                        Focus::RoomReload => self.send_command(crate::messages::Command::new(crate::messages::CommandKind::Look)).await,
                        Focus::RoomMove => {
                            let mut actions = vec!["Return".to_string()];
                            let mut directions = self.knowledge.room.room.exits
                                .keys()
                                .map(|i| i.to_string())
                                .collect();
                            actions.append(&mut directions);
                            self.popup = Some(crate::tui::Popup::action(
                                "move",
                                "Move to",
                                actions,
                            ));
                            None
                        }
                        Focus::RoomPlayers => {
                            if let Some(player) = self.notebook.page::<crate::tui::RoomPage>(Page::Room as usize).players.selected() {
                                self.popup = Some(crate::tui::Popup::action(
                                    "player",
                                    player,
                                    vec![
                                        "Return".to_string(),
                                        "Invite".to_string(),
                                    ],
                                ));
                            }
                            None
                        }
                        Focus::RoomNPCs => {
                            if let Some(npc) = self.notebook.page::<crate::tui::RoomPage>(Page::Room as usize).npcs.selected() {
                                self.popup = Some(crate::tui::Popup::describe(
                                    "npc",
                                    Box::new(npc.clone()),
                                    vec![
                                        "Talk".to_string(),
                                        "Ask for a quest".to_string(),
                                    ],
                                ));
                            }
                            None
                        }
                        Focus::RoomItems => {
                            if let Some(item) = self.notebook.page::<crate::tui::RoomPage>(Page::Room as usize).items.selected() {
                                self.popup = Some(crate::tui::Popup::describe(
                                    "item",
                                    Box::new(item.clone()),
                                    vec!["Take".to_string()],
                                ));
                            }
                            None
                        }
                        Focus::StatsInventory => {
                            if let Some(item) = self.notebook.page::<crate::tui::StatsPage>(Page::Stats as usize).inventory.selected() {
                                self.popup = Some(crate::tui::Popup::describe(
                                    "item",
                                    Box::new(item.clone()),
                                    vec!["Drop".to_string()],
                                ));
                            }
                            None
                        }
                        Focus::QuestsQuests => {
                            if let Some(quest) = self.notebook.page::<crate::tui::QuestsPage>(Page::Quests as usize).quests.selected() {
                                self.popup = Some(crate::tui::Popup::describe(
                                    "quest",
                                    Box::new(quest.clone()),
                                    vec!["Abandon".to_string()],
                                ));
                            }
                            None
                        }
                        Focus::GroupCreate => {
                            self.popup = Some(crate::tui::Popup::input(
                                "group_create",
                                "Enter the group names",
                            ));
                            None
                        }
                        Focus::GroupPlayers => None,
                        Focus::GroupLeave => {
                            self.popup = Some(crate::tui::Popup::ask(
                                "group_leave",
                                "Leave Group",
                                "Do you really want to leave the group ?",
                            ));
                            None
                        }
                        Focus::GroupInvitations => {
                            if let Some(group) = self.notebook.page::<crate::tui::GroupPage>(Page::Group as usize).invitations.selected() {
                                self.popup = Some(crate::tui::Popup::action(
                                    "invite",
                                    group,
                                    vec![
                                        "Return".to_string(),
                                        "Accept".to_string(),
                                        "Dismiss".to_string(),
                                    ],
                                ));
                            }
                            None
                        }
                        Focus::Chat => {
                            let scope = self.notebook.page::<crate::tui::ChatPage>(Page::Chat as usize).chat.scope.to_string();
                            let input = self.notebook.page::<crate::tui::ChatPage>(Page::Chat as usize).chat.input.input.consume();
                            self.send_command(crate::messages::Command {
                                kind: crate::messages::CommandKind::Chat,
                                payload: crate::messages::Payload::new(&[
                                    crate::messages::PayloadKind::String(scope),
                                    crate::messages::PayloadKind::String(input),
                                ]),
                            }).await
                        }
                    }
                }
                _ => None,
            }
        }
    }

    async fn process_response(&mut self, response: crate::messages::Response) -> Option<crate::messages::Error> {
        match if self.commands.is_empty() {
            None
        } else {
            Some(self.commands.remove(0))
        } {
            Some(command) => match command.kind {
                crate::messages::CommandKind::AbandonQuest => {
                    let mut quest = String::new();
                    if response.payload.is_empty() && command.payload.extract(&mut [
                        crate::messages::PayloadExtractor::String(&mut quest),
                    ]).is_ok() && let Some(quest) = self.knowledge.player.quests.get_mut(&quest) {
                        quest.status = crate::game::QuestStatus::Abandoned;
                        self.notebook.page::<crate::tui::QuestsPage>(Page::Quests as usize).update(&self.knowledge);
                        None
                    } else {
                        Some(crate::messages::Error::UnexpectedServerResponse)
                    }
                }
                crate::messages::CommandKind::Attack => None,
                crate::messages::CommandKind::Chat => {
                    if response.payload.is_empty() {
                        None
                    } else {
                        Some(crate::messages::Error::UnexpectedServerResponse)
                    }
                }
                crate::messages::CommandKind::Connect => if response.payload.extract(&mut [
                    crate::messages::PayloadExtractor::String(&mut "connected".to_string()),
                ]).is_ok() {
                    self.client.state = crate::network::ClientState::Authenticated;
                    self.knowledge.player.username.clear();
                    if command.payload.extract(&mut [
                        crate::messages::PayloadExtractor::String(&mut self.knowledge.player.username),
                    ]).is_err() {
                        return Some(crate::messages::Error::UnexpectedServerResponse);
                    }
                    self.send_command(crate::messages::Command::new(crate::messages::CommandKind::Look)).await
                } else {
                    Some(crate::messages::Error::UnexpectedServerResponse)
                }
                crate::messages::CommandKind::Describe => {
                    let mut data = crate::game::WorldData::default();
                    if response.payload.extract(&mut [
                        crate::messages::PayloadExtractor::Json(&mut data),
                    ]).is_ok() && self.knowledge.update(data).is_ok() {
                        None
                    } else {
                        Some(crate::messages::Error::UnexpectedServerResponse)
                    }
                }
                crate::messages::CommandKind::Drop => {
                    if let crate::messages::PayloadKind::String(mut s) = command.payload.args[0].clone() {
                        if response.payload.extract(&mut [
                            crate::messages::PayloadExtractor::KeyValue {
                                key: &mut "dropped".to_string(),
                                value: &mut s,
                            },
                        ]).is_ok() {
                            if self.send_command(crate::messages::Command::new(crate::messages::CommandKind::Inventory)).await.is_none() &&
                                self.send_command(crate::messages::Command::new(crate::messages::CommandKind::Quests)).await.is_none() &&
                                self.send_command(crate::messages::Command::new(crate::messages::CommandKind::Look)).await.is_none() {
                                None
                            } else {
                                Some(crate::messages::Error::SendFailed)
                            }
                        } else {
                            Some(crate::messages::Error::UnexpectedServerResponse)
                        }
                    } else {
                        None
                    }
                }
                crate::messages::CommandKind::GroupDescribe => {
                    let mut group = crate::game::Group::default();
                    if response.payload.extract(&mut [
                        crate::messages::PayloadExtractor::Json(&mut group),
                    ]).is_ok() {
                        self.knowledge.change_group(Some(group));
                        self.notebook.page::<crate::tui::GroupPage>(Page::Group as usize).update(&self.knowledge);
                        None
                    } else {
                        Some(crate::messages::Error::UnexpectedServerResponse)
                    }
                }
                crate::messages::CommandKind::GroupCreate | crate::messages::CommandKind::GroupJoin => {
                    let mut group = String::new();
                    if response.payload.extract(&mut [
                        crate::messages::PayloadExtractor::KeyValue {
                            key: &mut "group".to_string(),
                            value: &mut group,
                        },
                    ]).is_ok() {
                        self.change_focus(Focus::GroupPlayers);
                        self.send_command(crate::messages::Command::new(crate::messages::CommandKind::GroupDescribe)).await
                    } else {
                        Some(crate::messages::Error::UnexpectedServerResponse)
                    }
                }
                crate::messages::CommandKind::GroupInvite => {
                    if response.payload.is_empty() {
                        None
                    } else {
                        Some(crate::messages::Error::UnexpectedServerResponse)
                    }
                }
                crate::messages::CommandKind::GroupLeave => {
                    if response.payload.is_empty() {
                        self.knowledge.change_group(None);
                        self.change_focus(Focus::GroupCreate);
                        None
                    } else {
                        Some(crate::messages::Error::UnexpectedServerResponse)
                    }
                }
                crate::messages::CommandKind::Inventory => {
                    if response.payload.extract(&mut [
                        crate::messages::PayloadExtractor::Json(&mut self.knowledge.player.items),
                    ]).is_ok() {
                        self.notebook.page::<crate::tui::StatsPage>(Page::Stats as usize).update(&self.knowledge);
                        None
                    } else {
                        Some(crate::messages::Error::UnexpectedServerResponse)
                    }
                }
                crate::messages::CommandKind::Look => {
                    let mut room = crate::game::RoomState::default();
                    if response.payload.extract(&mut [
                        crate::messages::PayloadExtractor::Json(&mut room),
                    ]).is_ok() {
                        self.knowledge.change_room(room);
                        self.notebook.page::<crate::tui::RoomPage>(Page::Room as usize).update(&self.knowledge);
                        None
                    } else {
                        Some(crate::messages::Error::UnexpectedServerResponse)
                    }
                }
                crate::messages::CommandKind::Move => {
                    let mut room = String::new();
                    if response.payload.extract(&mut [
                        crate::messages::PayloadExtractor::KeyValue {
                            key: &mut "room".to_string(),
                            value: &mut room,
                        },
                    ]).is_ok() {
                        self.send_command(crate::messages::Command::new(crate::messages::CommandKind::Look)).await
                    } else {
                        Some(crate::messages::Error::UnexpectedServerResponse)
                    }
                }
                crate::messages::CommandKind::Quest => {
                    let mut quest = crate::game::Quest::default();
                    if response.payload.extract(&mut [
                        crate::messages::PayloadExtractor::Json(&mut quest),
                    ]).is_ok() {
                        self.send_command(crate::messages::Command::new(crate::messages::CommandKind::Quests)).await
                    } else {
                        Some(crate::messages::Error::UnexpectedServerResponse)
                    }
                }
                crate::messages::CommandKind::Quests => {
                    let mut quests: Vec<crate::game::QuestProgress> = Vec::new();
                    if response.payload.extract(&mut [
                        crate::messages::PayloadExtractor::Json(&mut quests),
                    ]).is_ok() {
                        self.knowledge.player.quests.clear();
                        for quest in quests.into_iter() {
                            self.knowledge.player.quests.insert(quest.quest.clone(), quest);
                        }
                        self.notebook.page::<crate::tui::QuestsPage>(Page::Quests as usize).update(&self.knowledge);
                        None
                    } else {
                        Some(crate::messages::Error::UnexpectedServerResponse)
                    }
                }
                crate::messages::CommandKind::Quit => if response.payload.extract(&mut [
                    crate::messages::PayloadExtractor::String(&mut "bye".to_string()),
                ]).is_ok() {
                    self.client.close();
                    None
                } else {
                    Some(crate::messages::Error::UnexpectedServerResponse)
                }
                crate::messages::CommandKind::Status => None,
                crate::messages::CommandKind::Take => {
                    if response.payload.extract(&mut [
                        crate::messages::PayloadExtractor::KeyValue {
                            key: &mut "taken".to_string(),
                            value: &mut "".to_string(),
                        },
                    ]).is_ok() {
                        if self.send_command(crate::messages::Command::new(crate::messages::CommandKind::Look)).await.is_none() &&
                            self.send_command(crate::messages::Command::new(crate::messages::CommandKind::Quests)).await.is_none() &&
                            self.send_command(crate::messages::Command::new(crate::messages::CommandKind::Inventory)).await.is_none() {
                            None
                        } else {
                            Some(crate::messages::Error::SendFailed)
                        }
                    } else {
                        Some(crate::messages::Error::UnexpectedServerResponse)
                    }
                }
                crate::messages::CommandKind::Talk => {
                    let mut dialogue = String::new();
                    if response.payload.extract(&mut [
                        crate::messages::PayloadExtractor::String(&mut dialogue),
                    ]).is_ok() {
                        if let crate::messages::PayloadKind::String(id) = &command.payload.args[0] && let Some(npc) = self.knowledge.npcs.get(id) {
                            self.popup = Some(crate::tui::Popup::info(
                                &npc.name,
                                &dialogue,
                            ));
                        }
                        None
                    } else {
                        Some(crate::messages::Error::UnexpectedServerResponse)
                    }
                }
                crate::messages::CommandKind::Who => {
                    let mut players = String::new();
                    if response.payload.extract(&mut [
                        crate::messages::PayloadExtractor::KeyValue {
                            key: &mut "players".to_string(),
                            value: &mut players,
                        },
                    ]).is_ok() && let Ok(v) = players.parse::<usize>() {
                        self.knowledge.players = v;
                        None
                    } else {
                        Some(crate::messages::Error::UnexpectedServerResponse)
                    }
                }
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
                    Some(crate::messages::Error::UnexpectedServerResponse)
                } else {
                    self.knowledge.addr = self.client.addr.clone();
                    self.knowledge.proto = self.client.proto.clone();
                    None
                }
            }
        }
    }

    async fn process_event(&mut self, event: crate::messages::Event) -> Option<crate::messages::Error> {
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
                self.notebook.page::<crate::tui::ChatPage>(Page::Chat as usize).chat.push(message);
            }
            crate::messages::EventKind::Invite => {
                let mut group = String::new();
                if event.payload.extract(&mut [
                    crate::messages::PayloadExtractor::String(&mut group),
                ]).is_err() {
                    return Some(crate::messages::Error::UnexpectedServerResponse);
                }
                self.knowledge.invitations.insert(group);
                self.notebook.page::<crate::tui::GroupPage>(Page::Group as usize).update(&self.knowledge);
            }
            crate::messages::EventKind::Join => {
                let mut player = String::new();
                if event.payload.extract(&mut [
                    crate::messages::PayloadExtractor::String(&mut player),
                ]).is_err() || self.knowledge.player.group.is_empty() || !self.knowledge.group.players.insert(player) {
                    return Some(crate::messages::Error::UnexpectedServerResponse);
                }
                self.notebook.page::<crate::tui::GroupPage>(Page::Group as usize).update(&self.knowledge);
            }
            crate::messages::EventKind::Leave => {
                let mut player = String::new();
                if event.payload.extract(&mut [
                    crate::messages::PayloadExtractor::String(&mut player),
                ]).is_err() || self.knowledge.player.group.is_empty() || !self.knowledge.group.players.remove(&player) {
                    return Some(crate::messages::Error::UnexpectedServerResponse);
                }
                self.notebook.page::<crate::tui::GroupPage>(Page::Group as usize).update(&self.knowledge);
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
            crate::messages::EventKind::PresenceEnter => {
                let mut player = String::new();
                if event.payload.extract(&mut [
                    crate::messages::PayloadExtractor::String(&mut player),
                ]).is_err() || !self.knowledge.room.players.insert(player) {
                    return Some(crate::messages::Error::UnexpectedServerResponse);
                }
                self.notebook.page::<crate::tui::RoomPage>(Page::Room as usize).update(&self.knowledge);
            }
            crate::messages::EventKind::PresenceLeave => {
                let mut player = String::new();
                if event.payload.extract(&mut [
                    crate::messages::PayloadExtractor::String(&mut player),
                ]).is_err() || !self.knowledge.room.players.remove(&player) {
                    return Some(crate::messages::Error::UnexpectedServerResponse);
                }
                self.notebook.page::<crate::tui::RoomPage>(Page::Room as usize).update(&self.knowledge);
            }
            crate::messages::EventKind::QuestComplete => {
                let mut quest = String::new();
                if event.payload.extract(&mut [
                    crate::messages::PayloadExtractor::String(&mut quest),
                ]).is_ok() && let Some(quest) = self.knowledge.player.quests.get_mut(&quest) {
                    quest.status = crate::game::QuestStatus::Completed;
                    self.notebook.page::<crate::tui::QuestsPage>(Page::Quests as usize).update(&self.knowledge);
                    self.send_command(crate::messages::Command::new(crate::messages::CommandKind::Inventory)).await;
                } else {
                    return Some(crate::messages::Error::UnexpectedServerResponse);
                }
            }
        }
        None
    }

    fn render(&mut self, frame: &mut ratatui::Frame) {
        let [header, body] = ratatui::layout::Layout::vertical([
            ratatui::layout::Constraint::Length(3),
            ratatui::layout::Constraint::Fill(1),
        ])
            .areas(frame.area());
        crate::tui::Header.render_with_data(&mut self.knowledge, header, frame.buffer_mut());
        self.notebook.render_with_data(&mut self.knowledge, body, frame.buffer_mut());
        if let Some(popup) = &mut self.popup {
            popup.render_with_data(&mut self.knowledge, body, frame.buffer_mut());
        }
    }
}
