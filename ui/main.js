const { invoke } = window.__TAURI__.tauri
const { listen } = window.__TAURI__.event
import { update_viewer } from '/viewer.js'

// Generate the body of the application.
const instructions = new EntityManager(instructions_cfg)
body.appendChild(instructions.frame)

const carrier_points = new EntityManager(carrier_points_cfg)
body.appendChild(carrier_points.frame)

generate_morphology.addEventListener("click", (e) => {
  const save_file = get_entity_data()
  invoke('generate_morphology', {save_file: save_file})
    .then((nodes) => {update_viewer(instructions.data, nodes)})
})

function reset() {
  const result = confirm('Create new model?\nThis will discard the current model.')
  result.then((choice) => {
    if (choice) {
      set_entity_data()
    }
  })
}

function save() {
  const data = get_entity_data()
  const blob = new Blob([data], {type : 'application/json'})
  const url  = URL.createObjectURL(blob)
  const link = document.createElement('a')
  link.href  = url
  link.download = "morphology.json"
  link.click()
  URL.revokeObjectURL(url)
}

function load(file) {
  const reader = new FileReader()
  reader.onload = (event => set_entity_data(event.target.result))
  reader.readAsText(file)
}

// Auto-load from persistent app storage or fallback to default initial state.
set_entity_data(localStorage.getItem("morphology_wizard_autosave"))
// Auto-save on a regular basis, in clase of program crash.
function auto_save() {
  localStorage.setItem("morphology_wizard_autosave", get_entity_data())
}
setInterval(auto_save, 30 * 1000);
// Auto-save on page hide (minimize or suspend).
document.addEventListener("visibilitychange", () => {
  if (document.hidden) {
    auto_save()
  }
})
// Auto-save on tauri shutdown (tauri does not call on beforeunload hook).
async function register_save_on_exit() {
  const unlisten = await listen("tauri://close-requested", (event) => {
    auto_save()
  });
}
register_save_on_exit()
