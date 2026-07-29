# Local build stubs

Used when system `-dev` packages are not installed.

| Path | Purpose |
| --- | --- |
| `include/` | Headers from `libhidapi-dev` and `libudev-dev` |
| `pkgconfig/` | pkg-config files pointing at these headers and system libs |
| `lib/` | Optional local linker name `libudev.so` (not committed) |

## Optional local `libudev.so` symlink

If linking fails with `unable to find library -ludev` while only
`libudev.so.1` is installed:

```bash
ln -sfn /usr/lib/$(uname -m)-linux-gnu/libudev.so.1 .deps/lib/libudev.so
# or on some systems:
# ln -sfn /usr/lib64/libudev.so.1 .deps/lib/libudev.so
```

Runtime still requires packages such as `libhidapi-hidraw0` and `libudev1`.

## Why `.gitkeep`?

Git does not track empty directories. `.deps/lib/.gitkeep` is an empty
placeholder so the `lib/` directory exists after clone. Real shared-library
symlinks under `lib/` stay untracked (see root `.gitignore`) because they are
machine-specific absolute links.
