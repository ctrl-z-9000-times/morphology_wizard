// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod carrier_points;

use carrier_points::CarrierPoints;
use serde::Deserialize;
use std::collections::HashMap;
use std::f64::consts::PI;
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
        "quit" => {
            std::process::exit(0);
        }
        _ => {}
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Instruction {
    Soma {
        name: String,
        carrier_points: Vec<String>,
        soma_diameter: f64,
    },
    Dendrite {
        name: String,
        carrier_points: Vec<String>,
        roots: Vec<String>,
        balancing_factor: f64,
        maximum_branches: u32,
        minimum_diameter: f64,
        dendrite_taper: f64,
        maximum_segment_length: f64,
    },
    Axon {
        name: String,
        carrier_points: Vec<String>,
        roots: Vec<String>,
        balancing_factor: f64,
        extension_distance: f64,
        extension_angle: f64,
        branch_distance: f64,
        branch_angle: f64,
        maximum_branches: u32,
        minimum_diameter: f64,
        maximum_segment_length: f64,
        reach_all_carrier_points: bool,
    },
}
impl Instruction {
    fn take_name(&mut self) -> String {
        match self {
            Self::Soma { name, .. } | Self::Dendrite { name, .. } | Self::Axon { name, .. } => std::mem::take(name),
        }
    }
}

#[tauri::command(rename_all = "snake_case")]
fn generate_morphology(mut instructions: Vec<Instruction>, carrier_points: Vec<CarrierPoints>) {
    //
    let instruction_names: HashMap<String, u32> = instructions
        .iter_mut()
        .enumerate()
        .map(|(index, instr)| (instr.take_name(), index as u32))
        .collect();
    let map_instruction_names_to_indices =
        |names: Vec<String>| names.iter().map(|name| instruction_names[name]).collect();
    //
    let carrier_points_names: HashMap<String, CarrierPoints> = carrier_points
        .into_iter()
        .map(|mut param| (param.take_name(), param))
        .collect();
    let generate_carrier_points = |names: Vec<String>| {
        let mut points = vec![];
        for name in &names {
            points.append(&mut carrier_points_names[name].generate_points());
        }
        points
    };
    // Fixup and transform the instructions into the propper data structure.
    let instructions: Vec<_> = instructions
        .into_iter()
        .map(|instr| match instr {
            Instruction::Soma {
                carrier_points,
                soma_diameter,
                ..
            } => morphology_wizard::Instruction {
                morphology: None,
                soma_diameter: Some(soma_diameter),
                carrier_points: generate_carrier_points(carrier_points),
                roots: vec![],
            },
            Instruction::Dendrite {
                carrier_points,
                roots,
                balancing_factor,
                maximum_branches,
                minimum_diameter,
                dendrite_taper,
                maximum_segment_length,
                ..
            } => morphology_wizard::Instruction {
                morphology: Some(morphology_wizard::Morphology {
                    balancing_factor,
                    extension_distance: f64::INFINITY,
                    extension_angle: PI,
                    branch_distance: f64::INFINITY,
                    branch_angle: PI,
                    extend_before_branch: false,
                    maximum_branches,
                    minimum_diameter,
                    dendrite_taper,
                    maximum_segment_length,
                    reach_all_carrier_points: true,
                }),
                soma_diameter: None,
                carrier_points: generate_carrier_points(carrier_points),
                roots: map_instruction_names_to_indices(roots),
            },
            Instruction::Axon {
                carrier_points,
                roots,
                balancing_factor,
                extension_distance,
                extension_angle,
                branch_distance,
                branch_angle,
                maximum_branches,
                minimum_diameter,
                maximum_segment_length,
                reach_all_carrier_points,
                ..
            } => morphology_wizard::Instruction {
                morphology: Some(morphology_wizard::Morphology {
                    balancing_factor,
                    extension_distance,
                    extension_angle,
                    branch_distance,
                    branch_angle,
                    extend_before_branch: true,
                    maximum_branches,
                    minimum_diameter,
                    dendrite_taper: 0.0,
                    maximum_segment_length,
                    reach_all_carrier_points,
                }),
                soma_diameter: None,
                carrier_points: generate_carrier_points(carrier_points),
                roots: map_instruction_names_to_indices(roots),
            },
        })
        .collect();

    let nodes = dbg!(morphology_wizard::create(&dbg!(instructions)));
}
