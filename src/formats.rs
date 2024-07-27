use crate::{carrier_points::CarrierPoints, Instruction, Morphology, Node};
use serde::Deserialize;
use std::collections::HashMap;
use std::f64::consts::PI;

#[derive(Debug, Deserialize)]
struct SaveFile {
    instructions: Vec<SaveInstruction>,
    carrier_points: Vec<CarrierPoints>,
}
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum SaveInstruction {
    Soma {
        name: String,
        carrier_points: Vec<String>,
        soma_diameter: f64,
    },
    Dendrite {
        name: String,
        carrier_points: Vec<String>,
        roots: Vec<String>,
        balancing_factor: f64,
        maximum_branches: u32,
        minimum_diameter: f64,
        dendrite_taper: f64,
        maximum_segment_length: f64,
    },
    Axon {
        name: String,
        carrier_points: Vec<String>,
        roots: Vec<String>,
        balancing_factor: f64,
        extension_distance: f64,
        extension_angle: f64,
        branch_distance: f64,
        branch_angle: f64,
        maximum_branches: u32,
        minimum_diameter: f64,
        maximum_segment_length: f64,
        reach_all_carrier_points: bool,
    },
}
impl SaveInstruction {
    fn take_name(&mut self) -> String {
        match self {
            Self::Soma { name, .. } | Self::Dendrite { name, .. } | Self::Axon { name, .. } => std::mem::take(name),
        }
    }
    fn carrier_points(&self) -> &[String] {
        match self {
            Self::Soma { carrier_points, .. }
            | Self::Dendrite { carrier_points, .. }
            | Self::Axon { carrier_points, .. } => carrier_points,
        }
    }
    fn roots(&self) -> &[String] {
        match self {
            Self::Soma { .. } => &[],
            Self::Dendrite { roots, .. } | Self::Axon { roots, .. } => roots,
        }
    }
    fn morphology(&self) -> Option<Morphology> {
        match *self {
            Self::Soma { .. } => None,
            Self::Dendrite {
                balancing_factor,
                maximum_branches,
                minimum_diameter,
                dendrite_taper,
                maximum_segment_length,
                ..
            } => Some(Morphology {
                balancing_factor,
                extension_distance: f64::INFINITY,
                extension_angle: PI,
                branch_distance: f64::INFINITY,
                branch_angle: PI,
                extend_before_branch: false,
                maximum_branches,
                minimum_diameter,
                dendrite_taper,
                maximum_segment_length,
                reach_all_carrier_points: true,
            }),
            Self::Axon {
                balancing_factor,
                extension_distance,
                extension_angle,
                branch_distance,
                branch_angle,
                maximum_branches,
                minimum_diameter,
                maximum_segment_length,
                reach_all_carrier_points,
                ..
            } => Some(Morphology {
                balancing_factor,
                extension_distance,
                extension_angle,
                branch_distance,
                branch_angle,
                extend_before_branch: true,
                maximum_branches,
                minimum_diameter,
                dendrite_taper: 0.0,
                maximum_segment_length,
                reach_all_carrier_points,
            }),
        }
    }
}

/// Parse a save-file from the morphology-wizard's graphical user interface.
pub fn import_save(save_file: &str) -> Result<Vec<Instruction>, serde_json::Error> {
    let SaveFile {
        mut instructions,
        carrier_points,
    } = serde_json::from_str(save_file)?;
    //
    let instruction_names: HashMap<String, u32> = instructions
        .iter_mut()
        .enumerate()
        .map(|(index, instr)| (instr.take_name(), index as u32))
        .collect();
    //
    let carrier_points_names: HashMap<String, CarrierPoints> = carrier_points
        .into_iter()
        .map(|mut param| (param.take_name(), param))
        .collect();
    // Fixup and transform the instructions into the propper data structure.
    Ok(instructions
        .into_iter()
        .map(|instr| {
            //
            let roots = instr.roots().iter().map(|name| instruction_names[name]).collect();
            //
            let carrier_points = instr.carrier_points().iter().fold(vec![], |mut points, name| {
                points.append(&mut carrier_points_names[name].generate_points());
                points
            });
            //
            let morphology = instr.morphology();
            //
            match instr {
                SaveInstruction::Soma { soma_diameter, .. } => Instruction {
                    morphology,
                    soma_diameter: Some(soma_diameter),
                    carrier_points,
                    roots,
                },
                SaveInstruction::Dendrite { .. } => Instruction {
                    morphology,
                    soma_diameter: None,
                    carrier_points,
                    roots,
                },
                SaveInstruction::Axon { .. } => Instruction {
                    morphology,
                    soma_diameter: None,
                    carrier_points,
                    roots,
                },
            }
        })
        .collect())
}

/// Enumerates the neurons and identifies which neuron each node is part of.
///
/// Returns (neuron-labels, total-number-of-neurons)  
/// Where neuron-labels runs parallel to the nodes list.  
fn label_neurons(nodes: &[Node]) -> (Vec<u32>, u32) {
    let mut labels = Vec::with_capacity(nodes.len());
    let mut num_neurons = 0;
    for node in nodes {
        if let Some(parent_index) = node.parent_index() {
            labels.push(labels[parent_index as usize]);
        } else {
            labels.push(num_neurons);
            num_neurons += 1;
        }
    }
    return (labels, num_neurons);
}

/// Format a list of nodes in the SWC neuron morphology format.
///
/// Returns a list of SWC files, one per neuron.
///
/// <http://www.neuronland.org/NLMorphologyConverter/MorphologyFormats/SWC/Spec.html>
pub fn export_swc(instructions: &[Instruction], nodes: &[Node]) -> Vec<String> {
    let (neurons, num_neurons) = label_neurons(nodes);
    let mut swc_files = vec![String::new(); num_neurons as usize];

    // Write a header with: the software version, a timestamp, and the neuron number.
    // The pound symbol "#" is a line comment.
    let timestamp = chrono::Local::now().to_rfc2822();
    for (neuron_index, file) in swc_files.iter_mut().enumerate() {
        *file = format!(
            "# Morphology Wizard\n# Version: {}\n# {timestamp}\n# Neuron {neuron_index} / {num_neurons}\n",
            env!("CARGO_PKG_VERSION")
        );
    }

    for (index, node) in nodes.iter().enumerate() {
        let index = index + 1;
        let instr = &instructions[node.instruction_index() as usize];
        let node_type = if instr.is_soma() {
            1
        } else if instr.is_axon() {
            2
        } else if instr.is_dendrite() {
            3
        } else {
            unreachable!()
        };
        let [x, y, z] = node.coordinates();
        let radius = 0.5 * node.diameter();
        let parent = match node.parent_index() {
            Some(index) => index + 1,
            None => 0,
        };
        let neuron_index = neurons[index] as usize;
        swc_files[neuron_index].push_str(&format!("{index} {node_type} {x} {y} {z} {radius} {parent}\n"));
    }
    swc_files
}

/// Format a list of nodes in the NeuroML v2 neuron description format.
///
/// <https://docs.neuroml.org>
pub fn export_nml(_instructions: &[Instruction], _nodes: &[Node]) -> String {
    // https://docs.neuroml.org/Userdocs/ImportingMorphologyFiles.html
    // https://docs.neuroml.org/Userdocs/Specification.html

    todo!()
}
