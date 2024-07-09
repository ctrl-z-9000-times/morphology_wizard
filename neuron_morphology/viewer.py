from panda3d.core import Geom, GeomNode, GeomTriangles, GeomVertexFormat, GeomVertexData, GeomVertexWriter
from direct.showbase.ShowBase import ShowBase
from direct.task import Task
from panda3d.core import KeyboardButton, MouseButton, LVecBase3, Quat, getDefaultCoordinateSystem, WindowProperties
from primatives import Sphere, Cylinder

class MorphologyViewer(ShowBase):
    def __init__(self):
        ShowBase.__init__(self)
        self.scene = None
        # Setup keyboard movement.
        self.base_speed = 2.0e4
        self.sprint_multiplier = 20.0
        self.task_mgr.add(self._keyboard_movement, 'keyboard_movement')
        # Setup mouse look.
        self.mouse_sensitivity = 4.0
        self._mouse1_down = False
        self.disableMouse()
        self._set_mouse_visible(True)
        self.task_mgr.add(self._mouse_movement, 'mouse_movement')

        # DEBUG: Print all events.
        # self.messenger.toggleVerbose()

        # DEBUG: Print all attributes of this ShowBase instance.
        # for x in sorted(dir(self)): print(x)

    def update_scene(self, nodes, vertices, indices):
        # Discard any existing scene.
        if self.scene is not None:
            self.scene.removeNode()

        # Copy the geometric data into panda3d's internal data structures.
        v_data = GeomVertexData('neuron_morphology', GeomVertexFormat.getV3(), Geom.UHStatic)
        v_data.unclean_set_num_rows(round(len(vertices) / 3))
        v_array = v_data.modify_array(0)
        view = memoryview(v_array).cast('B').cast('f')
        view[:] = vertices

        t_data = GeomTriangles(Geom.UH_static)
        t_data.set_index_type(Geom.NT_uint32)
        t_array = t_data.modify_vertices()
        t_array.unclean_set_num_rows(len(indices))
        view = memoryview(t_array).cast('B').cast('I')
        view[:] = indices

        # Setup the game engine to render the new mesh.
        geom = Geom(v_data)
        geom.addPrimitive(t_data)
        node = GeomNode('neuron_morphology')
        node.addGeom(geom)
        self.scene = self.render.attachNewNode(node)

    def _keyboard_movement(self, task):
        w_button = KeyboardButton.ascii_key('w')
        a_button = KeyboardButton.ascii_key('a')
        s_button = KeyboardButton.ascii_key('s')
        d_button = KeyboardButton.ascii_key('d')
        lctrl_button  = KeyboardButton.lcontrol()
        space_button  = KeyboardButton.space()
        lshift_button = KeyboardButton.lshift()
        up_button     = KeyboardButton.up()
        down_button   = KeyboardButton.down()
        left_button   = KeyboardButton.left()
        right_button  = KeyboardButton.right()
        enter_button  = KeyboardButton.enter()
        rctrl_button  = KeyboardButton.rcontrol()
        rshift_button = KeyboardButton.rshift()

        forward_speed = 0.0
        if self._button_down(w_button) or self._button_down(up_button):
            forward_speed += self.base_speed
        if self._button_down(s_button) or self._button_down(down_button):
            forward_speed -= 0.5 * self.base_speed

        sideways_speed = 0.0
        if self._button_down(a_button) or self._button_down(left_button):
            sideways_speed -= 0.75 * self.base_speed
        if self._button_down(d_button) or self._button_down(right_button):
            sideways_speed += 0.75 * self.base_speed

        upwards_speed = 0.0
        if self._button_down(space_button) or self._button_down(enter_button):
            upwards_speed += 0.75 * self.base_speed
        if self._button_down(lctrl_button) or self._button_down(rctrl_button):
            upwards_speed -= 0.75 * self.base_speed

        if self._button_down(lshift_button) or self._button_down(rshift_button):
            forward_speed  *= self.sprint_multiplier
            sideways_speed *= self.sprint_multiplier
            upwards_speed  *= self.sprint_multiplier

        self._move_forward( task.dt * forward_speed)
        self._move_sideways(task.dt * sideways_speed)
        self._move_upwards( task.dt * upwards_speed)
        return Task.cont

    def _button_down(self, button):
        return self.mouseWatcherNode.is_button_down(button)

    def _move_forward(self, distance):
        direction = self.camera.getQuat().getForward()
        self.camera.setPos(self.camera.getPos() + direction * distance)

    def _move_sideways(self, distance):
        direction = self.camera.getQuat().getRight()
        self.camera.setPos(self.camera.getPos() + direction * distance)

    def _move_upwards(self, distance):
        # direction = self.camera.getQuat().getUp()
        direction = LVecBase3(0.0, 0.0, 1.0)
        self.camera.setPos(self.camera.getPos() + direction * distance)

    def _mouse_movement(self, task):
        mouse1_button = MouseButton.one()

        if not self.mouseWatcherNode.hasMouse():
            self._mouse1_up()
            return Task.cont

        if self._button_down(mouse1_button):
            # Check for initial button down event.
            if not self._mouse1_down:
                self._mouse1_down = True
                self._set_mouse_visible(False)
            else:
                x, y = self.mouseWatcherNode.getMouse()
                # Check for invalid mouse coordiantes, which can happen if the
                # window is still updating the mouse properties.
                if abs(x) > 1.0 or abs(y) > 1.0:
                    x = 0.0
                    y = 0.0
                # Scale the mouse positions from relative positions [-1 to 1] to
                # absolute pixel offsets from the center of the window.
                w, h = self.win.getSize()
                x *= 0.5 * w
                y *= 0.5 * h
                h, p, r = self.camera.getHpr()
                # print(h, p, r)
                h -= 1000.0 * x * task.dt * self.mouse_sensitivity
                p += 1000.0 * y * task.dt * self.mouse_sensitivity
                # Clip rotation into sane bounds and gimble lock it.
                h = h % 360.0
                p = min(90.0, max(-90.0, p))
                self.camera.setH(h)
                self.camera.setP(p)
            self._recenter_mouse()
        else:
            self._mouse1_up()

        return Task.cont

    def _mouse1_up(self):
        if self._mouse1_down:
            self._mouse1_down = False
            self._set_mouse_visible(True)

    def _set_mouse_visible(self, visible):
        props = WindowProperties()
        if visible:
            props.setCursorHidden(False)
            props.setMouseMode(WindowProperties.M_absolute)
        else:
            props.setCursorHidden(True)
            props.setMouseMode(WindowProperties.M_relative)
        self.win.requestProperties(props)

    def _recenter_mouse(self):
        self.win.movePointer(0,
            int(self.win.getProperties().getXSize() / 2),
            int(self.win.getProperties().getYSize() / 2))
