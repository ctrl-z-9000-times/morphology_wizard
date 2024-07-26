use crate::{Instruction, Node};

/// Format a list of nodes in the SWC neuron morphology format.
///
/// <http://www.neuronland.org/NLMorphologyConverter/MorphologyFormats/SWC/Spec.html>
pub fn export_swc(instructions: &[Instruction], nodes: &[Node]) -> String {
    //
    // TODO: How to deal with multiple neurons in one SWC file?

    // TODO: Make a header that contains the software versions, parameters, and a timestamp.
    // The pound symbol "#" is a line comment.

    let mut swc = String::new();
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
        swc.push_str(&format!("{index} {node_type} {x} {y} {z} {radius} {parent}\n"));
    }
    swc
}

/// Format a list of nodes in the NeuroML v2 neuron description format.
///
/// <https://docs.neuroml.org>
pub fn export_nml(_instructions: &[Instruction], _nodes: &[Node]) -> String {
    // https://docs.neuroml.org/Userdocs/ImportingMorphologyFiles.html
    // https://docs.neuroml.org/Userdocs/Specification.html

    todo!()
}
