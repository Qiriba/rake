# rake

rake allows you to view and render 3D objects in a simple window. 
It uses the SDL2 library for graphics rendering and provides basic controls for navigating the scene.

## Script

If you do not want to worry about the prerequisites and flags, you ran simply run the rake.sh script.

```bash
sudo ./rake.sh
```

**Note**: This script uses `apt` to install the required packages.

## Prerequisites

This linux port expects the following packages to be installed:

- `libsdl2-dev`

## Running the program

When running the program, you need to specify the path to the 3D object file you want to render.  
This is done via the `-f` flag, followed by the path to the file.

You can also specify the `-t` flag to set the texture file.

In this repository there is an example file included, `capsule.obj` and a texture file `capsule.jpg`.

### Important Note

Rust reads the flags in a special way, so you need to specify the flags **after** separating them with `--`.

**Example**:

```bash
cargo run --release -- -f capsule.obj -t capsule.jpg
```

## Usage

These are the commands to use the rake renderer:

- `tab` - capture the mouse for looking around
- `w`, `a`, `s`, `d` - move forward, left, backward, right
- `space` - jump
- `t` - toggle wireframe mode
- `b` - toggle backface culling (only works in wireframe mode)
- `ESC` - exit the program

The program will render a shaded background to guide you to the object.