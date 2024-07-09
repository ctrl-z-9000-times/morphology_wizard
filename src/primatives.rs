use crate::linalg;
use std::f32::consts::PI;

///
pub fn cylinder(a: &[f32; 3], b: &[f32; 3], a_diam: f32, b_diam: f32, num_slices: u32) -> (Vec<[f32; 3]>, Vec<u32>) {
    let mut vertices = Vec::<[f32; 3]>::with_capacity(2 * (num_slices as usize + 1));
    let a_radius = 0.5 * a_diam;
    let b_radius = 0.5 * b_diam;
    let mut cyl_vec = linalg::sub(a, b);
    let length = linalg::mag(&cyl_vec);
    // Generate the vertices for a standard upright cylinder.
    for slice in 0..=num_slices {
        let ang_frq = 2.0 * PI * (slice as f32 / num_slices as f32);
        let x = ang_frq.cos();
        let y = ang_frq.sin();
        vertices.push([x * a_radius, y * a_radius, 0.0]);
        vertices.push([x * b_radius, y * b_radius, length]);
    }
    // Rotate the vertices so that the cylinder's tip points towards B.
    cyl_vec[0] /= length;
    cyl_vec[1] /= length;
    cyl_vec[2] /= length;
    let ref_vec = [0.0, 0.0, 1.0];
    let rot_mat = linalg::rotate_align(&ref_vec, &cyl_vec);
    for vertex in vertices.iter_mut() {
        linalg::vec_mat_mult(vertex, &rot_mat);
    }
    // Offset the vertices so that the cylinder's base is at A.
    for vertex in vertices.iter_mut() {
        vertex[0] += a[0];
        vertex[1] += a[1];
        vertex[2] += a[2];
    }
    // Make the index array.
    let mut indices = Vec::with_capacity(2 * 3 * num_slices as usize);
    for i in 0..num_slices {
        indices.push(2 * i + 2);
        indices.push(2 * i + 1);
        indices.push(2 * i);
        indices.push(2 * i + 1);
        indices.push(2 * i + 2);
        indices.push(2 * i + 3);
    }
    (vertices, indices)
}

pub fn sphere(center: &[f32; 3], radius: f32, slices: u32) -> (Vec<[f32; 3]>, Vec<u32>) {
    let stacks = slices;
    let mut vertices = Vec::<[f32; 3]>::with_capacity(((stacks + 1) * (slices + 1)) as usize);
    // Calculate the Vertices.
    for i in 0..=stacks {
        let phi = PI * (i as f32 / stacks as f32);
        for j in 0..=slices {
            let theta = 2.0 * PI * (j as f32 / slices as f32);
            vertices.push([theta.cos() * phi.sin(), phi.cos(), theta.sin() * phi.sin()]);
        }
    }
    // Scale and offset the vertices.
    for vertex in vertices.iter_mut() {
        vertex[0] *= radius;
        vertex[1] *= radius;
        vertex[2] *= radius;
        vertex[0] += center[0];
        vertex[1] += center[1];
        vertex[2] += center[2];
    }
    // Calculate The Index Positions
    let num_triangles = 2 * slices * (stacks + 1);
    let mut indices = Vec::with_capacity(3 * num_triangles as usize);
    for i in 0..slices * (stacks + 1) {
        indices.push(i);
        indices.push(i + slices + 1);
        indices.push(i + slices);
        indices.push(i + slices + 1);
        indices.push(i);
        indices.push(i + 1);
    }
    (vertices, indices)
}
