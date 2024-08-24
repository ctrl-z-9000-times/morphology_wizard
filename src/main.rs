// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use morphology_wizard::{create, CarrierPoints, GuiInstruction, Instruction, Node, SaveFile};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::api::dialog::{confirm, FileDialogBuilder};
use tauri::{
    AppHandle, GlobalShortcutManager, GlobalWindowEvent, Manager, State, Window, WindowBuilder, WindowEvent,
    WindowMenuEvent,
};

fn main() {
    tauri::Builder::default()
        .manage(Mutex::<AppState>::default())
        .menu(menu::main_window())
        .on_menu_event(menu_event)
        .on_window_event(window_event)
        .invoke_handler(tauri::generate_handler![
            menu::set_item,
            auto_load,
            request_update_viewer,
            data_sync::instr_create,
            data_sync::instr_change,
            data_sync::instr_delete,
            data_sync::instr_move,
            data_sync::instr_rename,
            data_sync::points_create,
            data_sync::points_change,
            data_sync::points_delete,
            data_sync::points_rename,
        ])
        .setup(setup)
        .run(tauri::generate_context!())
        .expect("Error while running Morphology Wizard");
}

mod menu {
    use tauri::{AppHandle, CustomMenuItem, Manager, Menu, Submenu};

    /// Update the text on a menu item for all windows.
    #[tauri::command(rename_all = "snake_case")]
    pub fn set_item(app: AppHandle, item: &str, text: &str) -> Result<(), String> {
        for window in app.windows().values() {
            if let Some(menu_item) = window.menu_handle().try_get_item(item) {
                menu_item.set_title(text).map_err(|err| err.to_string())?;
            }
        }
        Ok(())
    }

    pub fn main_window() -> Menu {
        Menu::new().add_submenu(file()).add_submenu(edit()).add_submenu(help())
    }
    pub fn viewer_window() -> Menu {
        Menu::new().add_submenu(file()).add_submenu(view()).add_submenu(help())
    }
    fn file() -> Submenu {
        Submenu::new(
            "File",
            Menu::new()
                .add_item(CustomMenuItem::new("new".to_string(), "New").accelerator("Ctrl+N"))
                .add_item(CustomMenuItem::new("open".to_string(), "Open").accelerator("Ctrl+O"))
                .add_item(CustomMenuItem::new("save".to_string(), "Save").accelerator("Ctrl+S"))
                .add_item(CustomMenuItem::new("save-as".to_string(), "Save As ...").accelerator("Ctrl+Shift+S"))
                .add_item(CustomMenuItem::new("swc".to_string(), "Export SWC"))
                // .add_item(CustomMenuItem::new("nml".to_string(), "Export NeuroML"))
                .add_item(CustomMenuItem::new("nrn".to_string(), "Export NEURON"))
                .add_item(CustomMenuItem::new("quit".to_string(), "Quit").accelerator("Ctrl+Q")),
        )
    }
    fn edit() -> Submenu {
        Submenu::new(
            "Windows",
            Menu::new()
                .add_item(CustomMenuItem::new("instr".to_string(), "Growth Instructions").accelerator("Ctrl+1"))
                .add_item(CustomMenuItem::new("points".to_string(), "Carrier Points").accelerator("Ctrl+2"))
                .add_item(CustomMenuItem::new("viewer".to_string(), "Preview Morphology").accelerator("Ctrl+3")),
        )
    }
    fn view() -> Submenu {
        Submenu::new(
            "View",
            Menu::new()
                .add_item(CustomMenuItem::new("live".to_string(), "Enable Live Viewer"))
                .add_item(CustomMenuItem::new("camera".to_string(), "Reset Camera"))
                .add_item(CustomMenuItem::new("regen".to_string(), "New Carrier Points"))
                .add_item(CustomMenuItem::new("full".to_string(), "Fullscreen").accelerator("F11")),
        )
    }
    fn help() -> Submenu {
        Submenu::new(
            "Help",
            Menu::new()
                // .add_item(CustomMenuItem::new("docs".to_string(), "Documentation").accelerator("Ctrl+H"))
                .add_item(CustomMenuItem::new("bugs".to_string(), "Report a Bug"))
                // .add_item(CustomMenuItem::new("home".to_string(), "Visit Homepage"))
                .add_item(CustomMenuItem::new("about".to_string(), "About Morphology Wizard")),
        )
    }
}

fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    // Immediately close the extra windows. Otherwise they prevents the app from exiting cleanly.
    if let Some(window) = app.get_window("viewer") {
        window.close()?;
    }
    if let Some(window) = app.get_window("about") {
        window.close()?;
    }
    // Register keyboard shortcuts that aren't covered by the menu's accelerator keys.
    let mut shortcuts = app.global_shortcut_manager();
    let app_handle = app.app_handle();
    shortcuts.register("F11", move || fullscreen(app_handle.clone()))?;

    // Auto-save every 60 seconds.
    tauri::async_runtime::spawn(auto_save_recurring(app.app_handle(), 60));

    Ok(())
}

fn window_event(event: GlobalWindowEvent) {
    match event.event() {
        WindowEvent::Destroyed => {
            if event.window().label() == "wizard" {
                let app = event.window().app_handle();
                auto_save(app.clone());
                quit(app) // If the main window closes then quit the whole application.
            }
        }
        _ => {}
    }
}

// Closes all windows.
fn quit(app: AppHandle) {
    for window in app.windows().values() {
        // Ignore any errors since the program is already on the way out the door.
        let _ignored = window.close();
    }
}

fn menu_event(event: WindowMenuEvent) {
    let window = event.window();
    let app = window.app_handle();
    let menu_item_id = event.menu_item_id();
    match menu_item_id {
        "new" => new_file(app),
        "save" => save_file(app, false),
        "save-as" => save_file(app, true),
        "open" => open_file(window.clone()),
        "swc" => export_swc(app.state()),
        "nml" => export_nml(app.state()),
        "nrn" => export_nrn(app.state()),
        "quit" => quit(app),
        "viewer" => open_viewer(app),
        // "live" => live(app),
        "regen" => regen_points(app),
        "full" => fullscreen(app),
        "docs" => open_webpage("todo!"),
        "bugs" => open_webpage("https://github.com/ctrl-z-9000-times/morphology_wizard/issues"),
        "home" => open_webpage("todo!"),
        "about" => about(app),
        _ => window.emit("menu_event", menu_item_id).unwrap(),
    }
}

fn open_webpage(url: &str) {
    if let Err(err) = webbrowser::open(url) {
        dbg!(err);
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct AppState {
    // Save the core application data and keep it in-sync with the front-end.
    instructions: Vec<GuiInstruction>,
    carrier_points: HashMap<String, CarrierPoints>,

    // Cache the results of the most recent run, as well as the instructions
    // that generated them. These are needed by the viewer and the export functions.
    nodes: Vec<Node>,
    instr_cache: Vec<Instruction>,
    // Save the carrier points and re-use them as long as possible. This is
    // indexed by (instruction-name, carrier-point-name) so that instructions
    // can share carrier point volumes and have different random points within
    // the shared volume.
    points_cache: HashMap<(String, String), Vec<[f64; 3]>>,

    /// Save file for the current session.
    current_file: Option<PathBuf>,
}

impl AppState {
    /// Returns JSON.
    fn get_data(&self) -> String {
        // Make an identical save file structure that holds references instead of
        // owning data to reduce unnecessary copying.
        #[derive(Serialize)]
        struct SaveFileRef<'a> {
            instructions: &'a [GuiInstruction],
            carrier_points: Vec<&'a CarrierPoints>,
        }
        serde_json::to_string(&SaveFileRef {
            instructions: &self.instructions,
            carrier_points: self.carrier_points.values().collect(),
        })
        .unwrap()
    }

    fn set_data(&mut self, instructions: Vec<GuiInstruction>, carrier_points: Vec<CarrierPoints>) {
        self.instructions = instructions;
        self.carrier_points = carrier_points
            .into_iter()
            .map(|points| (points.name().to_string(), points))
            .collect();
        //
        self.clear_cache();
    }

    fn clear_cache(&mut self) {
        self.points_cache.clear();
    }

    fn update_nodes(&mut self) {
        // Make instruction name->index look-up table.
        let instruction_names: HashMap<&str, u32> = self
            .instructions
            .iter()
            .enumerate()
            .map(|(index, instr)| (instr.name(), index as u32))
            .collect();
        // Fixup and transform the instructions into the morphology wizard's internal data structure.
        self.instr_cache = self
            .instructions
            .iter()
            .map(|instr| {
                // Map the roots from instruction names to indices.
                let roots = instr
                    .roots()
                    .iter()
                    // Ignore missing (deleted) roots.
                    .filter_map(|name| instruction_names.get(name.as_str()).cloned())
                    .collect();
                //
                let mut carrier_points = vec![];
                for name in instr.carrier_points() {
                    if let Some(parameters) = self.carrier_points.get(name) {
                        let points = self
                            .points_cache
                            .entry((instr.name().to_string(), name.to_string()))
                            .or_insert_with(|| parameters.generate_points());
                        carrier_points.extend_from_slice(points);
                    }
                }
                //
                instr.instruction(carrier_points, roots)
            })
            .collect();
        // Ignore invalid roots.
        for (index, instr) in self.instr_cache.iter_mut().enumerate() {
            instr.roots.retain(|&r| r < index as u32);
        }
        // Run the core algorithm.
        self.nodes = create(&self.instr_cache)
            .map_err(|err| popup_error_message(&err.to_string()))
            .unwrap_or(vec![]);
        // Free the combined carrier point lists, since they're re-computed
        // every time the algorithm runs anyways. Otherwise they would be
        // transmitted to the front-end even though they're not needed.
        for instr in &mut self.instr_cache {
            std::mem::take(&mut instr.carrier_points);
        }
    }
}

/// The front-end pushes all data changes to the back-end as soon as they happen using these commands.
mod data_sync {
    use super::{update_viewer, AppHandle, AppState, CarrierPoints, GuiInstruction, Manager, Mutex, State};

    #[tauri::command(rename_all = "snake_case")]
    pub fn instr_create(app: AppHandle, instr: GuiInstruction) {
        {
            let app_state: State<Mutex<AppState>> = app.state();
            let app_state = &mut app_state.lock().unwrap();
            app_state.instructions.push(instr);
        }
        update_viewer(app);
    }
    #[tauri::command(rename_all = "snake_case")]
    pub fn instr_change(app: AppHandle, index: usize, instr: GuiInstruction) {
        {
            let app_state: State<Mutex<AppState>> = app.state();
            let app_state = &mut app_state.lock().unwrap();
            debug_assert_eq!(app_state.instructions[index].name(), instr.name());
            app_state.instructions[index] = instr;
        }
        update_viewer(app);
    }
    #[tauri::command(rename_all = "snake_case")]
    pub fn instr_delete(app: AppHandle, index: usize) {
        {
            let app_state: State<Mutex<AppState>> = app.state();
            let app_state = &mut app_state.lock().unwrap();
            app_state.instructions.remove(index);
        }
        update_viewer(app);
    }
    #[tauri::command(rename_all = "snake_case")]
    pub fn instr_move(app: AppHandle, index_1: usize, index_2: usize) {
        {
            let app_state: State<Mutex<AppState>> = app.state();
            let app_state = &mut app_state.lock().unwrap();
            app_state.instructions.swap(index_1, index_2);
        }
        update_viewer(app);
    }
    #[tauri::command(rename_all = "snake_case")]
    pub fn instr_rename(app: AppHandle, old_name: &str, new_name: &str) {
        {
            let app_state: State<Mutex<AppState>> = app.state();
            let app_state = &mut app_state.lock().unwrap();
            for instr in app_state.instructions.iter_mut() {
                // Update the instruction's name.
                if instr.name() == old_name {
                    instr.set_name(new_name.to_string())
                }
                // Update all cross references to the instruction.
                for root in instr.roots_mut() {
                    if root == old_name {
                        *root = new_name.to_string();
                    }
                }
            }
            // Update the carrier points cache.
            app_state.points_cache = app_state
                .points_cache
                .drain()
                .map(|((mut instr, carrier), points)| {
                    if instr == old_name {
                        instr = new_name.to_string();
                    }
                    ((instr, carrier), points)
                })
                .collect();
        }
        update_viewer(app);
    }
    #[tauri::command(rename_all = "snake_case")]
    pub fn points_create(app: AppHandle, points: CarrierPoints) {
        {
            let app_state: State<Mutex<AppState>> = app.state();
            let app_state = &mut app_state.lock().unwrap();
            app_state.carrier_points.insert(points.name().to_string(), points);
        }
        update_viewer(app);
    }
    #[tauri::command(rename_all = "snake_case")]
    pub fn points_change(app: AppHandle, points: CarrierPoints) {
        {
            let app_state: State<Mutex<AppState>> = app.state();
            let app_state = &mut app_state.lock().unwrap();
            // Clear the carrier points cache.
            let name = points.name();
            app_state
                .points_cache
                .retain(|(_instr, carrier), _points| carrier != name);
            // Updated the primary data.
            let entry = app_state.carrier_points.get_mut(name).unwrap();
            *entry = points;
        }
        update_viewer(app);
    }
    #[tauri::command(rename_all = "snake_case")]
    pub fn points_delete(app: AppHandle, name: &str) {
        {
            let app_state: State<Mutex<AppState>> = app.state();
            let app_state = &mut app_state.lock().unwrap();
            let deleted = app_state.carrier_points.remove(name);
            // Clear the carrier points cache.
            app_state
                .points_cache
                .retain(|(_instr, carrier), _points| carrier != name);
            debug_assert!(deleted.is_some());
        }
        update_viewer(app);
    }
    #[tauri::command(rename_all = "snake_case")]
    pub fn points_rename(app: AppHandle, old_name: &str, new_name: &str) {
        {
            let app_state: State<Mutex<AppState>> = app.state();
            let app_state = &mut app_state.lock().unwrap();
            // Update the carrier points name.
            let mut points = app_state.carrier_points.remove(old_name).unwrap();
            points.set_name(new_name.to_string());
            app_state.carrier_points.insert(new_name.to_string(), points);
            // Update all cross references to the old name.
            for instr in app_state.instructions.iter_mut() {
                for points in instr.carrier_points_mut() {
                    if points == old_name {
                        *points = new_name.to_string();
                    }
                }
            }
            // Update the carrier points cache.
            app_state.points_cache = app_state
                .points_cache
                .drain()
                .map(|((instr, mut carrier), points)| {
                    if carrier == old_name {
                        carrier = new_name.to_string();
                    }
                    ((instr, carrier), points)
                })
                .collect();
        }
        update_viewer(app);
    }
}

fn new_file(app: AppHandle) {
    let title = "Create new model?";
    let message = "This will discard the current model.";
    let window = app.get_window("wizard").unwrap();
    confirm(Some(&window.clone()), title, message, move |yes| {
        if yes {
            {
                let app_state = &mut app.state::<Mutex<AppState>>();
                let mut app_state = app_state.lock().unwrap();
                app_state.instructions.clear();
                app_state.carrier_points.clear();
                app_state.current_file.take();
            }
            window.emit("set_data", ()).unwrap();
        }
    })
}

fn save_file(app: AppHandle, save_as: bool) {
    let json = {
        let app_state = &app.state::<Mutex<AppState>>();
        let app_state = app_state.lock().unwrap();
        let json = app_state.get_data();
        // If use clicked "Save As" then ignore the file that the model was
        // loaded from and always prompt them for a new file
        if !save_as {
            if let Some(path) = &app_state.current_file {
                write(path, &json);
                return;
            }
        }
        json
    };
    let mut dialog = FileDialogBuilder::new().set_file_name("morphology.json");
    if let Some(path) = save_directory() {
        dialog = dialog.set_directory(path);
    }
    dialog.save_file(move |path| {
        if let Some(path) = path {
            write(&path, &json);
            //
            let app_state = &app.state::<Mutex<AppState>>();
            let app_state = &mut app_state.lock().unwrap();
            app_state.current_file = Some(path);
        }
    });
}

fn auto_save(app: AppHandle) {
    let Some(path) = autosave_path(&app) else { return };
    let json = {
        let app_state = &app.state::<Mutex<AppState>>();
        let app_state = app_state.lock().unwrap();
        app_state.get_data()
    };
    if let Err(err) = std::fs::write(&path, json) {
        eprintln!("Error: failed to write autosave file {err}");
    }
}

async fn auto_save_recurring(app: AppHandle, period_seconds: u64) {
    use tokio::time::{sleep, Duration};
    loop {
        sleep(Duration::from_secs(period_seconds)).await;
        auto_save(app.clone());
    }
}

/// Default directory to for save files.
fn save_directory() -> Option<PathBuf> {
    tauri::api::path::desktop_dir()
}

/// Locate the autosave file.
fn autosave_path(app: &AppHandle) -> Option<PathBuf> {
    let Some(mut path) = app.path_resolver().app_data_dir() else {
        eprintln!("Error: autosave can not find application data directory");
        return None;
    };
    path.push("autosave.json");
    Some(path)
}

fn open_file(window: Window) {
    let mut dialog = FileDialogBuilder::new()
        .set_file_name("morphology.json")
        .add_filter(".json", &["json"]);
    if let Some(path) = save_directory() {
        dialog = dialog.set_directory(path);
    }
    dialog.pick_file(move |path| {
        let Some(path) = path else { return };
        //
        let Ok(json) = std::fs::read_to_string(&path)
            .map_err(|err| popup_error_message(&format!("Failed to open file {path:?}\n{err}")))
        else {
            return;
        };
        //
        let Ok(data) = serde_json::from_str::<SaveFile>(&json)
            .map_err(|err| popup_error_message(&format!("Failed to open file {path:?}\n{err}")))
        else {
            return;
        };
        // Send the data to the front-end.
        window.emit("set_data", &json).unwrap();
        // Save the data in the back-end.
        {
            let app_state = window.state::<Mutex<AppState>>();
            let app_state = &mut app_state.lock().unwrap();
            app_state.set_data(data.instructions, data.carrier_points);
            app_state.current_file = Some(path);
        }
        // Compute the new nodes and send them to the viewer.
        update_viewer(window.app_handle());
    })
}

#[tauri::command(rename_all = "snake_case")]
fn auto_load(window: Window) {
    let app = window.app_handle();
    let Some(path) = autosave_path(&app) else { return };

    // Check if autosave file is missing to suppress obnoxious error messages.
    if !path.exists() {
        return;
    }
    // Load the saved data.
    let json = match std::fs::read_to_string(&path) {
        Ok(json) => json,
        Err(err) => {
            eprintln!("Error: failed to open autosave file: {err}");
            return;
        }
    };
    let data = match serde_json::from_str::<SaveFile>(&json) {
        Ok(data) => data,
        Err(err) => {
            eprintln!("Error: failed to parse autosave file: {err}");
            return;
        }
    };
    // Send the data to the front-end.
    window.emit("set_data", &json).unwrap();
    // Save the data in the back-end.
    let app_state = app.state::<Mutex<AppState>>();
    let app_state = &mut app_state.lock().unwrap();
    app_state.set_data(data.instructions, data.carrier_points);
}

fn export_swc(app_state: State<Mutex<AppState>>) {
    // Generate the SWC files.
    let swc = {
        // First update the cached nodes.
        let app_state = &mut app_state.lock().unwrap();
        app_state.update_nodes();
        morphology_wizard::export_swc(&app_state.instr_cache, &app_state.nodes)
    };
    // Prompt for SWC file path.
    let mut dialog = FileDialogBuilder::new()
        .set_title("Export SWC File")
        .set_file_name("morphology.swc");
    if let Some(path) = save_directory() {
        dialog = dialog.set_directory(path);
    }
    dialog.save_file(move |path| {
        if let Some(path) = path {
            write(&path, &swc);
        }
    });
}

fn export_nml(app_state: State<Mutex<AppState>>) {
    // Generate the NML file.
    let nml = {
        // First update the cached nodes.
        let app_state = &mut app_state.lock().unwrap();
        app_state.update_nodes();
        morphology_wizard::export_nml(&app_state.instr_cache, &app_state.nodes)
    };
    // Prompt for NML file path.
    let mut dialog = FileDialogBuilder::new()
        .set_title("Export NeuroML File")
        .set_file_name("morphology.nml");
    if let Some(path) = save_directory() {
        dialog = dialog.set_directory(path);
    }
    dialog.save_file(move |path| {
        if let Some(path) = path {
            write(&path, &nml);
        }
    });
}

fn export_nrn(app_state: State<Mutex<AppState>>) {
    // Generate the NEURON script.
    let nrn = {
        // First update the cached nodes.
        let app_state = &mut app_state.lock().unwrap();
        app_state.update_nodes();
        morphology_wizard::export_nrn(&app_state.instr_cache, &app_state.nodes)
    };
    // Prompt for python script file path.
    let mut dialog = FileDialogBuilder::new()
        .set_title("Export NEURON Script")
        .set_file_name("morphology.py");
    if let Some(path) = save_directory() {
        dialog = dialog.set_directory(path);
    }
    dialog.save_file(move |path| {
        if let Some(path) = path {
            write(&path, &nrn);
        }
    });
}

fn write(path: &Path, data: &str) {
    let result = std::fs::write(path, data);
    if let Err(err) = result {
        popup_error_message(&format!("Failed to save to file {path:?}\n{err}"));
    }
}

fn popup_error_message(message: &str) {
    use tauri::api::dialog;
    let title = "Error!";
    dialog::MessageDialogBuilder::new(title, message)
        .kind(dialog::MessageDialogKind::Error)
        .buttons(dialog::MessageDialogButtons::Ok)
        .show(|_ok| {});
}

fn open_viewer(app: AppHandle) {
    // If the viewer window is already open then simply bring it into focus.
    if let Some(window) = app.get_window("viewer") {
        let _ = window.unminimize().map_err(|err| dbg!(err));
        let _ = window.set_focus().map_err(|err| dbg!(err));
    } else {
        // Open the viewer window.
        if let Err(err) = WindowBuilder::from_config(
            &app,
            app.config()
                .tauri
                .windows
                .iter()
                .find(|w| w.label == "viewer")
                .unwrap()
                .clone(),
        )
        .menu(menu::viewer_window())
        .visible(true)
        .build()
        {
            dbg!(err);
            return;
        }
    }
}

/// Receive this command from the viewer window once it's ready to accept its initial geometry.
#[tauri::command(rename_all = "snake_case")]
fn request_update_viewer(app: AppHandle) {
    update_viewer(app);
}

fn fullscreen(app: AppHandle) {
    // If the viewer window isn't open then F11 should not do anything.
    let Some(window) = app.get_window("viewer") else { return };

    // The F11 shortcut should only work if the viewer window is selected.
    let Ok(is_focused) = window.is_focused().map_err(|err| dbg!(err)) else {
        return;
    };
    if !is_focused {
        return;
    }

    // Get the current fullscreen state.
    let Ok(is_fullscreen) = window.is_fullscreen().map_err(|err| dbg!(err)) else {
        return;
    };

    // Toggle fullscreen.
    if let Err(err) = window.set_fullscreen(!is_fullscreen) {
        dbg!(err);
        return;
    }
    // Toggle the menu bar.
    if is_fullscreen {
        if let Err(err) = window.menu_handle().show() {
            dbg!(err);
        }
    } else {
        if let Err(err) = window.menu_handle().hide() {
            dbg!(err);
        }
    }
    /*
    // This crashes the program because after 15 years wayland still hasn't
    // reached feature parity with X11.

    // Enable the ESC key when it's in fullscreen mode.
    let mut shortcuts = app.global_shortcut_manager();
    if is_fullscreen {
        if let Err(err) = shortcuts.unregister("ESC") {
            dbg!(err);
        }
    } else {
        if let Err(err) = shortcuts.register("ESC", move || fullscreen(app.clone())) {
            dbg!(err);
        }
    }
    */
}

/// Regenerate the morphology using new carrier points.
fn regen_points(app: AppHandle) {
    {
        let app_state: State<Mutex<AppState>> = app.state();
        let app_state = &mut app_state.lock().unwrap();
        app_state.clear_cache();
    }
    update_viewer(app);
}

fn update_viewer(app: AppHandle) {
    // If the viewer window is closed then do nothing.
    let Some(window) = app.get_window("viewer") else { return };

    // Update the app state, recompute the nodes as necessary.
    let app_state: State<Mutex<AppState>> = app.state();
    let app_state = &mut app_state.lock().unwrap();
    app_state.update_nodes();

    // Send the updated data to the viewer window.
    #[derive(Serialize, Copy, Clone)]
    struct Payload<'a> {
        instructions: &'a [Instruction],
        nodes: &'a [Node],
    }
    window
        .emit(
            "update_viewer",
            Payload {
                instructions: &app_state.instr_cache,
                nodes: &app_state.nodes,
            },
        )
        .unwrap()
}

fn about(app: AppHandle) {
    // Check that the window is not already open.
    if let Some(window) = app.get_window("about") {
        let _ = window.unminimize().map_err(|err| dbg!(err));
        let _ = window.set_focus().map_err(|err| dbg!(err));
        return;
    }
    // Open the about window.
    let _ = WindowBuilder::from_config(
        &app,
        app.config()
            .tauri
            .windows
            .iter()
            .find(|w| w.label == "about")
            .unwrap()
            .clone(),
    )
    .menu(tauri::Menu::new()) // Suppress inheriting the menu from the main window.
    .visible(true)
    .build()
    .map_err(|err| dbg!(err));
}
