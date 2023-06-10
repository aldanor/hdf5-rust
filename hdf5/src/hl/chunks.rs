use crate::internal_prelude::*;

#[cfg(feature = "1.10.5")]
use hdf5_sys::h5d::{H5Dget_chunk_info, H5Dget_num_chunks};

#[cfg(feature = "1.10.5")]
#[derive(Clone, Debug, PartialEq, Eq)]
/// Information on a chunk in a Dataset
pub struct ChunkInfo {
    /// Array with a size equal to the dataset’s rank whose elements contain 0-based
    /// logical positions of the chunk’s first element in each dimension.
    pub offset: Vec<hsize_t>,
    /// Filter mask that indicates which filters were used with the chunk when written.
    ///
    /// A zero value indicates that all enabled filters are applied on the chunk.
    /// A filter is skipped if the bit corresponding to the filter’s position in
    /// the pipeline (0 ≤ position < 32) is turned on.
    pub filter_mask: c_uint,
    /// Chunk address in the file.
    pub addr: haddr_t,
    /// Chunk size in bytes.
    pub size: hsize_t,
}

#[cfg(feature = "1.10.5")]
impl ChunkInfo {
    pub(crate) fn new(ndim: usize) -> Self {
        let offset = vec![0; ndim];
        Self { offset, filter_mask: 0, addr: 0, size: 0 }
    }

    /// Returns positional indices of disabled filters.
    pub fn disabled_filters(&self) -> Vec<usize> {
        (0..32).filter(|i| self.filter_mask & (1 << i) != 0).collect()
    }
}

#[cfg(feature = "1.10.5")]
pub(crate) fn chunk_info(ds: &Dataset, index: usize) -> Option<ChunkInfo> {
    if !ds.is_chunked() {
        return None;
    }
    h5lock!(ds.space().map_or(None, |s| {
        let mut chunk_info = ChunkInfo::new(ds.ndim());
        h5check(H5Dget_chunk_info(
            ds.id(),
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

#[cfg(feature = "1.10.5")]
pub(crate) fn get_num_chunks(ds: &Dataset) -> Option<usize> {
    if !ds.is_chunked() {
        return None;
    }
    h5lock!(ds.space().map_or(None, |s| {
        let mut n: hsize_t = 0;
        h5check(H5Dget_num_chunks(ds.id(), s.id(), &mut n)).map(|_| n as _).ok()
    }))
}

#[cfg(feature = "1.14.0")]
mod v1_14_0 {
    use super::*;
    use hdf5_sys::h5d::H5Dchunk_iter;

    /// Borrowed version of [ChunkInfo](crate::dataset::ChunkInfo)
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct ChunkInfoRef<'a> {
        pub offset: &'a [hsize_t],
        pub filter_mask: c_uint,
        pub addr: haddr_t,
        pub size: hsize_t,
    }

    impl<'a> ChunkInfoRef<'a> {
        /// Returns positional indices of disabled filters.
        pub fn disabled_filters(&self) -> Vec<usize> {
            (0..32).filter(|i| self.filter_mask & (1 << i) != 0).collect()
        }
    }

    impl<'a> From<ChunkInfoRef<'a>> for ChunkInfo {
        fn from(val: ChunkInfoRef<'a>) -> Self {
            Self {
                offset: val.offset.to_owned(),
                filter_mask: val.filter_mask,
                addr: val.addr,
                size: val.size,
            }
        }
    }

    #[repr(C)]
    struct RustCallback<F> {
        pub ndims: hsize_t,
        pub callback: F,
    }

    extern "C" fn chunks_callback<F>(
        offset: *const hsize_t, filter_mask: c_uint, addr: haddr_t, size: hsize_t,
        op_data: *mut c_void,
    ) -> herr_t
    where
        F: FnMut(ChunkInfoRef) -> i32,
    {
        unsafe {
            std::panic::catch_unwind(|| {
                let data: *mut RustCallback<F> = op_data.cast::<RustCallback<F>>();
                let ndims = (*data).ndims;
                let callback = &mut (*data).callback;

                let offset = std::slice::from_raw_parts(offset, ndims as usize);

                let info = ChunkInfoRef { offset, filter_mask, addr, size };

                callback(info)
            })
            .unwrap_or(-1)
        }
    }

    pub(crate) fn visit<F>(ds: &Dataset, callback: F) -> Result<()>
    where
        F: for<'a> FnMut(ChunkInfoRef<'a>) -> i32,
    {
        let mut data = RustCallback::<F> { ndims: ds.ndim() as _, callback };

        h5try!(H5Dchunk_iter(
            ds.id(),
            H5P_DEFAULT,
            Some(chunks_callback::<F>),
            std::ptr::addr_of_mut!(data).cast()
        ));

        Ok(())
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn chunks_visit() {
            with_tmp_file(|f| {
                let ds = f.new_dataset::<i16>().no_chunk().shape((4, 4)).create("nochunk").unwrap();
                assert_err_re!(visit(&ds, |_| 0), "not a chunked dataset");

                let ds =
                    f.new_dataset::<i16>().shape([3, 2]).chunk([1, 1]).create("chunk").unwrap();
                ds.write(&ndarray::arr2(&[[1, 2], [3, 4], [5, 6]])).unwrap();

                let mut i = 0;
                let f = |c: ChunkInfoRef| {
                    match i {
                        0 => assert_eq!(c.offset, [0, 0]),
                        1 => assert_eq!(c.offset, [0, 1]),
                        2 => assert_eq!(c.offset, [1, 0]),
                        3 => assert_eq!(c.offset, [1, 1]),
                        4 => assert_eq!(c.offset, [2, 0]),
                        5 => assert_eq!(c.offset, [2, 1]),
                        _ => unreachable!(),
                    }
                    assert_eq!(c.size, std::mem::size_of::<i16>() as u64);
                    i += 1;
                    0
                };
                visit(&ds, f).unwrap();
                assert_eq!(i, 6);
            })
        }
    }
}
#[cfg(feature = "1.14.0")]
pub use v1_14_0::*;
