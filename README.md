# hlbsp2obj
Program allows you to convert half-life's `.bsp` to `.obj` file
## How it works
Build (`cargo build --release`) and run with `--help` or `-h` to get help info.
## TO-DO list
- [x] **map convertation**: Works fine, triangulation is possible using `-t` flag. 
- [x] **normals and uv-s export**: Normals, UV-s work fine.
- [x] **textures export & mtl files**: Implemented, but blender rejects to render correct obj file.
- [x] **texture grouping**: Temporary solution, because itertools doesn't already have needed functional
