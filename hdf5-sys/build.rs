#![allow(clippy::must_use_candidate)]
#![allow(clippy::option_if_let_else)]

use std::convert::TryInto;
use std::env;
use std::error::Error;
use std::fmt::{self, Debug};
use std::fs;
use std::os::raw::{c_int, c_uint};
use std::path::{Path, PathBuf};
use std::process::Command;

use regex::Regex;

fn feature_enabled(feature: &str) -> bool {
    env::var(format!("CARGO_FEATURE_{}", feature)).is_ok()
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Version {
    pub major: u8,
    pub minor: u8,
    pub micro: u8,
}

impl Version {
    pub const fn new(major: u8, minor: u8, micro: u8) -> Self {
        Self { major, minor, micro }
    }

    pub fn parse(s: &str) -> Option<Self> {
        let re = Regex::new(r"^(1)\.(8|10|12|14)\.(\d\d?)(_\d+)?((-|.)(patch)?\d+)?$").ok()?;
        let captures = re.captures(s)?;
        Some(Self {
            major: captures.get(1).and_then(|c| c.as_str().parse::<u8>().ok())?,
            minor: captures.get(2).and_then(|c| c.as_str().parse::<u8>().ok())?,
            micro: captures.get(3).and_then(|c| c.as_str().parse::<u8>().ok())?,
        })
    }

    pub fn is_valid(self) -> bool {
        self >= Self { major: 1, minor: 8, micro: 4 }
    }
}

impl Debug for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.micro)
    }
}

#[allow(dead_code)]
fn run_command(cmd: &str, args: &[&str]) -> Option<String> {
    let out = Command::new(cmd).args(args).output();
    if let Ok(ref r1) = out {
        if r1.status.success() {
            let r2 = String::from_utf8(r1.stdout.clone());
            if let Ok(r3) = r2 {
                return Some(r3.trim().to_string());
            }
        }
    }
    None
}

#[allow(dead_code)]
fn is_inc_dir<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().join("H5pubconf.h").is_file() || path.as_ref().join("H5pubconf-64.h").is_file()
}

#[allow(dead_code)]
fn is_root_dir<P: AsRef<Path>>(path: P) -> bool {
    is_inc_dir(path.as_ref().join("include"))
}

#[allow(dead_code)]
fn is_msvc() -> bool {
    // `cfg!(target_env = "msvc")` will report wrong value when using
    // MSVC toolchain targeting GNU.
    std::env::var("CARGO_CFG_TARGET_ENV").unwrap() == "msvc"
}

#[allow(non_snake_case, non_camel_case_types)]
fn get_runtime_version_single<P: AsRef<Path>>(path: P) -> Result<Version, Box<dyn Error>> {
    type H5open_t = unsafe extern "C" fn() -> c_int;
    type H5get_libversion_t = unsafe extern "C" fn(*mut c_uint, *mut c_uint, *mut c_uint) -> c_int;

    let lib = unsafe { libloading::Library::new(path.as_ref()) }?;
    let H5open = unsafe { lib.get::<H5open_t>(b"H5open")? };
    let H5get_libversion = unsafe { lib.get::<H5get_libversion_t>(b"H5get_libversion")? };

    let mut v: (c_uint, c_uint, c_uint) = (0, 0, 0);
    let res = unsafe {
        if H5open() != 0 {
            Err("H5open()".into())
        } else if H5get_libversion(&mut v.0, &mut v.1, &mut v.2) != 0 {
            Err("H5get_libversion()".into())
        } else {
            Ok(Version::new(
                v.0.try_into().unwrap(),
                v.1.try_into().unwrap(),
                v.2.try_into().unwrap(),
            ))
        }
    };
    // On macos libraries using TLS will corrupt TLS from rust. We delay closing
    // the library until program exit by forgetting the library
    std::mem::forget(lib);
    res
}

fn validate_runtime_version(config: &Config) {
    println!("Looking for HDF5 library binary...");
    let libfiles = &["libhdf5.dylib", "libhdf5.so", "hdf5.dll", "libhdf5-0.dll", "libhdf5-310.dll"];
    let mut link_paths = config.link_paths.clone();
    if cfg!(all(unix, not(target_os = "macos"))) {
        if let Some(ldv) = run_command("ld", &["--verbose"]) {
            let re = Regex::new(r#"SEARCH_DIR\("=?(?P<path>[^"]+)"\)"#).unwrap();
            println!("Adding extra link paths (ld)...");
            for caps in re.captures_iter(&ldv) {
                let path = &caps["path"];
                println!("    {}", path);
                link_paths.push(path.into());
            }
        } else {
            println!("Unable to add extra link paths (ld).");
        }
    }
    for link_path in &link_paths {
        if let Ok(paths) = fs::read_dir(link_path) {
            for path in paths.flatten() {
                let path = path.path();
                if let Some(filename) = path.file_name() {
                    let filename = filename.to_str().unwrap_or("");
                    if path.is_file() && libfiles.contains(&filename) {
                        println!("Attempting to load: {:?}", path);
                        match get_runtime_version_single(&path) {
                            Ok(version) => {
                                println!("    => runtime version = {:?}", version);
                                if version == config.header.version {
                                    println!("HDF5 library runtime version matches headers.");
                                    return;
                                }
                                panic!(
                                    "Invalid HDF5 runtime version (expected: {:?}).",
                                    config.header.version
                                );
                            }
                            Err(err) => {
                                println!("    => {}", err);
                            }
                        }
                    }
                }
            }
        }
    }
    panic!("Unable to infer HDF5 library runtime version (can't find the binary).");
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Header {
    pub have_stdbool_h: bool,
    pub have_direct: bool,
    pub have_parallel: bool,
    pub have_threadsafe: bool,
    pub have_no_deprecated: bool,
    pub have_filter_deflate: bool,
    pub version: Version,
}

impl Header {
    pub fn parse<P: AsRef<Path>>(inc_dir: P) -> Self {
        let inc_dir = inc_dir.as_ref();

        let header = get_conf_header(inc_dir);
        println!("Parsing HDF5 config from:\n    {:?}", header);

        let contents = fs::read_to_string(header).unwrap();
        let mut hdr = Self::default();

        let num_def_re = Regex::new(r"(?m)^#define\s+(H5_[A-Z_]+)\s+([0-9]+)\s*$").unwrap();
        for captures in num_def_re.captures_iter(&contents) {
            let name = captures.get(1).unwrap().as_str();
            let value = captures.get(2).unwrap().as_str().parse::<i64>().unwrap();
            if name == "H5_HAVE_STDBOOL_H" {
                hdr.have_stdbool_h = value > 0;
            } else if name == "H5_HAVE_DIRECT" {
                hdr.have_direct = value > 0;
            } else if name == "H5_HAVE_PARALLEL" {
                hdr.have_parallel = value > 0;
            } else if name == "H5_HAVE_THREADSAFE" {
                hdr.have_threadsafe = value > 0;
            } else if name == "H5_HAVE_FILTER_DEFLATE" {
                hdr.have_filter_deflate = value > 0;
            } else if name == "H5_NO_DEPRECATED_SYMBOLS" {
                hdr.have_no_deprecated = value > 0;
            }
        }

        let str_def_re = Regex::new(r#"(?m)^#define\s+(H5_[A-Z_]+)\s+"([^"]+)"\s*$"#).unwrap();
        for captures in str_def_re.captures_iter(&contents) {
            let name = captures.get(1).unwrap().as_str();
            let value = captures.get(2).unwrap().as_str();
            if name == "H5_VERSION" {
                if let Some(version) = Version::parse(value) {
                    hdr.version = version;
                } else {
                    panic!("Invalid H5_VERSION: {:?}", value);
                }
            };
        }

        assert!(hdr.version.is_valid(), "Invalid H5_VERSION in the header: {:?}", hdr.version);

        hdr
    }
}

fn get_conf_header<P: AsRef<Path>>(inc_dir: P) -> PathBuf {
    let inc_dir = inc_dir.as_ref();

    if inc_dir.join("H5pubconf.h").is_file() {
        inc_dir.join("H5pubconf.h")
    } else if inc_dir.join("H5pubconf-64.h").is_file() {
        inc_dir.join("H5pubconf-64.h")
    } else {
        panic!("H5pubconf header not found in include directory");
    }
}

#[derive(Clone, Debug, Default)]
pub struct LibrarySearcher {
    pub version: Option<Version>,
    pub inc_dir: Option<PathBuf>,
    pub link_paths: Vec<PathBuf>,
    pub user_provided_dir: bool,
    pub pkg_conf_found: bool,
}

#[cfg(any(all(unix, not(target_os = "macos")), windows))]
mod pkgconf {
    use super::{is_inc_dir, LibrarySearcher};

    pub fn find_hdf5_via_pkg_config(config: &mut LibrarySearcher) {
        if config.inc_dir.is_some() {
            return;
        }

        // If we're going to windows-gnu we can use pkg-config, but only so long as
        // we're coming from a windows host.
        if cfg!(windows) {
            std::env::set_var("PKG_CONFIG_ALLOW_CROSS", "1");
        }

        // Try pkg-config. Note that HDF5 only ships pkg-config metadata
        // in CMake builds (which is not what homebrew uses, for example).
        // Still, this would work sometimes on Linux.
        let mut pc = pkg_config::Config::new();
        pc.cargo_metadata(false);
        println!("Attempting to find HDF5 via pkg-config...");
        if let Ok(library) = pc.probe("hdf5") {
            println!("Found HDF5 pkg-config entry");
            println!("    Include paths:");
            for dir in &library.include_paths {
                println!("    - {:?}", dir);
            }
            println!("    Link paths:");
            for dir in &library.link_paths {
                println!("    - {:?}", dir);
            }
            for dir in &library.include_paths {
                if is_inc_dir(dir) {
                    config.inc_dir = Some(dir.into());
                    config.link_paths.clone_from(&library.link_paths);
                    break;
                }
            }
            if let Some(ref inc_dir) = config.inc_dir {
                println!("Located HDF5 headers at:");
                println!("    {:?}", inc_dir);
            } else {
                println!("Unable to locate HDF5 headers from pkg-config info.");
            }

            config.pkg_conf_found = true;
        }
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
mod unix {
    pub use super::pkgconf::find_hdf5_via_pkg_config;
    use super::{is_inc_dir, LibrarySearcher};

    pub fn find_hdf5_in_default_location(config: &mut LibrarySearcher) {
        if config.inc_dir.is_some() {
            return;
        }
        for (inc_dir, lib_dir) in &[
            ("/usr/include/hdf5/serial", "/usr/lib/x86_64-linux-gnu/hdf5/serial"),
            ("/usr/include", "/usr/lib/x86_64-linux-gnu"),
            ("/usr/include", "/usr/lib64"),
        ] {
            if is_inc_dir(inc_dir) {
                println!("Found HDF5 headers at:\n    {:?}", inc_dir);
                println!("Adding to link path:\n    {:?}", lib_dir);
                config.inc_dir = Some(inc_dir.into());
                config.link_paths.push(lib_dir.into());
                break;
            }
        }
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use super::*;

    pub fn find_hdf5_via_homebrew(config: &mut LibrarySearcher) {
        if config.inc_dir.is_some() {
            return;
        }
        // We have to explicitly support homebrew since the HDF5 bottle isn't
        // packaged with pkg-config metadata.
        let (v18, v110, v112, v114) = if let Some(version) = config.version {
            (
                version.major == 1 && version.minor == 8,
                version.major == 1 && version.minor == 10,
                version.major == 1 && version.minor == 12,
                version.major == 1 && version.minor == 14,
            )
        } else {
            (false, false, false, false)
        };
        println!(
            "Attempting to find HDF5 via Homebrew ({})...",
            if v18 {
                "1.8.*"
            } else if v110 {
                "1.10.*"
            } else if v112 {
                "1.12.*"
            } else if v114 {
                "1.14.*"
            } else {
                "any version"
            }
        );
        if !(v18 || v110 || v112) {
            if let Some(out) = run_command("brew", &["--prefix", "hdf5@1.14"]) {
                if is_root_dir(&out) {
                    config.inc_dir = Some(PathBuf::from(out).join("include"));
                }
            }
        }
        if !(v18 || v110) {
            if let Some(out) = run_command("brew", &["--prefix", "hdf5@1.12"]) {
                if is_root_dir(&out) {
                    config.inc_dir = Some(PathBuf::from(out).join("include"));
                }
            }
        }
        if config.inc_dir.is_none() && !v18 {
            if let Some(out) = run_command("brew", &["--prefix", "hdf5@1.10"]) {
                if is_root_dir(&out) {
                    config.inc_dir = Some(PathBuf::from(out).join("include"));
                }
            }
        }
        if config.inc_dir.is_none() {
            if let Some(out) = run_command("brew", &["--prefix", "hdf5@1.8"]) {
                if is_root_dir(&out) {
                    config.inc_dir = Some(PathBuf::from(out).join("include"));
                }
            }
        }
        if config.inc_dir.is_none() {
            if let Some(out) = run_command("brew", &["--prefix", "hdf5-mpi"]) {
                if is_root_dir(&out) {
                    config.inc_dir = Some(PathBuf::from(out).join("include"));
                }
            }
        }
        if let Some(ref inc_dir) = config.inc_dir {
            println!("Found Homebrew HDF5 headers at:");
            println!("    {:?}", inc_dir);
        }
    }
}

#[cfg(windows)]
mod windows {
    pub use super::pkgconf::find_hdf5_via_pkg_config;
    use super::*;

    use std::io;

    use serde::de::Error;
    use serde::{Deserialize, Deserializer};
    use serde_derive::Deserialize as DeriveDeserialize;
    use winreg::enums::HKEY_LOCAL_MACHINE;
    use winreg::RegKey;

    impl<'de> Deserialize<'de> for Version {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let s = String::deserialize(deserializer)?;
            Version::parse(&s).ok_or_else(|| Error::custom("invalid version"))
        }
    }

    #[derive(Clone, DeriveDeserialize)]
    struct App {
        #[serde(rename = "DisplayName")]
        name: String,
        #[serde(rename = "DisplayVersion")]
        version: Version,
        #[serde(rename = "InstallLocation")]
        location: PathBuf,
    }

    impl Debug for App {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{} {:?} ({:?})", self.name, self.version, self.location)
        }
    }

    impl App {
        fn check_hdf5(&self, version: Option<Version>) -> bool {
            version.unwrap_or(self.version) == self.version
                && &self.name == "HDF5"
                && self.version.is_valid()
        }
    }

    fn get_installed_apps() -> io::Result<Vec<App>> {
        const KEY: &'static str = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall";
        let root = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey(KEY)?;
        let mut installed = Vec::new();
        for key in root.enum_keys().filter_map(Result::ok) {
            let app = root.open_subkey(key).ok().and_then(|v| v.decode::<App>().ok());
            if let Some(app) = app {
                installed.push(app);
            }
        }
        Ok(installed)
    }

    fn get_hdf5_app(version: Option<Version>) -> Option<App> {
        if let Some(version) = version {
            println!("Searching for installed HDF5 with version {:?}...", version);
        } else {
            println!("Searching for installed HDF5 (any version)...")
        }
        let apps = get_installed_apps().ok()?;
        let mut apps: Vec<_> = apps.iter().filter(|app| app.check_hdf5(version)).collect();
        apps.sort_by_key(|app| app.version);
        if apps.is_empty() {
            println!("Found no HDF5 installations.");
            return None;
        }
        let latest = apps[apps.len() - 1];
        if apps.len() == 1 {
            println!("Found exactly one HDF5 installation:");
        } else {
            println!("Found multiple HDF5 installations:");
        };
        for app in &apps {
            println!("- {:?}", app);
        }
        if apps.len() > 1 {
            println!("Selecting the latest version ({:?}):", latest.version);
            println!("- {:?}", latest);
        }
        Some(latest.clone())
    }

    pub fn find_hdf5_via_winreg(config: &mut LibrarySearcher) {
        // Official HDF5 binaries on Windows are built for MSVC toolchain only.
        if config.inc_dir.is_some() || !is_msvc() {
            return;
        }
        // Check the list of installed programs, see if there's HDF5 anywhere;
        // if the version is provided, try to match that, otherwise pick the
        // latest version available.
        if let Some(app) = get_hdf5_app(config.version) {
            config.version = Some(app.version);
            config.inc_dir = Some(PathBuf::from(app.location).join("include"));
        }
    }

    pub fn validate_env_path(config: &LibrarySearcher) {
        if let Some(ref inc_dir) = config.inc_dir {
            let var_path = env::var("PATH").unwrap_or_else(|_| Default::default());
            let bin_dir = inc_dir.parent().unwrap().join("bin").canonicalize().unwrap();
            for path in env::split_paths(&var_path) {
                if let Ok(path) = path.canonicalize() {
                    if path == bin_dir {
                        println!("Found in PATH: {:?}", path);
                        return;
                    }
                }
            }
            panic!("{:?} not found in PATH.", bin_dir);
        }
    }
}

impl LibrarySearcher {
    pub fn new_from_env() -> Self {
        let mut config = Self::default();
        if let Some(var) = env::var_os("HDF5_DIR") {
            println!("Setting HDF5 root from environment variable:");
            println!("    HDF5_DIR = {:?}", var);
            let root = PathBuf::from(var);

            assert!(!root.is_relative(), "HDF5_DIR cannot be relative.");
            assert!(root.is_dir(), "HDF5_DIR is not a directory.");

            config.user_provided_dir = true;
            config.inc_dir = Some(root.join("include"));
        }
        if is_msvc() {
            // in order to allow HDF5_DIR to be pointed to a conda environment, we have
            // to support MSVC as a special case (where the root is in $PREFIX/Library)
            if let Some(ref inc_dir) = config.inc_dir {
                if let Some(root_dir) = inc_dir.parent() {
                    let alt_inc_dir = root_dir.join("Library").join("include");
                    if !is_inc_dir(inc_dir) && is_inc_dir(&alt_inc_dir) {
                        println!("Detected MSVC conda environment, changing headers dir to:");
                        println!("    {:?}", alt_inc_dir);
                        config.inc_dir = Some(alt_inc_dir);
                    }
                }
            }
        }
        if let Ok(var) = env::var("HDF5_VERSION") {
            println!("Setting HDF5 version from environment variable:");
            println!("    HDF5_VERSION = {:?}", var);
            if let Some(v) = Version::parse(&var) {
                config.version = Some(v);
            } else {
                panic!("Invalid HDF5 version: {}", var);
            }
        }
        config
    }

    pub fn try_locate_hdf5_library(&mut self) {
        #[cfg(all(unix, not(target_os = "macos")))]
        {
            self::unix::find_hdf5_via_pkg_config(self);
            self::unix::find_hdf5_in_default_location(self);
        }
        #[cfg(target_os = "macos")]
        {
            self::macos::find_hdf5_via_homebrew(self);
        }
        #[cfg(windows)]
        {
            self::windows::find_hdf5_via_winreg(self);
            self::windows::find_hdf5_via_pkg_config(self);
            // the check below is for dynamic linking only
            self::windows::validate_env_path(self);
        }
        if let Some(ref inc_dir) = self.inc_dir {
            if cfg!(unix) {
                if let Some(envdir) = inc_dir.parent() {
                    if self.user_provided_dir {
                        let lib_dir = format!("{}/lib", envdir.to_string_lossy());
                        println!("Custom HDF5_DIR provided; rpath can be set via:");
                        println!("    RUSTFLAGS=\"-C link-args=-Wl,-rpath,{}\"", lib_dir);
                        if cfg!(target_os = "macos") {
                            println!("On some OS X installations, you may also need to set:");
                            println!("    DYLD_FALLBACK_LIBRARY_PATH=\"{}\"", lib_dir);
                        }
                    }
                }
            }
        } else {
            panic!("Unable to locate HDF5 root directory and/or headers.");
        }
    }

    pub fn finalize(self) -> Config {
        if let Some(ref inc_dir) = self.inc_dir {
            assert!(is_inc_dir(inc_dir), "Invalid HDF5 headers directory: {:?}", inc_dir);
            let mut link_paths = self.link_paths;
            if link_paths.is_empty() {
                if let Some(root_dir) = inc_dir.parent() {
                    link_paths.push(root_dir.join("lib"));
                    link_paths.push(root_dir.join("bin"));
                }
            }
            let header = Header::parse(inc_dir);
            if let Some(version) = self.version {
                assert_eq!(header.version, version, "HDF5 header version mismatch",);
            }
            let config = Config { inc_dir: inc_dir.clone(), link_paths, header };
            // Don't check version if pkg-config finds the library and this is a windows target.
            // We trust the pkg-config provided path, to avoid updating dll names every time
            // the package updates.
            if !(self.pkg_conf_found && cfg!(windows)) {
                validate_runtime_version(&config);
            }
            config.check_against_features_required();
            config
        } else {
            panic!("Unable to determine HDF5 location (set HDF5_DIR to specify it manually).");
        }
    }
}

#[derive(Clone, Debug)]
pub struct Config {
    pub inc_dir: PathBuf,
    pub link_paths: Vec<PathBuf>,
    pub header: Header,
}

impl Config {
    pub fn emit_link_flags(&self) {
        println!("cargo:rustc-link-lib=dylib=hdf5");
        for dir in &self.link_paths {
            println!("cargo:rustc-link-search=native={}", dir.to_str().unwrap());
        }
        println!("cargo:rerun-if-env-changed=HDF5_DIR");
        println!("cargo:rerun-if-env-changed=HDF5_VERSION");

        if is_msvc() {
            println!("cargo:msvc_dll_indirection=1");
        }
        println!("cargo:include={}", self.inc_dir.to_str().unwrap());

        println!("cargo:library=hdf5");

        if feature_enabled("HL") {
            println!("cargo:hl_library=hdf5_hl");
        }
    }

    pub fn emit_cfg_flags(&self) {
        let version = self.header.version;
        assert!(version >= Version::new(1, 8, 4), "required HDF5 version: >=1.8.4");
        let mut vs: Vec<_> = (5..=21).map(|v| Version::new(1, 8, v)).collect(); // 1.8.[5-23]
        vs.extend((0..=8).map(|v| Version::new(1, 10, v))); // 1.10.[0-10]
        vs.extend((0..=2).map(|v| Version::new(1, 12, v))); // 1.12.[0-2]
        vs.extend((0..=1).map(|v| Version::new(1, 14, v))); // 1.14.[0-1]
        for v in vs.into_iter().filter(|&v| version >= v) {
            println!("cargo:rustc-cfg=feature=\"{}.{}.{}\"", v.major, v.minor, v.micro);
            println!("cargo:version_{}_{}_{}=1", v.major, v.minor, v.micro);
        }
        if self.header.have_stdbool_h {
            println!("cargo:rustc-cfg=have_stdbool_h");
            // there should be no need to export have_stdbool_h downstream
        }
        if self.header.have_direct {
            println!("cargo:rustc-cfg=feature=\"have-direct\"");
            println!("cargo:have_direct=1");
        }
        if self.header.have_parallel {
            println!("cargo:rustc-cfg=feature=\"have-parallel\"");
            println!("cargo:have_parallel=1");
        }
        if self.header.have_threadsafe {
            println!("cargo:rustc-cfg=feature=\"have-threadsafe\"");
            println!("cargo:have_threadsafe=1");
        }
        if self.header.have_filter_deflate {
            println!("cargo:rustc-cfg=feature=\"have-filter-deflate\"");
            println!("cargo:have_filter_deflate=1");
        }
    }

    fn check_against_features_required(&self) {
        let h = &self.header;
        for (flag, feature, native) in [
            (!h.have_no_deprecated, "deprecated", "HDF5_ENABLE_DEPRECATED_SYMBOLS"),
            (h.have_threadsafe, "threadsafe", "HDF5_ENABLE_THREADSAFE"),
            (h.have_filter_deflate, "zlib", "HDF5_ENABLE_Z_LIB_SUPPORT"),
        ] {
            if feature_enabled(&feature.to_ascii_uppercase()) {
                assert!(
                    flag,
                    "Enabled feature {feature:?} but the HDF5 library was not built with {native}"
                );
            }
        }
    }
}

fn main() {
    if feature_enabled("STATIC") && std::env::var_os("HDF5_DIR").is_none() {
        get_build_and_emit();
    } else {
        let mut searcher = LibrarySearcher::new_from_env();
        searcher.try_locate_hdf5_library();
        let config = searcher.finalize();
        println!("{:#?}", config);
        config.emit_link_flags();
        config.emit_cfg_flags();
    }
}

fn get_build_and_emit() {
    println!("cargo:rerun-if-changed=build.rs");

    if feature_enabled("ZLIB") {
        let zlib_lib = env::var("DEP_HDF5SRC_ZLIB").unwrap();
        println!("cargo:zlib={}", &zlib_lib);
        let zlib_lib_header = env::var("DEP_HDF5SRC_ZLIB").unwrap();
        println!("cargo:zlib={}", &zlib_lib_header);
        println!("cargo:rustc-link-lib=static={}", &zlib_lib);
    }

    if feature_enabled("HL") {
        let hdf5_hl_lib = env::var("DEP_HDF5SRC_HL_LIBRARY").unwrap();
        println!("cargo:rustc-link-lib=static={}", &hdf5_hl_lib);
        println!("cargo:hl_library={}", &hdf5_hl_lib);
    }

    let hdf5_root = env::var("DEP_HDF5SRC_ROOT").unwrap();
    println!("cargo:root={}", &hdf5_root);
    let hdf5_incdir = env::var("DEP_HDF5SRC_INCLUDE").unwrap();
    println!("cargo:include={}", &hdf5_incdir);
    let hdf5_lib = env::var("DEP_HDF5SRC_LIBRARY").unwrap();
    println!("cargo:library={}", &hdf5_lib);

    println!("cargo:rustc-link-search=native={}/lib", &hdf5_root);
    println!("cargo:rustc-link-lib=static={}", &hdf5_lib);

    let header = Header::parse(&hdf5_incdir);
    let config = Config { header, inc_dir: "".into(), link_paths: Vec::new() };
    config.emit_cfg_flags();
}
