const { listen } = window.__TAURI__.event
const { invoke } = window.__TAURI__.tauri
import { EntityManager, set_data } from "entity_maker"
import { instructions_cfg }   from "instructions_cfg"
import { carrier_points_cfg } from "carrier_points_cfg"

// Generate the body of the application.
const instructions = new EntityManager(instructions_cfg)
instructions_div.appendChild(instructions.domElement)

const carrier_points = new EntityManager(carrier_points_cfg)
carrier_points_div.appendChild(carrier_points.domElement)

// Switch to the growth instructions tab.
function instructions_tab() {
    instructions_div.style.display = ""
    carrier_points_div.style.display = "none"
}

// Switch to the carrier points tab.
function carrier_points_tab() {
    instructions_div.style.display = "none"
    carrier_points_div.style.display = ""
}

// Listen for user tab-switching events.
await listen("menu_event", (event) => {
    if (event.payload == "instr") {
        instructions_tab()
    }
    else if (event.payload == "points") {
        carrier_points_tab()
    }
    else {
        console.warn(`Unhandled menu item ${event.payload}`)
    }
})

// Initially show the growth instructions tab.
instructions_tab()

// Detect changes to the entity data and preemptively transmit them to the back-end.
instructions.create_hooks.push((instr) => {
    invoke("instr_create", {instr: instr})
})
instructions.delete_hooks.push(() => {
    invoke("instr_delete", {index: instructions.get_selected_index()})
})
instructions.move_hooks.push((index_1, index_2) => {
    invoke("instr_move", {index_1: index_1, index_2: index_2})
})
instructions.change_hooks.push((instr) => {
    invoke("instr_change", {index: instructions.get_selected_index(), instr: instr})
})
instructions.rename_hooks.push((old_name, new_name) => {
    invoke("instr_rename", {old_name: old_name, new_name: new_name})
})
carrier_points.create_hooks.push((points) => {
    invoke("points_create", {points: points})
})
carrier_points.change_hooks.push((points) => {
    invoke("points_change", {points: points})
})
carrier_points.delete_hooks.push((points) => {
    invoke("points_delete", {name: points.name})
})
carrier_points.rename_hooks.push((old_name, new_name) => {
    invoke("points_rename", {old_name: old_name, new_name: new_name})
})

// Allow the back-end to call "set_data".
await listen("set_data", (event) => {
    set_data(event.payload)
})

// The back-end can't load the autosave file until the front-end has setup the "set_data" listener.
invoke("auto_load")
