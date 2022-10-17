# Changelog

## Unreleased

### Added

- Support for HDF5 version 1.13.0.
- Support field renaming via `#[hdf5(rename = "new_name")]` helper attribute.
- Add a `ByteReader` which implements `std::io::{Read, Seek}` for 1D `u8`
  datasets. Usage via `Dataset::as_byte_reader()`.

### Changed

- The `H5Type` derive macro now uses `proc-macro-error` to emit error messages.
- MSRV is now `1.54` following a bump in a dependency.

### Fixed

- Fixed a bug where `H5Pget_fapl_direct` was only included when HDF5 was compiled
  with feature `have-parallel` instead of `have-direct`.
- Fixed a missing symbol when building `hdf5-src` with `libz-sys`.
- Fixed a bug where errors were only silenced on the main thread.
- Fixed a memory leak when opening datasets.

## 0.8.1

Release date: Nov 21, 2021.

### Added

- `Error` now implements `From<Infallible>`, which allows passing convertible
  extents (like tuples of integers) where `impl TryInto<Extents>` is required.
- Support for HDF5 versions 1.12.1 and 1.10.8.
- `#[derive(H5Type)]` now supports structs / tuple structs with `repr(packed)`.
- `#[derive(H5Type)]` now supports structs / tuple structs with
  `repr(transparent)` (the generated HDF5 type is equivalent to the type of
  the field and is not compound).

### Changed

- Renamed `filters::gzip_available()` to `deflate_available()` (the old name is
  present but marked as deprecated).
- Code dependent on HDF5 version in `hdf5` and `hdf5-sys` crates now uses features
  instead of cfg options: `cfg(feature = "1.10.1")` instead of `cfg(hdf5_1_10_1)`.
  The main initial reason for that is for HDF5 versions to show up in the official
  documentation on docs.rs.
- Similar to the above, there's `have-direct`, `have-parallel` and `have-threadsafe`
  features that reflect the build configuration of the underlying HDF5 library.

### Fixed

- Fixed a bug where all blosc filter settings were discarded / zeroed out.
- Fixed errors when recovering filter pipelines from stored datasets.
- Fixed a bug where filter availability was computed incorrectly.

## 0.8.0

Release date: Oct 23, 2021.

### Added

- Complete rewrite of `DatasetBuilder`; dataset creation API is now different
  and not backwards-compatible (however, it integrates all the new features
  and is more flexible and powerful). It is now possible to create and write
  datasets in one step. Refer to the API docs for full reference.
- Added new `Extents` type matching HDF5 extents types: null (no elements),
  scalar, simple (fixed dimensionality); it is used to query and specify shapes
  of datasets. Extents objects are convertible from numbers and also tuples,
  slices and vectors of indices -- all of which can be used whenever passing
  extents is required (e.g., when creating a new dataset or an attribute).
- Added new `Selection` type with the surrounding API closely matching native
  HDF5 selection API. This includes 'all' selection, point-wise selection and
  hyperslab selection (only 'regular' hyperslabs are supported -- that is,
  hyperslabs that can be represented as a single multi-dimensional box some of
  whose dimensions may be infinite). Selection objects are convertible from
  integers, ranges, tuples and arrays of integers and ranges; one can also use
  `s!` macro from `ndarray` crate if needed. Selections can be provided when
  reading and writing slices.
- Support for LZF / Blosc filters has been added. These filters are enabled by
  "lzf" / "blosc" cargo features and depend on `lzf-sys` / `blosc-src` crates
  respectively. Blosc filter is a meta-filter providing multi-threaded access
  to the best-in-class compression codecs like Zstd and LZ4 and is recommended
  to use as a default when compression performance is critical.
- Added new `Filter` type to unify the filters API; if LZF / Blosc filters are
  enabled, this enum also contains the corresponding variants. It is now also
  possible to provide user-defined filters with custom filter IDs and configs.
- Added wrappers for dataset creation property list (DCPL) API. This provides
  access to the properties that can be specified at dataset creation time
  (e.g., layout, chunking, fill values, external file linking, virtual maps,
  object time tracking, attribute creation order, and a few other settings).
- Virtual dataset maps (VDS API in HDF5 1.10+) are now supported.
- Added wrappers for link creation property list (LCPL) API.
- File creation property list (FCPL) API has been extended to include a few
  previously missing properties (object time tracking, attribute creation order
  and few other settings).
- Added "h5-alloc" feature to `hdf5-types` crate. It enables using the HDF5
  library allocator for varlen types and dynamic values. This may be necessary
  on platforms where different allocators may be used in different libraries
  (e.g. dynamic libraries on Windows) or if `libhdf5` is compiled with the
  memchecker option enabled. This option is force-enabled by default if using
  a dll version of the HDF5 library on Windows.
- Added new `DynValue` type which represents a dynamic self-describing HDF5
  object that also knows how to deallocate itself. It supports all the HDF5
  types including compound types, strings and arrays.
- Added support for attributes and a new `Attribute` object type. The attribute
  API uses the new dataset API with some restrictions imposed by HDF5 library
  (e.g. one can not perform partial IO, attributes must be read all at once).
- `hdf5-sys` now exports new functions added in HDF5 1.10.6 and 1.10.7.
- Added to `Dataset`:
  - `layout`: get the dataset layout.
  - `dapl`, `access_plist`: get the dataset access plist.
  - `dcpl`, `create_plist`: get the dataset create plist.
- Added to `Location`:
  - `loc_info`, `loc_info_by_name`: retrieve information on a location.
  - `loc_type`, `loc_type_by_name`: retrieve location type.
  - `open_by_token`: open a location by its token (physical address).
- Added to `Group`:
  - `iter_visit`: closure API for iterating over objects in a group.
  - `iter_visit_default`: like `iter_visit` but with default iteration order.
  - `get_all_of_type`: find all objects in a group of a given type.
  - Shortcut methods for finding all objects in a group of a given type:
    `datasets`, `groups`, `named_datatypes`, `link_external`.
- Added to `Handle`:
  - `id_type`: get native HDF5 object type.
  - `try_borrow`: instantiate a handle but don't take ownership of the object.
  - `Handle` now implements `Debug`.
- Added to `ObjectClass`:
  - `cast()`: safe counterpart to `cast_unchecked()`.
- Added to `Object`:
  - Safe downcast methods: `as_file`, `as_group`, `as_datatype`,
    `as_dataspace`, `as_dataset`, `as_attr`, `as_location`,
    `as_container`, `as_plist`.
- Added to `FileAccessBuilder`:
  - `libver_earliest`, `libver_v18`, `libver_v110`, `libver_latest`:
    shortcuts for setting the minimum library version.
- Added to `FileAccess`:
  - `libver`: shortcut for getting the minimum library version.

 ### Changed

- Required Rust compiler version is now `1.51`.
- Removed `num-integer` and `num-traits` dependencies.
- `Dataspace` type has been reworked and can be now constructed from an
  `Extents` object and sliced with a `Selection` object.
- `Dataset::fill_value` now returns an object of the newly added `DynValue`
  type; this object is self-describing and knows how to free itself.
- Automatic chunking now uses a fill-from-back approach instead of the
  previously used method which was borrowed from `h5py`.
- Removed the old `Filters` type (replaced by `Filter` that represents a
  single filter).
- `write_slice`, `read_slice`, `read_slice_1d`, `read_slice_2d` in `Container`
  now take any object convertible to `Selection` (instead of `SliceInfo`).
- `Dataset::chunks` has been renamed to `Dataset::chunk`.
- Const generics support (MSRV 1.51): `hdf5-types` now uses const generics for
  array types, allowing fixed-size arrays of arbitrary sizes. `Array` trait has
  been removed. String types are now generic over size: `FixedAscii<N>` and
  `FixedUnicode<N>`.
- `ndarray` dependency has been updated to `0.15`.
- The version of HDF5 in `hdf5-src` has been updated from 1.10.6 to 1.10.7.
- `zlib` dependency is no longer included with `default-features`.
- The crate no longer calls `H5close` automatically on program exit.
- Errors are now silenced, and will not be written to stderr by default.
- `silence_errors` now works globally instead of using guards.
- Errors are no longer automatically expanded into error stacks when
  encountered. This can be still done manually (e.g. via printing the error).
- Handles to HDF5 identifiers are no longer tracked via a global registry;
  identifier safety is now enforced via stricter semantics of ownership.
- Handles to `File` objects will no longer close all objects contained in the
  file when dropped -- weak file close degree is now used instead. For the old
  behaviour see `FileCloseDegree::Strong`.
- HDF5 global variables no longer create a `lazy_static` per variable.
- Unsafe `cast()` in `ObjectClass` has been renamed to `cast_unchecked()`.
- Bump `winreg` (Windows only) to 0.10, `pretty_assertions` (dev) to 1.0.
- Updated the example in the readme to showcase the new features.

### Fixed

- A potential memory leak of identifier handles has been identified and fixed.
- A potential race condition occurring in multi-thread library initialisation
  has been identified and fixed.

### Removed

- Free-standing functions `get_id_type`, `is_valid_id`, `is_valid_user_id`
  have been removed in favor of `Handle` methods.

## 0.7.1

Release date: Jan 27, 2021.

### Added

- Slices can now be used where trait item `Dimension` is required.
- Arrays of arbitrary sizes are now supported in `hdf5-types`. This requires
  the crate feature `const_generics` and minimum Rust version of 1.51.

### Changed

- Dependencies are bumped to the newest major versions; `ndarray` users may
  now use both version `0.13` and version `0.14`.

### Fixed

- Cross-compilation of `hdf5-src` from Unix to Windows will now use the correct
  name of the static library when linking.

## 0.7.0

Release date: Aug 9, 2020.

### Added

- HDF5 C library can now be built from source and linked in statically, enabled
  via `hdf5-sys/static` feature (as of this release, the version of the bundled
  sources of HDF5 is 1.10.6). CMake is required for building. For further
  details, see the docs for `hdf5-sys`.
- Thanks to static build option, the documentation will now be built on
  [docs.rs](https://docs.rs/crate/hdf5); if it builds successfully, this
  will be the official documentation source from now on.
- Add support for HDF5 1.12 on all platforms and include it in CI.

### Changed

- Switched CI from Travis/AppVeyor to GitHub Actions; for each pull request, we
  now run around 30 concurrent builds which provides with a much wider coverage
  than previously and has already revealed some issues that have been fixed.
  Platforms covered: macOS 10.15, Windows Server 2019, Ubuntu 16.04/18.04/20.04;
  HDF5 installation methods covered: conda, apt, homebrew, also official
  binaries on Windows. We now also test MPI versions of the library for
  both MPICH and OpenMPI on macOS / Linux.

### Fixed

- We now force the variable-length allocator that HDF5 uses when reading data
  to use `libc::malloc` and `libc::free`, so that they can be deallocated
  properly by `VarLenString` and `VarLenArray` in `hdf5-types`. Previously,
  this could cause a rare but serious failure for Windows builds when the
  default allocator used for vlen types by HDF5 was not matching the
  libc deallocator.
- Use `std::panic::catch_unwind` in all cases where we use extern C callbacks,
  so that they are panic-safe.
- `Reader::read_raw` and `Reader::read_slice` should now be `Drop`-safe in the
  event where the read operation fails and the destination type is not trivial.

## 0.6.1

Release date: Apr 12, 2020.

### Added

- Implement `Default` for `H5D_layout_t`, `H5D_alloc_time_t`, `H5T_cset_t`,
  `H5D_fill_time_t`, `H5D_fill_value_t` (based on what their values are set to
  in default property lists).
- Derive `Debug` for `ErrorFrame` and `ErrorStack`.
- Implement `Display` for `TypeDescriptor`.
- Implement `Dimension` for `[Ix]`, `&[Ix]`, `[Ix; N]` and `&[Ix; N]`.

### Changed

- `h5check!`, `h5lock!`, `h5try!`, `h5call!` and `h5check()` are now public.
- `globals` module containing HDF5 runtime constants is now also public.
- Switch to using 1.0 versions of `syn`, `quote` and `proc-macro2` (which
  required a bit of a rewrite of `hdf5-derive`).
- Bump `ascii` to 1.0, update `hdf5-types` to be compatible.
- Bump other dependencies to their latest versions (`parking_lot` to 0.10,
  `ndarray` to 0.13, `bitflags` to 1.2, `lazy_static` to 1.4,
  `libloading` to 0.6, `winreg` to 0.7).
- Remove implementations of deprecated `Error::description()`.
- Switch to `trybuild` instead of `compiletest_rs` for derive-macro testing;
  enable full tests (including hdf5-derive) on both AppVeyor and Travis.
- Update the minimum Rust version to 1.40 (due to `ndarray` and `libloading`).

### Fixed

- Use `H5_free_memory()` instead of `libc::free()` to free HDF5-allocated
  resources if possible (1.8.13+); this fixes some deallocation errors.
- Fix wrong `H5D_DONT_FILTER_PARTIAL_CHUNKS` constant value.
- `H5Z_filter_t` is changed to `c_int` (was wrongfully set to `hid_t`).

## 0.6.0

Release date: Feb 17, 2020.

### Added

- Added support for HDF5 1.10.5 with bindings for new functions.
- `File::access_plist()` or `File::fapl()` to get file access plist.
- `File::create_plist()` or `File::fcpl()` to get file creation plist.
- Added wrappers for dataset access H5P API in `plist::DatasetAccess`.
- Added `is_library_threadsafe()` function.
- Added `Group::member_names()`.
- Added `Datatype::byte_order()`.
- Added `Dataset::num_chunks()` (1.10.5+).
- Added `Dataset::chunk_info()` (1.10.5+).

### Changed

- Changed `File` constructors, getting rid of string access modes:
  - `File::open(path, "r")` is now `File::open(path)`
  - `File::open(path, "r+")` is now `File::open_rw(path)`
  - `File::open(path, "w")` is now `File::create(path)`
  - `File::open(path, "x" | "w-")` is now `File::create_excl(path)`
  - `File::open(path, "a")` is now `File::append(path)`
- Also added `File::open_as(path, mode)` which accepts the mode enum.
- Rewritten `FileBuilder`: it no longer accepts userblock, driver etc.; all of
  these parameters can be set in the corresponding FAPL / FCPL:
  - `FileBuilder::set_access_plist()` or `FileBuilder::set_fapl()` to set the
    active file access plist to a given one.
  - `FileBuilder::access_plist()` or `FileBuilder::fapl()` to get a mutable
    reference to the FAPL builder -- any parameter can then be tweaked.
  - `FileBuilder::with_access_plist()` or `FileBuilder::with_fapl()` to get
    access to the FAPL builder in an inline way via a closure.
  - Same as the three above for `create_plist` / `fcpl`.
- As a result, all the newly added FAPL / FCPL functionality is fully
  accessible in the new `FileBuilder`. Also, driver strings are gone,
  everything is strongly typed now.
- It's no longer prohibited to set FCPL options when opening a file and not
  creating it -- it will simply be silently ignored (this simplifies the
  behavior and allows using a single file builder).
- Added an explicit `hdf5_types::string::StringError` error type, and
  `error-chain` dependency has now been dropped.
- `Error` is now convertible from `ndarray::ShapeError`; `ResultExt` trait has
  been removed.
- Renamed `hdf5_version()` to `library_version()`.

### Fixed

- Replaced deprecated `std::mem::uninitialized` with `std::mem::MaybeUninit`.
- Fixed a serde-related problem with building `hdf5-sys` on Windows.

## 0.5.2

Release date: Jul 14, 2019.

### Changed

- Allow chunk dimensions to exceed dataset dimensions for resizable datasets.
- Default HDF5 location should now be detected automatically on Fedora Linux.

## 0.5.1

Release date: Mar 8, 2019.

### Added

- Added `Group::link_exists()`.
- Re-export `silence_errors()` at the crate root.
- Added `from_id()` unsafe method at the crate root.

### Changed

- `#[derive(H5Type)]` no longer requires adding `hdf5-types` as a dependency.

## 0.5.0

Release date: Mar 8, 2019.

### Added

- Added support for HDF5 1.10.
- Added Rust equivalents of HDF5 primitives: arrays, Unicode strings and ASCII
  strings – all of them available in both fixed-size/variable-length flavours
  (see `hdf5-types` crate for details).
- Added `H5Type` trait that unifies the types that can be handled by the HDF5
  library. This trait is implemented by default for all scalar types, tuples,
  fixed-size arrays and all types in `hdf5-types` and can be used to create
  `Datatype` objects for known types.
- Implemented `#[derive(H5Type)]` proc macro that allows for seamless mapping
  of user-defined structs and enums to their HDF5 counterparts.
- Added high-level wrappers for file-creation H5P API (`plist::FileCreate`) and
  file-access H5P API (`plist::FileAccess`), covering almost the entirety of
  FCPL and FAPL property list functionality.
- Various improvements and additions to `PropertyList` type.
- Added support for file drivers: sec2/stdio/core/family/multi/split/log.
- Added support for MPIO driver (requires `H5_HAVE_PARALLEL` HDF5 flag and the
  crate has to be built with "mpio" feature enabled).
- Added support for direct VFD driver (requires `H5_HAVE_DIRECT` HDF5 flag).
- Added some missing bindings to `hdf5-sys`: driver-related FAPL bindings in
  `h5p` and `h5fd` (including MPIO and direct VFD drivers), MPIO bindings in
  `h5p`, `h5f` and `h5fd`.
- Added core reading/writing API in `Container`, with support for reading and
  writing scalars, 1-D/2-D/dynamic-dimensional arrays and raw slices. As a
  side effect, the main crate now depends on `ndarray`. `Dataset` now
  dereferences to `Container`.
- Added basic support for reading and writing dataset slices.
- When creating datasets, in-memory type layouts are normalized (converted to
  C representation).
- Added `packed` option to `DatasetBuilder` for creating packed HDF5 layouts.
- All object types now implement `Clone` (shallow copy, increases refcount).

### Changed

- Renamed `hdf5-rs` crate to `hdf5`.
- Renamed `libhdf5-sys` crate to `hdf5-sys` (importable as `hdf5_sys`).
- Renamed GitHub repository to `aldanor/hdf5-rust`.
- Updated the bindings and tests to the latest HDF5 versions (1.10.4 / 1.8.21).
- The build system has been reworked from the ground up:
  - `hdf5-lib` crate has been removed; all the build-time logic now resides in
    the build script of `hdf5-sys`.
  - The build script now looks for `HDF5_DIR` and `HDF5_VERSION` env vars.
  - `pkg-config` is now only launched on Linux.
  - macOS: the build script detects Homebrew installations (both 1.8 and 1.10).
  - Windows: we now scan the registry to detect official system-wide installs.
  - `HDF5_DIR` can be now pointed to a conda env for dynamic linking.
  - A few definitions from `H5pubconf.h` are now exposed as cfg definitions,
    like `h5_have_parallel`, `h5_have_threadsafe` and `h5_have_direct` (this
    requires locating the include folder and parsing the header at build time).
- Various cleanups in `hdf5-sys`: implemented `Default` and `Clone` where
  applicable, moved a few types and methods to matching parent modules.
- Major refactor: trait-based type hierarchy has been replaced with a
  `Deref`-based hierarchy instead (53eff4f). `ID` and `FromID` traits have been
  removed. Traits like `Location`, `Object` and a few others have been replaced
  with real types (wrappers around HDF5 handles, same as the concrete types
  like `File`). Subtypes then dereference into parent types, so the user can
  call methods of the parent type without having to import any traits into
  scope (for instance, `File` dereferences into `Group`, which dereferences
  into `Location`, which dereferences into `Object`).
- Dataspaces and property lists can now be properly copied via `.copy()` method
  (instead of `.clone()` which now yields a shallow copy increasing refcount).

### Fixed

- `hbool_t` is now mapped to unsigned integer of proper size (either 1 byte or
  4 bytes), depending on how the HDF5 library was built and on which platform.
- Added missing bindings for previous versions (mostly in `h5p` and `h5fd`).
- Querying the HDF5 error stack is now thread-safe.
- Error silencing (`silence_errors()`) is now thread-safe.
- Fixed wrong bindings for `H5AC_cache_config_t`.

### Removed

- Removed `hdf5-lib` crate (merged it into `hdf5-sys`, see above).
- Removed `remutex` crate, using locking primitives from `parking_lot` crate.
- `Container` trait has been removed in favor of `Group` type.

### Notes

- The version number jump is due to renaming crates `hdf5-rs` and `libhdf5-sys`
  to `hdf5` and `hdf5-sys`, respectively. Since there were already published
  crates with those names and the crates registry is meant to be immutable even
  if the crates are yanked, we had to bump the version so that it shadows all
  the older versions.

## 0.2.0

Release date: Jul 29, 2015.

### Added

- Full support of `msvc` target on Windows. CI tests on AppVeyor now use
  official releases of HDF5 binaries (1.8.16, VS2015, x86_x64). The `gnu`
  target are still unofficially supported but won't be tested.
- If `HDF5_LIBDIR` is not specified when building on Windows and `PATH`
  contains what looks like the `bin` folder of HDF5 installation, the library
  directory will be inferred automatically. The official HDF5 installers add
  the `bin` folder to user path, so the official MSVC releases should just work
  out of the box without having to set any environment variables.
- The library is now split into three crates: `hdf5-lib` (requests linkage to
  HDF5), `hdf5-sys` (contains bindings, requires `hdf5-lib` at build time in
  order to conditionally enable or disable certain HDF5 functionality), and
  `hdf5` (the user-facing crate, requires both lower-level crates at build time).
- Added `hdf5::hdf5_version` function.
- The minimum required version of the HDF5 library is now 1.8.4.
- Both `hdf5-sys` and `hdf5` crates can now use version attributes at compile
  time to enable/disable/change functionality. All functions and definitions
  that appeared in HDF5 versions past 1.8.4 are now conditionally enabled in
  `hdf5-sys`.
- Added bindings for HDF5 functions that were added in 1.8.15 and 1.8.16.
- Static global variables in HDF5 (H5E, H5P, H5T) are now linked based on HDF5
  version and not the target platform (`_ID_g` variables were introduced in
  1.8.14). When `msvc` target is used, dllimport stub prefixes are also
  accounted for. Constants exposed by `hdf5-sys` are now of reference type and
  need to be dereferenced (for `msvc`, they have to be dereferenced twice).

### Changed

- API simplification: many methods previously expecting `Into<String>` inputs
  now just take `&str`.
- `util::to_cstring` now takes `Borrow<str>` instead of `Into<String>` to avoid
  unnecessary allocations, and the return value is now wrapped in `Result` so
  that interior null bytes in input strings trigger an error.

### Fixed

- Fixed dangling pointer problems when strings were passed as pointers to
  the C API.
- Fixed target path not being passed correctly in `Container::link_soft`.

## 0.1.0

Release date: Jul 27, 2015.

Initial public version.
