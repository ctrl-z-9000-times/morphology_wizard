# Getting Started with the Morphology Wizard #

This tutorial will guide you through creating a simple neuron.

1) First open the morphology wizard program. It should look like this:

![The morphology wizard at startup](/tutorial/step_1.avif)

2) Click on the "New Soma" button and enter a name for the soma type.

![Making a new soma type](/tutorial/step_2.avif)

3) Click on the "New Dendrite" button and enter a name for the dendrite type.

4) Edit the "roots" property of the dendrite, and select your soma's name.  
Dendrites start growing from their "roots". You need to specify some roots before it can grow.

![Connecting the dendrite to the soma](/tutorial/step_4.avif)

5) Click on the "windows" menu at the top of the screen and select "carrier points".
The carrier points window allows you to specify the 3-D layout of the neuron.

![Highlighting the windows menu](/tutorial/step_5.avif)

6) Click on the "New Single Point" button and enter a name for the new point.
The soma will be located at these coordinates.

7) Click on the "New Sphere" button and enter a name for the new group of points.
The dendrites will be located inside of this sphere.

8) Edit the "Num Points" property of the sphere, increase it from 1 to 100.

![Editing the carrier points](/tutorial/step_8.avif)

9) Switch back to the "growth instructions" window by using the "windows" menu
at the top of the screen.

10) Select your soma from the list of instructions and edit the "carrier points"
property by selecting the carrier point that you made for the soma.

![Selecting the carrier points](/tutorial/step_10.avif)

11) Select your dendrite from the list of instructions and edit the "carrier
points" property by selecting the sphere of carrier points that you made for
the dendrite.

12) Open the "Morphology Viewer" window using the "windows" menu at the top of
the screen and clicking on the "preview morphology" button. You should see your
neuron.
    - Left click and drag on the screen to rotate around the neuron.
    - Right click and drag to pan sideways through the scene.
    - Mouse wheel up/down to zoom in/out on the neuron.

![Viewing your new neuron](/tutorial/step_12.avif)

