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

	let file_path = "exemple.json";

	println!("In file {file_path}");

    let contents = fs::read_to_string(file_path)
        .expect("Should have been able to read the file");

	let temp = contents.as_str();

	let data: Data = serde_json::from_str(temp);

    println!("With text:\n{}", data.items);

    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "My First GUI",
        options,
        Box::new(|_cc| Ok(Box::new(MyApp::default()))),
    )
}

#[derive(Default)]
struct MyApp {
    counter: i32,
	text: String,
	val: f32,
	check: bool
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
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
        });
    }
}
