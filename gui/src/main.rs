//! Launches MDeezl's desktop app. A release build is a windowed program, so
//! Windows opens no console beside it.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() -> eframe::Result {
    eframe::run_native(
        "MDeezl",
        mdeezl_gui::native_options(),
        Box::new(|_cc| Ok(Box::new(mdeezl_gui::App::default()))),
    )
}
