# Diligent Garbanzo

Diligent Garbanzo is a standard-compliant SVG renderer. By the nature of the SVG format, Diligent Garbanzo generates geometry at an infinite resolution.

> This project is still in progress. Features like texture rendering are not yet ready.

## Usage

To render an SVG file, pass in the path to the file as a command-line argument

```sh
cargo run <path to your SVG file>
```

To move the SVG around your screen, you can use the `left-mouse-button` to click and drag or use the `arrow-keys`.

To zoom in or out, you can use the `mouse-wheel` or the `i` / `o` keys on the keyboard.

## Installation

If you're installing from a fresh machine, make sure you have CMake and all the basic packages for working with C and C++. You'll need these because this Rust project statically links to the SDL2 C library which will need to be recompiled when building from scratch.

### Linux / WSL

> **Note if you are working with WSL**: Since this is a graphics library, this project needs to interact with your GPU and may require some driver tweaking. Using WSL for this project is possible but using Windows is much less painful since it has better driver support.

Ensure you have the following packages installed such that SDL2 can compile from scratch:

* cmake
* gcc
* g++
* libsdl2-dev

```bash
sudo apt-get install cmake gcc g++ libsdl2-dev
```

### Troubleshooting - Linux / WSL

#### Mesa Error

```text
MESA: error: ZINK: failed to choose pdev
glx: failed to create drisw screen
```

This is an issue with MESA drivers on Ubuntu 24.04. The following steps fixed my issue.

```bash
sudo add-apt-repository ppa:kisak/kisak-mesa
sudo apt update
sudo apt upgrade
```

#### Missing Vulkan Drivers

```text
WARNING: dzn is not a conformant Vulkan implementation, testing use only.
WARNING: dzn is not a conformant Vulkan implementation, testing use only.
WARNING: Some incorrect rendering might occur because the selected Vulkan device (<Device>) doesn't support base Zink requirements: feats.features.logicOp have_EXT_custom_border_color have_EXT_line_rasterization
```

**Please be aware that Vulkan is not supported in WSL**.
Check for device and platform support: <https://docs.vulkan.org/guide/latest/checking_for_support.html>

WSL sometimes doesn't play well with NVIDIA GPUs. Exporting the following flag might fix the issue:

```bash
MESA_D3D12_DEFAULT_ADAPTER_NAME=NVIDIA
```

And if that still doesn't work then make sure you have the following installed

```bash
sudo apt install mesa-utils libglu1-mesa-dev freeglut3-dev mesa-common-dev
```

### MacOS

Install CMake: <https://cmake.org/download/>
See the sdl2 crate landing page: <https://crates.io/crates/sdl2>

### Windows

Make sure you have a working C/C++ compiler installed. Installing Rust on Windows should already guarantee this.
Install CMake: <https://cmake.org/download/>
See the sdl2 crate landing page if you have any further problems: <https://crates.io/crates/sdl2>
