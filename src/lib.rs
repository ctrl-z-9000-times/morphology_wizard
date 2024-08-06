//! Create realistic neuron morphologies.
//!
//! This implements the TREES algorithm combined with the morphological
//! constraints of the ROOTS algorithm.
//!
//! ### References
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

mod carrier_points;
mod dendrite_diameter;
mod formats;
mod linalg;

use bitvec::prelude::*;
#[doc(hidden)]
pub use carrier_points::CarrierPoints;
pub use formats::*;
use kiddo::{immutable::float::kdtree::ImmutableKdTree, SquaredEuclidean};
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::f64::consts::PI;

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
///     https://doi.org:10.1186/1742-4682-4-21

#[cfg(feature = "pyo3")]
#[pymodule]
fn morphology_wizard(m: &Bound<PyModule>) -> PyResult<()> {
    m.add_class::<Morphology>()?;
    m.add_class::<Instruction>()?;
    m.add_class::<Node>()?;
    m.add_function(wrap_pyfunction!(crate::create_py, m)?)?;
    m.add_function(wrap_pyfunction!(crate::formats::import_save_py, m)?)?;
    m.add_function(wrap_pyfunction!(crate::formats::export_swc_py, m)?)?;
    m.add_function(wrap_pyfunction!(crate::formats::export_nml_py, m)?)?;
    m.add_function(wrap_pyfunction!(crate::formats::export_nrn_py, m)?)?;
    Ok(())
}

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
    /// True indicates an axonal morphology, false indicates a dendrite morphology.  
    pub extend_before_branch: bool,

    /// Maximum number of secondary branches that any segment can have.  
    /// Root nodes can have an unlimited number of children.  
    pub maximum_branches: u32,

    /// Minimum diameter for this type of neurite.  
    /// Units: microns  
    pub minimum_diameter: f64,

    /// Scale the size of the dendrite tapering effect.  
    /// A value of zero yields a dendrite with constant diameter and no tapering.
    /// Larger values yield larger and more tapered dendrites.  
    /// Must be greater than or equal to zero.  
    pub dendrite_taper: f64,

    /// Segments longer than this length will be automatically broken up into
    /// multiple shorter segments.
    pub maximum_segment_length: f64,

    /// Override the morphological constraints in order to reach all carrier points.
    pub reach_all_carrier_points: bool,
}

impl Default for Morphology {
    fn default() -> Self {
        Self {
            balancing_factor: 0.7,
            extension_angle: PI,
            extension_distance: f64::INFINITY,
            branch_angle: PI,
            branch_distance: f64::INFINITY,
            extend_before_branch: false,
            maximum_branches: 1,
            minimum_diameter: 1.0,
            dendrite_taper: 0.2,
            maximum_segment_length: f64::INFINITY,
            reach_all_carrier_points: true,
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
    ///     Always assign a new morphology to this attribute.  
    ///     Accessing this attribute creates a new copy of it.  
    #[serde(flatten, default, skip_serializing_if = "Option::is_none")]
    pub morphology: Option<Morphology>,

    /// Specifies the diameter of the root node in a neuron's tree structure.  
    /// Units: microns  
    ///
    /// This parameter is required for somas and invalid for dendrites and axons.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub soma_diameter: Option<f64>,

    /// Three dimensional locations where this instruction will grow to.  
    /// Units: microns  
    #[serde(default)]
    pub carrier_points: Vec<[f64; 3]>,

    /// Specifies the prior growth that this instruction will start from.
    /// Roots is a list of indices into the instructions list.
    ///
    /// All instructions in the roots list must have already been executed.  
    /// Roots are only applicable to dendrite / axon growth instructions.  
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
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
                assert!(morphology.dendrite_taper >= 0.0);
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
}

#[cfg_attr(feature = "pyo3", pymethods)]
impl Instruction {
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

/// Container for a piece of a neuron.
#[cfg_attr(feature = "pyo3", pyclass)]
#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub struct Node {
    coordinates: [f64; 3],
    carrier_point: bool,
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
    /// Is this node located at an actual carrier point or was it interpolated
    /// between two other carrier points.
    pub fn carrier_point(&self) -> bool {
        self.carrier_point
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
        !self.is_root()
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
            carrier_point: true,
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
        carrier_point: bool,
    ) -> Self {
        Self {
            coordinates,
            diameter,
            parent_index,
            instruction_index,
            num_children: 0,
            path_length,
            carrier_point,
        }
    }
}

/// Container for all of the data structures needed to execute a growth instruction.
struct WorkingData<'a> {
    instruction_index: u32,

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
    fn new(instruction_index: u32, instruction: &'a Instruction) -> Self {
        Self {
            instruction_index,
            morphology: instruction.morphology.as_ref().unwrap(),
            carrier_points: &instruction.carrier_points,
            occupied: bitbox![0; instruction.carrier_points.len()],
            kd_tree: ImmutableKdTree::new_from_slice(&instruction.carrier_points),
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

    /// Returns all potential segments starting from the given node, without
    /// considering any morphological constraints. This is used to restart the
    /// algorithm if it gets stuck before consuming all carrier points.
    fn consider_all_potential_segments_relaxed(
        &self,
        parent_index: u32,
        parent_node: &Node,
        potential_segments: &mut Vec<PotentialSegment>,
    ) {
        for carrier_index in self.occupied.iter_zeros() {
            let carrier_point = &self.carrier_points[carrier_index];
            let segment_length = linalg::distance(&parent_node.coordinates, carrier_point);
            let priority = self.priority(parent_node, segment_length);
            potential_segments.push(PotentialSegment {
                branch_num: 0,
                priority,
                carrier_index: carrier_index as u32,
                parent_index,
            });
        }
    }

    fn priority(&self, parent_node: &Node, segment_length: f64) -> f64 {
        let path_length = segment_length + parent_node.path_length;
        segment_length + self.morphology.balancing_factor * path_length
    }

    fn grow_segment(&mut self, nodes: &mut Vec<Node>, parent_index: u32, carrier_index: u32) {
        //
        let carrier_point = &self.carrier_points[carrier_index as usize];
        self.occupied.set(carrier_index as usize, true);
        //
        self.grow_segment_nodes(nodes, parent_index, carrier_point);

        // Now consider growing more segments off of this new node.
        let node_index = (nodes.len() - 1) as u32;
        self.consider_all_potential_segments(node_index, &nodes[node_index as usize]);
    }

    fn grow_segment_nodes(&self, nodes: &mut Vec<Node>, mut parent_index: u32, carrier_point: &[f64; 3]) {
        let mut parent_node = &mut nodes[parent_index as usize];
        parent_node.num_children += 1;
        // Insert an extra node on the surface of the soma.
        if parent_node.is_root() {
            let parent_coords = &parent_node.coordinates;
            let parent_radius = 0.5 * parent_node.diameter;
            let path_length = linalg::distance(parent_coords, carrier_point);
            // Check if the carrier point is inside of the soma's radius.
            if parent_radius >= path_length {
                nodes.push(Node::new_segment(
                    *carrier_point,
                    self.morphology.minimum_diameter,
                    parent_index,
                    self.instruction_index,
                    path_length,
                    true,
                ));
                return;
            }
            // Find the surface of the soma.
            let percent = parent_radius / path_length;
            let offset = linalg::scale(&linalg::sub(parent_coords, carrier_point), percent);
            let surface = linalg::add(parent_coords, &offset);
            //
            nodes.push(Node::new_segment(
                surface,
                self.morphology.minimum_diameter,
                parent_index,
                self.instruction_index,
                parent_radius,
                false,
            ));
            parent_index = (nodes.len() - 1) as u32;
            parent_node = &mut nodes[parent_index as usize];
            parent_node.num_children += 1;
        }
        // Split up segments that are too long.
        let start_coordinates = parent_node.coordinates;
        let parent_path_length = parent_node.path_length;
        let segment_length = linalg::distance(&start_coordinates, carrier_point);
        let num_segments = (segment_length / self.morphology.maximum_segment_length).ceil() as u32;
        // Make intermediate nodes along the length of the segment.
        for segment in 1..num_segments {
            let percent = segment as f64 / num_segments as f64;
            let offset = linalg::scale(&linalg::sub(&start_coordinates, carrier_point), percent);
            let coordinates = linalg::add(&start_coordinates, &offset);
            nodes.push(Node::new_segment(
                coordinates,
                self.morphology.minimum_diameter,
                parent_index,
                self.instruction_index,
                parent_path_length + percent * segment_length,
                false,
            ));
            parent_index = (nodes.len() - 1) as u32;
            parent_node = &mut nodes[parent_index as usize];
            parent_node.num_children += 1;
        }
        // Make the final segment reaching to the exact carrier point.
        nodes.push(Node::new_segment(
            *carrier_point,
            self.morphology.minimum_diameter,
            parent_index,
            self.instruction_index,
            parent_path_length + segment_length,
            true,
        ));
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
/// Returns a list of nodes in topologically sorted order.
#[cfg(feature = "pyo3")]
#[pyfunction(name = "create")]
pub(crate) fn create_py(py: Python<'_>, instructions: Vec<Instruction>) -> Vec<Node> {
    py.allow_threads(|| crate::create(&instructions))
}

/// Execute a list of neuron growth instructions.
///
/// Returns a list of nodes in topologically sorted order.
pub fn create(instructions: &[Instruction]) -> Vec<Node> {
    Instruction::check(instructions);
    // Preallocate space for all of the returned nodes.
    let num_nodes = instructions.iter().map(|inst| inst.carrier_points.len()).sum::<usize>();
    let mut nodes = Vec::<Node>::with_capacity(num_nodes);
    // Preemptively check for index overflow.
    assert!(instructions.len() < u32::MAX as usize);
    assert!(num_nodes < u32::MAX as usize);
    // The sections list keeps track of which nodes were created by each instruction.
    let mut sections = Vec::<u32>::with_capacity(instructions.len() + 1);
    sections.push(0);

    // Process each instruction.
    for (instr_index, instr) in instructions.iter().enumerate() {
        execute_instruction(instr_index, instr, &mut nodes, &sections);
        sections.push(nodes.len() as u32);
    }

    dendrite_diameter::QuadraticApprox::default().apply(instructions, &mut nodes);
    nodes
}

fn execute_instruction(instr_index: usize, instr: &Instruction, nodes: &mut Vec<Node>, sections: &[u32]) {
    if instr.carrier_points.is_empty() {
        return;
    }
    // Instructions without a morphology create new neurons.
    let Some(ref morph) = instr.morphology else {
        let soma_diameter = instr.soma_diameter.expect("Internal Error");
        for coordinates in instr.carrier_points.iter() {
            nodes.push(Node::new_root(*coordinates, soma_diameter, instr_index as u32));
        }
        return;
    };

    let mut data = WorkingData::new(instr_index as u32, instr);

    // Start with potential segments from all of the roots to all of the carrier points.
    for root_instr in instr.roots.iter().copied() {
        let section_start = sections[root_instr as usize];
        let section_end__ = sections[root_instr as usize + 1];
        for root_index in section_start..section_end__ {
            let root_node = &nodes[root_index as usize];
            if root_node.carrier_point {
                data.consider_all_potential_segments(root_index, root_node);
            }
        }
    }

    // Run the modified Prim's algorithm.
    loop {
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

            let coordinates = &instr.carrier_points[carrier_index as usize];

            if !check_morphological_constraints(morph, nodes, parent_index, coordinates) {
                continue;
            }
            // Check if the parent grew another segment in the time since this
            // segment was first considered. If so then update the potential
            // segment's branch number and put it back in the queue to be
            // re-evaluated at a later time.
            if morph.extend_before_branch {
                let num_siblings = nodes[parent_index as usize].num_children;
                if num_siblings > branch_num {
                    data.potential.push(PotentialSegment {
                        branch_num: num_siblings,
                        priority,
                        carrier_index,
                        parent_index,
                    });
                    continue;
                }
            }
            // Create the new segment and consider new growth opportunities.
            data.grow_segment(nodes, parent_index, carrier_index);
        }
        // If not all carrier points could be reached then relax the
        // morphological constraints and retry the instruction.
        if morph.reach_all_carrier_points && !data.occupied.all() {
            let mut potential = vec![];
            for root_instr in instr.roots.iter().copied() {
                let section_start = sections[root_instr as usize];
                let section_end__ = sections[root_instr as usize + 1];
                for root_index in section_start..section_end__ {
                    let root_node = &nodes[root_index as usize];
                    if root_node.carrier_point {
                        data.consider_all_potential_segments_relaxed(root_index, root_node, &mut potential);
                    }
                }
            }
            let this_section_start = *sections.last().unwrap();
            for root_index in this_section_start..nodes.len() as u32 {
                let root_node = &nodes[root_index as usize];
                if root_node.carrier_point {
                    data.consider_all_potential_segments_relaxed(root_index, root_node, &mut potential);
                }
            }
            // Select the best segment, grow it, and restart the regular algorithm.
            let Some(PotentialSegment {
                carrier_index,
                parent_index,
                ..
            }) = potential.into_iter().max()
            else {
                break;
            };
            data.grow_segment(nodes, parent_index, carrier_index);
            continue;
        }
        break;
    }
}

fn check_morphological_constraints(
    morph: &Morphology,
    nodes: &[Node],
    parent_index: u32,
    carrier_point: &[f64; 3],
) -> bool {
    let parent_node = &nodes[parent_index as usize];
    let num_siblings = parent_node.num_children;
    let parent_coords = &parent_node.coordinates;
    // Check number of branches (this constraint does not apply to the soma).
    if parent_node.is_segment() && num_siblings > morph.maximum_branches {
        return false;
    }
    // Check maximum extension/branching distance.
    let segment_length = linalg::distance(parent_coords, carrier_point);
    let maximum_distance = if parent_node.is_root() || num_siblings == 0 {
        morph.extension_distance
    } else {
        morph.branch_distance
    };
    if segment_length > maximum_distance {
        return false;
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
        let segment_vector = linalg::sub(parent_coords, carrier_point);
        let segment_angle = linalg::angle(&parent_vector, &segment_vector);
        if segment_angle > maximum_angle {
            return false;
        }
    }
    true
}
