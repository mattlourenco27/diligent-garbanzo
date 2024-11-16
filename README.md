# Diligent Garbanzo
He's great at drawing

# Installation
If you're installing from a fresh machine, make sure you have all the basic packages for working with C and C++. You'll need these because this Rust project links to the SDL2 C library which will need to be recompiled when building from scratch.

## Linux
Here are some packages that I needed to install before I could get sdl2 working:
* cmake
* gcc
* g++
* libsdl2-dev

```
sudo apt-get install cmake gcc g++ libsdl2-dev
```

### Troubleshooting
#### Mesa Error
```
MESA: error: ZINK: failed to choose pdev
glx: failed to create drisw screen
```

This is an issue with MESA drivers on Ubuntu 24.04. The following steps fixed my issue.
```
sudo add-apt-repository ppa:kisak/kisak-mesa
sudo apt update
sudo apt upgrade
```

#### Missing Vulkan Drivers
```
WARNING: dzn is not a conformant Vulkan implementation, testing use only.
WARNING: dzn is not a conformant Vulkan implementation, testing use only.
WARNING: Some incorrect rendering might occur because the selected Vulkan device (<Device>) doesn't support base Zink requirements: feats.features.logicOp have_EXT_custom_border_color have_EXT_line_rasterization
```
Check for device and platform support: https://docs.vulkan.org/guide/latest/checking_for_support.html

**Please be aware that Vulkan is not supported in WSL**

I had trouble with launching this project on WSL using an NVIDIA GPU. Exporting the following flag fixed my issue:
```
MESA_D3D12_DEFAULT_ADAPTER_NAME=NVIDIA
```

And if that still doesn't work then make sure you have the following installed
```
sudo apt install mesa-utils libglu1-mesa-dev freeglut3-dev mesa-common-dev
```

## MacOS & Windows
See the sdl2 crate landing page: https://crates.io/crates/sdl2