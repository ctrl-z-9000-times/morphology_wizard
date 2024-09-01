const soma_cfg = {
  name: "soma",
  properties: [
    {
      name: "soma_diameter",
      min: 0,
      default: 10,
    },
    {
      name: "carrier_points",
      type: "entity",
      targets: "carrier_points",
      multiple: true,
    },
  ]
}
const dendrite_cfg = {
  name: "dendrite",
  properties: [
    {
      name: "balancing_factor",
      desc: "Controls the trade-off between total dendrite length and conduction delay. Lower factors favor using less material, higher factors favor faster conduction.",
      min: 0,
      default: 0.7,
      step: 0.1,
    },
    {
      name: "maximum_branches",
      desc: "Maximum number of secondary branches that any segment can have. Root nodes can have an unlimited number of children.",
      type: "int",
      min: 0,
      default: 1,
    },
    {
      name: "minimum_diameter",
      desc: "Minimum diameter for this type of neurite.",
      units: "microns",
      min: 0,
      default: 1,
      step: 0.1,
    },
    {
      name: "dendrite_taper",
      desc: "Scales the size of the dendrite tapering effect. A value of zero will yield a constant diameter dendrite with no tapering. Larger values will yield larger dendrites.",
      min: 0,
      default: 0,
      step: 0.1,
    },
    {
      name: "maximum_segment_length",
      desc: "Segments longer than this length will be automatically broken up into multiple shorter segments.",
      min: 0,
      default: 30,
    },
    {
      name: "carrier_points",
      type: "entity",
      targets: "carrier_points",
      multiple: true,
    },
    {
      name: "roots",
      type: "entity",
      targets: "instructions",
      multiple: true,
    },
  ]
}
const axon_cfg = {
  name: "axon",
  properties: [
    {
      name: "balancing_factor",
      desc: "Controls the trade-off between total axon length and conduction delay. Lower factors favor using less material, higher factors favor faster conduction.",
      min: 0,
      default: 0.0,
      step: 0.1,
    },
    {
      name: "extension_distance",
      desc: "Maximum distance for primary extension segments.",
      units: "microns",
      min: 0,
      default: 100,
    },
    {
      name: "extension_angle",
      desc: "Maximum angle between a primary extension and its parent segment. This is sometimes also known as the meander.",
      units: "radians",
      min: 0,
      max: 3.1415926535897932385,
      default: 3.1415926535897932385,
      step: 0.01,
    },
    {
      name: "branch_distance",
      desc: "Maximum distance for secondary branching segments.",
      units: "microns",
      min: 0,
      default: 100,
    },
    {
      name: "branch_angle",
      desc: "Maximum angle between a secondary branch and its parent segment.",
      units: "radians",
      min: 0,
      max: 3.1415926535897932385,
      default: 2.0245819323134224,
      step: 0.01,
    },
    {
      name: "maximum_branches",
      desc: "Maximum number of secondary branches that any segment can have. Root nodes can always have an unlimited number of children.",
      type: "int",
      min: 0,
      default: 1,
    },
    {
      name: "minimum_diameter",
      desc: "Minimum diameter for this type of neurite.",
      units: "microns",
      min: 0,
      default: 1,
      step: 0.1,
    },
    {
      name: "maximum_segment_length",
      desc: "Segments longer than this length will be automatically broken up into multiple shorter segments.",
      min: 0,
      default: 30,
    },
    {
      name: "reach_all_carrier_points",
      desc: "",
      type: "bool",
      default: true,
    },
    {
      name: "carrier_points",
      type: "entity",
      targets: "carrier_points",
      multiple: true,
    },
    {
      name: "roots",
      type: "entity",
      targets: "instructions",
      multiple: true,
    },
  ]
}
export const instructions_cfg = {
  name: "instructions",
  title: "Growth Instructions",
  entities: [
    soma_cfg,
    dendrite_cfg,
    axon_cfg,
  ]
}
