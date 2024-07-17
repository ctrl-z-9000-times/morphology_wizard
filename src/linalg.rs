//! Basic linear algebra routines for working in three dimensions.

#![allow(dead_code)]

use num_traits::{one, zero, Float, NumAssignOps};

pub fn distance<F: Float>(a: &[F; 3], b: &[F; 3]) -> F {
    mag(&sub(a, b))
}

pub fn angle<F: Float>(a: &[F; 3], b: &[F; 3]) -> F {
    (dot(a, b) / mag(a) / mag(b)).acos()
}

pub fn dot<F: Float>(a: &[F; 3], b: &[F; 3]) -> F {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Vector length.
pub fn mag<F: Float>(x: &[F; 3]) -> F {
    (x[0].powi(2) + x[1].powi(2) + x[2].powi(2)).sqrt()
}

pub fn sub<F: Float>(a: &[F; 3], b: &[F; 3]) -> [F; 3] {
    [b[0] - a[0], b[1] - a[1], b[2] - a[2]]
}

pub fn add<F: Float>(a: &[F; 3], b: &[F; 3]) -> [F; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

pub fn scale<F: Float>(x: &[F; 3], f: F) -> [F; 3] {
    [f * x[0], f * x[1], f * x[2]]
}

pub fn cross<F: Float>(a: &[F; 3], b: &[F; 3]) -> [F; 3] {
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
pub fn rotate_align<F: Float + NumAssignOps>(a: &[F; 3], b: &[F; 3]) -> [[F; 3]; 3] {
    let c = dot(a, b); // Cosine of angle
    let c1 = c + one();
    if c1.abs() <= F::epsilon() {
        todo!()
    } else {
        let v = cross(a, b);
        // Skew symmetric cross-product matrix.
        let vx = [[zero(), -v[2], v[1]], [v[2], zero(), -v[0]], [-v[1], v[0], zero()]];
        // Calculate: identity-matrix + vx + vx^2 / c1
        let mut vx2 = mat3x3_sqr(&vx);
        max3x3_div_scalar(&mut vx2, c1);
        max3x3_add(&mut vx2, &vx);
        vx2[0][0] += one();
        vx2[1][1] += one();
        vx2[2][2] += one();
        vx2
    }
}

fn mat3x3_sqr<F: Float + NumAssignOps>(mat: &[[F; 3]; 3]) -> [[F; 3]; 3] {
    let mut msqr = [[zero(); 3]; 3];
    for row in 0..3 {
        for col in 0..3 {
            for inner in 0..3 {
                msqr[row][col] += mat[row][inner] * mat[inner][col];
            }
        }
    }
    msqr
}

fn max3x3_add<F: Float + NumAssignOps>(a: &mut [[F; 3]; 3], b: &[[F; 3]; 3]) {
    for row in 0..3 {
        for col in 0..3 {
            a[row][col] += b[row][col];
        }
    }
}

fn max3x3_div_scalar<F: Float + NumAssignOps>(mat: &mut [[F; 3]; 3], div: F) {
    let factor = one::<F>() / div;
    for row in 0..3 {
        for col in 0..3 {
            mat[row][col] *= factor;
        }
    }
}

pub fn vec_mat_mult<F: Float + NumAssignOps>(vec: &mut [F; 3], mat: &[[F; 3]; 3]) {
    let mut result = [zero(); 3];
    for col in 0..3 {
        for inner in 0..3 {
            result[col] += vec[inner] * mat[col][inner];
        }
    }
    *vec = result;
}
