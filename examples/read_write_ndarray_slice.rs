//! Read and write slices of ndarray datasets
//!
//! ``` ignore
//! cargo run --example read_write_ndarray_slice
//! ```

static FILE_NAME: &str = "out_read_write_ndarray_slice.h5";
static DSET_NAME: &str = "data";

/// Read slice of an ndarray dataset
///
/// # Errors
/// File or dataset do not exist.
///
/// # Panics
/// Panics when `into_dimensionality` fails.
pub fn read_ndarray_slice<A, D, T>(
    filename: &str, dsetname: &str, slice: ndarray::SliceInfo<T, D, D>,
) -> hdf5::Result<ndarray::Array<A, D>>
where
    A: hdf5::H5Type,
    D: ndarray::Dimension,
    T: AsRef<[ndarray::SliceInfoElem]>,
{
    // Open file
    let file = hdf5::File::open(filename)?;

    //Read dataset
    let data = file.dataset(dsetname)?;
    let y: ndarray::ArrayD<A> = data.read_slice(slice)?;

    // Dyn to static
    let x = y.into_dimensionality::<D>().unwrap();
    Ok(x)
}

/// Write a slice of an ndarray dataset to an
/// hdf5 file. Specify the full shape of the
/// dataset and the slice (with ndarrays `s!` macro).
/// Supplied Array and slices must be smaller than `full_shape`.
///
/// Creates new file or append to existing file.
///
/// # Errors
/// File does not exist or file and dataset exist,
/// but shapes mismatch.
///
/// # Panics
/// Mismatch of number of dimensions between array
/// and `full_shape`.
pub fn write_ndarray_slice<A, S, D, T, Sh>(
    filename: &str, dsetname: &str, array: &ndarray::ArrayBase<S, D>,
    slice: ndarray::SliceInfo<T, D, D>, full_shape: Sh,
) -> hdf5::Result<()>
where
    A: hdf5::H5Type,
    S: ndarray::Data<Elem = A>,
    D: ndarray::Dimension,
    T: AsRef<[ndarray::SliceInfoElem]>,
    Sh: Into<hdf5::Extents>,
{
    use std::convert::TryFrom;

    let full_shape = full_shape.into();
    assert!(
        array.ndim() == full_shape.ndim(),
        "Dimension mismatch of array and full_shape, {} vs. {}",
        array.ndim(),
        full_shape.ndim()
    );

    // Create new file or append
    let file = hdf5::File::append(filename)?;

    //Write dataset
    let dset = match file.dataset(dsetname) {
        // Overwrite
        Ok(dset) => dset,
        // Create new dataset
        std::prelude::v1::Err(..) => {
            file.new_dataset::<A>().no_chunk().shape(full_shape).create(dsetname)?
        }
    };
    dset.write_slice(array, hdf5::Hyperslab::try_from(slice)?)?;
    Ok(())
}

fn main() -> hdf5::Result<()> {
    // Create data
    let shape = [5, 6];
    let slice = ndarray::s![1..3, 2..4];
    let data = ndarray::Array2::<f64>::from_elem(shape, 2.);

    // Write slice
    let w_slice = data.slice(&slice).to_owned();
    write_ndarray_slice(FILE_NAME, DSET_NAME, &w_slice, slice, shape)?;

    // Read slice
    let r_slice: ndarray::Array2<f64> = read_ndarray_slice(FILE_NAME, DSET_NAME, slice)?;
    assert_eq!(w_slice, r_slice);

    // Read full array (see also `examples/read_write_ndarray.rs`)
    // This array should be zero everywhere outside the sliced region,
    // and thus unequal to the original full filled array.
    let r_full: ndarray::Array2<f64> =
        read_ndarray_slice(FILE_NAME, DSET_NAME, ndarray::s![0..shape[0], 0..shape[1]])?;
    assert_ne!(data, r_full);

    Ok(())
}
