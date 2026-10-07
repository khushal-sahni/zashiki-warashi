// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if let Some(code) = zashiki_warashi_lib::run_cli(&args) {
        std::process::exit(code);
    }
    zashiki_warashi_lib::run()
}
