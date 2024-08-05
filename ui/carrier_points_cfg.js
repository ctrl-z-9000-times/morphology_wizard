const point_cfg = {
  name: "point",
  title: "Single Point",
  properties: [
    {
      name:  "x",
      units: "microns",
      default: 0,
    },
    {
      name:  "y",
      units: "microns",
      default: 0,
    },
    {
      name:  "z",
      units: "microns",
      default: 0,
    },
  ]
}
const sphere_cfg = {
  name: "sphere",
  properties: [
    {
      name: "num_points",
      type: "int",
      min: 0,
      default: 1,
    },
    {
      name:  "center_x",
      units: "microns",
      default: 0,
    },
    {
      name:  "center_y",
      units: "microns",
      default: 0,
    },
    {
      name:  "center_z",
      units: "microns",
      default: 0,
    },
    {
      name:  "radius",
      units: "microns",
      min: 0,
      default: 50,
    },
  ]
}
const cylinder_cfg = {
  name: "cylinder",
  properties: [
    {
      name: "num_points",
      type: "int",
      min: 0,
      default: 1,
    },
    {
      name:  "top_x",
      units: "microns",
      default: 0,
    },
    {
      name:  "top_y",
      units: "microns",
      default: 0,
    },
    {
      name:  "top_z",
      units: "microns",
      default: 100,
    },
    {
      name:  "bottom_x",
      units: "microns",
      default: 0,
    },
    {
      name:  "bottom_y",
      units: "microns",
      default: 0,
    },
    {
      name:  "bottom_z",
      units: "microns",
      default: 0,
    },
    {
      name:  "radius",
      units: "microns",
      min: 0,
      default: 50,
    },
  ]
}
const cone_cfg = {
  name: "cone",
  properties: [
    {
      name: "num_points",
      type: "int",
      min: 0,
      default: 1,
    },
    {
      name:  "tip_x",
      units: "microns",
      default: 0,
    },
    {
      name:  "tip_y",
      units: "microns",
      default: 0,
    },
    {
      name:  "tip_z",
      units: "microns",
      default: 100,
    },
    {
      name:  "base_x",
      units: "microns",
      default: 0,
    },
    {
      name:  "base_y",
      units: "microns",
      default: 0,
    },
    {
      name:  "base_z",
      units: "microns",
      default: 0,
    },
    {
      name:  "radius",
      desc: "radius at the top of the cylinder",
      units: "microns",
      min: 0,
      default: 50,
    },
  ]
}
const box_cfg = {
  name: "box",
  title: "Axis Aligned Cube",
  properties: [
    {
      name: "num_points",
      type: "int",
      min: 0,
      default: 1,
    },
    {
      name:  "upper_x",
      units: "microns",
      default: 50,
    },
    {
      name:  "upper_y",
      units: "microns",
      default: 50,
    },
    {
      name:  "upper_z",
      units: "microns",
      default: 50,
    },
    {
      name:  "lower_x",
      units: "microns",
      default: -50,
    },
    {
      name:  "lower_y",
      units: "microns",
      default: -50,
    },
    {
      name:  "lower_z",
      units: "microns",
      default: -50,
    },
  ]
}
const import_cfg = {
  name: "import",
  properties: [
    {
      name:  "file",
      title: "CSV File",
      type:  "file",
      accept: ".csv",
    }
  ]
}
export const carrier_points_cfg = {
  name: "carrier_points",
  entities: [
    point_cfg,
    sphere_cfg,
    cylinder_cfg,
    cone_cfg,
    box_cfg,
    import_cfg,
  ]
}
