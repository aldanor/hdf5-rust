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
use std::ptr;

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
#[cfg(h5_have_direct)]
use hdf5_sys::h5p::{H5Pget_fapl_direct, H5Pset_fapl_direct};
#[cfg(feature = "mpio")]
use hdf5_sys::h5p::{H5Pget_fapl_mpio, H5Pset_fapl_mpio};

#[cfg(hdf5_1_10_1)]
use hdf5_sys::h5ac::{H5AC_cache_image_config_t, H5AC__CACHE_IMAGE__ENTRY_AGEOUT__NONE};
#[cfg(hdf5_1_10_2)]
use hdf5_sys::h5f::H5F_libver_t;
#[cfg(all(hdf5_1_10_0, h5_have_parallel))]
use hdf5_sys::h5p::{
    H5Pget_all_coll_metadata_ops, H5Pget_coll_metadata_write, H5Pset_all_coll_metadata_ops,
    H5Pset_coll_metadata_write,
};
#[cfg(hdf5_1_8_13)]
use hdf5_sys::h5p::{H5Pget_core_write_tracking, H5Pset_core_write_tracking};
#[cfg(hdf5_1_8_7)]
use hdf5_sys::h5p::{H5Pget_elink_file_cache_size, H5Pset_elink_file_cache_size};
#[cfg(hdf5_1_10_1)]
use hdf5_sys::h5p::{
    H5Pget_evict_on_close, H5Pget_mdc_image_config, H5Pget_page_buffer_size, H5Pset_evict_on_close,
    H5Pset_mdc_image_config, H5Pset_page_buffer_size,
};
#[cfg(hdf5_1_10_2)]
use hdf5_sys::h5p::{H5Pget_libver_bounds, H5Pset_libver_bounds};
#[cfg(hdf5_1_10_0)]
use hdf5_sys::h5p::{
    H5Pget_mdc_log_options, H5Pget_metadata_read_attempts, H5Pset_mdc_log_options,
    H5Pset_metadata_read_attempts,
};

#[cfg(h5_have_direct)]
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
        let class = self.class()?;
        if class != PropertyListClass::FileAccess {
            fail!("expected file access property list, got {:?}", class);
        }
        Ok(())
    }
}

impl Debug for FileAccess {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let _e = silence_errors();
        let mut formatter = f.debug_struct("FileAccess");
        formatter.field("alignment", &self.alignment());
        formatter.field("chunk_cache", &self.chunk_cache());
        formatter.field("fclose_degree", &self.fclose_degree());
        formatter.field("gc_references", &self.gc_references());
        formatter.field("small_data_block_size", &self.small_data_block_size());
        #[cfg(hdf5_1_10_2)]
        formatter.field("libver_bounds", &self.libver_bounds());
        #[cfg(hdf5_1_8_7)]
        formatter.field("elink_file_cache_size", &self.elink_file_cache_size());
        formatter.field("meta_block_size", &self.meta_block_size());
        #[cfg(hdf5_1_10_1)]
        formatter.field("page_buffer_size", &self.page_buffer_size());
        #[cfg(hdf5_1_10_1)]
        formatter.field("evict_on_close", &self.evict_on_close());
        #[cfg(hdf5_1_10_1)]
        formatter.field("mdc_image_config", &self.mdc_image_config());
        formatter.field("sieve_buf_size", &self.sieve_buf_size());
        #[cfg(hdf5_1_10_0)]
        formatter.field("metadata_read_attempts", &self.metadata_read_attempts());
        #[cfg(hdf5_1_10_0)]
        formatter.field("mdc_log_options", &self.mdc_log_options());
        #[cfg(all(hdf5_1_10_0, h5_have_parallel))]
        formatter.field("all_coll_metadata_ops", &self.all_coll_metadata_ops());
        #[cfg(all(hdf5_1_10_0, h5_have_parallel))]
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
        unsafe { self.deref().clone().cast() }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct CoreDriver {
    pub increment: usize,
    pub filebacked: bool,
    #[cfg(hdf5_1_8_13)]
    pub write_tracking: usize,
}

impl Default for CoreDriver {
    fn default() -> Self {
        Self {
            increment: 1024 * 1024,
            filebacked: false,
            #[cfg(hdf5_1_8_13)]
            write_tracking: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FamilyDriver {
    pub member_size: usize,
}

impl Default for FamilyDriver {
    fn default() -> Self {
        Self { member_size: H5F_FAMILY_DEFAULT as _ }
    }
}

bitflags! {
    pub struct LogFlags: u64 {
        const TRUNCATE = H5FD_LOG_TRUNCATE;
        const META_IO = H5FD_LOG_META_IO;
        const LOC_READ = H5FD_LOG_LOC_READ;
        const LOC_WRITE = H5FD_LOG_LOC_WRITE;
        const LOC_SEEK = H5FD_LOG_LOC_SEEK;
        const LOC_IO = H5FD_LOG_LOC_IO;
        const FILE_READ = H5FD_LOG_FILE_READ;
        const FILE_WRITE = H5FD_LOG_FILE_WRITE;
        const FILE_IO = H5FD_LOG_FILE_IO;
        const FLAVOR = H5FD_LOG_FLAVOR;
        const NUM_READ = H5FD_LOG_NUM_READ;
        const NUM_WRITE = H5FD_LOG_NUM_WRITE;
        const NUM_SEEK = H5FD_LOG_NUM_SEEK;
        const NUM_TRUNCATE = H5FD_LOG_NUM_TRUNCATE;
        const NUM_IO = H5FD_LOG_NUM_IO;
        const TIME_OPEN = H5FD_LOG_TIME_OPEN;
        const TIME_STAT = H5FD_LOG_TIME_STAT;
        const TIME_READ = H5FD_LOG_TIME_READ;
        const TIME_WRITE = H5FD_LOG_TIME_WRITE;
        const TIME_SEEK = H5FD_LOG_TIME_SEEK;
        const TIME_TRUNCATE = H5FD_LOG_TIME_TRUNCATE;
        const TIME_CLOSE = H5FD_LOG_TIME_CLOSE;
        const TIME_IO = H5FD_LOG_TIME_IO;
        const FREE = H5FD_LOG_FREE;
        const ALL = H5FD_LOG_ALL;
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LogOptions {
    logfile: Option<String>,
    flags: LogFlags,
    buf_size: usize,
}

impl Default for LogOptions {
    fn default() -> Self {
        Self { logfile: None, flags: LogFlags::LOC_IO, buf_size: 0 }
    }
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MultiFile {
    pub name: String,
    pub addr: u64,
}

impl MultiFile {
    pub fn new(name: &str, addr: u64) -> Self {
        Self { name: name.into(), addr }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MultiLayout {
    pub mem_super: u8,
    pub mem_btree: u8,
    pub mem_draw: u8,
    pub mem_gheap: u8,
    pub mem_lheap: u8,
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MultiDriver {
    pub files: Vec<MultiFile>,
    pub layout: MultiLayout,
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SplitDriver {
    pub meta_ext: String,
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
        if cfg!(hdf5_1_8_10) {
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

    #[derive(Debug)]
    pub struct MpioDriver {
        pub comm: MPI_Comm,
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

#[cfg(h5_have_direct)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DirectDriver {
    pub alignment: usize,
    pub block_size: usize,
    pub cbuf_size: usize,
}

#[cfg(h5_have_direct)]
impl Default for DirectDriver {
    fn default() -> Self {
        Self { alignment: 4096, block_size: 4096, cbuf_size: 16 * 1024 * 1024 }
    }
}

#[derive(Clone, Debug)]
pub enum FileDriver {
    Sec2,
    Stdio,
    Log,
    Core(CoreDriver),
    Family(FamilyDriver),
    Multi(MultiDriver),
    Split(SplitDriver),
    #[cfg(feature = "mpio")]
    Mpio(MpioDriver),
    #[cfg(h5_have_direct)]
    Direct(DirectDriver),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileCloseDegree {
    Default,
    Weak,
    Semi,
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
            _ => Self::Default,
        }
    }
}

impl Into<H5F_close_degree_t> for FileCloseDegree {
    fn into(self) -> H5F_close_degree_t {
        match self {
            Self::Weak => H5F_close_degree_t::H5F_CLOSE_WEAK,
            Self::Semi => H5F_close_degree_t::H5F_CLOSE_SEMI,
            Self::Strong => H5F_close_degree_t::H5F_CLOSE_STRONG,
            _ => H5F_close_degree_t::H5F_CLOSE_DEFAULT,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Alignment {
    pub threshold: u64,
    pub alignment: u64,
}

impl Default for Alignment {
    fn default() -> Self {
        Self { threshold: 1, alignment: 1 }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChunkCache {
    pub nslots: usize,
    pub nbytes: usize,
    pub w0: f64,
}

impl Default for ChunkCache {
    fn default() -> Self {
        Self { nslots: 521, nbytes: 1024 * 1024, w0: 0.75 }
    }
}

impl Eq for ChunkCache {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PageBufferSize {
    pub buf_size: usize,
    pub min_meta_perc: u32,
    pub min_raw_perc: u32,
}

impl Default for PageBufferSize {
    fn default() -> Self {
        Self { buf_size: 0, min_meta_perc: 0, min_raw_perc: 0 }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CacheIncreaseMode {
    Off,
    Threshold,
}

impl From<H5C_cache_incr_mode> for CacheIncreaseMode {
    fn from(mode: H5C_cache_incr_mode) -> Self {
        match mode {
            H5C_cache_incr_mode::H5C_incr__threshold => Self::Threshold,
            _ => Self::Off,
        }
    }
}

impl Into<H5C_cache_incr_mode> for CacheIncreaseMode {
    fn into(self) -> H5C_cache_incr_mode {
        match self {
            Self::Threshold => H5C_cache_incr_mode::H5C_incr__threshold,
            _ => H5C_cache_incr_mode::H5C_incr__off,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlashIncreaseMode {
    Off,
    AddSpace,
}

impl From<H5C_cache_flash_incr_mode> for FlashIncreaseMode {
    fn from(mode: H5C_cache_flash_incr_mode) -> Self {
        match mode {
            H5C_cache_flash_incr_mode::H5C_flash_incr__add_space => Self::AddSpace,
            _ => Self::Off,
        }
    }
}

impl Into<H5C_cache_flash_incr_mode> for FlashIncreaseMode {
    fn into(self) -> H5C_cache_flash_incr_mode {
        match self {
            Self::AddSpace => H5C_cache_flash_incr_mode::H5C_flash_incr__add_space,
            _ => H5C_cache_flash_incr_mode::H5C_flash_incr__off,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CacheDecreaseMode {
    Off,
    Threshold,
    AgeOut,
    AgeOutWithThreshold,
}

impl From<H5C_cache_decr_mode> for CacheDecreaseMode {
    fn from(mode: H5C_cache_decr_mode) -> Self {
        match mode {
            H5C_cache_decr_mode::H5C_decr__threshold => Self::Threshold,
            H5C_cache_decr_mode::H5C_decr__age_out => Self::AgeOut,
            H5C_cache_decr_mode::H5C_decr__age_out_with_threshold => Self::AgeOutWithThreshold,
            _ => Self::Off,
        }
    }
}

impl Into<H5C_cache_decr_mode> for CacheDecreaseMode {
    fn into(self) -> H5C_cache_decr_mode {
        match self {
            Self::Threshold => H5C_cache_decr_mode::H5C_decr__threshold,
            Self::AgeOut => H5C_cache_decr_mode::H5C_decr__age_out,
            Self::AgeOutWithThreshold => H5C_cache_decr_mode::H5C_decr__age_out_with_threshold,
            _ => H5C_cache_decr_mode::H5C_decr__off,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MetadataWriteStrategy {
    ProcessZeroOnly,
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

impl Into<c_int> for MetadataWriteStrategy {
    fn into(self) -> c_int {
        match self {
            Self::Distributed => H5AC_METADATA_WRITE_STRATEGY__DISTRIBUTED,
            _ => H5AC_METADATA_WRITE_STRATEGY__PROCESS_0_ONLY,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MetadataCacheConfig {
    pub rpt_fcn_enabled: bool,
    pub open_trace_file: bool,
    pub close_trace_file: bool,
    pub trace_file_name: String,
    pub evictions_enabled: bool,
    pub set_initial_size: bool,
    pub initial_size: usize,
    pub min_clean_fraction: f64,
    pub max_size: usize,
    pub min_size: usize,
    pub epoch_length: i64,
    pub incr_mode: CacheIncreaseMode,
    pub lower_hr_threshold: f64,
    pub increment: f64,
    pub apply_max_increment: bool,
    pub max_increment: usize,
    pub flash_incr_mode: FlashIncreaseMode,
    pub flash_multiple: f64,
    pub flash_threshold: f64,
    pub decr_mode: CacheDecreaseMode,
    pub upper_hr_threshold: f64,
    pub decrement: f64,
    pub apply_max_decrement: bool,
    pub max_decrement: usize,
    pub epochs_before_eviction: i32,
    pub apply_empty_reserve: bool,
    pub empty_reserve: f64,
    pub dirty_bytes_threshold: usize,
    pub metadata_write_strategy: MetadataWriteStrategy,
}

impl Eq for MetadataCacheConfig {}

impl Default for MetadataCacheConfig {
    fn default() -> Self {
        let min_clean_fraction = if cfg!(h5_have_parallel) { 0.3_f32 } else { 0.01_f32 };
        let flash_multiple = if cfg!(h5_have_parallel) { 1.0_f32 } else { 1.4_f32 };
        Self {
            rpt_fcn_enabled: false,
            open_trace_file: false,
            close_trace_file: false,
            trace_file_name: "".into(),
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

impl Into<H5AC_cache_config_t> for MetadataCacheConfig {
    fn into(self) -> H5AC_cache_config_t {
        const N: usize = H5AC__MAX_TRACE_FILE_NAME_LEN;
        let mut trace_file_name: [c_char; N + 1] = unsafe { mem::zeroed() };
        string_to_fixed_bytes(&self.trace_file_name, &mut trace_file_name[..N]);
        H5AC_cache_config_t {
            version: H5AC__CURR_CACHE_CONFIG_VERSION,
            rpt_fcn_enabled: self.rpt_fcn_enabled as _,
            open_trace_file: self.open_trace_file as _,
            close_trace_file: self.close_trace_file as _,
            trace_file_name,
            evictions_enabled: self.evictions_enabled as _,
            set_initial_size: self.set_initial_size as _,
            initial_size: self.initial_size as _,
            min_clean_fraction: self.min_clean_fraction as _,
            max_size: self.max_size as _,
            min_size: self.min_size as _,
            epoch_length: self.epoch_length as _,
            incr_mode: self.incr_mode.into(),
            lower_hr_threshold: self.lower_hr_threshold as _,
            increment: self.increment as _,
            apply_max_increment: self.apply_max_increment as _,
            max_increment: self.max_increment as _,
            flash_incr_mode: self.flash_incr_mode.into(),
            flash_multiple: self.flash_multiple as _,
            flash_threshold: self.flash_threshold as _,
            decr_mode: self.decr_mode.into(),
            upper_hr_threshold: self.upper_hr_threshold as _,
            decrement: self.decrement as _,
            apply_max_decrement: self.apply_max_decrement as _,
            max_decrement: self.max_decrement as _,
            epochs_before_eviction: self.epochs_before_eviction as _,
            apply_empty_reserve: self.apply_empty_reserve as _,
            empty_reserve: self.empty_reserve as _,
            dirty_bytes_threshold: self.dirty_bytes_threshold as _,
            metadata_write_strategy: self.metadata_write_strategy.into(),
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

#[cfg(hdf5_1_10_1)]
mod cache_image_config {
    use super::*;

    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    pub struct CacheImageConfig {
        pub generate_image: bool,
        pub save_resize_status: bool,
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

    impl Into<H5AC_cache_image_config_t> for CacheImageConfig {
        fn into(self) -> H5AC_cache_image_config_t {
            H5AC_cache_image_config_t {
                version: H5AC__CURR_CACHE_CONFIG_VERSION,
                generate_image: self.generate_image as _,
                save_resize_status: self.save_resize_status as _,
                entry_ageout: self.entry_ageout as _,
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

#[cfg(hdf5_1_10_1)]
pub use self::cache_image_config::*;

#[cfg(hdf5_1_10_0)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CacheLogOptions {
    pub is_enabled: bool,
    pub location: String,
    pub start_on_access: bool,
}

#[cfg(hdf5_1_10_0)]
impl Default for CacheLogOptions {
    fn default() -> Self {
        Self { is_enabled: false, location: "".into(), start_on_access: false }
    }
}

#[cfg(hdf5_1_10_2)]
mod libver {
    use super::*;

    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub enum LibraryVersion {
        Earliest = 0,
        V18 = 1,
        V110 = 2,
    }

    impl LibraryVersion {
        pub fn is_earliest(self) -> bool {
            self == Self::Earliest
        }

        pub const fn latest() -> Self {
            Self::V110
        }
    }

    impl Into<H5F_libver_t> for LibraryVersion {
        fn into(self) -> H5F_libver_t {
            match self {
                Self::V18 => H5F_libver_t::H5F_LIBVER_V18,
                Self::V110 => H5F_libver_t::H5F_LIBVER_V110,
                _ => H5F_libver_t::H5F_LIBVER_EARLIEST,
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

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct LibVerBounds {
        pub low: LibraryVersion,
        pub high: LibraryVersion,
    }

    impl Default for LibVerBounds {
        fn default() -> Self {
            Self { low: LibraryVersion::Earliest, high: LibraryVersion::latest() }
        }
    }
}

#[cfg(hdf5_1_10_2)]
pub use self::libver::*;

/// Builder used to create file access property list.
#[derive(Clone, Debug, Default)]
pub struct FileAccessBuilder {
    file_driver: Option<FileDriver>,
    log_options: LogOptions,
    #[cfg(hdf5_1_8_13)]
    write_tracking: Option<usize>,
    fclose_degree: Option<FileCloseDegree>,
    alignment: Option<Alignment>,
    chunk_cache: Option<ChunkCache>,
    #[cfg(hdf5_1_8_7)]
    elink_file_cache_size: Option<u32>,
    meta_block_size: Option<u64>,
    #[cfg(hdf5_1_10_1)]
    page_buffer_size: Option<PageBufferSize>,
    sieve_buf_size: Option<usize>,
    #[cfg(hdf5_1_10_1)]
    evict_on_close: Option<bool>,
    #[cfg(hdf5_1_10_0)]
    metadata_read_attempts: Option<u32>,
    mdc_config: Option<MetadataCacheConfig>,
    #[cfg(hdf5_1_10_1)]
    mdc_image_config: Option<CacheImageConfig>,
    #[cfg(hdf5_1_10_0)]
    mdc_log_options: Option<CacheLogOptions>,
    #[cfg(all(hdf5_1_10_0, h5_have_parallel))]
    all_coll_metadata_ops: Option<bool>,
    #[cfg(all(hdf5_1_10_0, h5_have_parallel))]
    coll_metadata_write: Option<bool>,
    gc_references: Option<bool>,
    small_data_block_size: Option<u64>,
    #[cfg(hdf5_1_10_2)]
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
        #[cfg(hdf5_1_10_2)]
        {
            let v = plist.get_libver_bounds()?;
            builder.libver_bounds(v.low, v.high);
        }
        #[cfg(hdf5_1_8_7)]
        {
            builder.elink_file_cache_size(plist.get_elink_file_cache_size()?);
        }
        builder.meta_block_size(plist.get_meta_block_size()?);
        #[cfg(hdf5_1_10_1)]
        {
            let v = plist.get_page_buffer_size()?;
            builder.page_buffer_size(v.buf_size, v.min_meta_perc, v.min_raw_perc);
            builder.evict_on_close(plist.get_evict_on_close()?);
            builder.mdc_image_config(plist.get_mdc_image_config()?.generate_image);
        }
        builder.sieve_buf_size(plist.get_sieve_buf_size()?);
        #[cfg(hdf5_1_10_0)]
        {
            builder.metadata_read_attempts(plist.get_metadata_read_attempts()?);
            let v = plist.get_mdc_log_options()?;
            builder.mdc_log_options(v.is_enabled, &v.location, v.start_on_access);
        }
        #[cfg(all(hdf5_1_10_0, h5_have_parallel))]
        {
            builder.all_coll_metadata_ops(plist.get_all_coll_metadata_ops()?);
            builder.coll_metadata_write(plist.get_coll_metadata_write()?);
        }
        builder.mdc_config(&plist.get_mdc_config()?);
        #[cfg(hdf5_1_8_13)]
        {
            if let FileDriver::Core(ref drv) = drv {
                builder.write_tracking(drv.write_tracking);
            }
        }
        Ok(builder)
    }

    pub fn fclose_degree(&mut self, fc_degree: FileCloseDegree) -> &mut Self {
        self.fclose_degree = Some(fc_degree);
        self
    }

    pub fn alignment(&mut self, threshold: u64, alignment: u64) -> &mut Self {
        self.alignment = Some(Alignment { threshold, alignment });
        self
    }

    pub fn chunk_cache(&mut self, nslots: usize, nbytes: usize, w0: f64) -> &mut Self {
        self.chunk_cache = Some(ChunkCache { nslots, nbytes, w0 });
        self
    }

    #[cfg(hdf5_1_8_7)]
    pub fn elink_file_cache_size(&mut self, efc_size: u32) -> &mut Self {
        self.elink_file_cache_size = Some(efc_size);
        self
    }

    pub fn meta_block_size(&mut self, size: u64) -> &mut Self {
        self.meta_block_size = Some(size);
        self
    }

    #[cfg(hdf5_1_10_1)]
    pub fn page_buffer_size(
        &mut self, buf_size: usize, min_meta_perc: u32, min_raw_perc: u32,
    ) -> &mut Self {
        self.page_buffer_size = Some(PageBufferSize { buf_size, min_meta_perc, min_raw_perc });
        self
    }

    pub fn sieve_buf_size(&mut self, size: usize) -> &mut Self {
        self.sieve_buf_size = Some(size);
        self
    }

    #[cfg(hdf5_1_10_1)]
    pub fn evict_on_close(&mut self, evict_on_close: bool) -> &mut Self {
        self.evict_on_close = Some(evict_on_close);
        self
    }

    #[cfg(hdf5_1_10_0)]
    pub fn metadata_read_attempts(&mut self, attempts: u32) -> &mut Self {
        self.metadata_read_attempts = Some(attempts);
        self
    }

    pub fn mdc_config(&mut self, config: &MetadataCacheConfig) -> &mut Self {
        self.mdc_config = Some(config.clone());
        self
    }

    #[cfg(hdf5_1_10_1)]
    pub fn mdc_image_config(&mut self, generate_image: bool) -> &mut Self {
        self.mdc_image_config = Some(CacheImageConfig {
            generate_image,
            save_resize_status: false,
            entry_ageout: H5AC__CACHE_IMAGE__ENTRY_AGEOUT__NONE,
        });
        self
    }

    #[cfg(hdf5_1_10_0)]
    pub fn mdc_log_options(
        &mut self, is_enabled: bool, location: &str, start_on_access: bool,
    ) -> &mut Self {
        self.mdc_log_options =
            Some(CacheLogOptions { is_enabled, location: location.into(), start_on_access });
        self
    }

    #[cfg(all(hdf5_1_10_0, h5_have_parallel))]
    pub fn all_coll_metadata_ops(&mut self, is_collective: bool) -> &mut Self {
        self.all_coll_metadata_ops = Some(is_collective);
        self
    }

    #[cfg(all(hdf5_1_10_0, h5_have_parallel))]
    pub fn coll_metadata_write(&mut self, is_collective: bool) -> &mut Self {
        self.coll_metadata_write = Some(is_collective);
        self
    }

    pub fn gc_references(&mut self, gc_ref: bool) -> &mut Self {
        self.gc_references = Some(gc_ref);
        self
    }

    pub fn small_data_block_size(&mut self, size: u64) -> &mut Self {
        self.small_data_block_size = Some(size);
        self
    }

    #[cfg(hdf5_1_10_2)]
    pub fn libver_bounds(&mut self, low: LibraryVersion, high: LibraryVersion) -> &mut Self {
        self.libver_bounds = Some(LibVerBounds { low, high });
        self
    }

    pub fn driver(&mut self, file_driver: &FileDriver) -> &mut Self {
        self.file_driver = Some(file_driver.clone());
        self
    }

    pub fn sec2(&mut self) -> &mut Self {
        self.driver(&FileDriver::Sec2)
    }

    pub fn stdio(&mut self) -> &mut Self {
        self.driver(&FileDriver::Stdio)
    }

    pub fn log_options(
        &mut self, logfile: Option<&str>, flags: LogFlags, buf_size: usize,
    ) -> &mut Self {
        self.log_options.logfile = logfile.map(Into::into);
        self.log_options.flags = flags;
        self.log_options.buf_size = buf_size;
        self.driver(&FileDriver::Log)
    }

    pub fn log(&mut self) -> &mut Self {
        self.log_options(None, LogFlags::LOC_IO, 0)
    }

    pub fn core_options(&mut self, increment: usize, filebacked: bool) -> &mut Self {
        let mut drv = CoreDriver::default();
        drv.increment = increment;
        drv.filebacked = filebacked;
        self.driver(&FileDriver::Core(drv))
    }

    pub fn core_filebacked(&mut self, filebacked: bool) -> &mut Self {
        let mut drv = CoreDriver::default();
        drv.filebacked = filebacked;
        self.driver(&FileDriver::Core(drv))
    }

    pub fn core(&mut self) -> &mut Self {
        self.driver(&FileDriver::Core(CoreDriver::default()))
    }

    #[cfg(hdf5_1_8_13)]
    pub fn write_tracking(&mut self, page_size: usize) -> &mut Self {
        self.write_tracking = Some(page_size);
        self
    }

    pub fn family(&mut self) -> &mut Self {
        self.driver(&FileDriver::Family(FamilyDriver::default()))
    }

    pub fn family_options(&mut self, member_size: usize) -> &mut Self {
        self.driver(&FileDriver::Family(FamilyDriver { member_size }))
    }

    pub fn multi_options(
        &mut self, files: &[MultiFile], layout: &MultiLayout, relax: bool,
    ) -> &mut Self {
        self.driver(&FileDriver::Multi(MultiDriver {
            files: files.to_vec(),
            layout: layout.clone(),
            relax,
        }))
    }

    pub fn multi(&mut self) -> &mut Self {
        self.driver(&FileDriver::Multi(MultiDriver::default()))
    }

    pub fn split_options(&mut self, meta_ext: &str, raw_ext: &str) -> &mut Self {
        self.driver(&FileDriver::Split(SplitDriver {
            meta_ext: meta_ext.into(),
            raw_ext: raw_ext.into(),
        }))
    }

    pub fn split(&mut self) -> &mut Self {
        self.driver(&FileDriver::Split(SplitDriver::default()))
    }

    #[cfg(feature = "mpio")]
    pub fn mpio(&mut self, comm: mpi_sys::MPI_Comm, info: Option<mpi_sys::MPI_Info>) -> &mut Self {
        // We use .unwrap() here since MPI will almost surely terminate the process anyway.
        self.driver(&FileDriver::Mpio(MpioDriver::try_new(comm, info).unwrap()))
    }

    #[cfg(h5_have_direct)]
    pub fn direct_options(
        &mut self, alignment: usize, block_size: usize, cbuf_size: usize,
    ) -> &mut Self {
        self.driver(&FileDriver::Direct(DirectDriver { alignment, block_size, cbuf_size }))
    }

    #[cfg(h5_have_direct)]
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
        h5try!(H5Pset_fapl_core(id, drv.increment as _, drv.filebacked as _));
        #[cfg(hdf5_1_8_13)]
        {
            if let Some(page_size) = self.write_tracking {
                h5try!(H5Pset_core_write_tracking(id, (page_size > 0) as _, page_size.max(1) as _));
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
            drv.relax as _,
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

    #[cfg(h5_have_direct)]
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
            #[cfg(h5_have_direct)]
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
        if let Some(v) = self.fclose_degree {
            h5try!(H5Pset_fclose_degree(id, v.into()));
        }
        if let Some(v) = self.gc_references {
            h5try!(H5Pset_gc_references(id, v as _));
        }
        if let Some(v) = self.small_data_block_size {
            h5try!(H5Pset_small_data_block_size(id, v as _));
        }
        #[cfg(hdf5_1_10_2)]
        {
            if let Some(v) = self.libver_bounds {
                h5try!(H5Pset_libver_bounds(id, v.low.into(), v.high.into()));
            }
        }
        #[cfg(hdf5_1_8_7)]
        {
            if let Some(v) = self.elink_file_cache_size {
                h5try!(H5Pset_elink_file_cache_size(id, v as _));
            }
        }
        if let Some(v) = self.meta_block_size {
            h5try!(H5Pset_meta_block_size(id, v as _));
        }
        #[cfg(hdf5_1_10_1)]
        {
            if let Some(v) = self.page_buffer_size {
                h5try!(H5Pset_page_buffer_size(
                    id,
                    v.buf_size as _,
                    v.min_meta_perc as _,
                    v.min_raw_perc as _,
                ));
            }
            if let Some(v) = self.evict_on_close {
                h5try!(H5Pset_evict_on_close(id, v as _));
            }
            if let Some(v) = self.mdc_image_config {
                h5try!(H5Pset_mdc_image_config(id, &v.into() as *const _));
            }
        }
        if let Some(v) = self.sieve_buf_size {
            h5try!(H5Pset_sieve_buf_size(id, v as _));
        }
        #[cfg(hdf5_1_10_0)]
        {
            if let Some(v) = self.metadata_read_attempts {
                h5try!(H5Pset_metadata_read_attempts(id, v as _));
            }
            if let Some(ref v) = self.mdc_log_options {
                let location = to_cstring(v.location.as_ref())?;
                h5try!(H5Pset_mdc_log_options(
                    id,
                    v.is_enabled as _,
                    location.as_ptr(),
                    v.start_on_access as _,
                ));
            }
        }
        #[cfg(all(hdf5_1_10_0, h5_have_parallel))]
        {
            if let Some(v) = self.all_coll_metadata_ops {
                h5try!(H5Pset_all_coll_metadata_ops(id, v as _));
            }
            if let Some(v) = self.coll_metadata_write {
                h5try!(H5Pset_coll_metadata_write(id, v as _));
            }
        }
        if let Some(ref v) = self.mdc_config {
            h5try!(H5Pset_mdc_config(id, &v.clone().into() as *const _));
        }
        Ok(())
    }

    pub fn finish(&self) -> Result<FileAccess> {
        h5lock!({
            let plist = FileAccess::try_new()?;
            self.populate_plist(plist.id())?;
            Ok(plist)
        })
    }
}

/// File access property list.
impl FileAccess {
    pub fn try_new() -> Result<Self> {
        Self::from_id(h5try!(H5Pcreate(*H5P_FILE_ACCESS)))
    }

    pub fn copy(&self) -> Self {
        unsafe { self.deref().copy().cast() }
    }

    pub fn build() -> FileAccessBuilder {
        FileAccessBuilder::new()
    }

    #[doc(hidden)]
    fn get_core(&self) -> Result<CoreDriver> {
        let mut drv = CoreDriver::default();
        let mut increment: size_t = 0;
        let mut filebacked: hbool_t = 0;
        h5try!(H5Pget_fapl_core(self.id(), &mut increment as *mut _, &mut filebacked as *mut _));
        drv.increment = increment as _;
        drv.filebacked = filebacked > 0;
        #[cfg(hdf5_1_8_13)]
        {
            let mut is_enabled: hbool_t = 0;
            let mut page_size: size_t = 0;
            h5try!(H5Pget_core_write_tracking(
                self.id(),
                &mut is_enabled as *mut _,
                &mut page_size as *mut _,
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
            &mut relax as *mut _,
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
                files.push(MultiFile::new(&string_from_cstr(name), addr as _));
            }
            *layout.get_mut(i - 1) = 0xff - mapping[j];
        }
        let relax = relax > 0;
        let drv = MultiDriver { files, layout, relax };
        drv.validate().map(|_| drv)
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
    #[cfg(h5_have_direct)]
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
        #[cfg(h5_have_direct)]
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
            if let Some(split) = SplitDriver::from_multi(&multi) {
                Ok(FileDriver::Split(split))
            } else {
                Ok(FileDriver::Multi(multi))
            }
        } else {
            fail!("unknown or unsupported file driver (id: {})", drv_id);
        }
    }

    pub fn driver(&self) -> FileDriver {
        self.get_driver().unwrap_or(FileDriver::Sec2)
    }

    #[doc(hidden)]
    pub fn get_fclose_degree(&self) -> Result<FileCloseDegree> {
        h5get!(H5Pget_fclose_degree(self.id()): H5F_close_degree_t).map(|x| x.into())
    }

    pub fn fclose_degree(&self) -> FileCloseDegree {
        self.get_fclose_degree().unwrap_or_else(|_| FileCloseDegree::default())
    }

    #[doc(hidden)]
    pub fn get_alignment(&self) -> Result<Alignment> {
        h5get!(H5Pget_alignment(self.id()): hsize_t, hsize_t).map(|(threshold, alignment)| {
            Alignment { threshold: threshold as _, alignment: alignment as _ }
        })
    }

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

    pub fn chunk_cache(&self) -> ChunkCache {
        self.get_chunk_cache().unwrap_or_else(|_| ChunkCache::default())
    }

    #[cfg(hdf5_1_8_7)]
    #[doc(hidden)]
    pub fn get_elink_file_cache_size(&self) -> Result<u32> {
        h5get!(H5Pget_elink_file_cache_size(self.id()): c_uint).map(|x| x as _)
    }

    #[cfg(hdf5_1_8_7)]
    pub fn elink_file_cache_size(&self) -> u32 {
        self.get_elink_file_cache_size().unwrap_or(0)
    }

    #[doc(hidden)]
    pub fn get_meta_block_size(&self) -> Result<u64> {
        h5get!(H5Pget_meta_block_size(self.id()): hsize_t).map(|x| x as _)
    }

    pub fn meta_block_size(&self) -> u64 {
        self.get_meta_block_size().unwrap_or(2048)
    }

    #[cfg(hdf5_1_10_1)]
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

    #[cfg(hdf5_1_10_1)]
    pub fn page_buffer_size(&self) -> PageBufferSize {
        self.get_page_buffer_size().unwrap_or_else(|_| PageBufferSize::default())
    }

    #[doc(hidden)]
    pub fn get_sieve_buf_size(&self) -> Result<usize> {
        h5get!(H5Pget_sieve_buf_size(self.id()): size_t).map(|x| x as _)
    }

    pub fn sieve_buf_size(&self) -> usize {
        self.get_sieve_buf_size().unwrap_or(64 * 1024)
    }

    #[cfg(hdf5_1_10_1)]
    #[doc(hidden)]
    pub fn get_evict_on_close(&self) -> Result<bool> {
        h5get!(H5Pget_evict_on_close(self.id()): hbool_t).map(|x| x > 0)
    }

    #[cfg(hdf5_1_10_1)]
    pub fn evict_on_close(&self) -> bool {
        self.get_evict_on_close().unwrap_or(false)
    }

    #[cfg(hdf5_1_10_0)]
    #[doc(hidden)]
    pub fn get_metadata_read_attempts(&self) -> Result<u32> {
        h5get!(H5Pget_metadata_read_attempts(self.id()): c_uint).map(|x| x as _)
    }

    #[cfg(hdf5_1_10_0)]
    pub fn metadata_read_attempts(&self) -> u32 {
        self.get_metadata_read_attempts().unwrap_or(1)
    }

    #[doc(hidden)]
    pub fn get_mdc_config(&self) -> Result<MetadataCacheConfig> {
        let mut config: H5AC_cache_config_t = unsafe { mem::zeroed() };
        config.version = H5AC__CURR_CACHE_CONFIG_VERSION;
        h5call!(H5Pget_mdc_config(self.id(), &mut config)).map(|_| config.into())
    }

    pub fn mdc_config(&self) -> MetadataCacheConfig {
        self.get_mdc_config().ok().unwrap_or_else(MetadataCacheConfig::default)
    }

    #[cfg(hdf5_1_10_1)]
    #[doc(hidden)]
    pub fn get_mdc_image_config(&self) -> Result<CacheImageConfig> {
        let mut config: H5AC_cache_image_config_t = unsafe { mem::zeroed() };
        config.version = H5AC__CURR_CACHE_CONFIG_VERSION;
        h5call!(H5Pget_mdc_image_config(self.id(), &mut config)).map(|_| config.into())
    }

    #[cfg(hdf5_1_10_1)]
    pub fn mdc_image_config(&self) -> CacheImageConfig {
        self.get_mdc_image_config().ok().unwrap_or_else(CacheImageConfig::default)
    }

    #[cfg(hdf5_1_10_0)]
    #[doc(hidden)]
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
            location: string_from_cstr(buf.as_ptr()),
            start_on_access: start_on_access > 0,
        })
    }

    #[cfg(hdf5_1_10_0)]
    pub fn mdc_log_options(&self) -> CacheLogOptions {
        self.get_mdc_log_options().ok().unwrap_or_else(CacheLogOptions::default)
    }

    #[cfg(all(hdf5_1_10_0, h5_have_parallel))]
    #[doc(hidden)]
    pub fn get_all_coll_metadata_ops(&self) -> Result<bool> {
        h5get!(H5Pget_all_coll_metadata_ops(self.id()): hbool_t).map(|x| x > 0)
    }

    #[cfg(all(hdf5_1_10_0, h5_have_parallel))]
    pub fn all_coll_metadata_ops(&self) -> bool {
        self.get_all_coll_metadata_ops().unwrap_or(false)
    }

    #[cfg(all(hdf5_1_10_0, h5_have_parallel))]
    #[doc(hidden)]
    pub fn get_coll_metadata_write(&self) -> Result<bool> {
        h5get!(H5Pget_coll_metadata_write(self.id()): hbool_t).map(|x| x > 0)
    }

    #[cfg(all(hdf5_1_10_0, h5_have_parallel))]
    pub fn coll_metadata_write(&self) -> bool {
        self.get_coll_metadata_write().unwrap_or(false)
    }

    #[doc(hidden)]
    pub fn get_gc_references(&self) -> Result<bool> {
        h5get!(H5Pget_gc_references(self.id()): c_uint).map(|x| x > 0)
    }

    pub fn gc_references(&self) -> bool {
        self.get_gc_references().unwrap_or(false)
    }

    #[doc(hidden)]
    pub fn get_small_data_block_size(&self) -> Result<u64> {
        h5get!(H5Pget_small_data_block_size(self.id()): hsize_t).map(|x| x as _)
    }

    pub fn small_data_block_size(&self) -> u64 {
        self.get_small_data_block_size().unwrap_or(2048)
    }

    #[cfg(hdf5_1_10_2)]
    #[doc(hidden)]
    pub fn get_libver_bounds(&self) -> Result<LibVerBounds> {
        h5get!(H5Pget_libver_bounds(self.id()): H5F_libver_t, H5F_libver_t)
            .map(|(low, high)| LibVerBounds { low: low.into(), high: high.into() })
    }

    #[cfg(hdf5_1_10_2)]
    pub fn libver_bounds(&self) -> LibVerBounds {
        self.get_libver_bounds().ok().unwrap_or_else(LibVerBounds::default)
    }
}
