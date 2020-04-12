//! File creation properties.

use std::fmt::{self, Debug};
use std::ops::Deref;

use bitflags::bitflags;

#[cfg(hdf5_1_10_1)]
use hdf5_sys::h5f::H5F_fspace_strategy_t;
use hdf5_sys::h5o::{
    H5O_SHMESG_ALL_FLAG, H5O_SHMESG_ATTR_FLAG, H5O_SHMESG_DTYPE_FLAG, H5O_SHMESG_FILL_FLAG,
    H5O_SHMESG_NONE_FLAG, H5O_SHMESG_PLINE_FLAG, H5O_SHMESG_SDSPACE_FLAG,
};
use hdf5_sys::h5p::{
    H5Pcreate, H5Pget_istore_k, H5Pget_shared_mesg_index, H5Pget_shared_mesg_nindexes,
    H5Pget_shared_mesg_phase_change, H5Pget_sizes, H5Pget_sym_k, H5Pget_userblock, H5Pset_istore_k,
    H5Pset_shared_mesg_index, H5Pset_shared_mesg_nindexes, H5Pset_shared_mesg_phase_change,
    H5Pset_sym_k, H5Pset_userblock,
};
#[cfg(hdf5_1_10_1)]
use hdf5_sys::h5p::{
    H5Pget_file_space_page_size, H5Pget_file_space_strategy, H5Pset_file_space_page_size,
    H5Pset_file_space_strategy,
};

use crate::globals::H5P_FILE_CREATE;
use crate::internal_prelude::*;

/// File creation properties.
#[repr(transparent)]
#[derive(Clone)]
pub struct FileCreate(Handle);

impl ObjectClass for FileCreate {
    const NAME: &'static str = "file create property list";
    const VALID_TYPES: &'static [H5I_type_t] = &[H5I_GENPROP_LST];

    fn from_handle(handle: Handle) -> Self {
        Self(handle)
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
    fn eq(&self, other: &Self) -> bool {
        <PropertyList as PartialEq>::eq(self, other)
    }
}

impl Eq for FileCreate {}

/// Size of the offsets and lengths used in a file.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SizeofInfo {
    /// Offset size in bytes.
    pub sizeof_addr: usize,
    /// Length size in bytes.
    pub sizeof_size: usize,
}

/// Size of parameters used to control the symbol table nodes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SymbolTableInfo {
    /// Symbol table tree rank.
    ///
    /// `tree_rank` is one half the rank of a B-tree that stores a symbol table
    /// for a group. Internal nodes of the symbol table are on average 75% full.
    /// That is, the average rank of the tree is 1.5 times the value of ik. The
    /// HDF5library uses `tree_rank * 2` as the maximum number of entries before
    /// splitting a B-tree node. Since only 2 bytes are used in storing the
    /// number of entries for a B-tree node in an HDF5 file, `tree_rank * 2`
    /// cannot exceed 65536.
    ///
    /// The default value for `tree_rank` is 16.
    pub tree_rank: u32,
    /// Symbol table node size.
    ///
    /// `node_size` is one half of the number of symbols that can be stored in a
    /// symbol table node. A symbol table node is the leaf of a symbol table tree
    /// which is used to store a group. When symbols are inserted randomly into a
    /// group, the group's symbol table nodes are 75% full on average. That is,
    /// they contain 1.5 times the number of symbols specified by `node_size`.
    ///
    /// The default value for `node_size` is 4.
    pub node_size: u32,
}

/// Threshold values for storage of shared message indexes.
///
/// These phase change thresholds determine the point at which the index
/// storage mechanism changes from a more compact list format to a more
/// performance-oriented B-tree format, and vice-versa.
///
/// By default, a shared object header message index is initially stored as a
/// compact list. When the number of messages in an index exceeds the threshold
/// value of `max_list`, storage switches to a B-tree for improved performance.
/// If the number of messages subsequently falls below the `min_btree` threshold,
/// the index will revert to the list format.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PhaseChangeInfo {
    /// Threshold above which storage of a shared object header message index
    /// shifts from list to B-tree.
    ///
    /// If `max_list` is set to 0 (zero), shared object header message indexes
    /// in the file will be created as B-trees and will never revert to lists.
    pub max_list: u32,
    /// Threshold below which storage of a shared object header message index
    /// reverts to list format.
    pub min_btree: u32,
}

bitflags! {
    /// Types of messages that can be stored in a shared message index.
    pub struct SharedMessageType: u32 {
        /// No shared messages.
        const NONE = H5O_SHMESG_NONE_FLAG;
        /// Simple dataspace message.
        const SIMPLE_DATASPACE = H5O_SHMESG_SDSPACE_FLAG;
        /// Datatype message.
        const DATATYPE = H5O_SHMESG_DTYPE_FLAG;
        /// Fill value message.
        const FILL_VALUE = H5O_SHMESG_FILL_FLAG;
        /// Filter pipeline message.
        const FILTER_PIPELINE = H5O_SHMESG_PLINE_FLAG;
        /// Attribute message.
        const ATTRIBUTE = H5O_SHMESG_ATTR_FLAG;
        /// All message types.
        const ALL = H5O_SHMESG_ALL_FLAG;
    }
}

impl Default for SharedMessageType {
    fn default() -> Self {
        Self::NONE
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
    /// Mechanisms used: free-space managers, aggregators or embedded paged
    /// aggregation and the virtual file driver.
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
        Self::FreeSpaceManager { paged: false, persist: false, threshold: 1 }
    }
}

/// Builder used to create file creation property list.
#[derive(Clone, Debug, Default)]
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

impl FileCreateBuilder {
    /// Creates a new file creation property list builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new builder from an existing property list.
    pub fn from_plist(plist: &FileCreate) -> Result<Self> {
        let mut builder = Self::default();
        builder.userblock(plist.get_userblock()?);
        let v = plist.get_sym_k()?;
        builder.sym_k(v.tree_rank, v.node_size);
        builder.istore_k(plist.get_istore_k()?);
        let v = plist.get_shared_mesg_phase_change()?;
        builder.shared_mesg_phase_change(v.max_list, v.min_btree);
        builder.shared_mesg_indexes(&plist.get_shared_mesg_indexes()?);
        #[cfg(hdf5_1_10_1)]
        {
            builder.file_space_page_size(plist.get_file_space_page_size()?);
            builder.file_space_strategy(plist.get_file_space_strategy()?);
        }
        Ok(builder)
    }

    /// Sets user block size.
    ///
    /// Sets the user block size of a file creation property list. The default
    /// user block size is 0; it may be set to any power of 2 equal to 512 or
    /// greater (512, 1024, 2048, etc.).
    pub fn userblock(&mut self, size: u64) -> &mut Self {
        self.userblock = Some(size);
        self
    }

    /// Sets the size of parameters used to control the symbol table nodes.
    ///
    /// Passing in a value of zero (0) for one of the parameters (`tree_rank` or
    /// `node_size`) retains the current value.
    ///
    /// For further details, see [`SymbolTableInfo`](struct.SymbolTableInfo.html).
    pub fn sym_k(&mut self, tree_rank: u32, node_size: u32) -> &mut Self {
        self.sym_k = Some(SymbolTableInfo { tree_rank, node_size });
        self
    }

    /// Sets the size of the parameter used to control the B-trees for indexing
    /// chunked datasets.
    ///
    /// `istore_k` is one half the rank of a tree that stores chunked raw data.
    /// On average, such a tree will be 75% full, or have an average rank of 1.5
    /// times the value of `istore_k`.
    ///
    /// The HDF5 library uses `istore_k * 2` as the maximum number of entries
    /// before splitting a B-tree node. Since only 2 bytes are used in storing the
    /// number of entries for a B-tree node in an HDF5 file, `istore_k * 2`
    /// cannot exceed 65536.
    ///
    /// The default value for `istore_k` is 32.
    pub fn istore_k(&mut self, ik: u32) -> &mut Self {
        self.istore_k = Some(ik);
        self
    }

    /// Sets shared object header message storage phase change thresholds.
    ///
    /// For further details, see [`PhaseChangeInfo`](struct.PhaseChangeInfo.html).
    pub fn shared_mesg_phase_change(&mut self, max_list: u32, min_btree: u32) -> &mut Self {
        self.shared_mesg_phase_change = Some(PhaseChangeInfo { max_list, min_btree });
        self
    }

    /// Configures shared object header message indexes.
    ///
    /// For each specified index, sets the types of messages that may be stored
    /// and the minimum size of each message
    pub fn shared_mesg_indexes(&mut self, indexes: &[SharedMessageIndex]) -> &mut Self {
        self.shared_mesg_indexes = Some(indexes.into());
        self
    }

    #[cfg(hdf5_1_10_1)]
    /// Sets the file space page size.
    ///
    /// The minimum size is 512. Setting a value less than 512 will result in
    /// an error. The library default size for the file space page size when
    /// not set is 4096.
    pub fn file_space_page_size(&mut self, fsp_size: u64) -> &mut Self {
        self.file_space_page_size = Some(fsp_size);
        self
    }

    #[cfg(hdf5_1_10_1)]
    /// Sets the file space handling strategy and persisting free-space values.
    ///
    /// This setting cannot be changed for the life of the file.
    ///
    /// For further details, see [`FileSpaceStrategy`](enum.FileSpaceStrategy.html).
    pub fn file_space_strategy(&mut self, strategy: FileSpaceStrategy) -> &mut Self {
        self.file_space_strategy = Some(strategy);
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

    pub fn copy(&self) -> Self {
        unsafe { self.deref().copy().cast() }
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
        let (strategy, persist, threshold) =
            h5get!(H5Pget_file_space_strategy(self.id()): H5F_fspace_strategy_t, hbool_t, hsize_t)?;
        Ok(match strategy {
            H5F_fspace_strategy_t::H5F_FSPACE_STRATEGY_FSM_AGGR => {
                FileSpaceStrategy::FreeSpaceManager {
                    paged: false,
                    persist: persist != 0,
                    threshold: threshold as _,
                }
            }
            H5F_fspace_strategy_t::H5F_FSPACE_STRATEGY_PAGE => {
                FileSpaceStrategy::FreeSpaceManager {
                    paged: true,
                    persist: persist != 0,
                    threshold: threshold as _,
                }
            }
            H5F_fspace_strategy_t::H5F_FSPACE_STRATEGY_AGGR => FileSpaceStrategy::PageAggregation,
            _ => FileSpaceStrategy::None,
        })
    }

    /// Retrieves the size of a user block.
    pub fn userblock(&self) -> u64 {
        self.get_userblock().unwrap_or(0)
    }

    /// Retrieves the size of the offsets and lengths used in the file.
    pub fn sizes(&self) -> SizeofInfo {
        self.get_sizes().unwrap_or_else(|_| SizeofInfo::default())
    }

    /// Retrieves the size of the symbol table B-tree 1/2 rank and the symbol
    /// table leaf node 1/2 size.
    pub fn sym_k(&self) -> SymbolTableInfo {
        self.get_sym_k().unwrap_or_else(|_| SymbolTableInfo::default())
    }

    /// Queries the 1/2 rank of an indexed storage B-tree.
    pub fn istore_k(&self) -> u32 {
        self.get_istore_k().unwrap_or(0)
    }

    /// Retrieves shared object header message phase change information.
    pub fn shared_mesg_phase_change(&self) -> PhaseChangeInfo {
        self.get_shared_mesg_phase_change().unwrap_or_else(|_| PhaseChangeInfo::default())
    }

    /// Retrieves configuration settings for shared message indexes.
    pub fn shared_mesg_indexes(&self) -> Vec<SharedMessageIndex> {
        self.get_shared_mesg_indexes().unwrap_or_else(|_| Vec::new())
    }

    /// Retrieves the file space page size.
    #[cfg(hdf5_1_10_1)]
    pub fn file_space_page_size(&self) -> u64 {
        self.get_file_space_page_size().unwrap_or(0)
    }

    /// Retrieves the file space handling strategy.
    #[cfg(hdf5_1_10_1)]
    pub fn file_space_strategy(&self) -> FileSpaceStrategy {
        self.get_file_space_strategy().unwrap_or_else(|_| FileSpaceStrategy::default())
    }
}
