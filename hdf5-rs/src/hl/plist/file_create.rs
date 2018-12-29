//! File creation properties.

use std::fmt::{self, Debug};
use std::ops::Deref;

use bitflags::bitflags;

#[cfg(hdf5_1_10_1)]
use libhdf5_sys::h5f::H5F_fspace_strategy_t;
#[cfg(hdf5_1_10_0)]
use libhdf5_sys::h5f::{H5F_info2_t, H5Fget_info2};
use libhdf5_sys::h5o::{
    H5O_SHMESG_ALL_FLAG, H5O_SHMESG_ATTR_FLAG, H5O_SHMESG_DTYPE_FLAG, H5O_SHMESG_FILL_FLAG,
    H5O_SHMESG_NONE_FLAG, H5O_SHMESG_PLINE_FLAG, H5O_SHMESG_SDSPACE_FLAG,
};
#[cfg(not(hdf5_1_10_0))]
use libhdf5_sys::h5p::H5Pget_version;
use libhdf5_sys::h5p::{
    H5Pcreate, H5Pget_istore_k, H5Pget_shared_mesg_index, H5Pget_shared_mesg_nindexes,
    H5Pget_shared_mesg_phase_change, H5Pget_sizes, H5Pget_sym_k, H5Pget_userblock, H5Pset_istore_k,
    H5Pset_shared_mesg_index, H5Pset_shared_mesg_nindexes, H5Pset_shared_mesg_phase_change,
    H5Pset_sym_k, H5Pset_userblock,
};
#[cfg(hdf5_1_10_1)]
use libhdf5_sys::h5p::{
    H5Pget_file_space_page_size, H5Pget_file_space_strategy, H5Pset_file_space_page_size,
    H5Pset_file_space_strategy,
};

use crate::globals::H5P_FILE_CREATE;
use crate::internal_prelude::*;

/// File creation properties.
#[repr(transparent)]
pub struct FileCreate(Handle);

impl ObjectClass for FileCreate {
    const NAME: &'static str = "file-create property list";
    const VALID_TYPES: &'static [H5I_type_t] = &[H5I_GENPROP_LST];

    fn from_handle(handle: Handle) -> Self {
        FileCreate(handle)
    }

    fn handle(&self) -> &Handle {
        &self.0
    }

    fn validate(&self) -> Result<()> {
        let class = self.class()?;
        if class != PropertyListClass::FileCreate {
            fail!("expected file create property list, got {:?}", class);
        }
        Ok(())
    }
}

impl Debug for FileCreate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let _e = silence_errors();
        let mut formatter = f.debug_struct("FileCreate");
        formatter
            .field("userblock", &self.userblock())
            .field("sizes", &self.sizes())
            .field("sym_k", &self.sym_k())
            .field("istore_k", &self.istore_k())
            .field("version", &self.version())
            .field("shared_mesg_phase_change", &self.shared_mesg_phase_change())
            .field("shared_mesg_indexes", &self.shared_mesg_indexes());
        #[cfg(hdf5_1_10_1)]
        {
            formatter
                .field("file_space_page_size", &self.file_space_page_size())
                .field("file_space_strategy", &self.file_space_strategy());
        }
        formatter.finish()
    }
}

impl Deref for FileCreate {
    type Target = PropertyList;

    fn deref(&self) -> &PropertyList {
        unsafe { self.transmute() }
    }
}

impl PartialEq for FileCreate {
    fn eq(&self, other: &FileCreate) -> bool {
        <PropertyList as PartialEq>::eq(self, other)
    }
}

/// Version information of various objects in a file.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct VersionInfo {
    /// Super block version number.
    pub superblock: u32,
    /// Global freelist version number.
    pub freelist: u32,
    /// Shared object header version number.
    pub shared_header: u32,
}

/// Size of the offsets and lengths used in a file.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SizeofInfo {
    /// Offset size in bytes.
    pub sizeof_addr: usize,
    /// Length size in bytes.
    pub sizeof_size: usize,
}

/// Size of prameters used to control the symbol table nodes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SymbolTableInfo {
    /// Symbol table tree rank.
    pub tree_rank: u32,
    /// Symbol table node size.
    pub node_size: u32,
}

/// Shared object header message phase change information.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PhaseChangeInfo {
    /// Threshold above which storage of a shared object header message index
    /// shifts from list to B-tree.
    pub max_list: u32,
    /// Threshold below which storage of a shared object header message index
    /// reverts to list format.
    pub min_btree: u32,
}

bitflags! {
    /// Types of messages that can be stored in a shared message index.
    pub struct SharedMessageType: u32 {
        const NONE = H5O_SHMESG_NONE_FLAG;
        const SIMPLE_DATASPACE = H5O_SHMESG_SDSPACE_FLAG;
        const DATATYPE = H5O_SHMESG_DTYPE_FLAG;
        const FILL_VALUE = H5O_SHMESG_FILL_FLAG;
        const FILTER_PIPELINE = H5O_SHMESG_PLINE_FLAG;
        const ATTRIBUTE = H5O_SHMESG_ATTR_FLAG;
        const ALL = H5O_SHMESG_ALL_FLAG;
    }
}

impl Default for SharedMessageType {
    fn default() -> Self {
        SharedMessageType::NONE
    }
}

/// Configuration settings for a shared message index.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SharedMessageIndex {
    /// Types of messages that may be stored in the index.
    pub message_types: SharedMessageType,
    /// Minimum message size.
    pub min_message_size: u32,
}

/// File space handling strategy.
#[cfg(hdf5_1_10_1)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum FileSpaceStrategy {
    /// Mechanisms used: free-space managers, aggregators or embedded paged aggregation
    /// and the virtual file driver.
    FreeSpaceManager {
        /// Whether to use embedded paged aggregation.
        paged: bool,
        /// Whether free space is persistent or not.
        persist: bool,
        /// The free-space section size threshold value.
        threshold: u64,
    },
    /// Mechanisms used: aggregators and virtual file driver.
    PageAggregation,
    /// Mechanisms used: the virtual file driver.
    None,
}

#[cfg(hdf5_1_10_1)]
impl Default for FileSpaceStrategy {
    fn default() -> Self {
        FileSpaceStrategy::FreeSpaceManager { paged: false, persist: false, threshold: 1 }
    }
}

/// Builder used to create file creation property list.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FileCreateBuilder {
    userblock: Option<u64>,
    sym_k: Option<SymbolTableInfo>,
    istore_k: Option<u32>,
    shared_mesg_phase_change: Option<PhaseChangeInfo>,
    shared_mesg_indexes: Option<Vec<SharedMessageIndex>>,
    #[cfg(hdf5_1_10_1)]
    file_space_page_size: Option<u64>,
    #[cfg(hdf5_1_10_1)]
    file_space_strategy: Option<FileSpaceStrategy>,
}

/// Builder used to create file creation property list.
impl FileCreateBuilder {
    /// Creates a new file creation property list builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new builder from an existing property list.
    pub fn from_plist(plist: &FileCreate) -> Result<Self> {
        let mut builder = Self::default();
        builder
            .userblock(plist.get_userblock()?)
            .sym_k(plist.get_sym_k()?)
            .istore_k(plist.get_istore_k()?)
            .shared_mesg_phase_change(plist.get_shared_mesg_phase_change()?)
            .shared_mesg_indexes(plist.get_shared_mesg_indexes()?);
        #[cfg(hdf5_1_10_1)]
        {
            builder
                .file_space_page_size(plist.get_file_space_page_size()?)
                .file_space_strategy(plist.get_file_space_strategy()?);
        }
        Ok(builder)
    }

    pub fn userblock(&mut self, value: u64) -> &mut Self {
        self.userblock = Some(value);
        self
    }

    pub fn sym_k(&mut self, value: SymbolTableInfo) -> &mut Self {
        self.sym_k = Some(value);
        self
    }

    pub fn istore_k(&mut self, value: u32) -> &mut Self {
        self.istore_k = Some(value);
        self
    }

    pub fn shared_mesg_phase_change(&mut self, value: PhaseChangeInfo) -> &mut Self {
        self.shared_mesg_phase_change = Some(value);
        self
    }

    pub fn shared_mesg_indexes<S>(&mut self, value: S) -> &mut Self
    where
        S: Into<Vec<SharedMessageIndex>>,
    {
        self.shared_mesg_indexes = Some(value.into());
        self
    }

    #[cfg(hdf5_1_10_1)]
    pub fn file_space_page_size(&mut self, value: u64) -> &mut Self {
        self.file_space_page_size = Some(value);
        self
    }

    #[cfg(hdf5_1_10_1)]
    pub fn file_space_strategy(&mut self, value: FileSpaceStrategy) -> &mut Self {
        self.file_space_strategy = Some(value);
        self
    }

    fn populate_plist(&self, id: hid_t) -> Result<()> {
        if let Some(v) = self.userblock {
            h5try!(H5Pset_userblock(id, v as _));
        }
        if let Some(v) = self.sym_k {
            h5try!(H5Pset_sym_k(id, v.tree_rank as _, v.node_size as _));
        }
        if let Some(v) = self.istore_k {
            h5try!(H5Pset_istore_k(id, v as _));
        }
        if let Some(v) = self.shared_mesg_phase_change {
            h5try!(H5Pset_shared_mesg_phase_change(id, v.max_list as _, v.min_btree as _));
        }
        if let Some(ref v) = self.shared_mesg_indexes {
            h5try!(H5Pset_shared_mesg_nindexes(id, v.len() as _));
            for (i, v) in v.iter().enumerate() {
                h5try!(H5Pset_shared_mesg_index(
                    id,
                    i as _,
                    v.message_types.bits() as _,
                    v.min_message_size as _,
                ));
            }
        }
        #[cfg(hdf5_1_10_1)]
        {
            if let Some(v) = self.file_space_page_size {
                h5try!(H5Pset_file_space_page_size(id, v as _));
            }
            if let Some(v) = self.file_space_strategy {
                let (strategy, persist, threshold) = match v {
                    FileSpaceStrategy::FreeSpaceManager { paged, persist, threshold } => {
                        let strategy = if paged {
                            H5F_fspace_strategy_t::H5F_FSPACE_STRATEGY_PAGE
                        } else {
                            H5F_fspace_strategy_t::H5F_FSPACE_STRATEGY_FSM_AGGR
                        };
                        (strategy, persist as _, threshold as _)
                    }
                    FileSpaceStrategy::PageAggregation => {
                        (H5F_fspace_strategy_t::H5F_FSPACE_STRATEGY_AGGR, 0, 0)
                    }
                    _ => (H5F_fspace_strategy_t::H5F_FSPACE_STRATEGY_NONE, 0, 0),
                };
                h5try!(H5Pset_file_space_strategy(id, strategy, persist, threshold));
            }
        }
        Ok(())
    }

    pub fn finish(&self) -> Result<FileCreate> {
        h5lock!({
            let plist = FileCreate::try_new()?;
            self.populate_plist(plist.id())?;
            Ok(plist)
        })
    }
}

/// File creation property list.
impl FileCreate {
    pub fn try_new() -> Result<Self> {
        Self::from_id(h5try!(H5Pcreate(*H5P_FILE_CREATE)))
    }

    pub fn build() -> FileCreateBuilder {
        FileCreateBuilder::new()
    }

    #[doc(hidden)]
    pub fn get_userblock(&self) -> Result<u64> {
        h5get!(H5Pget_userblock(self.id()): hsize_t).map(|x| x as _)
    }

    #[doc(hidden)]
    pub fn get_sizes(&self) -> Result<SizeofInfo> {
        h5get!(H5Pget_sizes(self.id()): size_t, size_t).map(|(sizeof_addr, sizeof_size)| {
            SizeofInfo { sizeof_addr: sizeof_addr as _, sizeof_size: sizeof_size as _ }
        })
    }

    #[doc(hidden)]
    pub fn get_sym_k(&self) -> Result<SymbolTableInfo> {
        h5get!(H5Pget_sym_k(self.id()): c_uint, c_uint).map(|(tree_rank, node_size)| {
            SymbolTableInfo { tree_rank: tree_rank as _, node_size: node_size as _ }
        })
    }

    #[doc(hidden)]
    pub fn get_istore_k(&self) -> Result<u32> {
        h5get!(H5Pget_istore_k(self.id()): c_uint).map(|x| x as _)
    }

    #[doc(hidden)]
    pub fn get_version(&self) -> Result<VersionInfo> {
        // expected to fail if not attached to a file, that's ok
        let _e = silence_errors();

        #[cfg(not(hdf5_1_10_0))]
        {
            h5get!(H5Pget_version(self.id()): c_uint, c_uint, c_uint, c_uint).map(
                |(super_, free, _, sohm)| VersionInfo {
                    superblock: super_ as _,
                    freelist: free as _,
                    shared_header: sohm as _,
                },
            )
        }
        #[cfg(hdf5_1_10_0)]
        {
            h5get!(H5Fget_info2(self.id()): H5F_info2_t).map(|info| VersionInfo {
                superblock: info.super_.version as _,
                freelist: info.free.version as _,
                shared_header: info.sohm.version as _,
            })
        }
    }

    #[doc(hidden)]
    pub fn get_shared_mesg_phase_change(&self) -> Result<PhaseChangeInfo> {
        h5get!(H5Pget_shared_mesg_phase_change(self.id()): c_uint, c_uint).map(
            |(max_list, min_btree)| PhaseChangeInfo {
                max_list: max_list as _,
                min_btree: min_btree as _,
            },
        )
    }

    #[doc(hidden)]
    pub fn get_shared_mesg_indexes(&self) -> Result<Vec<SharedMessageIndex>> {
        let n = h5get_d!(H5Pget_shared_mesg_nindexes(self.id()): c_uint);
        let mut indexes = Vec::with_capacity(n as _);
        for i in 0..n {
            let (mut flags, mut min_size): (c_uint, c_uint) = (0, 0);
            h5try!(H5Pget_shared_mesg_index(self.id(), i, &mut flags, &mut min_size));
            indexes.push(SharedMessageIndex {
                message_types: SharedMessageType::from_bits_truncate(flags as _),
                min_message_size: min_size as _,
            });
        }
        Ok(indexes)
    }

    #[doc(hidden)]
    #[cfg(hdf5_1_10_1)]
    pub fn get_file_space_page_size(&self) -> Result<u64> {
        h5get!(H5Pget_file_space_page_size(self.id()): hsize_t).map(|x| x as _)
    }

    #[doc(hidden)]
    #[cfg(hdf5_1_10_1)]
    pub fn get_file_space_strategy(&self) -> Result<FileSpaceStrategy> {
        use self::H5F_fspace_strategy_t::*;
        let (strategy, persist, threshold) =
            h5get!(H5Pget_file_space_strategy(self.id()): H5F_fspace_strategy_t, hbool_t, hsize_t)?;
        Ok(match strategy {
            H5F_FSPACE_STRATEGY_FSM_AGGR => FileSpaceStrategy::FreeSpaceManager {
                paged: false,
                persist: persist != 0,
                threshold: threshold as _,
            },
            H5F_FSPACE_STRATEGY_PAGE => FileSpaceStrategy::FreeSpaceManager {
                paged: true,
                persist: persist != 0,
                threshold: threshold as _,
            },
            H5F_FSPACE_STRATEGY_AGGR => FileSpaceStrategy::PageAggregation,
            _ => FileSpaceStrategy::None,
        })
    }

    /// Retrieves the size of a user block.
    pub fn userblock(&self) -> u64 {
        self.get_userblock().unwrap_or_else(|_| Default::default())
    }

    /// Retrieves the size of the offsets and lengths used in the file.
    pub fn sizes(&self) -> SizeofInfo {
        self.get_sizes().unwrap_or_else(|_| Default::default())
    }

    /// Retrieves the size of the symbol table B-tree 1/2 rank and the symbol
    /// table leaf node 1/2 size.
    pub fn sym_k(&self) -> SymbolTableInfo {
        self.get_sym_k().unwrap_or_else(|_| Default::default())
    }

    /// Queries the 1/2 rank of an indexed storage B-tree.
    pub fn istore_k(&self) -> u32 {
        self.get_istore_k().unwrap_or_else(|_| Default::default())
    }

    /// Retrieves the version information of various objects in the file.
    pub fn version(&self) -> VersionInfo {
        self.get_version().unwrap_or_else(|_| Default::default())
    }

    /// Retrieves shared object header message phase change information.
    pub fn shared_mesg_phase_change(&self) -> PhaseChangeInfo {
        self.get_shared_mesg_phase_change().unwrap_or_else(|_| Default::default())
    }

    /// Retrieves configuration settings for shared message indexes.
    pub fn shared_mesg_indexes(&self) -> Vec<SharedMessageIndex> {
        self.get_shared_mesg_indexes().unwrap_or_else(|_| Default::default())
    }

    /// Retrieves the file space page size.
    #[cfg(hdf5_1_10_1)]
    pub fn file_space_page_size(&self) -> u64 {
        self.get_file_space_page_size().unwrap_or_else(|_| Default::default())
    }

    /// Retrieves the file space handling strategy.
    #[cfg(hdf5_1_10_1)]
    pub fn file_space_strategy(&self) -> FileSpaceStrategy {
        self.get_file_space_strategy().unwrap_or_else(|_| Default::default())
    }
}
