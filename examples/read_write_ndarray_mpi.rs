//! Read and write ndarray datasets in parallel.
//!
//! This will fail if the systems `hdf5` library is not build
//! with mpi support.
//!
//! ``` ignore
//! cargo build --example read_write_ndarray_mpi --features mpio
//! mpirun -np 2 ./target/debug/examples/read_write_ndarray_mpi
//! ```

static FILE_NAME: &str = "out_read_write_ndarray_mpi.h5";
static DSET_NAME: &str = "data";

/// Read slice of an ndarray dataset
///
/// # Errors
/// File or dataset do not exist.
///
/// # Panics
/// Panics when `into_dimensionality` fails.
///
/// # TODO
/// Check that slices do not overlap.
pub fn read_ndarray_mpi<A, D, T>(
    comm: mpi_sys::MPI_Comm, filename: &str, dsetname: &str, slice: ndarray::SliceInfo<T, D, D>,
) -> hdf5::Result<ndarray::Array<A, D>>
where
    A: hdf5::H5Type,
    D: ndarray::Dimension,
    T: AsRef<[ndarray::SliceInfoElem]>,
{
    // Open file
    let file = hdf5::FileBuilder::new().with_fapl(|p| p.mpio(comm, None)).open(filename)?;

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
///
/// # TODO
/// Check that slices do not overlap.
pub fn write_ndarray_mpi<A, S, D, T, Sh>(
    comm: mpi_sys::MPI_Comm, filename: &str, dsetname: &str, array: &ndarray::ArrayBase<S, D>,
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

    // Create file with mpi access
    let file = hdf5::FileBuilder::new().with_fapl(|p| p.mpio(comm, None)).append(filename)?;

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

use std::os::raw::c_int;
use std::ptr;

fn main() -> hdf5::Result<()> {
    // Initialize mpi
    let mut initialized: c_int = 1;
    unsafe { mpi_sys::MPI_Initialized(&mut initialized) };
    if initialized == 0 {
        unsafe { mpi_sys::MPI_Init(ptr::null_mut(), ptr::null_mut()) };
    }
    let world_comm = unsafe { mpi_sys::RSMPI_COMM_WORLD };
    let mut rank: c_int = 1;
    unsafe { mpi_sys::MPI_Comm_rank(world_comm, &mut rank) };

    // Create data
    let shape = [5];
    let slice = if rank == 0 { ndarray::s![..3] } else { ndarray::s![3..] };
    let data = ndarray::Array1::<f64>::from_elem(shape, rank as f64 + 1.);
    // Slice data. Each processor must hold a different subset.
    let data_slice = data.slice(&slice);

    // let mut data = ndarray::Array2::<f64>::zeros(shape);
    // data.fill(2.);

    // Write slice
    write_ndarray_mpi(world_comm, FILE_NAME, DSET_NAME, &data_slice, slice, shape)?;

    // Read slice
    let r_slice: ndarray::Array1<f64> = read_ndarray_mpi(world_comm, FILE_NAME, DSET_NAME, slice)?;
    assert_eq!(data_slice, r_slice);

    // Read full array (see also `examples/read_write_ndarray.rs`)
    // This array should be zero everywhere outside the sliced region,
    // and thus unequal to the original full filled array.
    let r_full: ndarray::Array1<f64> =
        read_ndarray_mpi(world_comm, FILE_NAME, DSET_NAME, ndarray::s![0..shape[0]])?;
    assert_ne!(data, r_full);
    assert_eq!(ndarray::array![1., 1., 1., 2., 2.], r_full);
    Ok(())
}
