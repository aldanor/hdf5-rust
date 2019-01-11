## 0.3.0-alpha.1 (unreleased)

Updates:

- Added Rust equivalents of HDF5 primitives: arrays, Unicode strings and ASCII strings – all of 
  them available in both fixed-size or variable-length flavours (`hdf5-types` crate).
- Added `H5Type` trait that unifies the types that can be handled by the HDF5 library. This trait
  is implemented by default for all scalar types, tuples, fixed-size arrays and all types in
  `hdf5-types`.
- Implemented `#[derive(H5Type)]` proc macro that allows for seamless mapping of user-defined 
  structs and enums to their HDF5 counterparts.
- `Datatype` can now be constructed directly from an `H5Type`-compatible type (or from/to
  a type descriptor).
- Added support for HDF5 1.10.
- Updated the bindings and test to the latest HDF5 versions (1.10.4 and 1.8.21).
- Added missing bindings for previous versions (mostly in `h5p` and `h5fd` modules).
- Removed `remutex` crate, using locking primitives from `parking_lot` crate instead.
- Major refactor: trait-based type hierarchy has been replaced with a `Deref`-based
  hierarchy instead (53eff4f). `ID` and `FromID` traits have been removed. Traits like `Location`,
  `Object` and a few other have been replaced with real types (wrappers around HDF5 handles, same
  as the concrete types like `File`). Subtypes then dereference into parent types, so the
  user can user methods of the parent type without having to import any traits into scope
  (for instance, `File` dereferences into `Group`, which dereferences into `Location`,
  which dereferences into `Object`).
- `Container` trait has been removed, all of its functionality is moved into `Group` type.
- Added high-level wrappers for file-creation H5P API (`plist::FileCreate`) and
  file-access H5P API (`plist::FileAccess`).
- Various improvements and additions to `PropertyList` type.
- Added support for various file drivers (sec2/stdio/core/family/multi/split/log).
- Querying the HDF5 error stack is now thread-safe.
- Error silencing (`silence_errors()`) is now thread-safe.
- Various clean ups in `libhdf5-sys`: implemented `Default` and `Clone` where
  applicable, moved a few types and methods to matching parent modules.
- `Dataset` now dereferences into `Container` (dataset/attribute shared functionality).
- The main crate now depends on `ndarray` (multi-dimensional arrays).
- Added core reading/writing API for `Container`, with support for reading/writing scalars, 
  1-d, 2-d, and dynamic-dimensional arrays, and raw slices.
- Added basic support for reading and writing dataset slices.
- Added `packed` option to `DatasetBuilder` (for creating packed HDF5 datasets).
- When creating datasets, in-memory type layouts are normalized (converted to C repr).

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
- Added `h5::hdf5_version` function.
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
