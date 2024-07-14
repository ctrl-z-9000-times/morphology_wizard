from lib import *
from viewer import MorphologyViewer
import numpy as np
import control_panels

root_instr = Instruction()
# root_instr.carrier_points = np.random.uniform(-100, 100, size=(10, 3))
root_instr.carrier_points = [[0,0,0]]
root_instr.soma_diameter = 10

hdist = lambda x: (x[0]**2 + x[1]**2)**0.5
def cylinder_points(min_height, max_height, radius, num_points):
    carrier_points = np.random.uniform([-radius, -radius, min_height], [radius, radius, max_height], size=(num_points, 3))
    carrier_points = [x for x in carrier_points if (x[0]**2 + x[1]**2) <= radius**2]
    return carrier_points

def torus_points(min_height, max_height, min_radius, max_radius, num_points):
    carrier_points = np.random.uniform([-max_radius, -max_radius, min_height], [max_radius, max_radius, max_height], size=(num_points, 3))
    carrier_points = [x for x in carrier_points if (x[0]**2 + x[1]**2) >= min_radius**2]
    carrier_points = [x for x in carrier_points if (x[0]**2 + x[1]**2) <= max_radius**2]
    return carrier_points

apical_instr = Instruction()
apical_instr.carrier_points = cylinder_points(80, 120, 160, 150)
apical_instr.roots = [0]
apical_instr.morphology = Morphology()
print(apical_instr.morphology)

oblique_instr = Instruction()
oblique_instr.carrier_points = cylinder_points(20, 80, 40, 10)
oblique_instr.roots = [1]
oblique_instr.morphology = Morphology()

basal_instr = Instruction()
# basal_instr.carrier_points = cylinder_points(-10, 10, 300, 500)
basal_instr.carrier_points = cylinder_points(-10, 10, 300, 500) + torus_points(-20, 20, 250, 300, 200)
basal_instr.roots = [0]
basal_morph = Morphology()
basal_morph.balancing_factor = 2.2
basal_morph.extension_distance = np.inf
basal_instr.morphology = basal_morph

nodes, vertices, indices = create_and_render([
    root_instr,
    # apical_instr,
    # oblique_instr,
    basal_instr,
])

print("number of nodes:", len(nodes))

viewer = MorphologyViewer()
viewer.update_scene(nodes, vertices, indices)
viewer.run()
exit()

from tkinter import *
from tkinter import ttk

root = Tk()
# window, root = control_panels.Toplevel("Morphology Wizard")

morph = control_panels.SettingsPanel(root)
morph.frame.grid(sticky='NESW')
morph.add_slider("extension_angle", [0, np.pi], default=np.pi)
morph.add_checkbox("extend_before_branch")

morph.add_slider("branch_angle", [0, np.pi], default=np.pi)
morph.add_slider("branch_distance", [0, 1000.0], default=100.0)

root.mainloop()

# class MorphologyParameters():
#     def __init__(self):
#         self.balancing_factor     = DoubleVar()
#         self.extension_distance   = DoubleVar()
#         self.extension_angle      = DoubleVar()
#         self.branch_distance      = DoubleVar()
#         self.branch_angle         = DoubleVar()
#         self.extend_before_branch = BooleanVar()
#         self.maximum_branches     = IntVar()

#     def load_parameters(self, parameters):
#         1/0

#     def save_parameters(self):
#         return 1/0

#     def make_panel(self, parent):
#         parent = ttk.Frame(parent, borderwidth=5, relief="ridge", padding=5)
#         parent.grid(sticky='NESW')
#         parent.columnconfigure(0, weight=1, pad=25)
#         parent.columnconfigure(1, weight=2, pad=25)
#         parent.columnconfigure(2, weight=0, pad=25)
#         row = 0

#         ttk.Label(parent, text="Morphology Parameters", font="TkHeadingFont")\
#             .grid(column=0, columnspan=3, row=row, pady=10)
#         row += 1

#         ttk.Label(parent, text="Balancing Factor").grid(column=0, row=row)
#         ttk.Spinbox(parent, from_=0.0, to=100.0, increment=0.1, textvariable=self.balancing_factor).grid(column=1, row=row)
#         row += 1

#         ttk.Label(parent, text="Extension Distance").grid(column=0, row=row)
#         ttk.Spinbox(parent, from_=0.0, to=1000.0, increment=10.0, textvariable=self.extension_distance).grid(column=1, row=row)
#         row += 1

#         ttk.Label(parent, text="Extension Angle").grid(column=0, row=row)
#         ttk.Scale(parent, orient=HORIZONTAL, length=200, from_=0, to=np.pi, variable=self.extension_angle).grid(column=1, row=row)
#         ttk.Label(parent, textvariable=self.extension_angle).grid(column=2, row=row)
#         row += 1

#         ttk.Label(parent, text="Branching Distance").grid(column=0, row=row)
#         ttk.Spinbox(parent, from_=0.0, to=1000.0, increment=10.0, textvariable=self.branch_distance).grid(column=1, row=row)
#         row += 1

#         ttk.Label(parent, text="Branching Angle").grid(column=0, row=row)
#         ttk.Scale(parent, orient=HORIZONTAL, length=200, from_=0, to=np.pi, variable=self.branch_angle).grid(column=1, row=row)
#         ttk.Label(parent, textvariable=self.branch_angle).grid(column=2, row=row)
#         row += 1

#         ttk.Label(parent, text="Extend Before Branching").grid(column=0, row=row)
#         ttk.Checkbutton(parent, variable=self.extend_before_branch).grid(column=1, row=row)
#         row += 1

#         ttk.Label(parent, text="Maximum Branches").grid(column=0, row=row)
#         ttk.Spinbox(parent, from_=0, to=1000, increment=1, textvariable=self.maximum_branches).grid(column=1, row=row)
#         row += 1

#         ttk.Button(parent, text="Generate", command=root.destroy).grid(column=0, row=row)

# root = Tk()
# MorphologyParameters().make_panel(root)
# root.mainloop()
