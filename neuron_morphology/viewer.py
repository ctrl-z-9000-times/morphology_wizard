from direct.showbase.ShowBase import ShowBase
from panda3d.core import Geom, GeomNode, GeomTriangles, GeomVertexFormat, GeomVertexData, GeomVertexWriter
from primatives import Sphere, Cylinder

class MorphologyViewer(ShowBase):
    def __init__(self):
        ShowBase.__init__(self)
        self.scene = None

    def update_scene(self, nodes):
        # Discard any existing scene.
        if self.scene is not None:
            self.scene.removeNode()

        # Create geometric primatives for every node in the morphology.
        primatives = []
        for node in nodes:
            if node.is_root():
                radius = 15
                slices=10
                primatives.append(Sphere(node.coordinates(), radius, slices))
            else:
                parent = nodes[node.parent_index()]
                radius = 3
                slices = 5
                primatives.append(Cylinder(node.coordinates(), parent.coordinates(), radius, slices))

        # Copy the geometric data into panda3d's internal data structures.
        v_data = GeomVertexData('neuron_morphology', GeomVertexFormat.getV3(), Geom.UHStatic)
        v_size = sum(len(obj3d.get_vertices()) for obj3d in primatives)
        v_data.setNumRows(v_size)
        v_writer = GeomVertexWriter(v_data, 'vertex')

        geom = Geom(v_data)
        offset = 0
        for object3d in primatives:
            for v in object3d.get_vertices():
                v_writer.addData3(*v)

            prim = GeomTriangles(Geom.UHStatic)
            for tri in object3d.get_indices():
                prim.addVertex(int(tri[0] + offset))
                prim.addVertex(int(tri[1] + offset))
                prim.addVertex(int(tri[2] + offset))
                prim.closePrimitive()
            geom.addPrimitive(prim)
            offset += len(object3d.get_vertices())

        node = GeomNode('neuron_morphology')
        node.addGeom(geom)
        self.scene = self.render.attachNewNode(node)
