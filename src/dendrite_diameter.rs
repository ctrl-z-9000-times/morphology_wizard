use crate::{Instruction, Node};

#[derive(Debug)]
pub struct QuadraticApprox {
    dendrite_lengths: Vec<f64>,
    polynomials: Vec<[f64; 3]>,
}

impl Default for QuadraticApprox {
    fn default() -> Self {
        Self::from_str(
            include_str!("quaddiameter_ldend.txt"),
            include_str!("quaddiameter_P_normalized.txt"),
        )
    }
}

impl QuadraticApprox {
    fn from_str(dendrite_length_data: &str, polynomial_data: &str) -> Self {
        let dendrite_lengths: Vec<f64> = dendrite_length_data
            .split_ascii_whitespace()
            .map(|x| x.parse().unwrap())
            .collect();
        let polynomials: Vec<[f64; 3]> = polynomial_data
            .lines()
            .map(|line| {
                let mut poly_terms_iter = line.split_ascii_whitespace().map(|x| x.parse().unwrap());
                let polynomial = [
                    poly_terms_iter.next().unwrap(),
                    poly_terms_iter.next().unwrap(),
                    poly_terms_iter.next().unwrap(),
                ];
                assert!(poly_terms_iter.next().is_none());
                polynomial
            })
            .collect();
        assert!(!dendrite_lengths.is_empty());
        assert_eq!(dendrite_lengths.len(), polynomials.len());
        assert!(is_sorted(&dendrite_lengths));
        assert!(is_unique(&dendrite_lengths));
        Self {
            polynomials,
            dendrite_lengths,
        }
    }

    pub fn apply(&self, instructions: &[Instruction], nodes: &mut [Node]) {
        // Track all of the paths from the tree's root to the terminals nodes.
        // Keep the path info alongside / parallel to the nodes.
        #[derive(Debug, Default, Copy, Clone)]
        struct PathAccumulator {
            num_paths: u32,
            sum_diams: f64,
        }
        let mut accum = vec![PathAccumulator::default(); nodes.len()];
        // Iterate through all leaf nodes / dendrite terminals.
        for terminal_index in 0..nodes.len() as u32 {
            let terminal_node = &nodes[terminal_index as usize];
            if !terminal_node.is_terminal() {
                continue;
            }
            // Get the diameter parameters.
            let instr = &instructions[terminal_node.instruction_index as usize];
            let Some(morph) = &instr.morphology else { continue };
            let scale = morph.dendrite_taper;
            let offset = morph.minimum_diameter;
            // Interpolate between the closest polynomial approximations for this length of dendrite.
            let (interp1_index, interp_data) = interp_points(&self.dendrite_lengths, terminal_node.path_length);
            let polynomial = match interp_data {
                None => self.polynomials[interp1_index],
                Some((interp1_weight, interp2_index, interp2_weight)) => add_arrays(
                    self.polynomials[interp1_index].map(|term| term * interp1_weight),
                    self.polynomials[interp2_index].map(|term| term * interp2_weight),
                ),
            };
            let polynomial = [
                polynomial[0] * scale,
                polynomial[1] * scale,
                polynomial[2] * scale + offset,
            ];
            // Calculate the optimal dendrite diameter for all nodes along the
            // path from this terminal to the soma.
            let mut cursor_index = terminal_index;
            let mut cursor_node = &nodes[cursor_index as usize];
            let normalization_factor = 1.0 / terminal_node.path_length;
            loop {
                let distance = cursor_node.path_length * normalization_factor;
                let diameter = polynomial[0] * distance.powi(2) + polynomial[1] * distance + polynomial[2];

                accum[cursor_index as usize].num_paths += 1;
                accum[cursor_index as usize].sum_diams += diameter;
                if cursor_node.is_segment() {
                    cursor_index = cursor_node.parent_index;
                    cursor_node = &nodes[cursor_index as usize];
                } else {
                    break;
                }
            }
        }
        // Average the diameters of all paths through each node.
        for (node, paths) in nodes.iter_mut().zip(&accum) {
            let instr = &instructions[node.instruction_index as usize];
            if instr.is_dendrite() {
                let mean_diameter = paths.sum_diams / paths.num_paths as f64;
                // Enforce the minimum_diameter constraint.
                let Some(morph) = &instr.morphology else { continue };
                node.diameter = morph.minimum_diameter.max(mean_diameter);
            }
        }
    }
}

fn is_sorted(data: &[f64]) -> bool {
    for pair in data.windows(2) {
        if pair[0] > pair[1] {
            return false;
        }
    }
    true
}

/// Are all of the elements unique?  
/// Argument data must be sorted.  
fn is_unique(data: &[f64]) -> bool {
    for pair in data.windows(2) {
        if pair[0] == pair[1] {
            return false;
        }
    }
    true
}

/// Find the index of the given value in the given data set. If there is no
/// exact match then this also returns the adjacent point and the interoplation
/// weights.
///
/// Argument data must be sorted and dedup'ed.  
fn interp_points(data: &[f64], query_value: f64) -> (usize, Option<(f64, usize, f64)>) {
    match data.binary_search_by(|probe| probe.total_cmp(&query_value)) {
        Ok(upper_index) => (upper_index, None),
        Err(upper_index) => {
            if upper_index == 0 {
                (upper_index, None)
            } else if upper_index == data.len() {
                (upper_index - 1, None)
            } else {
                let lower_index = upper_index - 1;
                let upper_value = data[upper_index];
                let lower_value = data[lower_index];
                let total_dist = upper_value - lower_value;
                let upper_dist = (upper_value - query_value) / total_dist;
                let lower_dist = (query_value - lower_value) / total_dist;
                (lower_index, Some((lower_dist, upper_index, upper_dist)))
            }
        }
    }
}

fn add_arrays<const N: usize>(mut a: [f64; N], b: [f64; N]) -> [f64; N] {
    for i in 0..N {
        a[i] += b[i]
    }
    a
}
