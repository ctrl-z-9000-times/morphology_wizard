from neuron_morphology import *
from viewer import MorphologyViewer
import numpy as np

root_instr = Instruction()
# root_instr.carrier_points = np.random.uniform(-100, 100, size=(10, 3))
root_instr.carrier_points = [[0,0,0]]

hdist = lambda x: (x[0]**2 + x[1]**2)**0.5
def cylinder_points(min_height, max_height, radius, num_points):
    carrier_points = np.random.uniform([-radius, -radius, min_height], [radius, radius, max_height], size=(num_points, 3))
    carrier_points = [x for x in carrier_points if (x[0]**2 + x[1]**2) <= radius**2]
    return carrier_points

apical_instr = Instruction()
apical_instr.carrier_points = cylinder_points(80, 120, 160, 1500)
apical_instr.roots = [0]
apical_instr.morphology = Morphology()
print(apical_instr.morphology)

oblique_instr = Instruction()
oblique_instr.carrier_points = cylinder_points(20, 80, 40, 100)
oblique_instr.roots = [1]
oblique_instr.morphology = Morphology()

basal_instr = Instruction()
basal_instr.carrier_points = cylinder_points(-40, 20, 280, 4000)
basal_instr.roots = [0]
basal_morph = Morphology()
basal_morph.balancing_factor = 1.2
basal_instr.morphology = basal_morph

nodes, vertices, indices = create_and_render([
    root_instr,
    apical_instr,
    oblique_instr,
    basal_instr,
])

print("number of nodes:", len(nodes))

viewer = MorphologyViewer()
viewer.update_scene(nodes, vertices, indices)
viewer.run()
