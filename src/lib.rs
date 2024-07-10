//! Create realistic neuron morphologies.
//!
//! This implements the TREES algorithm combined with the morphological
//! constraints of the ROOTS algorithm.
//!
//! **TREES:**  
//!     Cuntz H, Forstner F, Borst A, Hausser M (2010) One Rule to Grow Them
//!     All: A General Theory of Neuronal Branching and Its Practical
//!     Application. PLoS Comput Biol 6(8): e1000877.
//!     <https://doi.org/10.1371/journal.pcbi.1000877>
//!
//! **ROOTS:**  
//!     Bingham CS, Mergenthal A, Bouteiller J-MC, Song D, Lazzi G and Berger TW
//!     (2020) ROOTS: An Algorithm to Generate Biologically Realistic Cortical
//!     Axons and an Application to Electroceutical Modeling. Front. Comput.
//!     Neurosci. 14:13. <https://doi.org/10.3389/fncom.2020.00013>

#[cfg(feature = "pyo3")]
mod primatives;

use bitvec::prelude::*;
use kiddo::{immutable::float::kdtree::ImmutableKdTree, SquaredEuclidean};
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::f32::consts::PI;

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
    ///     Cuntz H, Forstner F, Borst A, Hausser M (2010) One Rule to Grow Them
    ///     All: A General Theory of Neuronal Branching and Its Practical
    ///     Application. PLoS Comput Biol 6(8): e1000877.
    ///     https://doi.org/10.1371/journal.pcbi.1000877
    ///
    /// ROOTS:
    ///     Bingham CS, Mergenthal A, Bouteiller J-MC, Song D, Lazzi G and Berger TW
    ///     (2020) ROOTS: An Algorithm to Generate Biologically Realistic Cortical
    ///     Axons and an Application to Electroceutical Modeling. Front. Comput.
    ///     Neurosci. 14:13. https://doi.org/10.3389/fncom.2020.00013
    #[pymodule]
    fn neuron_morphology(m: &Bound<PyModule>) -> PyResult<()> {
        m.add_class::<super::Morphology>()?;
        m.add_class::<super::Instruction>()?;
        m.add_class::<super::Node>()?;

        /// Execute a list of neuron growth instructions.
        ///
        /// Returns a list of Nodes in topologically sorted order.
        #[pyfn(m)]
        fn create(instructions: Vec<super::Instruction>) -> Vec<super::Node> {
            super::create(&instructions)
        }

        #[pyfn(m)]
        fn create_and_render(
            py: Python,
            instructions: Vec<super::Instruction>,
        ) -> (Vec<super::Node>, Bound<PyArray1<f32>>, Bound<PyArray1<u32>>) {
            let nodes = super::create(&instructions);
            //
            let mut vertices = Vec::new();
            let mut indicies = Vec::new();
            let mut offset = 0;
            for node in &nodes {
                if node.is_root() {
                    let diameter: f32 = 10.0;
                    let slices = 3.max((4.0 * diameter).round() as u32);
                    let (mut seg_v, mut seg_i) = crate::primatives::sphere(&node.coordinates, diameter, slices);
                    seg_i.iter_mut().for_each(|i| *i += offset);
                    indicies.append(&mut seg_i);
                    offset += seg_v.len() as u32;
                    vertices.append(&mut seg_v);
                } else {
                    let diam: f32 = 2.0;
                    let slices = 3.max((4.0 * diam).round() as u32);
                    let parent_node = &nodes[node.parent_index as usize];
                    let (mut seg_v, mut seg_i) =
                        crate::primatives::cylinder(&parent_node.coordinates, &node.coordinates, diam, diam, slices);
                    seg_i.iter_mut().for_each(|i| *i += offset);
                    indicies.append(&mut seg_i);
                    offset += seg_v.len() as u32;
                    vertices.append(&mut seg_v);
                }
            }
            let vertices = transmute_flatten_coordinates(vertices);
            (nodes, vertices.into_pyarray_bound(py), indicies.into_pyarray_bound(py))
        }

        Ok(())
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

/// Container for the morphological parameters of a neuron's dendrite or axon.
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all))]
#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub struct Morphology {
    /// The balancing factor controls the trade-off between minimizing the
    /// amount of neurite material and minimizing conduction delays.  
    /// Lower factors favor using less neurite material, higher factors favor
    /// more direct routes from each node to the soma.
    pub balancing_factor: f32,

    /// Maximum distance for primary extension segments.  
    pub extension_distance: f32,

    /// Maximum angle between a primary extension and its parent segment.  
    /// This is sometimes also known as the meander.  
    /// Units: Radians  
    pub extension_angle: f32,

    /// Maximum distance for secondary branching segments.  
    pub branch_distance: f32,

    /// Maximum angle between a secondary branch and its parent segment.  
    /// Units: Radians  
    pub branch_angle: f32,

    /// Prefer extending existing branches over making new branches.  
    pub extend_before_branch: bool,

    /// Maximum number of secondary branches that any segment can have.  
    /// Root nodes can have an unlimited number of children.  
    pub maximum_branches: u32,
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

    /// Three dimensional locations where this instruction will grow to.  
    /// The units are arbitrary and user defined.  
    pub carrier_points: Vec<[f32; 3]>,

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
            if let Some(ref morph) = instr.morphology {
                assert!(0.0 <= morph.balancing_factor);
                assert!(0.0 <= morph.extension_distance);
                assert!(0.0 <= morph.branch_distance);
                assert!((0.0..=PI).contains(&morph.extension_angle));
                assert!((0.0..=PI).contains(&morph.branch_angle));
                assert!(instr.roots.iter().all(|&root_instr| root_instr < instr_index as u32));
            } else {
                assert!(instr.roots.is_empty(), "Missing morphology parameters");
            }
        }
    }
}

/// Container for a location in a neuron.
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all))]
#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub struct Node {
    coordinates: [f32; 3],
    parent_index: u32,
    instruction_index: u32,
    num_children: u32,
    path_length: f32,
}

#[cfg_attr(feature = "pyo3", pymethods)]
impl Node {
    /// Three dimensional coordinates of this node.  
    /// Located at the center of the soma (for root nodes) or at the tip of the
    /// segment (for branch nodes).
    pub fn coordinates(&self) -> [f32; 3] {
        self.coordinates
    }
    /// Is this node a neuron's soma?
    pub fn is_root(&self) -> bool {
        self.parent_index == u32::MAX
    }
    /// Is this node a section of a dendrite or axon?
    pub fn is_segment(&self) -> bool {
        self.parent_index != u32::MAX
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
    pub fn path_length(&self) -> f32 {
        self.path_length
    }
    fn __str__(&self) -> String {
        format!("{self:#?}")
    }
}
impl Node {
    fn new_root(coordinates: [f32; 3], instruction_index: u32) -> Self {
        Self {
            coordinates,
            parent_index: u32::MAX,
            instruction_index,
            num_children: 0,
            path_length: 0.0,
        }
    }
}

/// Container for all of the data structures needed to execute a growth instruction.
struct WorkingData<'a> {
    morphology: &'a Morphology,

    carrier_points: &'a [[f32; 3]],

    /// Boolean flags to track which carrier points have already been used.
    occupied: BitBox,

    /// For constraining the algorithm to search within the maximum extension/branching distance constraints.
    kd_tree: ImmutableKdTree<f32, u32, 3, 32>,

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

    fn priority(&self, parent_node: &Node, segment_length: f32) -> f32 {
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
    priority: f32,

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
        // Instruction which are missing a morphology create new neurons / tree roots.
        let Some(ref morph) = instr.morphology else {
            for coordinates in instr.carrier_points.iter() {
                nodes.push(Node::new_root(*coordinates, instr_index as u32));
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
            if maximum_angle < PI - f32::EPSILON && parent_node.is_segment() {
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
            let path_length = parent_node.path_length + segment_length;
            nodes.push(Node {
                coordinates: *coordinates,
                parent_index,
                instruction_index: instr_index as u32,
                num_children: 0,
                path_length,
            });
            data.occupied.set(carrier_index as usize, true);
            nodes[parent_index as usize].num_children += 1;
            // Now consider growing more segments off of this new node.
            data.consider_all_potential_segments(node_index, &nodes[node_index as usize]);
        }
        // End of growth instruction.
        sections.push((section_start, nodes.len() as u32));
    }
    nodes
}

#[allow(dead_code)]
mod linalg {
    pub fn distance(a: &[f32; 3], b: &[f32; 3]) -> f32 {
        mag(&sub(a, b))
    }

    pub fn angle(a: &[f32; 3], b: &[f32; 3]) -> f32 {
        (dot(a, b) / mag(a) / mag(b)).acos()
    }

    pub fn dot(a: &[f32; 3], b: &[f32; 3]) -> f32 {
        a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
    }

    pub fn mag(x: &[f32; 3]) -> f32 {
        (x[0].powi(2) + x[1].powi(2) + x[2].powi(2)).sqrt()
    }

    pub fn sub(a: &[f32; 3], b: &[f32; 3]) -> [f32; 3] {
        [b[0] - a[0], b[1] - a[1], b[2] - a[2]]
    }

    pub fn cross(a: &[f32; 3], b: &[f32; 3]) -> [f32; 3] {
        [
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        ]
    }

    /// Calculate a rotation matrix to transform from vector A to vector B.
    ///
    /// Both argument must already be normalized (magnitude of one).
    ///
    /// https://math.stackexchange.com/questions/180418/calculate-rotation-matrix-to-align-vector-a-to-vector-b-in-3d/476311#476311
    pub fn rotate_align(a: &[f32; 3], b: &[f32; 3]) -> [[f32; 3]; 3] {
        let c = dot(a, b); // Cosine of angle
        let c1 = c + 1.0;
        if c1.abs() < f32::EPSILON {
            todo!()
        } else {
            let v = cross(a, b);
            // Skew symmetric cross-product matrix.
            let vx = [[0.0, -v[2], v[1]], [v[2], 0.0, -v[0]], [-v[1], v[0], 0.0]];
            // Calculate: identity-matrix + vx + vx^2 / c1
            let mut vx2 = mat3x3_sqr(&vx);
            max3x3_div_scalar(&mut vx2, c1);
            max3x3_add(&mut vx2, &vx);
            vx2[0][0] += 1.0;
            vx2[1][1] += 1.0;
            vx2[2][2] += 1.0;
            vx2
        }
    }

    fn mat3x3_sqr(mat: &[[f32; 3]; 3]) -> [[f32; 3]; 3] {
        let mut msqr = [[0.0; 3]; 3];
        for row in 0..3 {
            for col in 0..3 {
                for inner in 0..3 {
                    msqr[row][col] += mat[row][inner] * mat[inner][col];
                }
            }
        }
        msqr
    }

    fn max3x3_add(a: &mut [[f32; 3]; 3], b: &[[f32; 3]; 3]) {
        for row in 0..3 {
            for col in 0..3 {
                a[row][col] += b[row][col];
            }
        }
    }

    fn max3x3_div_scalar(mat: &mut [[f32; 3]; 3], div: f32) {
        let factor = 1.0 / div;
        for row in 0..3 {
            for col in 0..3 {
                mat[row][col] *= factor;
            }
        }
    }

    pub fn vec_mat_mult(vec: &mut [f32; 3], mat: &[[f32; 3]; 3]) {
        let mut result = [0.0, 0.0, 0.0];
        for col in 0..3 {
            for inner in 0..3 {
                result[col] += vec[inner] * mat[col][inner];
            }
        }
        *vec = result;
    }
}
