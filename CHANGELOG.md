## 0.2.0 (Apr 17, 2016)

Features:

- Full support of `msvc` target on Windows. CI tests on AppVeyor now use official reeases of HDF5
  binaries (1.8.16, VS2015, x86_x64). The `gnu` target are still unofficially supported but
  won't be tested.
- If `HDF5_LIBDIR` is not specified when building on Windows and `PATH` contains what looks like
  the `bin` folder of HDF5 installation, the library directory will be inferred automatically.
  The official HDF5 installers add the `bin` folder to user path, so the official MSVC releases
  should just work out of the box without having to set any environment variables.
- The library is now split into three crates: `libhdf5-lib` (requests linkage to HDF5),
  `libhdf5-sys` (contains bindings, requires `libhdf5-lib` at build time in order to conditionally
  enable or disable certain HDF5 functionality), and `hdf5-rs` (the user-facing crate, requires
  both lower-level crates at build time).
- Added `hdf5_rs::hdf5_version` function.
- The minimum required version of the HDF5 library is now 1.8.4.
- Both `libhdf5-sys` and `hdf5-rs` crates can now use version attributes at compile time to
  enable/disable/change functionality. All functions and definitions that appeared in HDF5 versions
  past 1.8.4 are now conditionally enabled in `libhdf5-sys`.
- Added bindings for HDF5 functions that were added in releases 1.8.15 and 1.8.16.
- Static global variables in HDF5 (H5E, H5P, H5T) are now linked based on HDF5 version and not
  the target platform (`_ID_g` variables were introduced in 1.8.14). When `msvc` target is used,
  the dllimport stub prefixes are also accounted for. The constants exposed by `libhdf5-sys` are
  now of reference type and need to be dereferenced upon use (for `msvc`, they have to be
  dereferenced twice).

Changes:

- API simplification: many methods previously expecting `Into<String>` inputs now just take `&str`.
- `util::to_cstring` now takes `Borrow<str>` instead of `Into<String>` so as to avoid
  unnecessary allocations, and the return value is now wrapped in `Result` so that interior
  null bytes in input strings trigger an error.

Bugfixes:

- Fixed dangling pointer problems when strings were being passed as pointers to the C API.
- Fixed target path not being passed correctly in `Container::link_soft`.

## 0.1.1 (July 2015)

Initial public version.
