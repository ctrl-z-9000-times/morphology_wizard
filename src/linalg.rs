//! Basic linear algebra routines for working in three dimensions.

#![allow(dead_code)]

pub fn distance(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    mag(&sub(a, b))
}

pub fn angle(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    (dot(a, b) / mag(a) / mag(b)).acos()
}

pub fn dot(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Vector length.
pub fn mag(x: &[f64; 3]) -> f64 {
    (x[0].powi(2) + x[1].powi(2) + x[2].powi(2)).sqrt()
}

/// Divide the vector by its length, returns the original length.
pub fn normalize(x: &mut [f64; 3]) -> f64 {
    let mag = mag(x);
    let scale = 1.0 / mag;
    x[0] *= scale;
    x[1] *= scale;
    x[2] *= scale;
    mag
}

pub fn sub(a: &[f64; 3], b: &[f64; 3]) -> [f64; 3] {
    [b[0] - a[0], b[1] - a[1], b[2] - a[2]]
}

pub fn add(a: &[f64; 3], b: &[f64; 3]) -> [f64; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

pub fn scale(x: &[f64; 3], f: f64) -> [f64; 3] {
    [f * x[0], f * x[1], f * x[2]]
}

pub fn cross(a: &[f64; 3], b: &[f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Calculate a rotation matrix to transform from vector A to vector B.
///
/// Both argument must already be normalized (magnitude of one).
///
/// https://math.stackexchange.com/questions/180418/calculate-rotation-matrix-to-align-vector-a-to-vector-b-in-3d/476311#476311
pub fn rotate_align(a: &[f64; 3], b: &[f64; 3]) -> [[f64; 3]; 3] {
    let c = dot(a, b); // Cosine of angle
    let c1 = c + 1.0;
    if c1.abs() <= f64::EPSILON {
        todo!()
    } else {
        let v = cross(a, b);
        // Skew symmetric cross-product matrix.
        let vx = [[0.0, -v[2], v[1]], [v[2], 0.0, -v[0]], [-v[1], v[0], 0.0]];
        // Calculate: identity-matrix + vx + vx^2 / c1
        let mut vx2 = mat3x3_sqr(&vx);
        max3x3_div_scalar(&mut vx2, c1);
        max3x3_add(&mut vx2, &vx);
        vx2[0][0] += 1.0;
        vx2[1][1] += 1.0;
        vx2[2][2] += 1.0;
        vx2
    }
}

fn mat3x3_sqr(mat: &[[f64; 3]; 3]) -> [[f64; 3]; 3] {
    let mut msqr = [[0.0; 3]; 3];
    for row in 0..3 {
        for col in 0..3 {
            for inner in 0..3 {
                msqr[row][col] += mat[row][inner] * mat[inner][col];
            }
        }
    }
    msqr
}

fn max3x3_add(a: &mut [[f64; 3]; 3], b: &[[f64; 3]; 3]) {
    for row in 0..3 {
        for col in 0..3 {
            a[row][col] += b[row][col];
        }
    }
}

fn max3x3_div_scalar(mat: &mut [[f64; 3]; 3], div: f64) {
    let factor = 1.0 / div;
    for row in 0..3 {
        for col in 0..3 {
            mat[row][col] *= factor;
        }
    }
}

pub fn vec_mat_mult(vec: &mut [f64; 3], mat: &[[f64; 3]; 3]) {
    let mut result = [0.0; 3];
    for col in 0..3 {
        for inner in 0..3 {
            result[col] += vec[inner] * mat[col][inner];
        }
    }
    *vec = result;
}
