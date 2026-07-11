use eframe::egui;
use egui_extras::{TableBuilder, Column};
use std::fs;
use serde_json::Result;
use dict::Dict;

struct Data {
	world: Dict<String>,
	items: Dict<String>,
	npcs: Dict<String>,
}

fn main() -> eframe::Result<()> {

	// let file_path = "exemple.json";

	// println!("In file {file_path}");

    // let contents = fs::read_to_string(file_path)
    //     .expect("Should have been able to read the file");

	// let temp = contents.as_str();

	// let data: Data = serde_json::from_str(temp);

    // println!("With text:\n{}", data.items);

    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "My First GUI",
        options,
        Box::new(|cc| {
			cc.egui_ctx.all_styles_mut(|style| {
			style.text_styles = [
				(egui::TextStyle::Heading, egui::FontId::new(50.0, egui::FontFamily::Proportional)),
                (egui::TextStyle::Body, egui::FontId::new(20.0, egui::FontFamily::Proportional)),
                (egui::TextStyle::Button, egui::FontId::new(30.0, egui::FontFamily::Proportional)),
                (egui::TextStyle::Monospace, egui::FontId::new(18.0, egui::FontFamily::Monospace)),
            ]
			.into();
			});
			Ok(Box::new(MyApp::default()))
		}),
    )
}

#[derive(Default, PartialEq)]
enum AppState {
    #[default]
    AskUsername,
	Connect,
    Rooms,
	Stats,
	Quests,
	Group,
	Chat,
}

#[derive(Default)]
struct MyApp {
    state: AppState,

	group_name: String,
	message: String,
    username_input: String,
    username: String,

    counter: i32,
	text: String,
	val: f32,
	check: bool,
}

impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {

		if self.state != AppState::AskUsername {
			egui::Panel::top("server_data")
				.resizable(false)
				.default_size(100.0)
				.show_inside(ui, |ui|{
					ui.centered_and_justified( |ui| {
						ui.horizontal( |ui| {
							ui.add_space(50.0);
							ui.label(format!("Connected at IP.IP.IP.IP:PORT proto = 1 as {}", self.username));
							ui.add_space(400.0);
							ui.label("PLAYERS players connected");
							ui.add_space(400.0);
							ui.heading("APPSTATE");
						});
					});
				});
		}

		if self.state != AppState::AskUsername && self.state != AppState::Connect {
			egui::Panel::bottom("state_data")
				.resizable(false)
				.max_size(350.0)
				.min_size(150.0)
				.show_inside(ui, |ui| {
					 if self.state == AppState::Rooms {
						TableBuilder::new(ui)
							.column(Column::remainder())
							.column(Column::remainder())
							.column(Column::remainder())
							.header(30.0, |mut header| {
								header.col(|ui| { ui.strong("Player"); });
								header.col(|ui| { ui.strong("NPCs"); });
								header.col(|ui| { ui.strong("Items"); });
							})
							.body(|mut body| {
								// for (name, kind, value) in &self.data_rows {
									body.row(18.0, |mut row| {
										row.col(|ui| { ui.label(format!("{}", self.username)); });
										row.col(|ui| { ui.label("NPC 1"); });
										row.col(|ui| { ui.label("Item 1"); });
									});
								// }
							});
					 }

					 if self.state == AppState::Stats {
						ui.add_space(20.0);
						ui.horizontal( |ui| {
							ui.strong("HP:");
							ui.label("HP/100");
						});
						ui.horizontal( |ui| {
							ui.strong("Armor:");
							ui.label("ARMOR");
							ui.add_space(200.0);
							ui.button("Unequip");
						});
						ui.horizontal( |ui| {
							ui.strong("Weapon:");
							ui.label("WEAPON");
							ui.add_space(200.0);
							ui.button("Unequip");
						});
					 }

					 if self.state == AppState::Quests {
						TableBuilder::new(ui)
							.column(Column::remainder())
							.column(Column::remainder())
							.column(Column::remainder())
							.column(Column::remainder())
							.header(30.0, |mut header| {
								header.col(|ui| { ui.strong("Quest Name"); });
								header.col(|ui| { ui.strong("From"); });
								header.col(|ui| { ui.strong("Progression"); });
								header.col(|ui| { ui.strong("Rewards"); });
							})
							.body(|mut body| {
								// for (name, kind, value) in &self.data_rows {
									body.row(18.0, |mut row| {
										row.col(|ui| { ui.label("QUEST_NAME"); });
										row.col(|ui| { ui.label("NPC"); });
										row.col(|ui| { ui.label("1/3 Coins"); });
										row.col(|ui| { ui.label("3 Coins"); });
									});
								// }
							});
					 }

					 if self.state == AppState::Group {
						ui.add_space(10.0);
						ui.horizontal( |ui| {
							ui.add_space(20.0);
							ui.strong("Invitations")
						});
						ui.horizontal( |ui| {
							ui.add_space(30.0);
							ui.label("INVITATION");
						});
					 }

					 if self.state == AppState::Chat {
						ui.centered_and_justified( |ui| {
							ui.horizontal( |ui| {
								ui.add_space(20.0);
								ui.strong("CHANNEL:");
								let response = ui.add(
									egui::TextEdit::singleline(&mut self.message)
										.hint_text("Your message")
										// .desired_height(50.0)
										.desired_width(1500.0),
								);
								ui.button("Send")
							});
						});
					 }

				});
		}

		if self.state != AppState::AskUsername && self.state != AppState::Connect {
			egui::Panel::left("nav_panel")
				.resizable(false)
				.default_size(150.0)
				.show_inside(ui, |ui| {
					ui.add_space(30.0);
					if ui.button("Rooms").clicked() {
						self.state = AppState::Rooms
					}
					ui.add_space(30.0);
					ui.separator();
					ui.add_space(30.0);
					if ui.button("Stats").clicked() {
						self.state = AppState::Stats
					}
					ui.add_space(30.0);
					ui.separator();
					ui.add_space(30.0);
					if ui.button("Quests").clicked() {
						self.state = AppState::Quests
					}
					ui.add_space(30.0);
					ui.separator();
					ui.add_space(30.0);
					if ui.button("Group").clicked() {
						self.state = AppState::Group
					}
					ui.add_space(30.0);
					ui.separator();
					ui.add_space(30.0);
					if ui.button("Chat").clicked() {
						self.state = AppState::Chat
					}
					ui.add_space(30.0);
					ui.separator();
					ui.add_space(30.0);
					if ui.button("Déconnexion").clicked() {
						self.state = AppState::AskUsername;
						self.username_input.clear();
					}
				});
            }

		egui::CentralPanel::default().show(ui, |ui| {
            match self.state {
                AppState::AskUsername => self.show_username_screen(ui),
				AppState::Connect => self.connect(ui),
				AppState::Rooms => self.rooms_ui(ui),
				AppState::Stats => self.stats_ui(ui),
				AppState::Quests => self.quests_ui(ui),
				AppState::Group => self.group_ui(ui),
				AppState::Chat => self.chat_ui(ui),
            }
        });
    }
}

impl MyApp {
    fn show_username_screen(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            ui.heading("Entrez votre nom d'utilisateur");
            ui.add_space(20.0);

            let response = ui.add(
                egui::TextEdit::singleline(&mut self.username_input)
                    .hint_text("Username")
                    .desired_width(200.0),
            );

            response.request_focus();

            let enter_pressed = response.lost_focus()
                && ui.input(|i| i.key_pressed(egui::Key::Enter));

            ui.add_space(10.0);
            let button_clicked = ui.button("Valider").clicked();

            if (enter_pressed || button_clicked) && !self.username_input.trim().is_empty() {
                self.username = self.username_input.trim().to_string();
                self.state = AppState::Connect;
            }
        });
    }

	fn connect(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            ui.heading(format!("Bienvenue, {} !", self.username));
            ui.add_space(200.0);
			ui.heading(egui::RichText::new("42").size(80.0).strong());
			ui.heading(egui::RichText::new("TAP").size(80.0).strong());
            ui.add_space(10.0);
            let button_clicked = ui.button("Connect").clicked();
			if (button_clicked) {
                self.state = AppState::Rooms;
            }
        });
    }

    fn rooms_ui(&mut self, ui: &mut egui::Ui) {
		ui.add_space(100.0);
		ui.vertical_centered( |ui| {
			ui.label("Room description");
		});
		ui.add_space(20.0);
		ui.centered_and_justified( |ui| {
			ui.label("Rooms");
		});
	}

    fn stats_ui(&mut self, ui: &mut egui::Ui) {
		ui.add_space(15.0);
		ui.strong("Inventory");
		ui.add_space(15.0);
		ui.separator();
		ui.add_space(15.0);
		TableBuilder::new(ui)
			.column(Column::exact(400.0))
			.column(Column::remainder())
			.column(Column::exact(100.0))
			.column(Column::exact(100.0))
			.column(Column::exact(100.0))
			.header(30.0, |mut header| {
				header.col(|ui| { ui.strong("Item Name"); });
				header.col(|ui| { ui.strong("Description"); });
				header.col(|ui| { ui.strong("Consume"); });
				header.col(|ui| { ui.strong("Equip"); });
				header.col(|ui| { ui.strong("Drop"); });
			})
			.body(|mut body| {
				// for (name, kind, value) in &self.data_rows {
					body.row(30.0, |mut row| {
						row.col(|ui| { ui.label("NAME"); });
						row.col(|ui| { ui.label("DESCRIPTIONH"); });
						row.col(|ui| { ui.button("consume"); });
						row.col(|ui| { ui.button("equip"); });
						row.col(|ui| { ui.button("drop"); });
					});
				// }
			});
	}

    fn quests_ui(&mut self, ui: &mut egui::Ui) {
		ui.add_space(30.0);
		ui.horizontal( |ui| {
			ui.add_space(300.0);
			ui.strong("Completed COMPLETED");
			ui.add_space(300.0);
			ui.strong("Actives ACTIVES");
			ui.add_space(300.0);
			ui.strong("Abandonned ABANDONED");
		});
		ui.add_space(30.0);
		ui.separator();
		ui.add_space(30.0);
		TableBuilder::new(ui)
			.column(Column::exact(100.0))
			.column(Column::exact(300.0))
			.column(Column::remainder())
			.column(Column::exact(100.0))
			.column(Column::exact(100.0))
			.header(30.0, |mut header| {
				header.col(|ui| { ui.strong("Status"); });
				header.col(|ui| { ui.strong("Quest Name"); });
				header.col(|ui| { ui.strong("Quest Desc."); });
				header.col(|ui| { ui.strong("Details"); });
				header.col(|ui| { ui.strong("Abandon"); });
			})
			.body(|mut body| {
				// for (name, kind, value) in &self.data_rows {
					body.row(30.0, |mut row| {
						row.col(|ui| { ui.label("STATUS"); });
						row.col(|ui| { ui.label("QUEST_NAME"); });
						row.col(|ui| { ui.label("QUEST_DESCRIPTION"); });
						row.col(|ui| { ui.button("details"); });
						row.col(|ui| { ui.button("abandon"); });
					});
				// }
			});
	}
    fn group_ui(&mut self, ui: &mut egui::Ui) {
		ui.add_space(20.0);
		ui.horizontal( |ui| {
			ui.button("Create Group");
			let response = ui.add(
				egui::TextEdit::singleline(&mut self.group_name)
					.hint_text("Your message")
					// .desired_height(50.0)
					.desired_width(500.0),
			);	
		});
		ui.add_space(20.0);
		ui.separator();
		ui.add_space(20.0);
		ui.strong("GROUP_NAME members");
		ui.label("MEMBER");
	}
    fn chat_ui(&mut self, ui: &mut egui::Ui) {
		ui.horizontal( |ui| {
			ui.strong("PLAYER_NAME : ");
			ui.label("MESSAGE_CONTENT");
		});
	}
}