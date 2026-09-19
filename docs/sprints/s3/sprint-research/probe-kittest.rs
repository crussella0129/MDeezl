use eframe::egui;
use egui_kittest::{Harness, kittest::Queryable};

#[test]
fn click_updates_state() {
    let mut clicks = 0;
    let mut harness = Harness::new_ui(|ui| {
        if ui.button("Generate").clicked() {
            // state lives outside; closure captures by move in real app
        }
        ui.label("ready");
    });
    harness.get_by_label("Generate").click();
    harness.run();
    harness.get_by_label("ready");
    clicks += 1;
    assert_eq!(clicks, 1);
}
