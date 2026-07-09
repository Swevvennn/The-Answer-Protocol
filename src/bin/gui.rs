use eframe::egui;
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
        Box::new(|_cc| Ok(Box::new(MyApp::default()))),
    )
}

#[derive(Default, PartialEq)]
enum AppState {
    #[default]
    AskUsername,
    ShowData,
}

#[derive(Default)]
struct MyApp {
    state: AppState,

    username_input: String,
    username: String,

    counter: i32,
	text: String,
	val: f32,
	check: bool,
}

impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            match self.state {
                AppState::AskUsername => self.show_username_screen(ui),
                AppState::ShowData => self.show_data_screen(ui),
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
                self.state = AppState::ShowData;
            }
        });
    }

    fn show_data_screen(&mut self, ui: &mut egui::Ui) {
        ui.heading(format!("Bienvenue, {} !", self.username));
        ui.separator();

        // --- ton contenu d'origine, inchangé ---
        ui.label("This is a label");
        ui.hyperlink("https://github.com/emilk/egui");
        ui.text_edit_singleline(&mut self.text);
        if ui.button("Click me").clicked() { }
        ui.add(egui::Slider::new(&mut self.val, 0.0..=100.0));
        ui.add(egui::DragValue::new(&mut self.val));
        ui.label(format!("Text: {}", self.text));
        ui.checkbox(&mut self.check, "Checkbox");
        ui.label(format!("Counter: {}", self.counter));

        if ui.button("Increment").clicked() {
            self.counter += 1;
        }

        if ui.button("Reset").clicked() {
            self.counter = 0;
        }
        // --- fin contenu d'origine ---

        ui.add_space(20.0);
        if ui.button("Se déconnecter").clicked() {
            self.state = AppState::AskUsername;
            self.username_input.clear();
        }
    }
}