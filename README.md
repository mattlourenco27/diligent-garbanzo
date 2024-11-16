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

## MacOS & Windows
See the sdl2 crate landing page: https://crates.io/crates/sdl2