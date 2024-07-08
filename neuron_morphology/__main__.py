from neuron_morphology import *
from viewer import MorphologyViewer
import numpy as np

root_instr = Instruction()
# root_instr.carrier_points = np.random.uniform(-100, 100, size=(10, 3))
root_instr.carrier_points = [[0,0,0]]

dend_instr = Instruction()
dend_instr.carrier_points = np.random.uniform([-300, -300, -5], [300,300, 5], size=(1000, 3))
morphology = Morphology()
morphology.balancing_factor = 0.7
morphology.extension_angle = np.pi / 6
morphology.branch_angle = np.pi / 3
morphology.extend_before_branch = True

dend_instr.morphology = morphology
dend_instr.roots = [0]

print(dend_instr.morphology)

nodes = create([
    root_instr,
    dend_instr,
])

print("number of nodes:", len(nodes))

viewer = MorphologyViewer()
viewer.update_scene(nodes)
viewer.run()
