const { listen } = window.__TAURI__.event
const { invoke } = window.__TAURI__.tauri
import * as THREE from 'three'
import { ArcballControls } from 'three/addons/controls/ArcballControls.js'
import { FlyControls }     from 'three/addons/controls/FlyControls.js'

// Create the basic structures for 3D rendering.
const renderer = new THREE.WebGLRenderer()
const scene    = new THREE.Scene()
viewport_div.appendChild(renderer.domElement)
// Main render loop, called every tick.
renderer.setAnimationLoop(() => {
    renderer.render(scene, camera3d)
})
// Log performance diagnostics to the console.
export function log_info() {
    console.log(renderer.info)
}

// Setup a 3D perspective camera.
let camera_fov     = 50         // Field Of View.
let camera_dims    = [640, 480] // Width & Height of render target.
let camera_dist    = 0.1        // Camera image plane distance.
let view_distance  = 10000      // Maximum viewing distance.
const camera3d     = new THREE.PerspectiveCamera(camera_fov, camera_dims[0] / camera_dims[1], camera_dist, view_distance)
renderer.setSize(camera_dims[0], camera_dims[1])

// Set the width and height of the render target.
// Argument "dims" is array of [width, height] in pixels.
// If argument is missing then this uses the window's width and height.
function update_camera_size(dims) {
    if (dims) {
        camera_dims = dims
    }
    else {
        camera_dims = [window.innerWidth, window.innerHeight]
    }
    renderer.setSize(camera_dims[0], camera_dims[1])
    camera3d.aspect = camera_dims[0] / camera_dims[1]
    camera3d.updateProjectionMatrix()
}

// Resize the view-port to take up the whole window.
update_camera_size()
window.addEventListener('resize', () => {
    update_camera_size()
})

// Reset's the camera's position and orientation.
function reset_camera() {
    camera3d.position.set( 0, 20, 100 )
    camera3d.up.set(0, 1 ,0)
    camera3d.lookAt(0, 0, 0)
    camera3d.updateProjectionMatrix()
    controls.update()
}

// Setup the user control scheme.
let controls = null
function ball_control() {
    if (controls) {
        controls.dispose()
    }
    controls = new ArcballControls(camera3d, renderer.domElement, scene)
}
function fps_control() {
    if (controls) {
        controls.dispose()
    }
    controls = new FlyControls(camera3d, renderer.domElement)
}
// Initially use trackball style controls.
ball_control()
reset_camera()

// Setup the 3D geometry.
const meshes_by_instr = []

function clear_viewer() {
    for (const mesh_list of meshes_by_instr) {
        scene.remove(...mesh_list)
    }
    while (meshes_by_instr.length > 0) {
        meshes_by_instr.pop()
    }
}

function update_viewer(instructions, nodes) {

    clear_viewer()

    while (meshes_by_instr.length < instructions.length) {
        meshes_by_instr.push([])
    }

    const material = new THREE.MeshBasicMaterial( { color: 0x00ff00 } )

    for (const node of nodes) {
        if (node.parent_index > nodes.length) {
            const radius   = 0.5 * node.diameter
            const slices   = Math.max(2, (0.5 * radius).toFixed())
            const geometry = new THREE.SphereGeometry(radius, 2 * slices, slices)
            const mesh     = new THREE.Mesh(geometry, material)
            mesh.position.set(...node.coordinates)
            scene.add(mesh)
            meshes_by_instr[node.instruction_index].push(mesh)
        }
        else {
            const radius        = 0.5 * node.diameter
            const parent        = nodes[node.parent_index]
            const parent_radius = 0.5 * parent.diameter
            const max_radius    = Math.max(radius, parent_radius)
            const radial_slices = Math.max(6, (0.5 * max_radius).toFixed())
            const node_coords   = new THREE.Vector3(...node.coordinates)
            const parent_coords = new THREE.Vector3(...parent.coordinates)
            const height        = parent_coords.distanceTo(node_coords)
            const geometry      = new THREE.CylinderGeometry(radius, parent_radius, height, radial_slices)
            const mesh          = new THREE.Mesh(geometry, material)
            const cylinder_axis = THREE.Object3D.DEFAULT_UP
            const segment_axis  = node_coords.sub(parent_coords).normalize()
            mesh.translateOnAxis(segment_axis, 0.5 * height)
            mesh.quaternion.setFromUnitVectors(cylinder_axis, segment_axis)
            const translate     = new THREE.Object3D()
            translate.position.add(parent_coords)
            translate.add(mesh)
            scene.add(translate)
            meshes_by_instr[node.instruction_index].push(translate)
        }
    }
}

// The back-end can push new geometry at any time.
await listen("update_viewer", (event) => {
    update_viewer(event.payload.instructions, event.payload.nodes)
})

// 
await listen("menu_event", (event) => {
    if (event.payload == "camera") {
        reset_camera()
    }
    else {
        console.warn(`Unhandled menu item ${event.payload}`)
    }
})

// Request the initial geometry after that we've registered the listeners.
invoke("request_update_viewer")
