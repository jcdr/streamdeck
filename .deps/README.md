Local build stubs when system `-dev` packages are not installed.

- `include/` — headers from `libhidapi-dev` and `libudev-dev`
- `pkgconfig/` — pkg-config files pointing at these headers and system libs
- `lib/libudev.so` — symlink to system `libudev.so.1` (linker name `libudev.so`)

Runtime still requires packages such as `libhidapi-hidraw0` and `libudev1`.
