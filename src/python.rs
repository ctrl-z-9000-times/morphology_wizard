use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

/// Create realistic neuron morphologies.
///
/// This implements the TREES algorithm combined with the morphological
/// constraints of the ROOTS algorithm.
///
/// TREES:
///     One Rule to Grow Them All: A General Theory of Neuronal Branching
///     and Its Practical Application.  
///     Cuntz H, Forstner F, Borst A, Hausser M (2010)  
///     PLoS Comput Biol 6(8): e1000877.  
///     https://doi.org/10.1371/journal.pcbi.1000877
///
/// ROOTS:
///     ROOTS: An Algorithm to Generate Biologically Realistic Cortical
///     Axons and an Application to Electroceutical Modeling.  
///     Bingham CS, Mergenthal A, Bouteiller J-MC, Song D, Lazzi G and
///     Berger TW (2020)  
///     Front. Comput. Neurosci. 14:13.  
///     https://doi.org/10.3389/fncom.2020.00013
///
/// Dendrite Diameter:
///     Optimization principles of dendritic structure.
///     Hermann Cuntz, Alexander Borst and Idan Segev (2007)
///     Theoretical Biology and Medical Modelling 2007, 4:21
///     <https://doi.org:10.1186/1742-4682-4-21>

#[pymodule]
fn morphology_wizard(m: &Bound<PyModule>) -> PyResult<()> {
    m.add_class::<crate::Morphology>()?;
    m.add_class::<crate::Instruction>()?;
    m.add_class::<crate::Node>()?;

    /// Execute a list of neuron growth instructions.
    ///
    /// Returns a list of Nodes in topologically sorted order.
    #[pyfn(m)]
    fn create(py: Python<'_>, instructions: Vec<crate::Instruction>) -> Vec<crate::Node> {
        py.allow_threads(|| crate::create(&instructions))
    }

    /*
    /// Format a list of nodes in the SWC neuron morphology format.
    ///
    /// http://www.neuronland.org/NLMorphologyConverter/MorphologyFormats/SWC/Spec.html
    #[pyfn(m)]
    fn create_swc(py: Python<'_>, nodes: Vec<crate::Node>) -> String {
        py.allow_threads(|| crate::formats::create_swc(&nodes))
    }

    /// Format a list of nodes in the NeuroML v2 neuron description format.
    ///
    /// https://docs.neuroml.org
    #[pyfn(m)]
    fn create_nml(py: Python<'_>, nodes: Vec<crate::Node>) -> String {
        py.allow_threads(|| crate::formats::create_nml(&nodes))
    }
    */

    /// Execute a list of neuron growth instructions and load the results
    /// into the NEURON simulator via its python API. Returns a list of
    /// lists, containing the Section objects generated by each growth
    /// instruction.
    ///
    /// https://neuron.yale.edu/neuron/
    #[pyfn(m)]
    fn export_nrn(py: Python<'_>, instructions: Vec<crate::Instruction>) -> PyResult<PyObject> {
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
    }
    Ok(())
}
