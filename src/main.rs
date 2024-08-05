// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use morphology_wizard::{create, import_save, Instruction, Node};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::api::dialog::FileDialogBuilder;
use tauri::{CustomMenuItem, Menu, State, Submenu};

#[derive(Debug, Default, Serialize, Deserialize)]
struct AppState {
    instruction_params: Vec<Instruction>,
    instruction_values: Vec<Instruction>,
    carrier_point_params: HashMap<String, CarrierPoints>,
    carrier_point_values: HashMap<String, Vec<[f32; 3]>>,
    nodes: Vec<Node>,
}

fn main() {
    tauri::Builder::default()
        .manage(Mutex::<AppState>::default())
        .menu(new_menu())
        .on_menu_event(|event| event.window().emit("menu_event", event.menu_item_id()).unwrap())
        .invoke_handler(tauri::generate_handler![
            set_instructions,
            set_carrier_points,
            get_nodes,
            save,
            export_swc,
            export_nml,
            export_nrn,
            set_live_preview,
            detach_preview,
            reattach_preview,
        ])
        .run(tauri::generate_context!())
        .expect("error while running morphology wizard");
}

fn new_menu() -> Menu {
    Menu::new()
        .add_submenu(Submenu::new(
            "File",
            Menu::new()
                .add_item(CustomMenuItem::new("new".to_string(), "New"))
                .add_item(CustomMenuItem::new("save".to_string(), "Save"))
                .add_item(CustomMenuItem::new("load".to_string(), "Load"))
                .add_item(CustomMenuItem::new("swc".to_string(), "Export SWC"))
                .add_item(CustomMenuItem::new("nml".to_string(), "Export NeuroML"))
                .add_item(CustomMenuItem::new("nrn".to_string(), "Export NEURON"))
                .add_item(CustomMenuItem::new("quit".to_string(), "Quit")),
        ))
        .add_submenu(Submenu::new(
            "Edit",
            Menu::new()
                .add_item(CustomMenuItem::new("instr".to_string(), "Growth Instructions"))
                .add_item(CustomMenuItem::new("points".to_string(), "Carrier Points")),
        ))
        .add_submenu(Submenu::new(
            "Preview",
            Menu::new()
                .add_item(CustomMenuItem::new("generate".to_string(), "Update Preview"))
                .add_item(CustomMenuItem::new("update".to_string(), "Enable Live Preview"))
                .add_item(CustomMenuItem::new("camera".to_string(), "Reset Camera"))
                .add_item(CustomMenuItem::new("preview".to_string(), "Show Preview"))
                .add_item(CustomMenuItem::new("camera".to_string(), "Detach Preview")),
        ))
        .add_submenu(Submenu::new(
            "Help",
            Menu::new()
                .add_item(CustomMenuItem::new("docs".to_string(), "Documentation"))
                .add_item(CustomMenuItem::new("home".to_string(), "Homepage"))
                .add_item(CustomMenuItem::new("src".to_string(), "Github"))
                .add_item(CustomMenuItem::new("about".to_string(), "About Morphology Wizard")),
        ))
}

#[tauri::command(rename_all = "snake_case")]
fn set_instructions(data: &str, instructions: State<Instructions>, nodes: State<Nodes>) -> Result<(), String> {
    //
    nodes.0.lock().unwrap().clear();
    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
fn set_carrier_points(data: &str, carrier_points: State<CarrierPoints>, nodes: State<Nodes>) -> Result<(), String> {
    //
    nodes.0.lock().unwrap().clear();
    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
fn get_nodes(
    instructions: State<Instructions>,
    carrier_points: State<CarrierPoints>,
    nodes: State<Nodes>,
) -> Result<Vec<Node>, String> {
    // let instructions = import_save(save_file).map_err(|err| err.to_string())?;
    // let nodes = dbg!(create(&dbg!(instructions)));
    // Ok(nodes)
    todo!()
}

#[tauri::command(rename_all = "snake_case")]
fn save(instructions: String) {
    FileDialogBuilder::new()
        .set_file_name("morphology.json")
        .save_file(|file_path| {
            // do something with the optional file path here
            // the file path is `None` if the user closed the dialog
            todo!();
        })
}

#[tauri::command(rename_all = "snake_case")]
fn export_swc(instructions: State<Instructions>, nodes: State<Nodes>) {
    let instructions = instructions.0.lock().unwrap();
    let nodes = nodes.0.lock().unwrap();
    let swc = morphology_wizard::export_swc(&instructions, &nodes);

    todo!();
    /*
    FileDialogBuilder::new()
        .set_title("Export to SWC file")
        .set_file_name("morphology.swc")
        .save_file(|file_path| {
            if let Some(file_path) = file_path {
                eprintln!("TODO: Write {swc:?} to {file_path:?}");
                todo!();
            }
        })
    */
}

#[tauri::command(rename_all = "snake_case")]
fn export_nml(instructions: State<Instructions>, nodes: State<Nodes>) {
    let instructions = instructions.0.lock().unwrap();
    let nodes = nodes.0.lock().unwrap();
    let nml = morphology_wizard::export_nml(&instructions, &nodes);

    FileDialogBuilder::new()
        .set_title("Export to NeuroML file")
        .set_file_name("morphology.nml")
        // .add_filter("nml", &[".nml"]) // TODO: What does this do?
        .save_file(move |file_path| {
            if let Some(file_path) = file_path {
                eprintln!("TODO: Write {nml:?} to {file_path:?}");
                todo!();
            }
        })
}

#[tauri::command(rename_all = "snake_case")]
fn export_nrn(instructions: State<Instructions>, nodes: State<Nodes>) {
    let instructions = instructions.0.lock().unwrap();
    let nodes = nodes.0.lock().unwrap();
    let nrn = morphology_wizard::export_nrn(&instructions, &nodes);

    FileDialogBuilder::new()
        .set_title("Export to NEURON script")
        .set_file_name("morphology.py")
        .save_file(move |file_path| {
            if let Some(file_path) = file_path {
                eprintln!("TODO: Write {nrn:?} to {file_path:?}");
                todo!();
            }
        })
}

#[tauri::command(rename_all = "snake_case")]
fn set_live_preview(win_handle: tauri::Window, enable: bool) {
    //
}

#[tauri::command(rename_all = "snake_case")]
fn detach_preview() {}

#[tauri::command(rename_all = "snake_case")]
fn reattach_preview() {}
