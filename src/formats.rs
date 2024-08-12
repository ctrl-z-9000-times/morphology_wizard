use crate::{carrier_points::CarrierPoints, Instruction, Morphology, Node};
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::f64::consts::PI;

/// Native data structure for the graphical user interface.
#[doc(hidden)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SaveFile {
    pub instructions: Vec<GuiInstruction>,
    pub carrier_points: Vec<CarrierPoints>,
}

#[doc(hidden)]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GuiInstruction {
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
impl GuiInstruction {
    pub fn name(&self) -> &str {
        match self {
            Self::Soma { name, .. } | Self::Dendrite { name, .. } | Self::Axon { name, .. } => name,
        }
    }
    pub fn set_name(&mut self, new_name: String) {
        match self {
            Self::Soma { name, .. } | Self::Dendrite { name, .. } | Self::Axon { name, .. } => *name = new_name,
        }
    }
    pub fn take_name(&mut self) -> String {
        match self {
            Self::Soma { name, .. } | Self::Dendrite { name, .. } | Self::Axon { name, .. } => std::mem::take(name),
        }
    }
    pub fn carrier_points(&self) -> &[String] {
        match self {
            Self::Soma { carrier_points, .. }
            | Self::Dendrite { carrier_points, .. }
            | Self::Axon { carrier_points, .. } => carrier_points,
        }
    }
    pub fn carrier_points_mut(&mut self) -> &mut [String] {
        match self {
            Self::Soma { carrier_points, .. }
            | Self::Dendrite { carrier_points, .. }
            | Self::Axon { carrier_points, .. } => carrier_points,
        }
    }
    pub fn roots(&self) -> &[String] {
        match self {
            Self::Soma { .. } => &[],
            Self::Dendrite { roots, .. } | Self::Axon { roots, .. } => roots,
        }
    }
    pub fn roots_mut(&mut self) -> &mut [String] {
        match self {
            Self::Soma { .. } => &mut [],
            Self::Dendrite { roots, .. } | Self::Axon { roots, .. } => roots,
        }
    }
    pub fn morphology(&self) -> Option<Morphology> {
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
    pub fn instruction(&self, carrier_points: Vec<[f64; 3]>, roots: Vec<u32>) -> Instruction {
        let morphology = self.morphology();
        match self {
            Self::Soma { soma_diameter, .. } => Instruction {
                morphology,
                soma_diameter: Some(*soma_diameter),
                carrier_points,
                roots,
            },
            Self::Dendrite { .. } => Instruction {
                morphology,
                soma_diameter: None,
                carrier_points,
                roots,
            },
            Self::Axon { .. } => Instruction {
                morphology,
                soma_diameter: None,
                carrier_points,
                roots,
            },
        }
    }
}

/// Parse a save-file from the morphology-wizard's graphical user interface.
/// This generates random carrier points as necessary.
///
/// Argument "save_file" is the content of the file (not the file name).
///
/// Returns a list of Instructions.
#[cfg(feature = "pyo3")]
#[pyfunction(name = "import_save")]
pub(crate) fn import_save_py(py: Python<'_>, save_file: &str) -> PyResult<Vec<Instruction>> {
    py.allow_threads(|| crate::import_save(&save_file))
        .map_err(|err| pyo3::exceptions::PyValueError::new_err(err.to_string()))
}

/// Parse a save-file from the morphology-wizard's graphical user interface.  
/// This generates random carrier points as necessary.
///
/// Argument `save_file` is the content of the file (not the file name).
///
/// Returns a list of Instructions.
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
            instr.instruction(carrier_points, roots)
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
/// Returns a list of the contents of the SWC files, one per neuron.  
/// The user is responsible for writing them to the file system.  
///
/// http://www.neuronland.org/NLMorphologyConverter/MorphologyFormats/SWC/Spec.html
#[cfg(feature = "pyo3")]
#[pyfunction(name = "export_swc")]
pub(crate) fn export_swc_py(py: Python<'_>, instructions: Vec<Instruction>, nodes: Vec<Node>) -> Vec<String> {
    py.allow_threads(|| crate::export_swc(&instructions, &nodes))
}

/// Format a list of nodes into the SWC neuron morphology format.
///
/// Returns a list of the contents of the SWC files, one per neuron.  
/// The user is responsible for writing them to the file system.  
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
            "# Morphology Wizard\n# Version: {}\n# {timestamp}\n# Neuron {} / {num_neurons}\n",
            env!("CARGO_PKG_VERSION"),
            neuron_index + 1
        );
    }

    for (index, node) in nodes.iter().enumerate() {
        let node_index = index + 1;
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
        swc_files[neuron_index].push_str(&format!("{node_index} {node_type} {x} {y} {z} {radius} {parent}\n"));
    }
    swc_files
}

/// Format a list of nodes in the NeuroML v2 neuron description format.
///
/// Returns the content of the NMLv2 file as a string,
/// the user is responsible for writing it to the file system.
///
/// https://docs.neuroml.org
#[cfg(feature = "pyo3")]
#[pyfunction(name = "export_nml")]
pub(crate) fn export_nml_py(py: Python<'_>, instructions: Vec<Instruction>, nodes: Vec<Node>) -> String {
    py.allow_threads(|| crate::export_nml(&instructions, &nodes))
}

/// Format a list of nodes into the NeuroML v2 neuron description format.
///
/// Returns the content of the NMLv2 file as a string, the user is responsible
/// for writing it to the file system.
///
/// <https://docs.neuroml.org>
pub fn export_nml(_instructions: &[Instruction], _nodes: &[Node]) -> String {
    // https://docs.neuroml.org/Userdocs/ImportingMorphologyFiles.html
    // https://docs.neuroml.org/Userdocs/Specification.html

    todo!()
}

///
#[cfg(feature = "pyo3")]
#[pyfunction(name = "export_nrn")]
pub(crate) fn export_nrn_py(py: Python<'_>, instructions: Vec<Instruction>, nodes: Vec<Node>) -> String {
    py.allow_threads(|| crate::export_nrn(&instructions, &nodes))
}

/// Format a list of nodes into a python script for the NEURON simulator.
///
/// Returns an executable python script which [TODO: Define the API]
/// Example usage:
/// ```
/// import morphology
/// sections = morphology.sections()
/// ```
///
/// Returns a list of lists, containing the Section objects generated by each
/// growth instruction.
///
/// <https://neuron.yale.edu/neuron/>  
/// <https://nrn.readthedocs.io>  
pub fn export_nrn(instructions: &[Instruction], nodes: &[Node]) -> String {
    let mut script = String::new();
    script.push_str("def sections():\n");
    script.push_str("    import neuron\n");
    script.push_str("    S = neuron.h.Section\n");
    script.push_str(&format!("    r = [[] for _ in range({})]\n", instructions.len()));
    // Store each node's location in the "sections_by_type" structure.
    let mut section_index = Vec::<u32>::with_capacity(nodes.len());
    let mut autoinc = vec![0u32; instructions.len()];
    //
    for node in nodes {
        let instr_index = node.instruction_index;
        let sec_idx = autoinc[instr_index as usize];
        autoinc[instr_index as usize] += 1;
        section_index.push(sec_idx);
        // Access the node's geometry.
        let [x, y, z] = node.coordinates;
        let d = node.diameter;
        // Make a new section for this node.
        script.push_str(&format!("    x = S(name=\"section[{instr_index}][{sec_idx}]\")\n"));
        // Make a new soma.
        if node.is_root() {
            let x1 = x - 0.5 * d;
            let x2 = x + 0.5 * d;
            script.push_str(&format!("    x.pt3dadd({x1}, {y}, {z}, {d})\n"));
            script.push_str(&format!("    x.pt3dadd({x2}, {y}, {z}, {d})\n"));
        } else {
            // Make a new branch.
            let parent_node = &nodes[node.parent_index as usize];
            let parent_instr = parent_node.instruction_index;
            let [px, py, pz] = parent_node.coordinates;
            // TODO: If parent is root then use the child's diameter on both ends?
            let pd = parent_node.diameter;
            let p_sec_idx = section_index[node.parent_index as usize];
            script.push_str(&format!("    x.pt3dadd({px}, {py}, {pz}, {pd})\n"));
            script.push_str(&format!("    x.pt3dadd({x}, {y}, {z}, {d})\n"));
            // Close termincal sections with a cap over the end.
            if node.is_terminal() {
                script.push_str(&format!("    x.pt3dadd({x}, {y}, {z}, 0)\n"));
            }
            // Connect to the center of the soma.
            if parent_node.is_root() {
                script.push_str(&format!("    x.connect(r[{parent_instr}][{p_sec_idx}](.5))\n"));
            } else {
                // Connect to the tip of the parent branch.
                script.push_str(&format!("    x.connect(r[{parent_instr}][{p_sec_idx}])\n"));
            }
        }
        script.push_str(&format!("    r[{instr_index}].append(x)\n"));
    }
    script.push_str("    return r\n");
    script
}
