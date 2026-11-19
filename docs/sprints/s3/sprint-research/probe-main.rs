use eframe::egui;
fn main() -> eframe::Result {
    eframe::run_ui_native("probe", Default::default(), |ui, _frame| {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.label("hello");
        });
    })
}
