use crate::{carrier_points::CarrierPoints, Instruction, Morphology, Node};
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
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
///
/// Argument "save_file" is the content of the file (not the file name).
#[cfg(feature = "pyo3")]
#[pyfunction(name = "import_save")]
pub(crate) fn import_save_py(py: Python<'_>, save_file: &str) -> PyResult<Vec<Instruction>> {
    py.allow_threads(|| crate::import_save(&save_file))
        .map_err(|err| pyo3::exceptions::PyValueError::new_err(err.to_string()))
}

/// Parse a save-file from the morphology-wizard's graphical user interface.
///
/// Argument `save_file` is the content of the file (not the file name).
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
/// sections = eval(script)
/// ```
///
/// Returns a list of lists, containing the Section objects generated by each
/// growth instruction.
///
/// <https://neuron.yale.edu/neuron/>  
/// <https://nrn.readthedocs.io>  
pub fn export_nrn(_instructions: &[Instruction], _nodes: &[Node]) -> String {
    todo!()
    /*
    // First import the "Section" class from the "neuron.h" python module.
    let neuron = PyModule::import_bound(py, "neuron")?;
    let neuron_h = neuron.getattr("h")?;
    let section = neuron_h.getattr("Section")?;
    let connect = intern!(py, "connect");
    let pt3dadd = intern!(py, "pt3dadd");
    let nseg = intern!(py, "nseg");
    let name = intern!(py, "name");

    // Release the GIL while running the core algorithm.
    let nodes = py.allow_threads(|| crate::create(&instructions));

    // Transfer the morphology into the neuron simulator.
    let sections_by_type: Vec<_> = (0..instructions.len()).map(|_| PyList::empty_bound(py)).collect();
    let mut sections = Vec::<Bound<PyAny>>::with_capacity(nodes.len());
    let mut autoinc = vec![0u32; instructions.len()];
    for node in &nodes {
        let [x, y, z] = node.coordinates;
        let d = node.diameter;
        let instr_index = node.instruction_index;
        // Make a new soma.
        if node.is_root() {
            let cell_num = autoinc[instr_index as usize];
            autoinc[instr_index as usize] += 1;
            let kwargs = PyDict::new_bound(py);
            kwargs.set_item(name, format!("section[{instr_index}][{cell_num}]"))?;
            let soma = section.call((), Some(&kwargs))?;
            soma.call_method1(pt3dadd, (x - 0.5 * d, y, z, d))?;
            soma.call_method1(pt3dadd, (x + 0.5 * d, y, z, d))?;
            soma.setattr(nseg, 1)?;
            sections_by_type[instr_index as usize].append(soma.clone().unbind())?;
            sections.push(soma);
        } else {
            // Make a new section.
            let parent_node = &nodes[node.parent_index as usize];
            let parent_sec = &sections[node.parent_index as usize];
            let [parent_x, parent_y, parent_z] = parent_node.coordinates;
            let parent_d = parent_node.diameter;
            // Special case for segments branching off of the soma.
            let sec = if parent_node.num_children != 1 || instr_index != parent_node.instruction_index {
                // Start a new section.
                let sec_num = autoinc[instr_index as usize];
                autoinc[instr_index as usize] += 1;
                let kwargs = PyDict::new_bound(py);
                kwargs.set_item(name, format!("section[{instr_index}][{sec_num}]"))?;
                let sec = section.call((), Some(&kwargs))?;
                if parent_node.is_root() {
                    if node.is_terminal() {
                        sec.call_method1(pt3dadd, (parent_x, parent_y, parent_z, d))?;
                    }
                    sec.call_method1(pt3dadd, (x, y, z, d))?;
                    let soma_center = parent_sec.call1((0.5,))?;
                    sec.call_method1(connect, (soma_center,))?;
                } else {
                    sec.call_method1(pt3dadd, (parent_x, parent_y, parent_z, parent_d))?;
                    sec.call_method1(pt3dadd, (x, y, z, d))?;
                    sec.call_method1(connect, (parent_sec,))?;
                }
                sections_by_type[instr_index as usize].append(sec.clone().unbind())?;
                sec
            } else {
                // Simple dendrite or axon extension.
                let sec = sections[node.parent_index as usize].clone();
                sec.call_method1(pt3dadd, (x, y, z, d))?;
                sec
            };
            // Close the section by capping over the end.
            if node.num_children == 0 {
                sec.call_method1(pt3dadd, (x, y, z, 0.0))?;
            }
            // Record this node's section so that child nodes can attach to it.
            sections.push(sec);
        }
    }
    Ok(PyList::new_bound(py, sections_by_type).into())
    */
}
