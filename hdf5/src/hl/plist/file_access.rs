//! File access properties.

/*
Not supported due to complexity combined with low likelihood of ever being used:

- Low level direct VFD access: H5P{set,get}_driver, H5Pget_driver_info
- Direct file image API: H5P{set,get}_file_image, H5P{set,get}_file_image_callbacks
- Custom file access property lists in multi/family drivers
- Interfacing directly with multi/family parts via types/offsets
*/

use std::fmt::{self, Debug};
use std::iter;
use std::mem;
use std::ops::Deref;
use std::ptr::{self, addr_of, addr_of_mut};

use bitflags::bitflags;

use hdf5_sys::h5ac::{
    H5AC_cache_config_t, H5AC_METADATA_WRITE_STRATEGY__DISTRIBUTED,
    H5AC_METADATA_WRITE_STRATEGY__PROCESS_0_ONLY, H5AC__CURR_CACHE_CONFIG_VERSION,
    H5AC__MAX_TRACE_FILE_NAME_LEN,
};
use hdf5_sys::h5c::{H5C_cache_decr_mode, H5C_cache_flash_incr_mode, H5C_cache_incr_mode};
use hdf5_sys::h5f::{H5F_close_degree_t, H5F_mem_t, H5F_FAMILY_DEFAULT};
use hdf5_sys::h5fd::H5FD_MEM_NTYPES;
use hdf5_sys::h5fd::{
    H5FD_LOG_ALL, H5FD_LOG_FILE_IO, H5FD_LOG_FILE_READ, H5FD_LOG_FILE_WRITE, H5FD_LOG_FLAVOR,
    H5FD_LOG_FREE, H5FD_LOG_LOC_IO, H5FD_LOG_LOC_READ, H5FD_LOG_LOC_SEEK, H5FD_LOG_LOC_WRITE,
    H5FD_LOG_META_IO, H5FD_LOG_NUM_IO, H5FD_LOG_NUM_READ, H5FD_LOG_NUM_SEEK, H5FD_LOG_NUM_TRUNCATE,
    H5FD_LOG_NUM_WRITE, H5FD_LOG_TIME_CLOSE, H5FD_LOG_TIME_IO, H5FD_LOG_TIME_OPEN,
    H5FD_LOG_TIME_READ, H5FD_LOG_TIME_SEEK, H5FD_LOG_TIME_STAT, H5FD_LOG_TIME_TRUNCATE,
    H5FD_LOG_TIME_WRITE, H5FD_LOG_TRUNCATE,
};
use hdf5_sys::h5p::{
    H5Pcreate, H5Pget_alignment, H5Pget_cache, H5Pget_driver, H5Pget_fapl_core, H5Pget_fapl_family,
    H5Pget_fapl_multi, H5Pget_fclose_degree, H5Pget_gc_references, H5Pget_mdc_config,
    H5Pget_meta_block_size, H5Pget_sieve_buf_size, H5Pget_small_data_block_size, H5Pset_alignment,
    H5Pset_cache, H5Pset_fapl_core, H5Pset_fapl_family, H5Pset_fapl_log, H5Pset_fapl_multi,
    H5Pset_fapl_sec2, H5Pset_fapl_split, H5Pset_fapl_stdio, H5Pset_fclose_degree,
    H5Pset_gc_references, H5Pset_mdc_config, H5Pset_meta_block_size, H5Pset_sieve_buf_size,
    H5Pset_small_data_block_size,
};
#[cfg(feature = "have-direct")]
use hdf5_sys::h5p::{H5Pget_fapl_direct, H5Pset_fapl_direct};
#[cfg(feature = "mpio")]
use hdf5_sys::h5p::{H5Pget_fapl_mpio, H5Pset_fapl_mpio};

#[cfg(feature = "1.10.1")]
use hdf5_sys::h5ac::{H5AC_cache_image_config_t, H5AC__CACHE_IMAGE__ENTRY_AGEOUT__NONE};
#[cfg(feature = "1.10.2")]
use hdf5_sys::h5f::H5F_libver_t;
#[cfg(all(feature = "1.10.0", feature = "have-parallel"))]
use hdf5_sys::h5p::{
    H5Pget_all_coll_metadata_ops, H5Pget_coll_metadata_write, H5Pset_all_coll_metadata_ops,
    H5Pset_coll_metadata_write,
};
#[cfg(feature = "1.8.13")]
use hdf5_sys::h5p::{H5Pget_core_write_tracking, H5Pset_core_write_tracking};
#[cfg(feature = "1.8.7")]
use hdf5_sys::h5p::{H5Pget_elink_file_cache_size, H5Pset_elink_file_cache_size};
#[cfg(feature = "1.10.1")]
use hdf5_sys::h5p::{
    H5Pget_evict_on_close, H5Pget_mdc_image_config, H5Pget_page_buffer_size, H5Pset_evict_on_close,
    H5Pset_mdc_image_config, H5Pset_page_buffer_size,
};
#[cfg(feature = "1.10.2")]
use hdf5_sys::h5p::{H5Pget_libver_bounds, H5Pset_libver_bounds};
#[cfg(feature = "1.10.0")]
use hdf5_sys::h5p::{
    H5Pget_mdc_log_options, H5Pget_metadata_read_attempts, H5Pset_mdc_log_options,
    H5Pset_metadata_read_attempts,
};

#[cfg(feature = "have-direct")]
use crate::globals::H5FD_DIRECT;
#[cfg(feature = "mpio")]
use crate::globals::H5FD_MPIO;
use crate::globals::{
    H5FD_CORE, H5FD_FAMILY, H5FD_LOG, H5FD_MULTI, H5FD_SEC2, H5FD_STDIO, H5P_FILE_ACCESS,
};
use crate::internal_prelude::*;

/// File access properties.
#[repr(transparent)]
pub struct FileAccess(Handle);

impl ObjectClass for FileAccess {
    const NAME: &'static str = "file access property list";
    const VALID_TYPES: &'static [H5I_type_t] = &[H5I_GENPROP_LST];

    fn from_handle(handle: Handle) -> Self {
        Self(handle)
    }

    fn handle(&self) -> &Handle {
        &self.0
    }

    fn validate(&self) -> Result<()> {
        ensure!(
            self.is_class(PropertyListClass::FileAccess),
            "expected file access property list, got {:?}",
            self.class()
        );
        Ok(())
    }
}

impl Debug for FileAccess {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut formatter = f.debug_struct("FileAccess");
        formatter.field("alignment", &self.alignment());
        formatter.field("chunk_cache", &self.chunk_cache());
        formatter.field("fclose_degree", &self.fclose_degree());
        formatter.field("gc_references", &self.gc_references());
        formatter.field("small_data_block_size", &self.small_data_block_size());
        #[cfg(feature = "1.10.2")]
        formatter.field("libver_bounds", &self.libver_bounds());
        #[cfg(feature = "1.8.7")]
        formatter.field("elink_file_cache_size", &self.elink_file_cache_size());
        formatter.field("meta_block_size", &self.meta_block_size());
        #[cfg(feature = "1.10.1")]
        formatter.field("page_buffer_size", &self.page_buffer_size());
        #[cfg(feature = "1.10.1")]
        formatter.field("evict_on_close", &self.evict_on_close());
        #[cfg(feature = "1.10.1")]
        formatter.field("mdc_image_config", &self.mdc_image_config());
        formatter.field("sieve_buf_size", &self.sieve_buf_size());
        #[cfg(feature = "1.10.0")]
        formatter.field("metadata_read_attempts", &self.metadata_read_attempts());
        #[cfg(feature = "1.10.0")]
        formatter.field("mdc_log_options", &self.mdc_log_options());
        #[cfg(all(feature = "1.10.0", feature = "have-parallel"))]
        formatter.field("all_coll_metadata_ops", &self.all_coll_metadata_ops());
        #[cfg(all(feature = "1.10.0", feature = "have-parallel"))]
        formatter.field("coll_metadata_write", &self.coll_metadata_write());
        formatter.field("mdc_config", &self.mdc_config());
        formatter.field("driver", &self.driver());
        formatter.finish()
    }
}

impl Deref for FileAccess {
    type Target = PropertyList;

    fn deref(&self) -> &PropertyList {
        unsafe { self.transmute() }
    }
}

impl PartialEq for FileAccess {
    fn eq(&self, other: &Self) -> bool {
        <PropertyList as PartialEq>::eq(self, other)
    }
}

impl Eq for FileAccess {}

impl Clone for FileAccess {
    fn clone(&self) -> Self {
        unsafe { self.deref().clone().cast_unchecked() }
    }
}

/// Core file driver properties.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct CoreDriver {
    /// Size, in bytes, of memory increments.
    pub increment: usize,
    /// Whether to write the file contents to disk when the file is closed.
    pub filebacked: bool,
    /// Size, in bytes, of write aggregation pages. Setting to 1 enables tracking with no paging.
    #[cfg(feature = "1.8.13")]
    pub write_tracking: usize,
}

impl Default for CoreDriver {
    fn default() -> Self {
        Self {
            increment: 1024 * 1024,
            filebacked: false,
            #[cfg(feature = "1.8.13")]
            write_tracking: 0,
        }
    }
}

/// Family file driver properties.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FamilyDriver {
    /// Size in bytes of each file member.
    pub member_size: usize,
}

impl Default for FamilyDriver {
    fn default() -> Self {
        Self { member_size: H5F_FAMILY_DEFAULT as _ }
    }
}

bitflags! {
    /// Flags specifying types of logging activity for the logging virtual file driver.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct LogFlags: u64 {
        /// Track truncate operations.
        const TRUNCATE = H5FD_LOG_TRUNCATE;
        /// Track "meta" operations (e.g. truncate).
        const META_IO = H5FD_LOG_META_IO;
        /// Track the location of every read.
        const LOC_READ = H5FD_LOG_LOC_READ;
        /// Track the location of every write.
        const LOC_WRITE = H5FD_LOG_LOC_WRITE;
        /// Track the location of every seek.
        const LOC_SEEK = H5FD_LOG_LOC_SEEK;
        /// Track all I/O locations and lengths.
        /// Equivalent to `LOC_READ | LOC_WRITE | LOC_SEEK`.
        const LOC_IO = H5FD_LOG_LOC_IO;
        /// Track the number of times each byte is read.
        const FILE_READ = H5FD_LOG_FILE_READ;
        /// Track the number of times each byte is written.
        const FILE_WRITE = H5FD_LOG_FILE_WRITE;
        /// Track the number of all types of I/O operations.
        /// Equivalent to `FILE_READ | FILE_WRITE`.
        const FILE_IO = H5FD_LOG_FILE_IO;
        /// Track the type of information stored at each byte.
        const FLAVOR = H5FD_LOG_FLAVOR;
        /// Track the total number of read operations.
        const NUM_READ = H5FD_LOG_NUM_READ;
        /// Track the total number of write operations.
        const NUM_WRITE = H5FD_LOG_NUM_WRITE;
        /// Track the total number of seek operations.
        const NUM_SEEK = H5FD_LOG_NUM_SEEK;
        /// Track the total number of truncate operations.
        const NUM_TRUNCATE = H5FD_LOG_NUM_TRUNCATE;
        /// Track the total number of all types of I/O operations.
        /// Equivalent to `NUM_READ | NUM_WRITE | NUM_SEEK | NUM_TRUNCATE`.
        const NUM_IO = H5FD_LOG_NUM_IO;
        /// Track the time spent in open operations.
        const TIME_OPEN = H5FD_LOG_TIME_OPEN;
        /// Track the time spent in stat operations.
        const TIME_STAT = H5FD_LOG_TIME_STAT;
        /// Track the time spent in read operations.
        const TIME_READ = H5FD_LOG_TIME_READ;
        /// Track the time spent in write operations.
        const TIME_WRITE = H5FD_LOG_TIME_WRITE;
        /// Track the time spent in seek operations.
        const TIME_SEEK = H5FD_LOG_TIME_SEEK;
        /// Track the time spent in truncate operations.
        const TIME_TRUNCATE = H5FD_LOG_TIME_TRUNCATE;
        /// Track the time spent in close operations.
        const TIME_CLOSE = H5FD_LOG_TIME_CLOSE;
        /// Track the time spent in each I/O operation.
        /// Equivalent to `TIME_OPEN | TIME_STAT | TIME_READ | TIME_WRITE | TIME_SEEK
        /// | TIME_TRUNCATE | TIME_CLOSE`.
        const TIME_IO = H5FD_LOG_TIME_IO;
        /// Track releases of space in the file.
        const FREE = H5FD_LOG_FREE;
        /// Track everything.
        const ALL = H5FD_LOG_ALL;
    }
}

impl Default for LogFlags {
    fn default() -> Self {
        Self::LOC_IO
    }
}

/// Logging virtual file driver properties.
#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct LogOptions {
    logfile: Option<String>,
    flags: LogFlags,
    buf_size: usize,
}

static FD_MEM_TYPES: &[H5F_mem_t] = &[
    H5F_mem_t::H5FD_MEM_DEFAULT,
    H5F_mem_t::H5FD_MEM_SUPER,
    H5F_mem_t::H5FD_MEM_BTREE,
    H5F_mem_t::H5FD_MEM_DRAW,
    H5F_mem_t::H5FD_MEM_GHEAP,
    H5F_mem_t::H5FD_MEM_LHEAP,
    H5F_mem_t::H5FD_MEM_OHDR,
];

/// Properties for a data storage used by the multi-file driver.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MultiFile {
    /// Name of the member file.
    pub name: String,
    /// Offset within virtual address space where the storage begins.
    pub addr: u64,
}

impl MultiFile {
    /// Creates a new `MultiFile`.
    pub fn new(name: &str, addr: u64) -> Self {
        Self { name: name.into(), addr }
    }
}

/// A mapping of memory usage types to storage indices.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MultiLayout {
    /// Index of the superblock.
    pub mem_super: u8,
    /// Index of the B-tree data.
    pub mem_btree: u8,
    /// Index of the raw data.
    pub mem_draw: u8,
    /// Index of the global heap data.
    pub mem_gheap: u8,
    /// Index of the local heap data.
    pub mem_lheap: u8,
    /// Index of the object headers.
    pub mem_object: u8,
}

impl Default for MultiLayout {
    fn default() -> Self {
        Self { mem_super: 0, mem_btree: 1, mem_draw: 2, mem_gheap: 3, mem_lheap: 4, mem_object: 5 }
    }
}

impl MultiLayout {
    pub(crate) fn get(&self, index: usize) -> &u8 {
        match index {
            0 => &self.mem_super,
            1 => &self.mem_btree,
            2 => &self.mem_draw,
            3 => &self.mem_gheap,
            4 => &self.mem_lheap,
            5 => &self.mem_object,
            _ => unreachable!(),
        }
    }

    pub(crate) fn get_mut(&mut self, index: usize) -> &mut u8 {
        match index {
            0 => &mut self.mem_super,
            1 => &mut self.mem_btree,
            2 => &mut self.mem_draw,
            3 => &mut self.mem_gheap,
            4 => &mut self.mem_lheap,
            5 => &mut self.mem_object,
            _ => unreachable!(),
        }
    }
}

/// Multi-file driver properties.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MultiDriver {
    /// The names and offsets of each type of data storage.
    pub files: Vec<MultiFile>,
    /// The mapping of memory usage types to file indices.
    pub layout: MultiLayout,
    /// Whether to allow read-only access to incomplete file sets.
    pub relax: bool,
}

impl Default for MultiDriver {
    fn default() -> Self {
        let m = u64::max_value() / 6;
        let files = vec![
            MultiFile::new("%s-s.h5", 0 * m),
            MultiFile::new("%s-b.h5", 1 * m),
            MultiFile::new("%s-r.h5", 2 * m),
            MultiFile::new("%s-g.h5", 3 * m),
            MultiFile::new("%s-l.h5", 4 * m),
            MultiFile::new("%s-o.h5", 5 * m),
        ];
        Self { files, layout: MultiLayout::default(), relax: false }
    }
}

impl MultiDriver {
    pub(crate) fn validate(&self) -> Result<()> {
        let n = self.files.len();
        if self.files.is_empty() || n > 6 {
            fail!("invalid number of multi files: {} (expected 1-6)", n);
        }
        let mut used = iter::repeat(false).take(n).collect::<Vec<_>>();
        for i in 0..6 {
            let j = *self.layout.get(i) as usize;
            if j >= n {
                fail!("invalid multi layout index: {} (expected 0-{})", j, n - 1);
            }
            used[j] = true;
        }
        if !used.iter().all(|x| *x) {
            fail!("invalid multi layout: some files are unused");
        }
        Ok(())
    }
}

/// Split file driver properties.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SplitDriver {
    /// Metadata filename extension.
    pub meta_ext: String,
    /// Raw data filename extension.
    pub raw_ext: String,
}

impl Default for SplitDriver {
    fn default() -> Self {
        Self { meta_ext: ".meta".into(), raw_ext: ".raw".into() }
    }
}

impl SplitDriver {
    pub(crate) fn from_multi(drv: &MultiDriver) -> Option<Self> {
        let mut layout = MultiLayout {
            mem_super: 0,
            mem_btree: 0,
            mem_draw: 1,
            mem_gheap: 0,
            mem_lheap: 0,
            mem_object: 0,
        };
        if cfg!(feature = "1.8.10") {
            layout.mem_gheap = 1; // was changed in 1.8.10
        }
        let is_split = drv.relax
            && drv.layout == layout
            && drv.files.len() == 2
            && drv.files[0].addr == 0
            && drv.files[1].addr == u64::max_value() / 2
            && drv.files[0].name.starts_with("%s")
            && drv.files[1].name.starts_with("%s");
        if is_split {
            Some(Self {
                meta_ext: drv.files[0].name[2..].into(),
                raw_ext: drv.files[1].name[2..].into(),
            })
        } else {
            None
        }
    }
}

#[cfg(feature = "mpio")]
mod mpio {
    use std::mem;

    use mpi_sys::{MPI_Comm, MPI_Info};

    use super::{c_int, Result};

    /// MPI-I/O file driver properties.
    #[derive(Debug)]
    pub struct MpioDriver {
        /// MPI-2 communicator.
        pub comm: MPI_Comm,
        /// MPI-2 info object.
        pub info: MPI_Info,
    }

    macro_rules! mpi_exec {
        ($func:ident, $($arg:tt)*) => (
            if unsafe { mpi_sys::$func($($arg)*) } != mpi_sys::MPI_SUCCESS as _ {
                fail!("{} failed", stringify!($func));
            }
        );
    }

    impl MpioDriver {
        pub(crate) fn try_new(comm: MPI_Comm, info: Option<MPI_Info>) -> Result<Self> {
            let mut comm_dup = unsafe { mem::zeroed() };
            let mut info_dup = unsafe { mem::zeroed() };
            mpi_exec!(MPI_Comm_dup, comm, &mut comm_dup);
            if let Some(info) = info {
                mpi_exec!(MPI_Info_dup, info, &mut info_dup);
            } else {
                mpi_exec!(MPI_Info_create, &mut info_dup);
            }
            Ok(Self { comm: comm_dup, info: info_dup })
        }
    }

    impl Clone for MpioDriver {
        fn clone(&self) -> Self {
            unsafe {
                let mut comm_dup = mem::zeroed();
                mpi_sys::MPI_Comm_dup(self.comm, &mut comm_dup);
                let mut info_dup = mem::zeroed();
                mpi_sys::MPI_Info_dup(self.info, &mut info_dup);
                Self { comm: comm_dup, info: info_dup }
            }
        }
    }

    impl Drop for MpioDriver {
        fn drop(&mut self) {
            let mut finalized: c_int = 1;
            unsafe {
                let code = mpi_sys::MPI_Finalized(&mut finalized);
                if code == mpi_sys::MPI_SUCCESS as _ && finalized == 0 {
                    mpi_sys::MPI_Info_free(&mut self.info);
                    mpi_sys::MPI_Comm_free(&mut self.comm);
                }
            }
        }
    }
}

#[cfg(feature = "mpio")]
pub use self::mpio::*;

/// Direct I/O driver properties.
#[cfg(feature = "have-direct")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DirectDriver {
    /// Required memory alignment boundary.
    pub alignment: usize,
    /// File system block size.
    pub block_size: usize,
    /// Size in bytes of the copy buffer.
    pub cbuf_size: usize,
}

#[cfg(feature = "have-direct")]
impl Default for DirectDriver {
    fn default() -> Self {
        Self { alignment: 4096, block_size: 4096, cbuf_size: 16 * 1024 * 1024 }
    }
}

/// A low-level file driver configuration.
#[derive(Clone, Debug)]
pub enum FileDriver {
    /// Uses POSIX filesystem functions to perform unbuffered access to a single file.
    Sec2,
    /// Uses functions from the standard C `stdio.h` to perform buffered access to a single file.
    Stdio,
    /// SEC2 with logging capabilities.
    Log,
    /// Keeps file contents in memory until the file is closed, enabling faster access.
    Core(CoreDriver),
    /// Partitions file address space into pieces and sends them to separate storage files.
    Family(FamilyDriver),
    /// Allows data to be stored in multiple files according to the type of data.
    Multi(MultiDriver),
    /// Special case of the Multi driver that stores metadata and raw data in separate files.
    Split(SplitDriver),
    /// Uses the MPI standard for communication and I/O.
    #[cfg(feature = "mpio")]
    Mpio(MpioDriver),
    /// SEC2 except data is accessed synchronously without being cached by the system.
    #[cfg(feature = "have-direct")]
    Direct(DirectDriver),
}

/// Options for what to do when trying to close a file while there are open objects inside it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileCloseDegree {
    /// Let the driver choose the behavior.
    ///
    /// All drivers set this to `Weak`, except for MPI-I/O, which sets it to `Semi`.
    Default,
    /// Terminate file identifier access, but delay closing until all objects are closed.
    Weak,
    /// Return an error if the file has open objects.
    Semi,
    /// Close all open objects, then close the file.
    Strong,
}

impl Default for FileCloseDegree {
    fn default() -> Self {
        Self::Default
    }
}

impl From<H5F_close_degree_t> for FileCloseDegree {
    fn from(cd: H5F_close_degree_t) -> Self {
        match cd {
            H5F_close_degree_t::H5F_CLOSE_WEAK => Self::Weak,
            H5F_close_degree_t::H5F_CLOSE_SEMI => Self::Semi,
            H5F_close_degree_t::H5F_CLOSE_STRONG => Self::Strong,
            H5F_close_degree_t::H5F_CLOSE_DEFAULT => Self::Default,
        }
    }
}

impl From<FileCloseDegree> for H5F_close_degree_t {
    fn from(v: FileCloseDegree) -> Self {
        match v {
            FileCloseDegree::Weak => Self::H5F_CLOSE_WEAK,
            FileCloseDegree::Semi => Self::H5F_CLOSE_SEMI,
            FileCloseDegree::Strong => Self::H5F_CLOSE_STRONG,
            FileCloseDegree::Default => Self::H5F_CLOSE_DEFAULT,
        }
    }
}

/// File alignment properties.
///
/// Any file object with size of at least `threshold` bytes will be aligned on an address that is
/// a multiple of `alignment`. Addresses are relative to the end of the user block.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Alignment {
    /// The byte size threshold.
    pub threshold: u64,
    /// The alignment value.
    pub alignment: u64,
}

impl Default for Alignment {
    /// Returns the default threshold and alignment of 1 (i.e. no alignment).
    fn default() -> Self {
        Self { threshold: 1, alignment: 1 }
    }
}

/// Raw data chunk cache parameters.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChunkCache {
    /// The number of objects in the cache.
    pub nslots: usize,
    /// The total size of the cache in bytes.
    pub nbytes: usize,
    /// The chunk preemption policy.
    pub w0: f64,
}

impl Default for ChunkCache {
    fn default() -> Self {
        Self { nslots: 521, nbytes: 1024 * 1024, w0: 0.75 }
    }
}

impl Eq for ChunkCache {}

/// Page buffer size properties.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PageBufferSize {
    /// Maximum size, in bytes, of the page buffer.
    pub buf_size: usize,
    /// Minimum metadata percentage to keep in the buffer before allowing pages with metadata to be
    /// evicted.
    pub min_meta_perc: u32,
    /// Minimum raw data percentage to keep in the buffer before allowing pages with raw data to be
    /// evicted.
    pub min_raw_perc: u32,
}

/// Automatic cache size increase mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CacheIncreaseMode {
    /// Automatic increase is disabled.
    Off,
    /// Automatic increase uses the hit rate threshold algorithm.
    Threshold,
}

impl From<H5C_cache_incr_mode> for CacheIncreaseMode {
    fn from(mode: H5C_cache_incr_mode) -> Self {
        match mode {
            H5C_cache_incr_mode::H5C_incr__threshold => Self::Threshold,
            H5C_cache_incr_mode::H5C_incr__off => Self::Off,
        }
    }
}

impl From<CacheIncreaseMode> for H5C_cache_incr_mode {
    fn from(v: CacheIncreaseMode) -> Self {
        match v {
            CacheIncreaseMode::Threshold => Self::H5C_incr__threshold,
            CacheIncreaseMode::Off => Self::H5C_incr__off,
        }
    }
}

/// Flash cache size increase mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlashIncreaseMode {
    /// Flash cache size increase is disabled.
    Off,
    /// Flash cache size increase uses the add space algorithm.
    AddSpace,
}

impl From<H5C_cache_flash_incr_mode> for FlashIncreaseMode {
    fn from(mode: H5C_cache_flash_incr_mode) -> Self {
        match mode {
            H5C_cache_flash_incr_mode::H5C_flash_incr__add_space => Self::AddSpace,
            H5C_cache_flash_incr_mode::H5C_flash_incr__off => Self::Off,
        }
    }
}

impl From<FlashIncreaseMode> for H5C_cache_flash_incr_mode {
    fn from(v: FlashIncreaseMode) -> Self {
        match v {
            FlashIncreaseMode::AddSpace => Self::H5C_flash_incr__add_space,
            FlashIncreaseMode::Off => Self::H5C_flash_incr__off,
        }
    }
}

/// Automatic cache size decrease mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CacheDecreaseMode {
    /// Automatic decrease is disabled.
    Off,
    /// Automatic decrease uses the hit rate threshold algorithm.
    Threshold,
    /// Automatic decrease uses the ageout algorithm.
    AgeOut,
    /// Automatic decrease uses the ageout with hit rate threshold algorithm.
    AgeOutWithThreshold,
}

impl From<H5C_cache_decr_mode> for CacheDecreaseMode {
    fn from(mode: H5C_cache_decr_mode) -> Self {
        match mode {
            H5C_cache_decr_mode::H5C_decr__threshold => Self::Threshold,
            H5C_cache_decr_mode::H5C_decr__age_out => Self::AgeOut,
            H5C_cache_decr_mode::H5C_decr__age_out_with_threshold => Self::AgeOutWithThreshold,
            H5C_cache_decr_mode::H5C_decr__off => Self::Off,
        }
    }
}

impl From<CacheDecreaseMode> for H5C_cache_decr_mode {
    fn from(v: CacheDecreaseMode) -> Self {
        match v {
            CacheDecreaseMode::Threshold => Self::H5C_decr__threshold,
            CacheDecreaseMode::AgeOut => Self::H5C_decr__age_out,
            CacheDecreaseMode::AgeOutWithThreshold => Self::H5C_decr__age_out_with_threshold,
            CacheDecreaseMode::Off => Self::H5C_decr__off,
        }
    }
}

/// A strategy for writing metadata to disk.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MetadataWriteStrategy {
    /// Only process zero is allowed to write dirty metadata to disk.
    ProcessZeroOnly,
    /// Process zero decides what entries to flush, but the flushes are distributed across
    /// processes.
    Distributed,
}

impl Default for MetadataWriteStrategy {
    fn default() -> Self {
        Self::Distributed
    }
}

impl From<c_int> for MetadataWriteStrategy {
    fn from(strategy: c_int) -> Self {
        match strategy {
            H5AC_METADATA_WRITE_STRATEGY__DISTRIBUTED => Self::Distributed,
            _ => Self::ProcessZeroOnly,
        }
    }
}

impl From<MetadataWriteStrategy> for c_int {
    fn from(v: MetadataWriteStrategy) -> Self {
        match v {
            MetadataWriteStrategy::Distributed => H5AC_METADATA_WRITE_STRATEGY__DISTRIBUTED,
            MetadataWriteStrategy::ProcessZeroOnly => H5AC_METADATA_WRITE_STRATEGY__PROCESS_0_ONLY,
        }
    }
}

/// Metadata cache configuration.
#[derive(Clone, Debug, PartialEq)]
pub struct MetadataCacheConfig {
    /// Whether the adaptive cache resize report function is enabled.
    pub rpt_fcn_enabled: bool,
    /// Whether `trace_file_name` should be used to open a trace file for the cache.
    pub open_trace_file: bool,
    /// Whether the current trace file (if any) should be closed.
    pub close_trace_file: bool,
    /// Full path of the trace file to be opened if `open_trace_file` is `true`.
    pub trace_file_name: String,
    /// Whether evictions from the metadata cache are enabled.
    pub evictions_enabled: bool,
    /// Whether the cache should be created with a user-specified initial size.
    pub set_initial_size: bool,
    /// Initial cache size in bytes if `set_initial_size` is `true`.
    pub initial_size: usize,
    /// Minimum fraction of the cache that must be kept clean or empty.
    pub min_clean_fraction: f64,
    /// Maximum number of bytes that adaptive cache resizing can select as the maximum cache size.
    pub max_size: usize,
    /// Minimum number of bytes that adaptive cache resizing can select as the minimum cache size.
    pub min_size: usize,
    /// Number of cache accesses between runs of the adaptive cache resize code.
    pub epoch_length: i64,
    /// Automatic cache size increase mode.
    pub incr_mode: CacheIncreaseMode,
    /// Hit rate threshold for the hit rate threshold cache size increment algorithm.
    pub lower_hr_threshold: f64,
    /// Factor by which the hit rate threshold cache size increment algorithm multiplies the current
    /// cache max size to obtain a tentative new size.
    pub increment: f64,
    /// Whether to apply an upper limit to the size of cache size increases.
    pub apply_max_increment: bool,
    /// Maximum number of bytes by which cache size can be increased in a single step,
    /// if applicable.
    pub max_increment: usize,
    /// Flash cache size increase mode.
    pub flash_incr_mode: FlashIncreaseMode,
    /// Factor by which the size of the triggering entry / entry size increase is multiplied to
    /// obtain the initial cache size increment.
    pub flash_multiple: f64,
    /// Factor by which the current maximum cache size is multiplied to obtain the minimum size
    /// entry / entry size increase which may trigger a flash cache size increase.
    pub flash_threshold: f64,
    /// Automatic cache size decrease mode.
    pub decr_mode: CacheDecreaseMode,
    /// Hit rate threshold for hit-rate-based cache size decrease algorithms.
    pub upper_hr_threshold: f64,
    /// Factor by which the hit rate threshold cache size decrease algorithm multiplies the current
    /// cache max size to obtain a tentative new size.
    pub decrement: f64,
    /// Whether an upper limit should be applied to the size of cache size decreases.
    pub apply_max_decrement: bool,
    /// Maximum number of bytes by which cache size can be decreased in a single step,
    /// if applicable.
    pub max_decrement: usize,
    /// Minimum number of epochs that an entry must remain unaccessed in cache before ageout-based
    /// reduction algorithms try to evict it.
    pub epochs_before_eviction: i32,
    /// Whether ageout-based decrement algorithms will maintain an empty reserve.
    pub apply_empty_reserve: bool,
    /// Empty reserve as a fraction of maximum cache size.
    /// Ageout-based algorithms will not decrease the maximum size unless the empty reserve can be
    /// met.
    pub empty_reserve: f64,
    /// Threshold number of bytes of dirty metadata that will trigger synchronization of
    /// parallel metadata caches.
    pub dirty_bytes_threshold: usize,
    /// Strategy for writing metadata to disk.
    pub metadata_write_strategy: MetadataWriteStrategy,
}

impl Eq for MetadataCacheConfig {}

impl Default for MetadataCacheConfig {
    fn default() -> Self {
        let min_clean_fraction = if cfg!(feature = "have-parallel") { 0.3_f32 } else { 0.01_f32 };
        let flash_multiple = if cfg!(feature = "have-parallel") { 1.0_f32 } else { 1.4_f32 };
        Self {
            rpt_fcn_enabled: false,
            open_trace_file: false,
            close_trace_file: false,
            trace_file_name: String::new(),
            evictions_enabled: true,
            set_initial_size: true,
            initial_size: 1 << 21,
            min_clean_fraction: f64::from(min_clean_fraction),
            max_size: 1 << 25,
            min_size: 1 << 20,
            epoch_length: 50_000,
            incr_mode: CacheIncreaseMode::Threshold,
            lower_hr_threshold: f64::from(0.9_f32),
            increment: 2.0,
            apply_max_increment: true,
            max_increment: 1 << 22,
            flash_incr_mode: FlashIncreaseMode::AddSpace,
            flash_multiple: f64::from(flash_multiple),
            flash_threshold: 0.25,
            decr_mode: CacheDecreaseMode::AgeOutWithThreshold,
            upper_hr_threshold: f64::from(0.999_f32),
            decrement: f64::from(0.9_f32),
            apply_max_decrement: true,
            max_decrement: 1 << 20,
            epochs_before_eviction: 3,
            apply_empty_reserve: true,
            empty_reserve: f64::from(0.1_f32),
            dirty_bytes_threshold: 1 << 18,
            metadata_write_strategy: MetadataWriteStrategy::default(),
        }
    }
}

impl From<MetadataCacheConfig> for H5AC_cache_config_t {
    fn from(v: MetadataCacheConfig) -> Self {
        const N: usize = H5AC__MAX_TRACE_FILE_NAME_LEN;
        let mut trace_file_name: [c_char; N + 1] = unsafe { mem::zeroed() };
        string_to_fixed_bytes(&v.trace_file_name, &mut trace_file_name[..N]);
        Self {
            version: H5AC__CURR_CACHE_CONFIG_VERSION,
            rpt_fcn_enabled: hbool_t::from(v.rpt_fcn_enabled),
            open_trace_file: hbool_t::from(v.open_trace_file),
            close_trace_file: hbool_t::from(v.close_trace_file),
            trace_file_name,
            evictions_enabled: hbool_t::from(v.evictions_enabled),
            set_initial_size: hbool_t::from(v.set_initial_size),
            initial_size: v.initial_size as _,
            min_clean_fraction: v.min_clean_fraction as _,
            max_size: v.max_size as _,
            min_size: v.min_size as _,
            epoch_length: v.epoch_length as _,
            incr_mode: v.incr_mode.into(),
            lower_hr_threshold: v.lower_hr_threshold as _,
            increment: v.increment as _,
            apply_max_increment: hbool_t::from(v.apply_max_increment),
            max_increment: v.max_increment as _,
            flash_incr_mode: v.flash_incr_mode.into(),
            flash_multiple: v.flash_multiple as _,
            flash_threshold: v.flash_threshold as _,
            decr_mode: v.decr_mode.into(),
            upper_hr_threshold: v.upper_hr_threshold as _,
            decrement: v.decrement as _,
            apply_max_decrement: hbool_t::from(v.apply_max_decrement),
            max_decrement: v.max_decrement as _,
            epochs_before_eviction: v.epochs_before_eviction as _,
            apply_empty_reserve: hbool_t::from(v.apply_empty_reserve),
            empty_reserve: v.empty_reserve as _,
            dirty_bytes_threshold: v.dirty_bytes_threshold as _,
            metadata_write_strategy: v.metadata_write_strategy.into(),
        }
    }
}

impl From<H5AC_cache_config_t> for MetadataCacheConfig {
    fn from(mdc: H5AC_cache_config_t) -> Self {
        const N: usize = H5AC__MAX_TRACE_FILE_NAME_LEN;
        let trace_file_name = string_from_fixed_bytes(&mdc.trace_file_name, N);
        Self {
            rpt_fcn_enabled: mdc.rpt_fcn_enabled > 0,
            open_trace_file: mdc.open_trace_file > 0,
            close_trace_file: mdc.close_trace_file > 0,
            trace_file_name,
            evictions_enabled: mdc.evictions_enabled > 0,
            set_initial_size: mdc.set_initial_size > 0,
            initial_size: mdc.initial_size as _,
            min_clean_fraction: mdc.min_clean_fraction as _,
            max_size: mdc.max_size as _,
            min_size: mdc.min_size as _,
            epoch_length: mdc.epoch_length as _,
            incr_mode: mdc.incr_mode.into(),
            lower_hr_threshold: mdc.lower_hr_threshold as _,
            increment: mdc.increment as _,
            apply_max_increment: mdc.apply_max_increment > 0,
            max_increment: mdc.max_increment as _,
            flash_incr_mode: mdc.flash_incr_mode.into(),
            flash_multiple: mdc.flash_multiple as _,
            flash_threshold: mdc.flash_threshold as _,
            decr_mode: mdc.decr_mode.into(),
            upper_hr_threshold: mdc.upper_hr_threshold as _,
            decrement: mdc.decrement as _,
            apply_max_decrement: mdc.apply_max_decrement > 0,
            max_decrement: mdc.max_decrement as _,
            epochs_before_eviction: mdc.epochs_before_eviction as _,
            apply_empty_reserve: mdc.apply_empty_reserve > 0,
            empty_reserve: mdc.empty_reserve as _,
            dirty_bytes_threshold: mdc.dirty_bytes_threshold as _,
            metadata_write_strategy: mdc.metadata_write_strategy.into(),
        }
    }
}

#[cfg(feature = "1.10.1")]
mod cache_image_config {
    use super::*;

    /// Metadata cache image configuration.
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    pub struct CacheImageConfig {
        /// Whether a cache image should be created on file close.
        pub generate_image: bool,
        /// Whether the cache image should include the adaptive cache resize configuration and
        /// status.
        pub save_resize_status: bool,
        /// Maximum number of times a prefetched entry can appear in subsequent cache images.
        pub entry_ageout: i32,
    }

    impl Default for CacheImageConfig {
        fn default() -> Self {
            Self {
                generate_image: false,
                save_resize_status: false,
                entry_ageout: H5AC__CACHE_IMAGE__ENTRY_AGEOUT__NONE,
            }
        }
    }

    impl From<CacheImageConfig> for H5AC_cache_image_config_t {
        fn from(v: CacheImageConfig) -> Self {
            Self {
                version: H5AC__CURR_CACHE_CONFIG_VERSION,
                generate_image: hbool_t::from(v.generate_image),
                save_resize_status: hbool_t::from(v.save_resize_status),
                entry_ageout: v.entry_ageout as _,
            }
        }
    }

    impl From<H5AC_cache_image_config_t> for CacheImageConfig {
        fn from(config: H5AC_cache_image_config_t) -> Self {
            Self {
                generate_image: config.generate_image > 0,
                save_resize_status: config.save_resize_status > 0,
                entry_ageout: config.entry_ageout as _,
            }
        }
    }
}

#[cfg(feature = "1.10.1")]
pub use self::cache_image_config::*;

/// Metadata cache logging options.
#[cfg(feature = "1.10.0")]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CacheLogOptions {
    /// Whether logging is enabled.
    pub is_enabled: bool,
    /// File path of the log. (Must be ASCII on Windows)
    pub location: String,
    /// Whether to begin logging as soon as the file is opened
    pub start_on_access: bool,
}

#[cfg(feature = "1.10.2")]
mod libver {
    use super::*;

    /// Options for which library format version to use when storing objects.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub enum LibraryVersion {
        /// Use the earliest possible format.
        Earliest = 0,
        /// Use the latest v18 format.
        V18 = 1,
        /// Use the latest v110 format.
        V110 = 2,
    }

    impl LibraryVersion {
        /// Returns `true` if the version is set to `Earliest`.
        pub fn is_earliest(self) -> bool {
            self == Self::Earliest
        }

        /// Returns the latest library version.
        pub const fn latest() -> Self {
            Self::V110
        }
    }

    impl From<LibraryVersion> for H5F_libver_t {
        fn from(v: LibraryVersion) -> Self {
            match v {
                LibraryVersion::V18 => Self::H5F_LIBVER_V18,
                LibraryVersion::V110 => Self::H5F_LIBVER_V110,
                LibraryVersion::Earliest => Self::H5F_LIBVER_EARLIEST,
            }
        }
    }

    impl From<H5F_libver_t> for LibraryVersion {
        fn from(libver: H5F_libver_t) -> Self {
            match libver {
                H5F_libver_t::H5F_LIBVER_V18 => Self::V18,
                H5F_libver_t::H5F_LIBVER_V110 => Self::V110,
                _ => Self::Earliest,
            }
        }
    }

    /// Library format version bounds for writing objects to a file.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct LibVerBounds {
        /// The earliest version to use for writing objects.
        pub low: LibraryVersion,
        /// The latest version to use for writing objects.
        pub high: LibraryVersion,
    }

    impl LibVerBounds {
        pub const fn new(low: LibraryVersion, high: LibraryVersion) -> Self {
            Self { low, high }
        }
    }

    impl Default for LibVerBounds {
        fn default() -> Self {
            Self { low: LibraryVersion::Earliest, high: LibraryVersion::latest() }
        }
    }

    impl From<LibraryVersion> for LibVerBounds {
        fn from(version: LibraryVersion) -> Self {
            Self { low: version, high: LibraryVersion::latest() }
        }
    }
}

#[cfg(feature = "1.10.2")]
pub use self::libver::*;

/// Builder used to create file access property list.
#[derive(Clone, Debug, Default)]
pub struct FileAccessBuilder {
    file_driver: Option<FileDriver>,
    log_options: LogOptions,
    #[cfg(feature = "1.8.13")]
    write_tracking: Option<usize>,
    fclose_degree: Option<FileCloseDegree>,
    alignment: Option<Alignment>,
    chunk_cache: Option<ChunkCache>,
    #[cfg(feature = "1.8.7")]
    elink_file_cache_size: Option<u32>,
    meta_block_size: Option<u64>,
    #[cfg(feature = "1.10.1")]
    page_buffer_size: Option<PageBufferSize>,
    sieve_buf_size: Option<usize>,
    #[cfg(feature = "1.10.1")]
    evict_on_close: Option<bool>,
    #[cfg(feature = "1.10.0")]
    metadata_read_attempts: Option<u32>,
    mdc_config: Option<MetadataCacheConfig>,
    #[cfg(feature = "1.10.1")]
    mdc_image_config: Option<CacheImageConfig>,
    #[cfg(feature = "1.10.0")]
    mdc_log_options: Option<CacheLogOptions>,
    #[cfg(all(feature = "1.10.0", feature = "have-parallel"))]
    all_coll_metadata_ops: Option<bool>,
    #[cfg(all(feature = "1.10.0", feature = "have-parallel"))]
    coll_metadata_write: Option<bool>,
    gc_references: Option<bool>,
    small_data_block_size: Option<u64>,
    #[cfg(feature = "1.10.2")]
    libver_bounds: Option<LibVerBounds>,
}

impl FileAccessBuilder {
    /// Creates a new file access property list builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new builder from an existing property list.
    pub fn from_plist(plist: &FileAccess) -> Result<Self> {
        let mut builder = Self::default();
        builder.fclose_degree(plist.get_fclose_degree()?);
        let v = plist.get_alignment()?;
        builder.alignment(v.threshold, v.alignment);
        let v = plist.get_chunk_cache()?;
        builder.chunk_cache(v.nslots, v.nbytes, v.w0);
        let drv = plist.get_driver()?;
        builder.driver(&drv);
        builder.gc_references(plist.get_gc_references()?);
        builder.small_data_block_size(plist.get_small_data_block_size()?);
        #[cfg(feature = "1.10.2")]
        {
            let v = plist.get_libver_bounds()?;
            builder.libver_bounds(v.low, v.high);
        }
        #[cfg(feature = "1.8.7")]
        {
            builder.elink_file_cache_size(plist.get_elink_file_cache_size()?);
        }
        builder.meta_block_size(plist.get_meta_block_size()?);
        #[cfg(feature = "1.10.1")]
        {
            let v = plist.get_page_buffer_size()?;
            builder.page_buffer_size(v.buf_size, v.min_meta_perc, v.min_raw_perc);
            builder.evict_on_close(plist.get_evict_on_close()?);
            builder.mdc_image_config(plist.get_mdc_image_config()?.generate_image);
        }
        builder.sieve_buf_size(plist.get_sieve_buf_size()?);
        #[cfg(feature = "1.10.0")]
        {
            builder.metadata_read_attempts(plist.get_metadata_read_attempts()?);
            let v = plist.get_mdc_log_options()?;
            builder.mdc_log_options(v.is_enabled, &v.location, v.start_on_access);
        }
        #[cfg(all(feature = "1.10.0", feature = "have-parallel"))]
        {
            builder.all_coll_metadata_ops(plist.get_all_coll_metadata_ops()?);
            builder.coll_metadata_write(plist.get_coll_metadata_write()?);
        }
        builder.mdc_config(&plist.get_mdc_config()?);
        #[cfg(feature = "1.8.13")]
        {
            if let FileDriver::Core(ref drv) = drv {
                builder.write_tracking(drv.write_tracking);
            }
        }
        Ok(builder)
    }

    /// Sets the file close degree
    ///
    /// If called with `FileCloseDegree::Strong`, the programmer is responsible
    /// for closing all items before closing the file. Failure to do so might
    /// invalidate newly created objects.
    pub fn fclose_degree(&mut self, fc_degree: FileCloseDegree) -> &mut Self {
        self.fclose_degree = Some(fc_degree);
        self
    }

    /// Sets the file alignment parameters.
    pub fn alignment(&mut self, threshold: u64, alignment: u64) -> &mut Self {
        self.alignment = Some(Alignment { threshold, alignment });
        self
    }

    /// Sets the raw data chunk cache parameters.
    pub fn chunk_cache(&mut self, nslots: usize, nbytes: usize, w0: f64) -> &mut Self {
        self.chunk_cache = Some(ChunkCache { nslots, nbytes, w0 });
        self
    }

    /// Sets the number of files that can be held open in an external link open file cache.
    #[cfg(feature = "1.8.7")]
    pub fn elink_file_cache_size(&mut self, efc_size: u32) -> &mut Self {
        self.elink_file_cache_size = Some(efc_size);
        self
    }

    /// Sets the minimum metadata block size in bytes.
    pub fn meta_block_size(&mut self, size: u64) -> &mut Self {
        self.meta_block_size = Some(size);
        self
    }

    /// Sets the page buffer size properties.
    #[cfg(feature = "1.10.1")]
    pub fn page_buffer_size(
        &mut self, buf_size: usize, min_meta_perc: u32, min_raw_perc: u32,
    ) -> &mut Self {
        self.page_buffer_size = Some(PageBufferSize { buf_size, min_meta_perc, min_raw_perc });
        self
    }

    /// Sets the maximum size of the data sieve buffer.
    pub fn sieve_buf_size(&mut self, size: usize) -> &mut Self {
        self.sieve_buf_size = Some(size);
        self
    }

    /// Sets whether object metadata should be evicted from cache when an object is closed.
    #[cfg(feature = "1.10.1")]
    pub fn evict_on_close(&mut self, evict_on_close: bool) -> &mut Self {
        self.evict_on_close = Some(evict_on_close);
        self
    }

    /// Sets the number of reads that the library will try when reading checksummed metadata in a
    /// file opened with SWMR access.
    #[cfg(feature = "1.10.0")]
    pub fn metadata_read_attempts(&mut self, attempts: u32) -> &mut Self {
        self.metadata_read_attempts = Some(attempts);
        self
    }

    /// Sets the metadata cache configuration.
    pub fn mdc_config(&mut self, config: &MetadataCacheConfig) -> &mut Self {
        self.mdc_config = Some(config.clone());
        self
    }

    /// Sets whether a cache image should be created on file close.
    #[cfg(feature = "1.10.1")]
    pub fn mdc_image_config(&mut self, generate_image: bool) -> &mut Self {
        self.mdc_image_config = Some(CacheImageConfig {
            generate_image,
            save_resize_status: false,
            entry_ageout: H5AC__CACHE_IMAGE__ENTRY_AGEOUT__NONE,
        });
        self
    }

    /// Sets metadata cache logging options.
    #[cfg(feature = "1.10.0")]
    pub fn mdc_log_options(
        &mut self, is_enabled: bool, location: &str, start_on_access: bool,
    ) -> &mut Self {
        self.mdc_log_options =
            Some(CacheLogOptions { is_enabled, location: location.into(), start_on_access });
        self
    }

    /// Sets whether metadata reads are collective.
    #[cfg(all(feature = "1.10.0", feature = "have-parallel"))]
    pub fn all_coll_metadata_ops(&mut self, is_collective: bool) -> &mut Self {
        self.all_coll_metadata_ops = Some(is_collective);
        self
    }

    /// Sets whether metadata writes are collective.
    #[cfg(all(feature = "1.10.0", feature = "have-parallel"))]
    pub fn coll_metadata_write(&mut self, is_collective: bool) -> &mut Self {
        self.coll_metadata_write = Some(is_collective);
        self
    }

    /// Sets whether reference garbage collection is enabled.
    pub fn gc_references(&mut self, gc_ref: bool) -> &mut Self {
        self.gc_references = Some(gc_ref);
        self
    }

    /// Sets the maximum size in bytes of a contiguous block reserved for small data.
    pub fn small_data_block_size(&mut self, size: u64) -> &mut Self {
        self.small_data_block_size = Some(size);
        self
    }

    /// Sets the range of library versions to use when writing objects.
    #[cfg(feature = "1.10.2")]
    pub fn libver_bounds(&mut self, low: LibraryVersion, high: LibraryVersion) -> &mut Self {
        self.libver_bounds = Some(LibVerBounds { low, high });
        self
    }

    /// Allows use of the earliest library version when writing objects.
    #[cfg(feature = "1.10.2")]
    pub fn libver_earliest(&mut self) -> &mut Self {
        self.libver_bounds(LibraryVersion::Earliest, LibraryVersion::latest())
    }

    /// Sets the earliest library version for writing objects to v18.
    #[cfg(feature = "1.10.2")]
    pub fn libver_v18(&mut self) -> &mut Self {
        self.libver_bounds(LibraryVersion::V18, LibraryVersion::latest())
    }

    /// Sets the earliest library version for writing objects to v110.
    #[cfg(feature = "1.10.2")]
    pub fn libver_v110(&mut self) -> &mut Self {
        self.libver_bounds(LibraryVersion::V110, LibraryVersion::latest())
    }

    /// Allows only the latest library version when writing objects.
    #[cfg(feature = "1.10.2")]
    pub fn libver_latest(&mut self) -> &mut Self {
        self.libver_bounds(LibraryVersion::latest(), LibraryVersion::latest())
    }

    /// Sets which file driver to use.
    pub fn driver(&mut self, file_driver: &FileDriver) -> &mut Self {
        self.file_driver = Some(file_driver.clone());
        self
    }

    /// Sets the file driver to SEC2 (POSIX).
    pub fn sec2(&mut self) -> &mut Self {
        self.driver(&FileDriver::Sec2)
    }

    /// Sets the file driver to STDIO.
    pub fn stdio(&mut self) -> &mut Self {
        self.driver(&FileDriver::Stdio)
    }

    /// Sets the file driver to SEC2 with logging and configures it.
    pub fn log_options(
        &mut self, logfile: Option<&str>, flags: LogFlags, buf_size: usize,
    ) -> &mut Self {
        self.log_options.logfile = logfile.map(Into::into);
        self.log_options.flags = flags;
        self.log_options.buf_size = buf_size;
        self.driver(&FileDriver::Log)
    }

    /// Sets the file driver to SEC2 with logging.
    pub fn log(&mut self) -> &mut Self {
        self.log_options(None, LogFlags::LOC_IO, 0)
    }

    /// Sets the file driver to Core and configures it.
    pub fn core_options(&mut self, increment: usize, filebacked: bool) -> &mut Self {
        let drv = CoreDriver { increment, filebacked, ..CoreDriver::default() };
        self.driver(&FileDriver::Core(drv))
    }

    /// Sets the file driver to Core and sets whether to write file contents to disk upon closing.
    pub fn core_filebacked(&mut self, filebacked: bool) -> &mut Self {
        let drv = CoreDriver { filebacked, ..CoreDriver::default() };
        self.driver(&FileDriver::Core(drv))
    }

    /// Sets the file driver to Core.
    pub fn core(&mut self) -> &mut Self {
        self.driver(&FileDriver::Core(CoreDriver::default()))
    }

    /// Sets the write tracking page size for the Core file driver.
    #[cfg(feature = "1.8.13")]
    pub fn write_tracking(&mut self, page_size: usize) -> &mut Self {
        self.write_tracking = Some(page_size);
        self
    }

    /// Sets the file driver to Family.
    pub fn family(&mut self) -> &mut Self {
        self.driver(&FileDriver::Family(FamilyDriver::default()))
    }

    /// Sets the file driver to Family and configures the file member size.
    pub fn family_options(&mut self, member_size: usize) -> &mut Self {
        self.driver(&FileDriver::Family(FamilyDriver { member_size }))
    }

    /// Sets the file driver to Multi and configures it.
    pub fn multi_options(
        &mut self, files: &[MultiFile], layout: &MultiLayout, relax: bool,
    ) -> &mut Self {
        self.driver(&FileDriver::Multi(MultiDriver {
            files: files.to_vec(),
            layout: layout.clone(),
            relax,
        }))
    }

    /// Sets the file driver to Multi.
    pub fn multi(&mut self) -> &mut Self {
        self.driver(&FileDriver::Multi(MultiDriver::default()))
    }

    /// Sets the file driver to Split and configures it.
    pub fn split_options(&mut self, meta_ext: &str, raw_ext: &str) -> &mut Self {
        self.driver(&FileDriver::Split(SplitDriver {
            meta_ext: meta_ext.into(),
            raw_ext: raw_ext.into(),
        }))
    }

    /// Sets the file driver to Split.
    pub fn split(&mut self) -> &mut Self {
        self.driver(&FileDriver::Split(SplitDriver::default()))
    }

    /// Sets the file driver to MPI-I/O and configures it.
    #[cfg(feature = "mpio")]
    pub fn mpio(&mut self, comm: mpi_sys::MPI_Comm, info: Option<mpi_sys::MPI_Info>) -> &mut Self {
        // We use .unwrap() here since MPI will almost surely terminate the process anyway.
        self.driver(&FileDriver::Mpio(MpioDriver::try_new(comm, info).unwrap()))
    }

    /// Sets the file driver to Direct and configures it.
    #[cfg(feature = "have-direct")]
    pub fn direct_options(
        &mut self, alignment: usize, block_size: usize, cbuf_size: usize,
    ) -> &mut Self {
        self.driver(&FileDriver::Direct(DirectDriver { alignment, block_size, cbuf_size }))
    }

    /// Sets the file driver to Direct.
    #[cfg(feature = "have-direct")]
    pub fn direct(&mut self) -> &mut Self {
        self.driver(&FileDriver::Direct(DirectDriver::default()))
    }

    fn set_log(&self, id: hid_t) -> Result<()> {
        let opt = &self.log_options;
        let flags = opt.flags.bits() as _;
        let buf_size = opt.buf_size as _;
        if let Some(ref logfile) = opt.logfile {
            let logfile = to_cstring(logfile.as_ref())?;
            h5try!(H5Pset_fapl_log(id, logfile.as_ptr(), flags, buf_size));
        } else {
            h5try!(H5Pset_fapl_log(id, ptr::null(), flags, buf_size));
        }
        Ok(())
    }

    fn set_core(&self, id: hid_t, drv: &CoreDriver) -> Result<()> {
        h5try!(H5Pset_fapl_core(id, drv.increment as _, hbool_t::from(drv.filebacked)));
        #[cfg(feature = "1.8.13")]
        {
            if let Some(page_size) = self.write_tracking {
                h5try!(H5Pset_core_write_tracking(
                    id,
                    hbool_t::from(page_size > 0),
                    page_size.max(1) as _
                ));
            }
        }
        Ok(())
    }

    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn set_family(id: hid_t, drv: &FamilyDriver) -> Result<()> {
        h5try!(H5Pset_fapl_family(id, drv.member_size as _, H5P_DEFAULT));
        Ok(())
    }

    fn set_multi(id: hid_t, drv: &MultiDriver) -> Result<()> {
        const N: usize = H5FD_MEM_NTYPES as _;
        debug_assert_eq!(FD_MEM_TYPES.len(), N as _);

        drv.validate()?;

        let mut memb_map: [H5F_mem_t; N] = unsafe { mem::zeroed() };
        let mut memb_fapl: [hid_t; N] = unsafe { mem::zeroed() };
        let mut memb_name: [*const c_char; N] = unsafe { mem::zeroed() };
        let mut memb_addr: [haddr_t; N] = unsafe { mem::zeroed() };

        let mut names = Vec::with_capacity(drv.files.len());
        for file in &drv.files {
            names.push(to_cstring(file.name.as_ref())?);
        }
        let default_name = to_cstring("%s-X.h5")?;

        for i in 0..N {
            memb_fapl[i] = H5P_DEFAULT;
            if i >= 1 {
                memb_map[i] = FD_MEM_TYPES[(1 + drv.layout.get(i - 1)) as usize];
            } else {
                memb_map[i] = H5F_mem_t::H5FD_MEM_DEFAULT;
            }
            if i == 0 {
                memb_name[i] = default_name.as_ptr();
                memb_addr[i] = 0;
            } else if i <= drv.files.len() {
                memb_name[i] = names[i - 1].as_ptr();
                memb_addr[i] = drv.files[i - 1].addr;
            } else {
                memb_name[i] = ptr::null();
                memb_addr[i] = 0;
            }
        }

        h5try!(H5Pset_fapl_multi(
            id,
            memb_map.as_ptr(),
            memb_fapl.as_ptr(),
            memb_name.as_ptr(),
            memb_addr.as_ptr(),
            hbool_t::from(drv.relax),
        ));

        Ok(())
    }

    fn set_split(id: hid_t, drv: &SplitDriver) -> Result<()> {
        let meta_ext = to_cstring(drv.meta_ext.as_ref())?;
        let raw_ext = to_cstring(drv.raw_ext.as_ref())?;
        h5try!(H5Pset_fapl_split(
            id,
            meta_ext.as_ptr(),
            H5P_DEFAULT,
            raw_ext.as_ptr(),
            H5P_DEFAULT
        ));
        Ok(())
    }

    #[cfg(feature = "mpio")]
    fn set_mpio(id: hid_t, drv: &MpioDriver) -> Result<()> {
        h5try!(H5Pset_fapl_mpio(id, drv.comm, drv.info));
        Ok(())
    }

    #[cfg(feature = "have-direct")]
    fn set_direct(id: hid_t, drv: &DirectDriver) -> Result<()> {
        h5try!(H5Pset_fapl_direct(id, drv.alignment as _, drv.block_size as _, drv.cbuf_size as _));
        Ok(())
    }

    fn set_driver(&self, id: hid_t, drv: &FileDriver) -> Result<()> {
        match drv {
            FileDriver::Sec2 => {
                h5try!(H5Pset_fapl_sec2(id));
            }
            FileDriver::Stdio => {
                h5try!(H5Pset_fapl_stdio(id));
            }
            FileDriver::Log => {
                self.set_log(id)?;
            }
            FileDriver::Core(drv) => {
                self.set_core(id, drv)?;
            }
            FileDriver::Family(drv) => {
                Self::set_family(id, drv)?;
            }
            FileDriver::Multi(drv) => {
                Self::set_multi(id, drv)?;
            }
            FileDriver::Split(drv) => {
                Self::set_split(id, drv)?;
            }
            #[cfg(feature = "mpio")]
            FileDriver::Mpio(drv) => {
                Self::set_mpio(id, drv)?;
            }
            #[cfg(feature = "have-direct")]
            FileDriver::Direct(drv) => {
                Self::set_direct(id, drv)?;
            }
        }
        Ok(())
    }

    fn populate_plist(&self, id: hid_t) -> Result<()> {
        if let Some(ref v) = self.file_driver {
            self.set_driver(id, v)?;
        }
        if let Some(v) = self.alignment {
            h5try!(H5Pset_alignment(id, v.threshold as _, v.alignment as _));
        }
        if let Some(v) = self.chunk_cache {
            h5try!(H5Pset_cache(id, 0, v.nslots as _, v.nbytes as _, v.w0 as _));
        }
        // The default is to use CLOSE_SEMI or CLOSE_WEAK, depending on VFL driver.
        // Both of these are unproblematic for our ownership
        if let Some(v) = self.fclose_degree {
            h5try!(H5Pset_fclose_degree(id, v.into()));
        }
        if let Some(v) = self.gc_references {
            h5try!(H5Pset_gc_references(id, c_uint::from(v)));
        }
        if let Some(v) = self.small_data_block_size {
            h5try!(H5Pset_small_data_block_size(id, v as _));
        }
        #[cfg(feature = "1.10.2")]
        {
            if let Some(v) = self.libver_bounds {
                h5try!(H5Pset_libver_bounds(id, v.low.into(), v.high.into()));
            }
        }
        #[cfg(feature = "1.8.7")]
        {
            if let Some(v) = self.elink_file_cache_size {
                h5try!(H5Pset_elink_file_cache_size(id, v as _));
            }
        }
        if let Some(v) = self.meta_block_size {
            h5try!(H5Pset_meta_block_size(id, v as _));
        }
        #[cfg(feature = "1.10.1")]
        {
            if let Some(v) = self.page_buffer_size {
                h5try!(H5Pset_page_buffer_size(
                    id,
                    v.buf_size as _,
                    v.min_meta_perc as _,
                    v.min_raw_perc as _,
                ));
            }
            if let Some(evict) = self.evict_on_close {
                // Issue #259: H5Pset_evict_on_close is not allowed to be called
                // even if the argument is `false` on e.g. parallel/mpio setups
                let has_evict_on_close = h5get!(H5Pget_evict_on_close(id): hbool_t).map(|x| x > 0);
                if evict != has_evict_on_close.unwrap_or(false) {
                    h5try!(H5Pset_evict_on_close(id, hbool_t::from(evict)));
                }
            }
            if let Some(v) = self.mdc_image_config {
                let v = v.into();
                h5try!(H5Pset_mdc_image_config(id, addr_of!(v)));
            }
        }
        if let Some(v) = self.sieve_buf_size {
            h5try!(H5Pset_sieve_buf_size(id, v as _));
        }
        #[cfg(feature = "1.10.0")]
        {
            if let Some(v) = self.metadata_read_attempts {
                h5try!(H5Pset_metadata_read_attempts(id, v as _));
            }
            if let Some(ref v) = self.mdc_log_options {
                let location = to_cstring(v.location.as_ref())?;
                h5try!(H5Pset_mdc_log_options(
                    id,
                    hbool_t::from(v.is_enabled),
                    location.as_ptr(),
                    hbool_t::from(v.start_on_access),
                ));
            }
        }
        #[cfg(all(feature = "1.10.0", feature = "have-parallel"))]
        {
            if let Some(v) = self.all_coll_metadata_ops {
                h5try!(H5Pset_all_coll_metadata_ops(id, v as _));
            }
            if let Some(v) = self.coll_metadata_write {
                h5try!(H5Pset_coll_metadata_write(id, v as _));
            }
        }
        if let Some(ref v) = self.mdc_config {
            let v = v.clone().into();
            h5try!(H5Pset_mdc_config(id, addr_of!(v)));
        }
        Ok(())
    }

    /// Copies the builder settings into a file access property list.
    pub fn apply(&self, plist: &mut FileAccess) -> Result<()> {
        h5lock!(self.populate_plist(plist.id()))
    }

    /// Constructs a new file access property list.
    pub fn finish(&self) -> Result<FileAccess> {
        h5lock!({
            let mut plist = FileAccess::try_new()?;
            self.apply(&mut plist).map(|()| plist)
        })
    }
}

/// File access property list.
impl FileAccess {
    /// Creates a new file access property list.
    pub fn try_new() -> Result<Self> {
        Self::from_id(h5try!(H5Pcreate(*H5P_FILE_ACCESS)))
    }

    /// Creates a copy of the property list.
    pub fn copy(&self) -> Self {
        unsafe { self.deref().copy().cast_unchecked() }
    }

    /// Creates a new file access property list builder.
    pub fn build() -> FileAccessBuilder {
        FileAccessBuilder::new()
    }

    #[doc(hidden)]
    fn get_core(&self) -> Result<CoreDriver> {
        let mut drv = CoreDriver::default();
        let mut increment: size_t = 0;
        let mut filebacked: hbool_t = 0;
        h5try!(H5Pget_fapl_core(self.id(), addr_of_mut!(increment), addr_of_mut!(filebacked)));
        drv.increment = increment as _;
        drv.filebacked = filebacked > 0;
        #[cfg(feature = "1.8.13")]
        {
            let mut is_enabled: hbool_t = 0;
            let mut page_size: size_t = 0;
            h5try!(H5Pget_core_write_tracking(
                self.id(),
                addr_of_mut!(is_enabled),
                addr_of_mut!(page_size),
            ));
            if is_enabled > 0 {
                drv.write_tracking = page_size;
            } else {
                drv.write_tracking = 0;
            }
        }
        Ok(drv)
    }

    #[doc(hidden)]
    fn get_family(&self) -> Result<FamilyDriver> {
        let member_size = h5get!(H5Pget_fapl_family(self.id()): hsize_t, hid_t)?.0;
        Ok(FamilyDriver { member_size: member_size as _ })
    }

    #[doc(hidden)]
    fn get_multi(&self) -> Result<MultiDriver> {
        const N: usize = H5FD_MEM_NTYPES as _;
        debug_assert_eq!(FD_MEM_TYPES.len(), N as _);
        let mut memb_map: [H5F_mem_t; N] = unsafe { mem::zeroed() };
        let mut memb_fapl: [hid_t; N] = unsafe { mem::zeroed() };
        let mut memb_name: [*const c_char; N] = unsafe { mem::zeroed() };
        let mut memb_addr: [haddr_t; N] = unsafe { mem::zeroed() };
        let mut relax: hbool_t = 0;
        h5try!(H5Pget_fapl_multi(
            self.id(),
            memb_map.as_mut_ptr(),
            memb_fapl.as_mut_ptr(),
            memb_name.as_mut_ptr(),
            memb_addr.as_mut_ptr(),
            addr_of_mut!(relax),
        ));
        let mut mapping: [u8; N] = unsafe { mem::zeroed() };
        let mut layout = MultiLayout::default();
        let mut files = Vec::new();
        for i in 1..N {
            let (map, name, addr) = (memb_map[i], memb_name[i], memb_addr[i]);
            let j = map as usize;
            ensure!(j < N, "member map index out of bounds: {} (expected 0-{})", j, N - 1);
            if mapping[j] == 0 {
                mapping[j] = 0xff - (files.len() as u8);
                files.push(MultiFile::new(
                    // SAFETY: name produced by HDF5 is nul-terminated and valid UTF-8
                    unsafe { &string_from_cstr(name) },
                    addr as _,
                ));
            }
            *layout.get_mut(i - 1) = 0xff - mapping[j];
        }
        for &memb_name in &memb_name {
            // SAFETY: the array contains pointers to strings allocated by the previous H5P call
            unsafe {
                crate::util::h5_free_memory(memb_name as *mut _);
            }
        }
        let relax = relax > 0;
        let drv = MultiDriver { files, layout, relax };
        drv.validate().map(|()| drv)
    }

    #[doc(hidden)]
    #[cfg(feature = "mpio")]
    fn get_mpio(&self) -> Result<MpioDriver> {
        let mut comm = mem::MaybeUninit::<mpi_sys::MPI_Comm>::uninit();
        let mut info = mem::MaybeUninit::<mpi_sys::MPI_Info>::uninit();
        h5try!(H5Pget_fapl_mpio(self.id(), comm.as_mut_ptr(), info.as_mut_ptr()));
        Ok(unsafe { MpioDriver { comm: comm.assume_init(), info: info.assume_init() } })
    }

    #[doc(hidden)]
    #[cfg(feature = "have-direct")]
    fn get_direct(&self) -> Result<DirectDriver> {
        let res = h5get!(H5Pget_fapl_direct(self.id()): size_t, size_t, size_t)?;
        Ok(DirectDriver { alignment: res.0 as _, block_size: res.1 as _, cbuf_size: res.2 as _ })
    }

    #[doc(hidden)]
    pub fn get_driver(&self) -> Result<FileDriver> {
        let drv_id = h5try!(H5Pget_driver(self.id()));
        #[cfg(feature = "mpio")]
        {
            if drv_id == *H5FD_MPIO {
                return self.get_mpio().map(FileDriver::Mpio);
            }
        }
        #[cfg(feature = "have-direct")]
        {
            if drv_id == *H5FD_DIRECT {
                return self.get_direct().map(FileDriver::Direct);
            }
        }
        if drv_id == *H5FD_SEC2 {
            Ok(FileDriver::Sec2)
        } else if drv_id == *H5FD_STDIO {
            Ok(FileDriver::Stdio)
        } else if drv_id == *H5FD_LOG {
            Ok(FileDriver::Log)
        } else if drv_id == *H5FD_CORE {
            self.get_core().map(FileDriver::Core)
        } else if drv_id == *H5FD_FAMILY {
            self.get_family().map(FileDriver::Family)
        } else if drv_id == *H5FD_MULTI {
            let multi = self.get_multi()?;
            SplitDriver::from_multi(&multi)
                .map_or(Ok(FileDriver::Multi(multi)), |split| Ok(FileDriver::Split(split)))
        } else {
            fail!("unknown or unsupported file driver (id: {})", drv_id);
        }
    }

    /// Returns the file driver properties.
    pub fn driver(&self) -> FileDriver {
        self.get_driver().unwrap_or(FileDriver::Sec2)
    }

    #[doc(hidden)]
    pub fn get_fclose_degree(&self) -> Result<FileCloseDegree> {
        h5get!(H5Pget_fclose_degree(self.id()): H5F_close_degree_t).map(Into::into)
    }

    /// Returns the file close degree.
    pub fn fclose_degree(&self) -> FileCloseDegree {
        self.get_fclose_degree().unwrap_or_else(|_| FileCloseDegree::default())
    }

    #[doc(hidden)]
    pub fn get_alignment(&self) -> Result<Alignment> {
        h5get!(H5Pget_alignment(self.id()): hsize_t, hsize_t).map(|(threshold, alignment)| {
            Alignment { threshold: threshold as _, alignment: alignment as _ }
        })
    }

    /// Returns the file alignment properties.
    pub fn alignment(&self) -> Alignment {
        self.get_alignment().unwrap_or_else(|_| Alignment::default())
    }

    #[doc(hidden)]
    pub fn get_chunk_cache(&self) -> Result<ChunkCache> {
        h5get!(H5Pget_cache(self.id()): c_int, size_t, size_t, c_double).map(
            |(_, nslots, nbytes, w0)| ChunkCache {
                nslots: nslots as _,
                nbytes: nbytes as _,
                w0: w0 as _,
            },
        )
    }

    /// Returns the raw data chunk cache properties.
    pub fn chunk_cache(&self) -> ChunkCache {
        self.get_chunk_cache().unwrap_or_else(|_| ChunkCache::default())
    }

    #[cfg(feature = "1.8.7")]
    #[doc(hidden)]
    pub fn get_elink_file_cache_size(&self) -> Result<u32> {
        h5get!(H5Pget_elink_file_cache_size(self.id()): c_uint).map(|x| x as _)
    }

    #[cfg(feature = "1.8.7")]
    pub fn elink_file_cache_size(&self) -> u32 {
        self.get_elink_file_cache_size().unwrap_or(0)
    }

    #[doc(hidden)]
    pub fn get_meta_block_size(&self) -> Result<u64> {
        h5get!(H5Pget_meta_block_size(self.id()): hsize_t).map(|x| x as _)
    }

    /// Returns the metadata block size.
    pub fn meta_block_size(&self) -> u64 {
        self.get_meta_block_size().unwrap_or(2048)
    }

    #[cfg(feature = "1.10.1")]
    #[doc(hidden)]
    pub fn get_page_buffer_size(&self) -> Result<PageBufferSize> {
        h5get!(H5Pget_page_buffer_size(self.id()): size_t, c_uint, c_uint).map(
            |(buf_size, min_meta_perc, min_raw_perc)| PageBufferSize {
                buf_size: buf_size as _,
                min_meta_perc: min_meta_perc as _,
                min_raw_perc: min_raw_perc as _,
            },
        )
    }

    /// Returns the page buffer size properties.
    #[cfg(feature = "1.10.1")]
    pub fn page_buffer_size(&self) -> PageBufferSize {
        self.get_page_buffer_size().unwrap_or_else(|_| PageBufferSize::default())
    }

    #[doc(hidden)]
    pub fn get_sieve_buf_size(&self) -> Result<usize> {
        h5get!(H5Pget_sieve_buf_size(self.id()): size_t).map(|x| x as _)
    }

    /// Returns the maximum data sieve buffer size.
    pub fn sieve_buf_size(&self) -> usize {
        self.get_sieve_buf_size().unwrap_or(64 * 1024)
    }

    #[cfg(feature = "1.10.1")]
    #[doc(hidden)]
    pub fn get_evict_on_close(&self) -> Result<bool> {
        h5get!(H5Pget_evict_on_close(self.id()): hbool_t).map(|x| x > 0)
    }

    /// Returns `true` if an object will be evicted from the metadata cache when the object is
    /// closed.
    #[cfg(feature = "1.10.1")]
    pub fn evict_on_close(&self) -> bool {
        self.get_evict_on_close().unwrap_or(false)
    }

    #[cfg(feature = "1.10.0")]
    #[doc(hidden)]
    pub fn get_metadata_read_attempts(&self) -> Result<u32> {
        h5get!(H5Pget_metadata_read_attempts(self.id()): c_uint).map(|x| x as _)
    }

    /// Returns the number of read attempts for SWMR access.
    #[cfg(feature = "1.10.0")]
    pub fn metadata_read_attempts(&self) -> u32 {
        self.get_metadata_read_attempts().unwrap_or(1)
    }

    #[doc(hidden)]
    pub fn get_mdc_config(&self) -> Result<MetadataCacheConfig> {
        let mut config: H5AC_cache_config_t = unsafe { mem::zeroed() };
        config.version = H5AC__CURR_CACHE_CONFIG_VERSION;
        h5call!(H5Pget_mdc_config(self.id(), &mut config)).map(|_| config.into())
    }

    /// Returns the metadata cache configuration.
    pub fn mdc_config(&self) -> MetadataCacheConfig {
        self.get_mdc_config().ok().unwrap_or_default()
    }

    #[cfg(feature = "1.10.1")]
    #[doc(hidden)]
    pub fn get_mdc_image_config(&self) -> Result<CacheImageConfig> {
        let mut config: H5AC_cache_image_config_t = unsafe { mem::zeroed() };
        config.version = H5AC__CURR_CACHE_CONFIG_VERSION;
        h5call!(H5Pget_mdc_image_config(self.id(), &mut config)).map(|_| config.into())
    }

    /// Returns the metadata cache image configuration.
    #[cfg(feature = "1.10.1")]
    pub fn mdc_image_config(&self) -> CacheImageConfig {
        self.get_mdc_image_config().ok().unwrap_or_default()
    }

    #[cfg(feature = "1.10.0")]
    #[doc(hidden)]
    #[allow(clippy::unnecessary_cast)]
    pub fn get_mdc_log_options(&self) -> Result<CacheLogOptions> {
        let mut is_enabled: hbool_t = 0;
        let mut location_size: size_t = 0;
        let mut start_on_access: hbool_t = 0;
        h5try!(H5Pget_mdc_log_options(
            self.id(),
            &mut is_enabled,
            ptr::null_mut(),
            &mut location_size,
            &mut start_on_access
        ));
        let mut buf = vec![0; 1 + (location_size as usize)];
        h5try!(H5Pget_mdc_log_options(
            self.id(),
            &mut is_enabled,
            buf.as_mut_ptr(),
            &mut location_size,
            &mut start_on_access
        ));
        Ok(CacheLogOptions {
            is_enabled: is_enabled > 0,
            // SAFETY: buf points to a valid UTF-8 CStr created by previous H5P call
            location: unsafe { string_from_cstr(buf.as_ptr()) },
            start_on_access: start_on_access > 0,
        })
    }

    /// Returns the metadata cache logging options.
    #[cfg(feature = "1.10.0")]
    pub fn mdc_log_options(&self) -> CacheLogOptions {
        self.get_mdc_log_options().ok().unwrap_or_default()
    }

    #[cfg(all(feature = "1.10.0", feature = "have-parallel"))]
    #[doc(hidden)]
    pub fn get_all_coll_metadata_ops(&self) -> Result<bool> {
        h5get!(H5Pget_all_coll_metadata_ops(self.id()): hbool_t).map(|x| x > 0)
    }

    /// Returns `true` if metadata reads are collective.
    #[cfg(all(feature = "1.10.0", feature = "have-parallel"))]
    pub fn all_coll_metadata_ops(&self) -> bool {
        self.get_all_coll_metadata_ops().unwrap_or(false)
    }

    #[cfg(all(feature = "1.10.0", feature = "have-parallel"))]
    #[doc(hidden)]
    pub fn get_coll_metadata_write(&self) -> Result<bool> {
        h5get!(H5Pget_coll_metadata_write(self.id()): hbool_t).map(|x| x > 0)
    }

    /// Returns `true` if metadata writes are collective.
    #[cfg(all(feature = "1.10.0", feature = "have-parallel"))]
    pub fn coll_metadata_write(&self) -> bool {
        self.get_coll_metadata_write().unwrap_or(false)
    }

    #[doc(hidden)]
    pub fn get_gc_references(&self) -> Result<bool> {
        h5get!(H5Pget_gc_references(self.id()): c_uint).map(|x| x > 0)
    }

    /// Returns `true` if reference garbage collection is enabled.
    pub fn gc_references(&self) -> bool {
        self.get_gc_references().unwrap_or(false)
    }

    #[doc(hidden)]
    pub fn get_small_data_block_size(&self) -> Result<u64> {
        h5get!(H5Pget_small_data_block_size(self.id()): hsize_t).map(|x| x as _)
    }

    /// Returns the size setting in bytes of the small data block.
    pub fn small_data_block_size(&self) -> u64 {
        self.get_small_data_block_size().unwrap_or(2048)
    }

    #[cfg(feature = "1.10.2")]
    #[doc(hidden)]
    pub fn get_libver_bounds(&self) -> Result<LibVerBounds> {
        h5get!(H5Pget_libver_bounds(self.id()): H5F_libver_t, H5F_libver_t)
            .map(|(low, high)| LibVerBounds { low: low.into(), high: high.into() })
    }

    /// Returns the library format version bounds for writing objects to a file.
    #[cfg(feature = "1.10.2")]
    pub fn libver_bounds(&self) -> LibVerBounds {
        self.get_libver_bounds().ok().unwrap_or_default()
    }

    /// Returns the lower library format version bound for writing objects to a file.
    #[cfg(feature = "1.10.2")]
    pub fn libver(&self) -> LibraryVersion {
        self.get_libver_bounds().ok().unwrap_or_default().low
    }
}
