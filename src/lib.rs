//! Create realistic neuron morphologies.
//!
//! This implements the TREES algorithm combined with the morphological
//! constraints of the ROOTS algorithm.
//!
//! **TREES:**  
//!     One Rule to Grow Them All: A General Theory of Neuronal Branching and
//!     Its Practical Application.  
//!     Cuntz H, Forstner F, Borst A, Hausser M (2010)  
//!     PLoS Comput Biol 6(8): e1000877.  
//!     <https://doi.org/10.1371/journal.pcbi.1000877>
//!
//! **ROOTS:**  
//!     ROOTS: An Algorithm to Generate Biologically Realistic Cortical Axons
//!     and an Application to Electroceutical Modeling.  
//!     Bingham CS, Mergenthal A, Bouteiller J-MC, Song D, Lazzi G and Berger TW (2020)  
//!     Front. Comput. Neurosci. 14:13.  
//!     <https://doi.org/10.3389/fncom.2020.00013>
//!
//! **Dendrite Diameter:**  
//!     Optimization principles of dendritic structure.  
//!     Hermann Cuntz, Alexander Borst and Idan Segev (2007)  
//!     Theoretical Biology and Medical Modelling 2007, 4:21  
//!     <https://doi.org:10.1186/1742-4682-4-21>

mod formats;
mod linalg;
mod primatives;

pub use formats::*;

#[cfg(feature = "pyo3")]
mod python {
    use numpy::{IntoPyArray, PyArray1};
    use pyo3::prelude::*;

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
    #[pyo3(name = "lib")]
    fn neuron_morphology(m: &Bound<PyModule>) -> PyResult<()> {
        m.add_class::<crate::Morphology>()?;
        m.add_class::<crate::Instruction>()?;
        m.add_class::<crate::Node>()?;

        /// Execute a list of neuron growth instructions.
        ///
        /// Returns a list of Nodes in topologically sorted order.
        #[pyfn(m)]
        fn create(instructions: Vec<crate::Instruction>) -> Vec<crate::Node> {
            crate::create(&instructions)
        }

        /// Execute a list of neuron growth instructions and format the results
        /// in the SWC neuron morphology format.
        ///
        /// http://www.neuronland.org/NLMorphologyConverter/MorphologyFormats/SWC/Spec.html
        #[pyfn(m)]
        fn create_swc(instructions: Vec<crate::Instruction>) -> String {
            crate::formats::create_swc(&instructions)
        }

        /// Execute a list of neuron growth instructions and format the results
        /// in the NeuroML v2 neuron description format.
        ///
        /// https://docs.neuroml.org
        #[pyfn(m)]
        fn create_nml(instructions: Vec<crate::Instruction>) -> String {
            crate::formats::create_nml(&instructions)
        }

        #[pyfn(m)]
        fn create_and_render(
            py: Python,
            instructions: Vec<crate::Instruction>,
        ) -> (Vec<crate::Node>, Bound<PyArray1<f32>>, Bound<PyArray1<u32>>) {
            let nodes = crate::create(&instructions);
            //
            let mut vertices = Vec::new();
            let mut indicies = Vec::new();
            let mut offset = 0;
            for node in &nodes {
                if node.is_root() {
                    let slices = 3.max((4.0 * node.diameter).round() as u32);
                    let (mut seg_v, mut seg_i) =
                        crate::primatives::sphere(&coords_f64_to_f32(node.coordinates), node.diameter as f32, slices);
                    seg_i.iter_mut().for_each(|i| *i += offset);
                    indicies.append(&mut seg_i);
                    offset += seg_v.len() as u32;
                    vertices.append(&mut seg_v);
                } else {
                    let parent_node = &nodes[node.parent_index as usize];
                    let node_diameter = node.diameter;
                    let parent_diameter = if parent_node.is_root() {
                        node_diameter
                    } else {
                        parent_node.diameter
                    };
                    let max_diameter = parent_diameter.max(node.diameter);
                    let slices = 3.max((4.0 * max_diameter).round() as u32);
                    let (mut seg_v, mut seg_i) = crate::primatives::cylinder(
                        &coords_f64_to_f32(parent_node.coordinates),
                        &coords_f64_to_f32(node.coordinates),
                        parent_diameter as f32,
                        node.diameter as f32,
                        slices,
                    );
                    seg_i.iter_mut().for_each(|i| *i += offset);
                    indicies.append(&mut seg_i);
                    offset += seg_v.len() as u32;
                    vertices.append(&mut seg_v);
                }
            }
            let vertices = transmute_flatten_coordinates(vertices);
            let vertices_py = vertices.into_pyarray_bound(py);
            let indicies_py = indicies.into_pyarray_bound(py);
            (nodes, vertices_py, indicies_py)
        }

        Ok(())
    }

    fn coords_f64_to_f32(data: [f64; 3]) -> [f32; 3] {
        [data[0] as f32, data[1] as f32, data[2] as f32]
    }

    /// Transmute the vector without needlessly copying the data.
    fn transmute_flatten_coordinates(data: Vec<[f32; 3]>) -> Vec<f32> {
        // Check the data alignment.
        assert_eq!(std::mem::align_of::<[f32; 3]>(), std::mem::align_of::<f32>());
        // Take manual control over the data vector.
        let mut data = std::mem::ManuallyDrop::new(data);
        unsafe {
            // Disassemble the vector.
            let ptr = data.as_mut_ptr();
            let mut len = data.len();
            let mut cap = data.capacity();
            // Transmute the vector.
            let ptr = std::mem::transmute::<*mut [f32; 3], *mut f32>(ptr);
            len *= 3;
            cap *= 3;
            // Reassemble and return the data.
            Vec::from_raw_parts(ptr, len, cap)
        }
    }
}

use bitvec::prelude::*;
use kiddo::{immutable::float::kdtree::ImmutableKdTree, SquaredEuclidean};
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::f64::consts::PI;

/// Container for the morphological parameters of a neuron's dendrite or axon.
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all))]
#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub struct Morphology {
    /// The balancing factor controls the trade-off between minimizing the
    /// amount of neurite material and minimizing conduction delays.  
    /// Lower factors favor using less neurite material, higher factors favor
    /// more direct routes from each node to the soma.
    pub balancing_factor: f64,

    /// Maximum distance for primary extension segments.  
    /// Units: microns  
    pub extension_distance: f64,

    /// Maximum angle between a primary extension and its parent segment.  
    /// This is sometimes also known as the meander.  
    /// Units: radians  
    pub extension_angle: f64,

    /// Maximum distance for secondary branching segments.  
    /// Units: microns  
    pub branch_distance: f64,

    /// Maximum angle between a secondary branch and its parent segment.  
    /// Units: radians  
    pub branch_angle: f64,

    /// Prefer extending existing branches over making new branches.  
    pub extend_before_branch: bool,

    /// Maximum number of secondary branches that any segment can have.  
    /// Root nodes can have an unlimited number of children.  
    pub maximum_branches: u32,

    /// Minimum diameter for this type of neurite.  
    /// Units: microns  
    pub minimum_diameter: f64,

    /// Scales the size of the dendrite tapering effect. A value of zero will
    /// yield a constant diameter dendrite with no tapering. Larger values will
    /// yeild larger dendrites. Must be greater than or equal to zero.
    pub diameter_taper: f64,
}

impl Default for Morphology {
    fn default() -> Self {
        Self {
            balancing_factor: 0.7,
            extension_angle: PI,
            extension_distance: 100.0,
            branch_angle: PI,
            branch_distance: 100.0,
            extend_before_branch: false,
            maximum_branches: 1,
            minimum_diameter: 1.0,
            diameter_taper: 0.2,
        }
    }
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl Morphology {
    #[new]
    fn __new__() -> Self {
        Self::default()
    }
    fn __str__(&self) -> String {
        format!("{self:#?}")
    }
}

#[cfg_attr(feature = "pyo3", pymethods)]
impl Morphology {
    pub fn is_dendrite(&self) -> bool {
        !self.extend_before_branch
    }
    pub fn is_axon(&self) -> bool {
        self.extend_before_branch
    }
}

/// Container for a single step of a neuron growth program.
///
/// If the morphology is missing, then this specifies a neuron soma,
/// otherwise this specifies a dendrite or axon.
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all))]
#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct Instruction {
    /// The morphological parameters control the dendrite / axon growth process.
    ///
    /// Python API:  
    ///     Do not modify this attribute in-place!  
    ///     Always assign an entire morphology to it.  
    ///     Accessing this attribute creates a new copy of it.  
    pub morphology: Option<Morphology>,

    /// Specifies the diameter of the root node in a neuron's tree structure.  
    /// Units: microns  
    ///
    /// This parameter is required for somas and invalid for dendrites and axons.
    pub soma_diameter: Option<f64>,

    /// Three dimensional locations where this instruction will grow to.  
    /// Units: microns  
    pub carrier_points: Vec<[f64; 3]>,

    /// Specifies the prior growth that this instruction will start from.
    /// Roots is a list of indices into the instructions list.
    ///
    /// All instructions in the roots list must have already been executed.  
    /// Roots are only applicable to dendrite / axon growth instructions.  
    pub roots: Vec<u32>,
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl Instruction {
    #[new]
    fn __new__() -> Self {
        Self::default()
    }
    fn __str__(&self) -> String {
        format!("{self:#?}")
    }
}

impl Instruction {
    fn check(instructions: &[Instruction]) {
        for (instr_index, instr) in instructions.iter().enumerate() {
            if let Some(ref morphology) = instr.morphology {
                assert!(morphology.balancing_factor >= 0.0);
                assert!(morphology.extension_distance >= 0.0);
                assert!(morphology.branch_distance >= 0.0);
                assert!((0.0..=PI).contains(&morphology.extension_angle));
                assert!((0.0..=PI).contains(&morphology.branch_angle));
                assert!(morphology.minimum_diameter > 0.0);
                assert!(morphology.diameter_taper >= 0.0);
                assert!(instr.roots.iter().all(|&root_instr| root_instr < instr_index as u32));
                assert!(instr.soma_diameter.is_none());
            } else {
                assert!(instr.roots.is_empty(), "Missing morphology parameters");
                let Some(soma_diameter) = instr.soma_diameter else {
                    panic!("Missing soma diameter");
                };
                assert!(soma_diameter > 0.0);
            }
        }
    }
    pub fn is_soma(&self) -> bool {
        self.morphology.is_none()
    }
    pub fn is_dendrite(&self) -> bool {
        if let Some(morphology) = self.morphology {
            morphology.is_dendrite()
        } else {
            false
        }
    }
    pub fn is_axon(&self) -> bool {
        if let Some(morphology) = self.morphology {
            morphology.is_axon()
        } else {
            false
        }
    }
}

/// Container for a location in a neuron.
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all))]
#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub struct Node {
    coordinates: [f64; 3],
    diameter: f64,
    parent_index: u32,
    instruction_index: u32,
    num_children: u32,
    path_length: f64,
}

#[cfg_attr(feature = "pyo3", pymethods)]
impl Node {
    /// Three dimensional coordinates of this node.  
    /// Located at the center of the soma (for root nodes) or at the tip of the
    /// segment (for branch nodes).
    pub fn coordinates(&self) -> [f64; 3] {
        self.coordinates
    }
    /// Diameter of the neuron at this node's coordinates, in microns.
    pub fn diameter(&self) -> f64 {
        self.diameter
    }
    /// Is this node the root of a neuron's tree structure?
    pub fn is_root(&self) -> bool {
        self.parent_index == u32::MAX
    }
    /// Is this node a section of a dendrite or axon?
    pub fn is_segment(&self) -> bool {
        self.parent_index != u32::MAX
    }
    /// Is this node a leaf node in a neuron's tree structure?  
    pub fn is_terminal(&self) -> bool {
        self.num_children == 0
    }
    /// Index into the Nodes list of this segment's parent node.  
    /// Returns None if this is a root node.  
    pub fn parent_index(&self) -> Option<u32> {
        if self.is_root() {
            None
        } else {
            Some(self.parent_index)
        }
    }
    /// Index into the instructions list of the instruction that grew this neuron.
    pub fn instruction_index(&self) -> u32 {
        self.instruction_index
    }
    /// Number of extensions and branches descended from this node.
    pub fn num_children(&self) -> u32 {
        self.num_children
    }
    /// Distance from this node to the soma, by traveling through the neuron.
    pub fn path_length(&self) -> f64 {
        self.path_length
    }
    fn __str__(&self) -> String {
        format!("{self:#?}")
    }
}
impl Node {
    fn new_root(coordinates: [f64; 3], diameter: f64, instruction_index: u32) -> Self {
        Self {
            coordinates,
            diameter,
            parent_index: u32::MAX,
            instruction_index,
            num_children: 0,
            path_length: 0.0,
        }
    }
    fn new_segment(
        coordinates: [f64; 3],
        diameter: f64,
        parent_index: u32,
        instruction_index: u32,
        path_length: f64,
    ) -> Self {
        Self {
            coordinates,
            diameter,
            parent_index,
            instruction_index,
            num_children: 0,
            path_length,
        }
    }
}

/// Container for all of the data structures needed to execute a growth instruction.
struct WorkingData<'a> {
    morphology: &'a Morphology,

    carrier_points: &'a [[f64; 3]],

    /// Boolean flags to track which carrier points have already been used.
    occupied: BitBox,

    /// For constraining the algorithm to search within the maximum extension/branching distance constraints.
    kd_tree: ImmutableKdTree<f64, u32, 3, 32>,

    /// Priority queue for deciding which potential segments to grow first.
    potential: BinaryHeap<PotentialSegment>,
}

impl<'a> WorkingData<'a> {
    fn new(instruction: &'a Instruction) -> Self {
        let morphology = instruction.morphology.as_ref().unwrap();
        let carrier_points = instruction.carrier_points.as_slice();
        Self {
            morphology,
            carrier_points,
            occupied: bitbox![0; carrier_points.len()],
            kd_tree: ImmutableKdTree::new_from_slice(carrier_points),
            potential: BinaryHeap::new(),
        }
    }

    /// Add potential segments from the given parent node to all carrier points.
    /// This also filters out segments which are obviously invalid.
    fn consider_all_potential_segments(&mut self, parent_index: u32, parent_node: &Node) {
        // First check the easy morphological constraints.
        if parent_node.is_segment() && parent_node.num_children > self.morphology.maximum_branches {
            return;
        }
        // Don't check the segment angle because it's expensive and it will need
        // to be checked later anyways.

        // Keep track of how many children the parent already has,
        // in order to give priority to extensions over new branches.
        let branch_num = if self.morphology.extend_before_branch {
            parent_node.num_children
        } else {
            0
        };

        // Find all carrier points which pass the extension/branching distance constraint.
        //
        // Use the maximum of the extension and branching parameters because
        // the parent could grow another segment in the time between when we
        // enqueue this potential segment and when we actually try to grow it.
        let maximum_distance = if parent_node.is_root() {
            self.morphology.extension_distance
        } else if parent_node.num_children == 0 {
            self.morphology.extension_distance.max(self.morphology.branch_distance)
        } else {
            self.morphology.branch_distance
        };
        if maximum_distance.is_finite() {
            for nearest_neighbor in self
                .kd_tree
                .within_unsorted::<SquaredEuclidean>(&parent_node.coordinates, maximum_distance.powi(2))
            {
                let segment_length = nearest_neighbor.distance.sqrt();
                let carrier_index = nearest_neighbor.item;
                // Check if the carrier point is already being used.
                if self.occupied[carrier_index as usize] {
                    continue;
                }
                //
                self.potential.push(PotentialSegment {
                    branch_num,
                    priority: self.priority(parent_node, segment_length),
                    carrier_index,
                    parent_index,
                });
            }
        } else {
            // Iterate through the unoccupied carrier points to make use of the bitvec package's optimizations.
            for carrier_index in self.occupied.iter_zeros() {
                let coordinates = &self.carrier_points[carrier_index];
                let segment_length = linalg::distance(&parent_node.coordinates, coordinates);
                //
                self.potential.push(PotentialSegment {
                    branch_num,
                    priority: self.priority(parent_node, segment_length),
                    carrier_index: carrier_index as u32,
                    parent_index,
                });
            }
        }
    }

    fn priority(&self, parent_node: &Node, segment_length: f64) -> f64 {
        let path_length = segment_length + parent_node.path_length;
        segment_length + self.morphology.balancing_factor * path_length
    }
}

#[derive(Debug)]
struct PotentialSegment {
    /// Number of sibling branches. If zero then this is a simple extension,
    /// otherwise this will grow a branch.
    branch_num: u32,

    /// Priority for growing this segment. Lower is better.
    priority: f64,

    /// Index into the user's carrier_points list.
    carrier_index: u32,

    /// Index into the returned Node list.
    parent_index: u32,
}

impl PartialEq for PotentialSegment {
    fn eq(&self, other: &Self) -> bool {
        self.carrier_index == other.carrier_index && self.parent_index == other.parent_index
    }
}

impl Eq for PotentialSegment {}

impl PartialOrd for PotentialSegment {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PotentialSegment {
    fn cmp(&self, other: &Self) -> Ordering {
        let branch_ord = self.branch_num.cmp(&other.branch_num);
        let priority_ord = self.priority.total_cmp(&other.priority);
        // Reverse the ordering because the priority queue uses a max-heap.
        branch_ord.then(priority_ord).reverse()
    }
}

/// Execute a list of neuron growth instructions.
///
/// Returns a list of Nodes in topologically sorted order.
pub fn create(instructions: &[Instruction]) -> Vec<Node> {
    Instruction::check(instructions);
    // Preallocate space for all of the returned nodes.
    let num_nodes = instructions.iter().map(|inst| inst.carrier_points.len()).sum::<usize>();
    let mut nodes = Vec::<Node>::with_capacity(num_nodes);
    // Preemptively check for index overflow.
    assert!(instructions.len() < u32::MAX as usize);
    assert!(num_nodes < u32::MAX as usize);
    // The sections list keeps track of which nodes were created by each instruction.
    let mut sections = Vec::<(u32, u32)>::with_capacity(instructions.len());

    // Process each instruction.
    for (instr_index, instr) in instructions.iter().enumerate() {
        let section_start = nodes.len() as u32;
        // Instructions which are missing a morphology create new neurons / tree roots.
        let Some(ref morph) = instr.morphology else {
            let soma_diameter = instr.soma_diameter.expect("Internal Error");
            for coordinates in instr.carrier_points.iter() {
                nodes.push(Node::new_root(*coordinates, soma_diameter, instr_index as u32));
            }
            sections.push((section_start, nodes.len() as u32));
            continue;
        };

        let mut data = WorkingData::new(instr);

        // Start with potential segments from all of the roots to all of the carrier points.
        for root_instr in instr.roots.iter() {
            let (section_start, section_end) = sections[*root_instr as usize];
            for root_index in section_start..section_end {
                data.consider_all_potential_segments(root_index, &nodes[root_index as usize]);
            }
        }

        // Run the modified Prim's algorithm.
        while let Some(PotentialSegment {
            branch_num,
            priority,
            carrier_index,
            parent_index,
        }) = data.potential.pop()
        {
            // Check if the carrier point is already being used.
            if data.occupied[carrier_index as usize] {
                continue;
            }
            // Check morphological constraints.
            let parent_node = &nodes[parent_index as usize];
            let num_siblings = parent_node.num_children;
            let parent_coords = &parent_node.coordinates;
            let coordinates = &instr.carrier_points[carrier_index as usize];
            let segment_length = linalg::distance(parent_coords, coordinates);
            //
            if parent_node.is_segment() && num_siblings > morph.maximum_branches {
                continue;
            }
            // Check maximum extension/branching distance.
            let maximum_distance = if parent_node.is_root() || num_siblings == 0 {
                morph.extension_distance
            } else {
                morph.branch_distance
            };
            if segment_length > maximum_distance {
                continue;
            }
            // Check maximum extension/branching angle.
            let maximum_angle = if num_siblings == 0 {
                morph.extension_angle
            } else {
                morph.branch_angle
            };
            if maximum_angle < PI - f64::EPSILON && parent_node.is_segment() {
                let grandparent_coords = &nodes[parent_node.parent_index as usize].coordinates;
                let parent_vector = linalg::sub(grandparent_coords, parent_coords);
                let segment_vector = linalg::sub(parent_coords, coordinates);
                let segment_angle = linalg::angle(&parent_vector, &segment_vector);
                if segment_angle > maximum_angle {
                    continue;
                }
            }
            // Check if the parent grew another segment in the time since this
            // segment was first considered. If so then update potential
            // segment's branch number and put it back into the queue to be
            // re-evaluated at a later time.
            if morph.extend_before_branch && branch_num < num_siblings {
                data.potential.push(PotentialSegment {
                    branch_num: num_siblings,
                    priority,
                    carrier_index,
                    parent_index,
                });
                continue;
            }
            // Create the new segment.
            let node_index = nodes.len() as u32;
            nodes.push(Node::new_segment(
                *coordinates,
                morph.minimum_diameter,
                parent_index,
                instr_index as u32,
                parent_node.path_length + segment_length,
            ));
            data.occupied.set(carrier_index as usize, true);
            nodes[parent_index as usize].num_children += 1;
            // Now consider growing more segments off of this new node.
            data.consider_all_potential_segments(node_index, &nodes[node_index as usize]);
        }
        // End of growth instruction.
        sections.push((section_start, nodes.len() as u32));
    }
    let dendrite_diameter = DendriteDiameterQuadraticApprox::default();
    dendrite_diameter.calculate_quadratic_diameters(instructions, &mut nodes);
    nodes
}

#[derive(Debug)]
struct DendriteDiameterQuadraticApprox {
    dendrite_lengths: Vec<f64>,
    polynomials: Vec<[f64; 3]>,
}

impl Default for DendriteDiameterQuadraticApprox {
    fn default() -> Self {
        Self::from_str(
            include_str!("quaddiameter_ldend.txt"),
            include_str!("quaddiameter_P_normalized.txt"),
        )
    }
}

impl DendriteDiameterQuadraticApprox {
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

    fn calculate_quadratic_diameters(&self, instructions: &[Instruction], nodes: &mut [Node]) {
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
            let scale = morph.diameter_taper;
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
            let polynomial = polynomial.map(|term| term * scale + offset);
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
