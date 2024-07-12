use crate::{create, Instruction};

/// Execute a list of neuron growth instructions and format the results in the
/// SWC neuron morphology format.
///
/// <http://www.neuronland.org/NLMorphologyConverter/MorphologyFormats/SWC/Spec.html>
pub fn create_swc(instructions: &[Instruction]) -> String {
    let nodes = create(instructions);

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

/// Execute a list of neuron growth instructions and format the results in the
/// NeuroML v2 neuron description format.
///
/// <https://docs.neuroml.org>
pub fn create_nml(instructions: &[Instruction]) -> String {
    let _nodes = create(instructions);

    // https://docs.neuroml.org/Userdocs/ImportingMorphologyFiles.html
    // https://docs.neuroml.org/Userdocs/Specification.html

    todo!()
}
