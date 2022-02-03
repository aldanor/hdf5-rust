//! Read and write ndarray datasets.
//!
//! ``` ignore
//! cargo run --example read_write_ndarray
//! ```

static FILE_NAME: &str = "out_read_write_ndarray.h5";
static DSET_NAME: &str = "data";

/// Read an ndarray dataset
///
/// # Errors
/// File or dataset do not exist.
///
/// # Panics
/// Panics when `into_dimensionality` fails.
pub fn read_ndarray<A, D>(filename: &str, dsetname: &str) -> hdf5::Result<ndarray::Array<A, D>>
where
    A: hdf5::H5Type,
    D: ndarray::Dimension,
{
    // Open file
    let file = hdf5::File::open(filename)?;

    //Read dataset
    let data = file.dataset(dsetname)?;
    let y: ndarray::ArrayD<A> = data.read_dyn::<A>()?;

    // Dyn to static
    let x = y.into_dimensionality::<D>().unwrap();
    Ok(x)
}

/// Write an ndarray dataset.
///
/// Creates new file or append to existing file.
///
/// # Errors
/// File does not exist or file and dataset exist,
/// but shapes mismatch.
pub fn write_ndarray<A, S, D>(
    filename: &str, dsetname: &str, array: &ndarray::ArrayBase<S, D>,
) -> hdf5::Result<()>
where
    A: hdf5::H5Type,
    S: ndarray::Data<Elem = A>,
    D: ndarray::Dimension,
{
    // Open file
    let file = hdf5::File::append(filename)?;

    //Write dataset
    let dset = match file.dataset(dsetname) {
        // Overwrite
        Ok(dset) => dset,
        // Create new dataset
        std::prelude::v1::Err(..) => {
            file.new_dataset::<A>().no_chunk().shape(array.shape()).create(dsetname)?
        }
    };
    dset.write(&array.view())?;
    Ok(())
}

fn main() -> hdf5::Result<()> {
    // Create data
    let shape = [5, 6];
    let data = ndarray::Array2::<f64>::from_elem(shape, 2.);

    // Write
    write_ndarray(FILE_NAME, DSET_NAME, &data)?;

    // Read
    let r_full: ndarray::Array2<f64> = read_ndarray(FILE_NAME, DSET_NAME)?;
    assert_eq!(data, r_full);

    Ok(())
}
