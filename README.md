# Morphology Wizard #

This program generates neuron morphologies. It can generate both biologically
realistic neurons and novel synthetic neurons. It implements the TREES
algorithm combined with the morphological constraints of the ROOTS algorithm.


## Installation & Usage ##

The morphology wizard has [**installers**](https://github.com/ctrl-z-9000-times/morphology_wizard/releases) for Windows, Mac, and Linux.  

Once you've successfully installed the program, please proceed to the [**tutorial**](/tutorial/tutorial.md).


## Build Instructions ##

There are three ways to use this project:
* Rust Library
* Python Library
* Graphical User Interface

### Building the Rust Library ###

The morphology wizard is not yet in the `crates.io` public repository. Instead
you must download the code and point your `Cargo.toml` file to the local path
dependency.

```sh
cargo add --git https://github.com/ctrl-z-9000-times/morphology_wizard.git
```

View the rust documentation by calling:

```sh
cargo doc --open --package morphology_wizard
```


### Building the Python Library ###

This project uses the [maturin](https://github.com/PyO3/maturin) framework for
building and distributing the python version of this library. To build it, call:

```sh
git clone https://github.com/ctrl-z-9000-times/morphology_wizard.git
cd morphology_wizard
maturin build
```

The resulting python installable package should be located in the folder: `morphology_wizard/target/wheels`  
Install it using the python package manager of your choice (for example `pip install`).

View the python documentation by calling:

```sh
pydoc morphology_wizard
```


### Building the User Interface ###

This project uses the [tauri](https://tauri.app/) framework for its graphical
user interface. To build an installer for the GUI application, call:

```sh
git clone https://github.com/ctrl-z-9000-times/morphology_wizard.git
cd morphology_wizard
cargo tauri icon
cargo tauri build
```

Tauri should put the resulting installers in the folder: `morphology_wizard/target/release/bundle/`


### Building Installers for all platforms ###

This project uses github actions to provision virtual machines for all supported
platforms. All builds are native, the tauri framework does not yet support
cross-compilation.  
To trigger the action: make a new tag using the format `app-v*`.


## References ##

**TREES:**  
    One Rule to Grow Them All: A General Theory of Neuronal Branching and
    Its Practical Application.  
    Cuntz H, Forstner F, Borst A, Hausser M (2010)  
    PLoS Comput Biol 6(8): e1000877.  
    <https://doi.org/10.1371/journal.pcbi.1000877>

**ROOTS:**  
    ROOTS: An Algorithm to Generate Biologically Realistic Cortical Axons
    and an Application to Electroceutical Modeling.  
    Bingham CS, Mergenthal A, Bouteiller J-MC, Song D, Lazzi G and Berger TW (2020)  
    Front. Comput. Neurosci. 14:13.  
    <https://doi.org/10.3389/fncom.2020.00013>

**Dendrite Diameter:**  
    Optimization principles of dendritic structure.  
    Hermann Cuntz, Alexander Borst and Idan Segev (2007)  
    Theoretical Biology and Medical Modelling 2007, 4:21  
    <https://doi.org:10.1186/1742-4682-4-21>

