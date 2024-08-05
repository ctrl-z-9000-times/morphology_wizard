import * as THREE from 'three';

import { ArcballControls } from 'three/addons/controls/ArcballControls.js';

const scene = new THREE.Scene();
const camera = new THREE.PerspectiveCamera( 75, window.innerWidth / window.innerHeight, 0.1, 1000 );

const renderer = new THREE.WebGLRenderer();
// renderer.setSize( window.innerWidth, window.innerHeight );
renderer.setSize( 640, 480 );
renderer.setAnimationLoop( animate );
viewport_frame.appendChild( renderer.domElement );

camera.position.z = 5;

const materials = [] // TODO?
const meshes_by_instr = []

export function clear_viewer() {
    for (const mesh_list of meshes_by_instr) {
        scene.remove(...mesh_list)
    }
    while (meshes_by_instr.length > 0) {
        meshes_by_instr.pop()
    }
}

export function update_viewer(instructions, nodes) {

    clear_viewer()

    while (meshes_by_instr.length < instructions.length) {
        meshes_by_instr.push([])
    }

    const material = new THREE.MeshBasicMaterial( { color: 0x00ff00 } );

    for (const node of nodes) {
        if (node.parent_index > nodes.length) {
            const radius   = 0.5 * node.diameter
            const slices   = Math.max(2, (0.5 * radius).toFixed())
            const geometry = new THREE.SphereGeometry(radius, 2 * slices, slices);
            const mesh     = new THREE.Mesh(geometry, material);
            mesh.position.set(...node.coordinates)
            scene.add(mesh);
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
            scene.add(translate);
            meshes_by_instr[node.instruction_index].push(translate)
        }
    }
}

function animate() {

    renderer.render( scene, camera );

}


const controls = new ArcballControls( camera, renderer.domElement, scene );
// const controls = new FlyControls( camera, renderer.domElement );

controls.addEventListener('change', function () {
    renderer.render(scene, camera);
});

camera.position.set( 0, 20, 100 );
controls.update();
