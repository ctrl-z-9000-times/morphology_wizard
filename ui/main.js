const { listen } = window.__TAURI__.event
const { invoke } = window.__TAURI__.tauri
import { EntityManager, get_entity_data, set_entity_data} from 'entity_maker'
import { instructions_cfg }   from 'instructions_cfg'
import { carrier_points_cfg } from 'carrier_points_cfg'
import { update_viewer } from 'viewer'

// Generate the body of the application.
const instructions = new EntityManager(instructions_cfg)
instructions_frame.appendChild(instructions.frame)

const carrier_points = new EntityManager(carrier_points_cfg)
carrier_points_frame.appendChild(carrier_points.frame)

// Switch to the growth instructions tab.
function instructions_tab() {
    instructions_frame.style.display = ""
    carrier_points_frame.style.display = "none"
}

// Switch to the carrier points tab.
function carrier_points_tab() {
    instructions_frame.style.display = "none"
    carrier_points_frame.style.display = ""
}

// Initially show the growth instructions tab.
instructions_tab()

// Automatic Save & Load uses the webview's built-in local storage.
function auto_save() {
    localStorage.setItem("morphology_wizard_autosave", get_entity_data())
}
function auto_load() {
    // If data is missing then this will fallback to default initial state.
    set_entity_data(localStorage.getItem("morphology_wizard_autosave"))
}

// Auto-save on a regular basis, in case of program crash.
setInterval(auto_save, 60 * 1000); // The delay is in milliseconds.

// Auto-save on page hide (minimize or suspend).
document.addEventListener("visibilitychange", () => {
    if (document.hidden) {
        auto_save()
    }
})

// Auto-save on tauri shutdown (tauri does not call the onbeforeunload hook).
async function register_save_on_exit() {
    await listen("tauri://close-requested", (event) => {
        auto_save()
    });
}
register_save_on_exit()

// Auto-load at startup from persistent app storage.
auto_load()

// 
var enable_preview = true
function toggle_preview() {
    enable_preview = !enable_preview
    if (enable_preview) {
        viewport_frame.style.display = ""
    }
    else {
        viewport_frame.style.display = "none"
    }
    invoke('set_live_preview', {enable: enable_preview})
}

// 
async function register_menu_callbacks() {
    await listen("menu_event", (event) => {
        if (event.payload == "new") {
            const result = confirm('Create new model?\nThis will discard the current model.')
            result.then((choice) => {
                if (choice) {
                    set_entity_data()
                }
            })
        }
        else if (event.payload == "save") {
            invoke('save', {instructions: get_entity_data()})
        }
        else if (event.payload == "load") {
            const reader = new FileReader()
            reader.onload = (event => set_entity_data(event.target.result))
            reader.readAsText(file)
        }
        else if (event.payload == "swc") {
            invoke('export_swc', {instructions: get_entity_data(), nodes: []})
                .then((data) => {save_file_dialog(data, "text/plain", "morphology.swc")})
        }
        else if (event.payload == "nml") {
            invoke('export_nml', {instructions: get_entity_data(), nodes: []})
                .then((data) => {save_file_dialog(data, "application/xml", "morphology.nml")})
        }
        else if (event.payload == "nrn") {
            invoke('export_nrn', {instructions: get_entity_data(), nodes: []})
                .then((data) => {save_file_dialog(data, "text/plain", "morphology.py")})
        }
        else if (event.payload == "quit") {
            auto_save()
            window.close()
        }
        else if (event.payload == "instr") {
            instructions_tab()
        }
        else if (event.payload == "points") {
            carrier_points_tab()
        }
        else if (event.payload == "preview") {
            toggle_preview()
        }
        else if (event.payload == "generate") {
            invoke('generate_morphology', {save_file: get_entity_data()})
                .then((nodes) => {update_viewer(instructions.get_data(), nodes)})
        }
        else {
            console.warn(`Unimplemented menu item ${event.payload}`)
        }
    })
}
register_menu_callbacks()
