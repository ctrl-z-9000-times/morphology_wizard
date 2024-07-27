// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{CustomMenuItem, Menu, MenuItem, Submenu, WindowMenuEvent};

fn main() {
    tauri::Builder::default()
        .menu(editor_menu_init())
        .on_menu_event(editor_menu_callback)
        .invoke_handler(tauri::generate_handler![generate_morphology])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn editor_menu_init() -> Menu {
    let new = CustomMenuItem::new("new".to_string(), "New");
    let save = CustomMenuItem::new("save".to_string(), "Save");
    let load = CustomMenuItem::new("load".to_string(), "Load");
    let swc = CustomMenuItem::new("swc".to_string(), "Export SWC");
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    //
    let file_menu = Submenu::new(
        "File",
        Menu::new()
            .add_item(new)
            .add_item(save)
            .add_item(load)
            .add_item(swc)
            .add_item(quit),
    );
    //
    let morph = CustomMenuItem::new("morph".to_string(), "Growth Instructions");
    let points = CustomMenuItem::new("points".to_string(), "Carrier Points");
    //
    Menu::new()
        .add_submenu(file_menu)
        .add_native_item(MenuItem::Separator)
        .add_item(morph)
        .add_native_item(MenuItem::Separator)
        .add_item(points)
}

fn editor_menu_callback(event: WindowMenuEvent) {
    match event.menu_item_id() {
        "new" => {}
        "save" => {}
        "load" => {}
        "swc" => {}
        "quit" => {
            std::process::exit(0);
        }
        other => eprintln!("Unimplemented menu button: {other}"),
    }
}

#[tauri::command(rename_all = "snake_case")]
fn generate_morphology(save_file: &str) -> Result<Vec<morphology_wizard::Node>, String> {
    let instructions = morphology_wizard::import_save(save_file).map_err(|err| err.to_string())?;
    let nodes = dbg!(morphology_wizard::create(&dbg!(instructions)));
    Ok(nodes)
}
