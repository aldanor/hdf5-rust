use std::fmt::{self, Debug};
use std::mem;
use std::ops::Deref;

use num_integer::div_floor;

#[cfg(hdf5_1_10_5)]
use hdf5_sys::h5d::{H5Dget_chunk_info, H5Dget_num_chunks};
use hdf5_sys::{
    h5::HADDR_UNDEF,
    h5d::{
        H5D_fill_value_t, H5D_layout_t, H5Dcreate2, H5Dcreate_anon, H5Dget_create_plist,
        H5Dget_offset, H5Dset_extent, H5D_FILL_TIME_ALLOC,
    },
    h5p::{
        H5Pcreate, H5Pfill_value_defined, H5Pget_chunk, H5Pget_fill_value, H5Pget_layout,
        H5Pget_obj_track_times, H5Pset_chunk, H5Pset_create_intermediate_group, H5Pset_fill_time,
        H5Pset_fill_value, H5Pset_obj_track_times,
    },
};

use crate::globals::H5P_LINK_CREATE;
use crate::internal_prelude::*;

/// Represents the HDF5 dataset object.
#[repr(transparent)]
#[derive(Clone)]
pub struct Dataset(Handle);

impl ObjectClass for Dataset {
    const NAME: &'static str = "dataset";
    const VALID_TYPES: &'static [H5I_type_t] = &[H5I_DATASET];

    fn from_handle(handle: Handle) -> Self {
        Self(handle)
    }

    fn handle(&self) -> &Handle {
        &self.0
    }

    // TODO: short_repr()
}

impl Debug for Dataset {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.debug_fmt(f)
    }
}

impl Deref for Dataset {
    type Target = Container;

    fn deref(&self) -> &Container {
        unsafe { self.transmute() }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Chunk {
    None,
    Auto,
    Infer,
    Manual(Vec<Ix>),
}

#[cfg(hdf5_1_10_5)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChunkInfo {
    /// Array with a size equal to the dataset’s rank whose elements contain 0-based
    /// logical positions of the chunk’s first element in each dimension.
    pub offset: Vec<u64>,
    /// Filter mask that indicates which filters were used with the chunk when written.
    /// A zero value indicates that all enabled filters are applied on the chunk.
    /// A filter is skipped if the bit corresponding to the filter’s position in
    /// the pipeline (0 ≤ position < 32) is turned on.
    pub filter_mask: u32,
    /// Chunk address in the file.
    pub addr: u64,
    /// Chunk size in bytes.
    pub size: u64,
}

#[cfg(hdf5_1_10_5)]
impl ChunkInfo {
    pub(crate) fn new(ndim: usize) -> Self {
        let mut offset = Vec::with_capacity(ndim);
        unsafe { offset.set_len(ndim) };
        Self { offset, filter_mask: 0, addr: 0, size: 0 }
    }
}

impl Dataset {
    /// Returns whether this dataset is resizable along some axis.
    pub fn is_resizable(&self) -> bool {
        h5lock!(self.space().ok().map_or(false, |s| s.is_resizable()))
    }

    /// Returns whether this dataset has a chunked layout.
    pub fn is_chunked(&self) -> bool {
        h5lock!({
            self.dcpl_id()
                .ok()
                .map_or(false, |dcpl_id| H5Pget_layout(dcpl_id) == H5D_layout_t::H5D_CHUNKED)
        })
    }

    #[cfg(hdf5_1_10_5)]
    /// Returns number of chunks if the dataset is chunked.
    pub fn num_chunks(&self) -> Option<usize> {
        if !self.is_chunked() {
            return None;
        }
        h5lock!(self.space().map_or(None, |s| {
            let mut n: hsize_t = 0;
            h5check(H5Dget_num_chunks(self.id(), s.id(), &mut n)).map(|_| n as _).ok()
        }))
    }

    #[cfg(hdf5_1_10_5)]
    /// Retrieves the chunk information for the chunk specified by its index.
    pub fn chunk_info(&self, index: usize) -> Option<ChunkInfo> {
        if !self.is_chunked() {
            return None;
        }
        h5lock!(self.space().map_or(None, |s| {
            let mut chunk_info = ChunkInfo::new(self.ndim());
            h5check(H5Dget_chunk_info(
                self.id(),
                s.id(),
                index as _,
                chunk_info.offset.as_mut_ptr(),
                &mut chunk_info.filter_mask,
                &mut chunk_info.addr,
                &mut chunk_info.size,
            ))
            .map(|_| chunk_info)
            .ok()
        }))
    }

    /// Returns the chunk shape if the dataset is chunked.
    pub fn chunks(&self) -> Option<Vec<Ix>> {
        h5lock!({
            self.dcpl_id().ok().and_then(|dcpl_id| {
                if self.is_chunked() {
                    Some({
                        let ndim = self.ndim();
                        let mut dims: Vec<hsize_t> = Vec::with_capacity(ndim);
                        dims.set_len(ndim);
                        H5Pget_chunk(dcpl_id, ndim as _, dims.as_mut_ptr());
                        dims.iter().map(|&x| x as _).collect()
                    })
                } else {
                    None
                }
            })
        })
    }

    /// Returns the filters used to create the dataset.
    pub fn filters(&self) -> Filters {
        h5lock!({
            let dcpl = PropertyList::from_id(H5Dget_create_plist(self.id()))?;
            Ok(Filters::from_dcpl(&dcpl)?)
        })
        .unwrap_or_else(|_: crate::error::Error| Filters::default())
    }

    /// Returns `true` if object modification time is tracked by the dataset.
    pub fn tracks_times(&self) -> bool {
        h5lock!({
            self.dcpl_id().ok().map_or(false, |dcpl_id| {
                let mut track_times: hbool_t = 0;
                h5lock!(H5Pget_obj_track_times(dcpl_id, &mut track_times as *mut _));
                track_times > 0
            })
        })
    }

    /// Returns the absolute byte offset of the dataset in the file if such offset is defined
    /// (which is not the case for datasets that are chunked, compact or not allocated yet).
    pub fn offset(&self) -> Option<u64> {
        let offset: haddr_t = h5lock!(H5Dget_offset(self.id()));
        if offset == HADDR_UNDEF {
            None
        } else {
            Some(offset as _)
        }
    }

    /// Returns default fill value for the dataset if such value is set. Note that conversion
    /// to the requested type is done by HDF5 which may result in loss of precision for
    /// floating-point values if the datatype differs from the datatype of of the dataset.
    pub fn fill_value<T: H5Type>(&self) -> Result<Option<T>> {
        h5lock!({
            let defined: *mut H5D_fill_value_t = &mut H5D_fill_value_t::H5D_FILL_VALUE_UNDEFINED;
            let dcpl_id = self.dcpl_id()?;
            h5try!(H5Pfill_value_defined(dcpl_id, defined));
            match *defined {
                H5D_fill_value_t::H5D_FILL_VALUE_ERROR => fail!("Invalid fill value"),
                H5D_fill_value_t::H5D_FILL_VALUE_UNDEFINED => Ok(None),
                _ => {
                    let datatype = Datatype::from_type::<T>()?;
                    let mut value = mem::MaybeUninit::<T>::uninit();
                    h5try!(
                        H5Pget_fill_value(dcpl_id, datatype.id(), value.as_mut_ptr() as *mut _,)
                    );
                    Ok(Some(value.assume_init()))
                }
            }
        })
    }

    fn dcpl_id(&self) -> Result<hid_t> {
        h5call!(H5Dget_create_plist(self.id()))
    }

    pub fn resize<D: Dimension>(&self, d: D) -> Result<()> {
        let mut dims: Vec<hsize_t> = vec![];
        for dim in &d.dims() {
            dims.push(*dim as _);
        }
        h5try!(H5Dset_extent(self.id(), dims.as_ptr()));
        Ok(())
    }
}

#[derive(Clone)]
pub struct DatasetBuilder<T> {
    packed: bool,
    filters: Filters,
    chunk: Chunk,
    parent: Result<Handle>,
    track_times: bool,
    resizable: bool,
    fill_value: Option<T>,
}

impl<T: H5Type> DatasetBuilder<T> {
    /// Create a new dataset builder and bind it to the parent container.
    pub fn new(parent: &Group) -> Self {
        h5lock!({
            // Store the reference to the parent handle and try to increase its reference count.
            let handle = Handle::try_new(parent.id());
            if let Ok(ref handle) = handle {
                handle.incref();
            }

            Self {
                packed: false,
                filters: Filters::default(),
                chunk: Chunk::Auto,
                parent: handle,
                track_times: false,
                resizable: false,
                fill_value: None,
            }
        })
    }

    pub fn packed(&mut self, packed: bool) -> &mut Self {
        self.packed = packed;
        self
    }

    pub fn fill_value(&mut self, fill_value: T) -> &mut Self {
        self.fill_value = Some(fill_value);
        self
    }

    /// Disable chunking.
    pub fn no_chunk(&mut self) -> &mut Self {
        self.chunk = Chunk::None;
        self
    }

    /// Enable automatic chunking only if chunking is required (default option).
    pub fn chunk_auto(&mut self) -> &mut Self {
        self.chunk = Chunk::Auto;
        self
    }

    /// Enable chunking with automatic chunk shape.
    pub fn chunk_infer(&mut self) -> &mut Self {
        self.chunk = Chunk::Infer;
        self
    }

    /// Set chunk shape manually.
    pub fn chunk<D: Dimension>(&mut self, chunk: D) -> &mut Self {
        self.chunk = Chunk::Manual(chunk.dims());
        self
    }

    /// Set the filters.
    pub fn filters(&mut self, filters: &Filters) -> &mut Self {
        self.filters = filters.clone();
        self
    }

    /// Enable or disable tracking object modification time (disabled by default).
    pub fn track_times(&mut self, track_times: bool) -> &mut Self {
        self.track_times = track_times;
        self
    }

    /// Make the dataset resizable along all axes (requires chunking).
    pub fn resizable(&mut self, resizable: bool) -> &mut Self {
        self.resizable = resizable;
        self
    }

    /// Enable gzip compression with a specified level (0-9).
    pub fn gzip(&mut self, level: u8) -> &mut Self {
        self.filters.gzip(level);
        self
    }

    /// Enable szip compression with a specified method (EC, NN) and level (0-32).
    ///
    /// If `nn` if set to `true` (default), the nearest neighbor method is used, otherwise
    /// the method is set to entropy coding.
    pub fn szip(&mut self, nn: bool, level: u8) -> &mut Self {
        self.filters.szip(nn, level);
        self
    }

    /// Enable or disable shuffle filter.
    pub fn shuffle(&mut self, shuffle: bool) -> &mut Self {
        self.filters.shuffle(shuffle);
        self
    }

    /// Enable or disable fletcher32 filter.
    pub fn fletcher32(&mut self, fletcher32: bool) -> &mut Self {
        self.filters.fletcher32(fletcher32);
        self
    }

    /// Enable scale-offset filter with a specified factor (0 means automatic).
    pub fn scale_offset(&mut self, scale_offset: u32) -> &mut Self {
        self.filters.scale_offset(scale_offset);
        self
    }

    fn make_dcpl(&self, datatype: &Datatype, extents: &Extents) -> Result<PropertyList> {
        let ndim = extents.ndim();
        let resizable = extents.is_resizable();
        let shape = extents.dims();
        h5lock!({
            let dcpl = self.filters.to_dcpl(datatype)?;
            let id = dcpl.id();

            h5try!(H5Pset_obj_track_times(id, self.track_times as _));

            if let Some(ref fill_value) = self.fill_value {
                h5try!(H5Pset_fill_value(id, datatype.id(), fill_value as *const _ as *const _));
            }

            if let Chunk::None = self.chunk {
                ensure!(
                    !self.filters.has_filters(),
                    "Chunking must be enabled when filters are present"
                );
                ensure!(!resizable, "Chunking must be enabled for resizable datasets");
            } else {
                let no_chunk = if let Chunk::Auto = self.chunk {
                    !self.filters.has_filters() && !self.resizable
                } else {
                    false
                };
                if !no_chunk {
                    ensure!(ndim > 0, "Chunking cannot be enabled for 0-dim datasets");

                    let dims = match self.chunk {
                        Chunk::Manual(ref c) => c.clone(),
                        _ => infer_chunk_size(&shape, datatype.size()),
                    };

                    ensure!(
                        dims.ndim() == ndim,
                        "Invalid chunk ndim: expected {}, got {}",
                        ndim,
                        dims.ndim()
                    );
                    ensure!(
                        dims.size() > 0,
                        "Invalid chunk: {:?} (all dimensions must be positive)",
                        dims
                    );

                    if !resizable {
                        ensure!(
                            dims.iter().zip(shape.iter()).all(|(&c, &s)| c <= s),
                            "Invalid chunk: {:?} (must not exceed data shape in any dimension)",
                            dims
                        );
                    }

                    let c_dims: Vec<hsize_t> = dims.iter().map(|&x| x as _).collect();
                    h5try!(H5Pset_chunk(id, dims.ndim() as _, c_dims.as_ptr()));

                    // For chunked datasets, write fill values at the allocation time.
                    h5try!(H5Pset_fill_time(id, H5D_FILL_TIME_ALLOC));
                }
            }

            Ok(dcpl)
        })
    }

    fn make_lcpl() -> Result<PropertyList> {
        h5lock!({
            let lcpl = PropertyList::from_id(h5try!(H5Pcreate(*H5P_LINK_CREATE)))?;
            h5call!(H5Pset_create_intermediate_group(lcpl.id(), 1)).and(Ok(lcpl))
        })
    }

    fn finalize<S: Into<Extents>>(&self, name: Option<&str>, extents: S) -> Result<Dataset> {
        let type_descriptor = if self.packed {
            <T as H5Type>::type_descriptor().to_packed_repr()
        } else {
            <T as H5Type>::type_descriptor().to_c_repr()
        };
        let mut extents = extents.into();
        if self.resizable {
            extents = extents.resizable();
        }
        h5lock!({
            let datatype = Datatype::from_descriptor(&type_descriptor)?;
            let parent = try_ref_clone!(self.parent);

            let dcpl = self.make_dcpl(&datatype, &extents)?;
            let dataspace = Dataspace::try_new(extents)?;

            if let Some(name) = name {
                let lcpl = Self::make_lcpl()?;
                let name = to_cstring(name)?;
                Dataset::from_id(h5try!(H5Dcreate2(
                    parent.id(),
                    name.as_ptr(),
                    datatype.id(),
                    dataspace.id(),
                    lcpl.id(),
                    dcpl.id(),
                    H5P_DEFAULT
                )))
            } else {
                Dataset::from_id(h5try!(H5Dcreate_anon(
                    parent.id(),
                    datatype.id(),
                    dataspace.id(),
                    dcpl.id(),
                    H5P_DEFAULT
                )))
            }
        })
    }

    /// Create the dataset and link it into the file structure.
    pub fn create<S: Into<Extents>>(&self, name: &str, shape: S) -> Result<Dataset> {
        self.finalize(Some(name), shape)
    }

    /// Create an anonymous dataset without linking it.
    pub fn create_anon<S: Into<Extents>>(&self, shape: S) -> Result<Dataset> {
        self.finalize(None, shape)
    }
}

fn infer_chunk_size(shape: &[Ix], typesize: usize) -> Vec<Ix> {
    // This algorithm is borrowed from h5py, though the idea originally comes from PyTables.

    const CHUNK_BASE: f64 = (16 * 1024) as _;
    const CHUNK_MIN: f64 = (8 * 1024) as _;
    const CHUNK_MAX: f64 = (1024 * 1024) as _;

    let ndim = shape.len();
    let size: usize = shape.iter().product();

    if ndim == 0 {
        return vec![];
    } else if size == 0 {
        return vec![1];
    }

    let mut chunks = shape.to_vec();
    let total = (typesize * size) as f64;
    let mut target: f64 = CHUNK_BASE * (total / (1024.0 * 1024.0)).log10().exp2();

    if target > CHUNK_MAX {
        target = CHUNK_MAX;
    } else if target < CHUNK_MIN {
        target = CHUNK_MIN;
    }

    // Loop over axes, dividing them by 2, stop when all of the following is true:
    // - chunk size is smaller than the target chunk size or is within 50% of target chunk size
    // - chunk size is smaller than the maximum chunk size
    for i in 0.. {
        let size: usize = chunks.iter().product();
        let bytes = (size * typesize) as f64;
        if (bytes < target * 1.5 && bytes < CHUNK_MAX) || size == 1 {
            break;
        }
        let axis = i % ndim;
        chunks[axis] = div_floor(chunks[axis] + 1, 2);
    }

    chunks
}

#[cfg(test)]
pub mod tests {
    use std::fs;
    use std::io::Read;

    use hdf5_sys::{h5d::H5Dwrite, h5s::H5S_ALL};

    use crate::filters::{gzip_available, szip_available};
    use crate::internal_prelude::*;

    use super::infer_chunk_size;

    #[test]
    pub fn test_infer_chunk_size() {
        assert_eq!(infer_chunk_size(&[], 1), vec![]);
        assert_eq!(infer_chunk_size(&[0], 1), vec![1]);
        assert_eq!(infer_chunk_size(&[1], 1), vec![1]);

        // generated regression tests vs h5py implementation
        assert_eq!(infer_chunk_size(&[65682868], 1), vec![64144]);
        assert_eq!(infer_chunk_size(&[56755037], 2), vec![27713]);
        assert_eq!(infer_chunk_size(&[56882283], 4), vec![27775]);
        assert_eq!(infer_chunk_size(&[21081789], 8), vec![10294]);
        assert_eq!(infer_chunk_size(&[5735, 6266], 1), vec![180, 392]);
        assert_eq!(infer_chunk_size(&[467, 4427], 2), vec![30, 554]);
        assert_eq!(infer_chunk_size(&[5579, 8323], 4), vec![88, 261]);
        assert_eq!(infer_chunk_size(&[1686, 770], 8), vec![106, 49]);
        assert_eq!(infer_chunk_size(&[344, 414, 294], 1), vec![22, 52, 37]);
        assert_eq!(infer_chunk_size(&[386, 192, 444], 2), vec![25, 24, 56]);
        assert_eq!(infer_chunk_size(&[277, 161, 460], 4), vec![18, 21, 58]);
        assert_eq!(infer_chunk_size(&[314, 22, 253], 8), vec![40, 3, 32]);
        assert_eq!(infer_chunk_size(&[89, 49, 91, 59], 1), vec![12, 13, 23, 15]);
        assert_eq!(infer_chunk_size(&[42, 92, 60, 80], 2), vec![6, 12, 15, 20]);
        assert_eq!(infer_chunk_size(&[15, 62, 62, 47], 4), vec![4, 16, 16, 12]);
        assert_eq!(infer_chunk_size(&[62, 51, 55, 64], 8), vec![8, 7, 7, 16]);
    }

    #[test]
    pub fn test_is_chunked() {
        with_tmp_file(|file| {
            assert_eq!(file.new_dataset::<u32>().create_anon(1).unwrap().is_chunked(), false);
            assert_eq!(
                file.new_dataset::<u32>().shuffle(true).create_anon(1).unwrap().is_chunked(),
                true
            );
        })
    }

    #[test]
    pub fn test_chunks() {
        with_tmp_file(|file| {
            assert_eq!(file.new_dataset::<u32>().create_anon(1).unwrap().chunks(), None);
            assert_eq!(file.new_dataset::<u32>().no_chunk().create_anon(1).unwrap().chunks(), None);
            assert_eq!(
                file.new_dataset::<u32>().chunk((1, 2)).create_anon((10, 20)).unwrap().chunks(),
                Some(vec![1, 2])
            );
            assert_eq!(
                file.new_dataset::<u32>().chunk_infer().create_anon((5579, 8323)).unwrap().chunks(),
                Some(vec![88, 261])
            );
            assert_eq!(
                file.new_dataset::<u32>().chunk_auto().create_anon((5579, 8323)).unwrap().chunks(),
                None
            );
            assert_eq!(
                file.new_dataset::<u32>()
                    .chunk_auto()
                    .shuffle(true)
                    .create_anon((5579, 8323))
                    .unwrap()
                    .chunks(),
                Some(vec![88, 261])
            );
        })
    }

    #[test]
    pub fn test_chunks_resizable_zero_size() {
        with_tmp_file(|file| {
            let ds = file
                .new_dataset::<u32>()
                .chunk((128,))
                .resizable(true)
                .create("chunked_empty", (0,))
                .unwrap();
            assert_eq!(ds.shape(), vec![0]);

            ds.resize((10,)).unwrap();
            assert_eq!(ds.shape(), vec![10]);

            ds.as_writer().write(&vec![3; 10]).unwrap();
        })
    }

    #[test]
    pub fn test_invalid_chunk() {
        with_tmp_file(|file| {
            let b = file.new_dataset::<u32>();
            assert_err!(
                b.clone().shuffle(true).no_chunk().create_anon(1),
                "Chunking must be enabled when filters are present"
            );
            assert_err!(
                b.clone().no_chunk().resizable(true).create_anon(1),
                "Chunking must be enabled for resizable datasets"
            );
            assert_err!(
                b.clone().chunk_infer().create_anon(()),
                "Chunking cannot be enabled for 0-dim datasets"
            );
            assert_err!(
                b.clone().chunk((1, 2)).create_anon(()),
                "Chunking cannot be enabled for 0-dim datasets"
            );
            assert_err!(
                b.clone().chunk((1, 2)).create_anon(1),
                "Invalid chunk ndim: expected 1, got 2"
            );
            assert_err!(
                b.clone().chunk((0, 2)).create_anon((1, 2)),
                "Invalid chunk: [0, 2] (all dimensions must be positive)"
            );
            assert_err!(
                b.clone().chunk((1, 3)).create_anon((1, 2)),
                "Invalid chunk: [1, 3] (must not exceed data shape in any dimension)"
            );
        })
    }

    #[test]
    pub fn test_shape_ndim_size() {
        with_tmp_file(|file| {
            let d = file.new_dataset::<f32>().create_anon((2, 3)).unwrap();
            assert_eq!(d.shape(), vec![2, 3]);
            assert_eq!(d.size(), 6);
            assert_eq!(d.ndim(), 2);
            assert_eq!(d.is_scalar(), false);

            let d = file.new_dataset::<u8>().create_anon(()).unwrap();
            assert_eq!(d.shape(), vec![]);
            assert_eq!(d.size(), 1);
            assert_eq!(d.ndim(), 0);
            assert_eq!(d.is_scalar(), true);
        })
    }

    #[test]
    pub fn test_filters() {
        with_tmp_file(|file| {
            assert_eq!(
                file.new_dataset::<u32>().create_anon(100).unwrap().filters(),
                Filters::default()
            );
            assert_eq!(
                file.new_dataset::<u32>()
                    .shuffle(true)
                    .create_anon(100)
                    .unwrap()
                    .filters()
                    .get_shuffle(),
                true
            );
            assert_eq!(
                file.new_dataset::<u32>()
                    .fletcher32(true)
                    .create_anon(100)
                    .unwrap()
                    .filters()
                    .get_fletcher32(),
                true
            );
            assert_eq!(
                file.new_dataset::<u32>()
                    .scale_offset(8)
                    .create_anon(100)
                    .unwrap()
                    .filters()
                    .get_scale_offset(),
                Some(8)
            );
            if gzip_available() {
                assert_eq!(
                    file.new_dataset::<u32>()
                        .gzip(7)
                        .create_anon(100)
                        .unwrap()
                        .filters()
                        .get_gzip(),
                    Some(7)
                );
            }
            if szip_available() {
                assert_eq!(
                    file.new_dataset::<u32>()
                        .szip(false, 4)
                        .create_anon(100)
                        .unwrap()
                        .filters()
                        .get_szip(),
                    Some((false, 4))
                );
            }
        });

        with_tmp_file(|file| {
            let filters = Filters::new().fletcher32(true).shuffle(true).clone();
            assert_eq!(
                file.new_dataset::<u32>().filters(&filters).create_anon(100).unwrap().filters(),
                filters
            );
        })
    }

    #[test]
    pub fn test_resizable() {
        with_tmp_file(|file| {
            assert_eq!(file.new_dataset::<u32>().create_anon(1).unwrap().is_resizable(), false);
            assert_eq!(
                file.new_dataset::<u32>().resizable(false).create_anon(1).unwrap().is_resizable(),
                false
            );
            assert_eq!(
                file.new_dataset::<u32>().resizable(true).create_anon(1).unwrap().is_resizable(),
                true
            );
        })
    }

    #[test]
    pub fn test_track_times() {
        with_tmp_file(|file| {
            assert_eq!(file.new_dataset::<u32>().create_anon(1).unwrap().tracks_times(), false);
            assert_eq!(
                file.new_dataset::<u32>().track_times(false).create_anon(1).unwrap().tracks_times(),
                false
            );
            assert_eq!(
                file.new_dataset::<u32>().track_times(true).create_anon(1).unwrap().tracks_times(),
                true
            );
        });

        with_tmp_path(|path| {
            let mut buf1: Vec<u8> = Vec::new();
            File::create(&path).unwrap().new_dataset::<u32>().create("foo", 1).unwrap();
            fs::File::open(&path).unwrap().read_to_end(&mut buf1).unwrap();

            let mut buf2: Vec<u8> = Vec::new();
            File::create(&path)
                .unwrap()
                .new_dataset::<u32>()
                .track_times(false)
                .create("foo", 1)
                .unwrap();
            fs::File::open(&path).unwrap().read_to_end(&mut buf2).unwrap();

            assert_eq!(buf1, buf2);

            let mut buf2: Vec<u8> = Vec::new();
            File::create(&path)
                .unwrap()
                .new_dataset::<u32>()
                .track_times(true)
                .create("foo", 1)
                .unwrap();
            fs::File::open(&path).unwrap().read_to_end(&mut buf2).unwrap();
            assert_ne!(buf1, buf2);
        });
    }

    #[test]
    pub fn test_storage_size_offset() {
        with_tmp_file(|file| {
            let ds = file.new_dataset::<u16>().create_anon(3).unwrap();
            assert_eq!(ds.storage_size(), 0);
            assert!(ds.offset().is_none());

            let buf: Vec<u16> = vec![1, 2, 3];
            h5call!(H5Dwrite(
                ds.id(),
                Datatype::from_type::<u16>().unwrap().id(),
                H5S_ALL,
                H5S_ALL,
                H5P_DEFAULT,
                buf.as_ptr() as *const _
            ))
            .unwrap();
            assert_eq!(ds.storage_size(), 6);
            assert!(ds.offset().is_some());
        })
    }

    #[test]
    pub fn test_datatype() {
        with_tmp_file(|file| {
            assert_eq!(
                file.new_dataset::<f32>().create_anon(1).unwrap().dtype().unwrap(),
                Datatype::from_type::<f32>().unwrap()
            );
        })
    }

    #[test]
    pub fn test_create_anon() {
        with_tmp_file(|file| {
            let ds = file.new_dataset::<u32>().create("foo/bar", (1, 2)).unwrap();
            assert!(ds.is_valid());
            assert_eq!(ds.shape(), vec![1, 2]);
            assert_eq!(ds.name(), "/foo/bar");
            assert_eq!(file.group("foo").unwrap().dataset("bar").unwrap().shape(), vec![1, 2]);

            let ds = file.new_dataset::<u32>().create_anon((2, 3)).unwrap();
            assert!(ds.is_valid());
            assert_eq!(ds.name(), "");
            assert_eq!(ds.shape(), vec![2, 3]);
        })
    }

    #[test]
    pub fn test_fill_value() {
        with_tmp_file(|file| {
            macro_rules! check_fill_value {
                ($ds:expr, $tp:ty, $v:expr) => {
                    assert_eq!(($ds).fill_value::<$tp>().unwrap(), Some(($v) as $tp));
                };
            }

            macro_rules! check_fill_value_approx {
                ($ds:expr, $tp:ty, $v:expr) => {{
                    let fill_value = ($ds).fill_value::<$tp>().unwrap().unwrap();
                    // FIXME: should inexact float->float casts be prohibited?
                    assert!((fill_value - (($v) as $tp)).abs() < (1.0e-6 as $tp));
                }};
            }

            macro_rules! check_all_fill_values {
                ($ds:expr, $v:expr) => {
                    check_fill_value!($ds, u8, $v);
                    check_fill_value!($ds, u16, $v);
                    check_fill_value!($ds, u32, $v);
                    check_fill_value!($ds, u64, $v);
                    check_fill_value!($ds, i8, $v);
                    check_fill_value!($ds, i16, $v);
                    check_fill_value!($ds, i32, $v);
                    check_fill_value!($ds, i64, $v);
                    check_fill_value!($ds, usize, $v);
                    check_fill_value!($ds, isize, $v);
                    check_fill_value_approx!($ds, f32, $v);
                    check_fill_value_approx!($ds, f64, $v);
                };
            }

            let ds = file.new_dataset::<u16>().create_anon(100).unwrap();
            check_all_fill_values!(ds, 0);

            let ds = file.new_dataset::<u16>().fill_value(42).create_anon(100).unwrap();
            check_all_fill_values!(ds, 42);

            let ds = file.new_dataset::<f32>().fill_value(1.234).create_anon(100).unwrap();
            check_all_fill_values!(ds, 1.234);
        })
    }
}
