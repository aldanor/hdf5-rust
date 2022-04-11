use crate::internal_prelude::*;
use hdf5_sys::h5d::H5Dchunk_iter;

/// Borrowed version of [ChunkInfo](crate::hl::Dataset::ChunkInfo)
#[derive(Debug)]
pub struct ChunkInfoBorrowed<'a> {
    pub offset: &'a [hsize_t],
    pub filter_mask: u32,
    pub addr: u64,
    pub size: u64,
}

struct RustCallback<F> {
    ndims: usize,
    callback: F,
}

extern "C" fn chunks_callback<F>(
    offset: *const hsize_t, filter_mask: u32, addr: haddr_t, nbytes: u32, op_data: *mut c_void,
) -> herr_t
where
    F: FnMut(ChunkInfoBorrowed) -> i32,
{
    unsafe {
        std::panic::catch_unwind(|| {
            let data: *mut RustCallback<F> = op_data.cast::<RustCallback<F>>();
            let ndims = (*data).ndims;
            let callback = &mut (*data).callback;

            let offset = std::slice::from_raw_parts(offset, ndims);

            let info =
                ChunkInfoBorrowed { offset, filter_mask, addr: addr as u64, size: nbytes as u64 };

            callback(info)
        })
        .unwrap_or(-1)
    }
}

pub(crate) fn visit<F>(ds: &Dataset, callback: F) -> Result<()>
where
    F: for<'a> FnMut(ChunkInfoBorrowed<'a>) -> i32,
{
    let mut data = RustCallback::<F> { ndims: ds.ndim(), callback };

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

            let ds = f.new_dataset::<i16>().shape([3, 2]).chunk([1, 1]).create("chunk").unwrap();
            ds.write(&ndarray::arr2(&[[1, 2], [3, 4], [5, 6]])).unwrap();

            let mut i = 0;
            let f = |c: ChunkInfoBorrowed| {
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
