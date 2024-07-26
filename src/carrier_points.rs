use crate::linalg;
use kiddo::{immutable::float::kdtree::ImmutableKdTree, SquaredEuclidean};
use rand::distributions::uniform::Uniform;
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::f64::consts::PI;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CarrierPoints {
    Point {
        name: String,
        x: f64,
        y: f64,
        z: f64,
    },
    Sphere {
        name: String,
        num_points: u32,
        center_x: f64,
        center_y: f64,
        center_z: f64,
        radius: f64,
    },
    Cylinder {
        name: String,
        num_points: u32,
        top_x: f64,
        top_y: f64,
        top_z: f64,
        bottom_x: f64,
        bottom_y: f64,
        bottom_z: f64,
        radius: f64,
    },
    Cone {
        name: String,
        num_points: u32,
        tip_x: f64,
        tip_y: f64,
        tip_z: f64,
        base_x: f64,
        base_y: f64,
        base_z: f64,
        radius: f64,
    },
    Box {
        name: String,
        num_points: u32,
        upper_x: f64,
        upper_y: f64,
        upper_z: f64,
        lower_x: f64,
        lower_y: f64,
        lower_z: f64,
    },
    Import {
        name: String,
        file: (Vec<String>, Vec<String>),
    },
}

impl CarrierPoints {
    #[allow(dead_code)]
    pub fn name(&mut self) -> String {
        match self {
            Self::Point { name, .. }
            | Self::Sphere { name, .. }
            | Self::Cylinder { name, .. }
            | Self::Cone { name, .. }
            | Self::Box { name, .. }
            | Self::Import { name, .. } => name.clone(),
        }
    }
    pub(crate) fn take_name(&mut self) -> String {
        match self {
            Self::Point { name, .. }
            | Self::Sphere { name, .. }
            | Self::Cylinder { name, .. }
            | Self::Cone { name, .. }
            | Self::Box { name, .. }
            | Self::Import { name, .. } => std::mem::take(name),
        }
    }
    pub fn num_points(&self) -> u32 {
        match self {
            Self::Import { .. } => todo!(),
            Self::Point { .. } => 1,
            Self::Sphere { num_points, .. }
            | Self::Cylinder { num_points, .. }
            | Self::Cone { num_points, .. }
            | Self::Box { num_points, .. } => *num_points,
        }
    }
    pub fn volume(&self) -> f64 {
        match *self {
            Self::Point { .. } => 0.0,
            Self::Import { .. } => 0.0,
            Self::Sphere { radius, .. } => 4.0 / 3.0 * PI * radius.powi(3),
            Self::Cylinder {
                top_x,
                top_y,
                top_z,
                bottom_x,
                bottom_y,
                bottom_z,
                radius,
                ..
            } => {
                let top = [top_x, top_y, top_z];
                let bottom = [bottom_x, bottom_y, bottom_z];
                let height = linalg::distance(&bottom, &top);
                height * PI * radius.powi(2)
            }
            Self::Cone {
                tip_x,
                tip_y,
                tip_z,
                base_x,
                base_y,
                base_z,
                radius,
                ..
            } => {
                let tip = [tip_x, tip_y, tip_z];
                let base = [base_x, base_y, base_z];
                let height = linalg::distance(&base, &tip);
                height * PI * radius.powi(2) / 3.0
            }
            Self::Box {
                upper_x,
                upper_y,
                upper_z,
                lower_x,
                lower_y,
                lower_z,
                ..
            } => (upper_x - lower_x) * (upper_y - lower_y) * (upper_z - lower_z),
        }
    }
    pub fn contains(&self, coordinates: &[f64; 3]) -> bool {
        match *self {
            Self::Point { x, y, z, .. } => coordinates[0] == x && coordinates[1] == y && coordinates[2] == z,
            Self::Sphere {
                center_x,
                center_y,
                center_z,
                radius,
                ..
            } => linalg::distance(coordinates, &[center_x, center_y, center_z]) <= radius,
            Self::Cylinder {
                top_x,
                top_y,
                top_z,
                bottom_x,
                bottom_y,
                bottom_z,
                radius,
                ..
            } => {
                // https://math.stackexchange.com/a/4864013
                let a = [top_x, top_y, top_z];
                let b = [bottom_x, bottom_y, bottom_z];
                let axis = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
                let a_vec = [coordinates[0] - a[0], coordinates[1] - a[1], coordinates[2] - a[2]];
                let b_vec = [coordinates[0] - b[0], coordinates[1] - b[1], coordinates[2] - b[2]];
                let dist = linalg::mag(&linalg::cross(&axis, &a_vec)) / linalg::mag(&axis);
                if dist > radius {
                    return false;
                }
                if linalg::dot(&a_vec, &axis) < 0.0 {
                    return false;
                }
                if linalg::dot(&b_vec, &axis) > 0.0 {
                    return false;
                }
                true
            }
            Self::Cone {
                tip_x,
                tip_y,
                tip_z,
                base_x,
                base_y,
                base_z,
                radius,
                ..
            } => {
                // https://stackoverflow.com/a/12826333
                let mut tip_base = [base_x - tip_x, base_y - tip_y, base_z - tip_z];
                let height = linalg::normalize(&mut tip_base);
                let tip_coord = [coordinates[0] - tip_x, coordinates[1] - tip_y, coordinates[2] - tip_z];
                let cone_dist = linalg::dot(&tip_coord, &tip_base);
                if cone_dist < 0.0 || cone_dist > height {
                    return false;
                }
                let radius = (cone_dist / height) * radius;
                let ortho_dist = linalg::distance(&tip_coord, &linalg::scale(&tip_base, cone_dist));
                ortho_dist <= radius
            }
            Self::Box {
                upper_x,
                upper_y,
                upper_z,
                lower_x,
                lower_y,
                lower_z,
                ..
            } => {
                coordinates[0] >= lower_x
                    && coordinates[1] >= lower_y
                    && coordinates[2] >= lower_z
                    && coordinates[0] <= upper_x
                    && coordinates[1] <= upper_y
                    && coordinates[2] <= upper_z
            }
            Self::Import { .. } => {
                todo!();
            }
        }
    }
    /// Axis Aligned Bounding Box.
    ///
    /// Returns pair of [lower, upper] coordinates, inclusive bounds.
    pub fn aabb(&self) -> [[f64; 3]; 2] {
        match *self {
            Self::Import { .. } => unimplemented!(),
            Self::Point { x, y, z, .. } => [[x, y, z], [x, y, z]],
            Self::Box {
                upper_x,
                upper_y,
                upper_z,
                lower_x,
                lower_y,
                lower_z,
                ..
            } => {
                let lower_bound = [lower_x, lower_y, lower_z];
                let upper_bound = [upper_x, upper_y, upper_z];
                [lower_bound, upper_bound]
            }
            Self::Sphere {
                center_x,
                center_y,
                center_z,
                radius,
                ..
            } => {
                let lower_bound = [center_x - radius, center_y - radius, center_z - radius];
                let upper_bound = [center_x + radius, center_y + radius, center_z + radius];
                [lower_bound, upper_bound]
            }
            Self::Cylinder {
                top_x,
                top_y,
                top_z,
                bottom_x,
                bottom_y,
                bottom_z,
                radius,
                ..
            } => {
                let lower_edge = [bottom_x.min(top_x), bottom_y.min(top_y), bottom_z.min(top_z)];
                let upper_edge = [bottom_x.max(top_x), bottom_y.max(top_y), bottom_z.max(top_z)];
                let lower_bound = [lower_edge[0] - radius, lower_edge[1] - radius, lower_edge[2] - radius];
                let upper_bound = [upper_edge[0] + radius, upper_edge[1] + radius, upper_edge[2] + radius];
                [lower_bound, upper_bound]
            }
            Self::Cone {
                tip_x,
                tip_y,
                tip_z,
                base_x,
                base_y,
                base_z,
                radius,
                ..
            } => {
                let lower_edge = [base_x.min(tip_x), base_y.min(tip_y), base_z.min(tip_z)];
                let upper_edge = [base_x.max(tip_x), base_y.max(tip_y), base_z.max(tip_z)];
                let lower_bound = [lower_edge[0] - radius, lower_edge[1] - radius, lower_edge[2] - radius];
                let upper_bound = [upper_edge[0] + radius, upper_edge[1] + radius, upper_edge[2] + radius];
                [lower_bound, upper_bound]
            }
        }
    }
    pub fn generate_points(&self) -> Vec<[f64; 3]> {
        match self {
            Self::Import { .. } => {
                todo!()
            }
            Self::Point { x, y, z, .. } => {
                vec![[*x, *y, *z]]
            }
            Self::Sphere { .. } | Self::Cylinder { .. } | Self::Cone { .. } | Self::Box { .. } => {
                let num_points = self.num_points();
                // Generate more points than we actually need so that we can
                // discard some points that are too close togther.
                let oversample = 2 * num_points;
                let mut carrier_points = Vec::with_capacity(oversample as usize);
                // Generate random coordinates inside of the AABB, and then
                // filter out the points that are outside of the actual shape.
                let rng = &mut rand::thread_rng();
                let [lower, upper] = self.aabb();
                let coordinate_generator = [
                    Uniform::new_inclusive(lower[0], upper[0]),
                    Uniform::new_inclusive(lower[1], upper[1]),
                    Uniform::new_inclusive(lower[2], upper[2]),
                ];
                while carrier_points.len() < oversample as usize {
                    let coordinates = [
                        coordinate_generator[0].sample(rng),
                        coordinate_generator[1].sample(rng),
                        coordinate_generator[2].sample(rng),
                    ];
                    if self.contains(&coordinates) {
                        carrier_points.push(coordinates);
                    }
                }
                // Find the average distance between neighboring points.
                let average_volume = self.volume() / num_points as f64;
                let average_distance = average_volume.powf(1.0 / 3.0);
                // Find the ideal minimum distance between every two points.
                let minimum_distance = 0.5 * average_distance;
                // Build a KD-Tree of the carrier points.
                let kd_tree = ImmutableKdTree::<f64, u32, 3, 32>::new_from_slice(&carrier_points);
                // Search for points that are too close together.
                let mut neighbors = Vec::with_capacity(oversample as usize);
                for (index, coordinates) in carrier_points.iter().enumerate() {
                    for nn in kd_tree.within_unsorted::<SquaredEuclidean>(coordinates, minimum_distance) {
                        if nn.item < index as u32 {
                            neighbors.push((nn.distance, index as u32, nn.item));
                        }
                    }
                }
                // Sort the neighbors by distance (ascending).
                neighbors.sort_unstable_by(|a, b| a.0.total_cmp(&b.0));
                // Discard extra points based on proximity to other points.
                let num_discard = oversample - num_points;
                let mut discard = HashSet::<u32>::with_capacity(num_discard as usize);
                for (_distance, index_a, index_b) in neighbors {
                    if discard.len() as u32 >= num_discard {
                        break;
                    }
                    if discard.contains(&index_a) || discard.contains(&index_b) {
                        continue;
                    }
                    discard.insert(index_a);
                }
                // Filter out the discarded coordinates.
                carrier_points
                    .into_iter()
                    .enumerate()
                    .filter_map(|(index, coordinates)| {
                        if !discard.contains(&(index as u32)) {
                            Some(coordinates)
                        } else {
                            None
                        }
                    })
                    // Discard any extra coordinates remaining.
                    .take(num_points as usize)
                    .collect()
            }
        }
    }
}
