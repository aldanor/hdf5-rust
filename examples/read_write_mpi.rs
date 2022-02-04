//! Read and write dynamically sized arrays in parallel with mpi.
//!
//! This will fail if the systems `hdf5` library is not build
//! with mpi support.
//!
//! ``` ignore
//! cargo build --example read_write_mpi --features mpio
//! mpirun -np 2 ./target/debug/examples/read_write_mpi
//! ```
use hdf5::{FileBuilder, Result, Selection};
use mpi_sys::{
    MPI_Comm_rank, MPI_Comm_size, MPI_Finalize, MPI_Init, MPI_Initialized, RSMPI_COMM_WORLD,
};
use ndarray::{array, s, Array1, ArrayD, Ix1};
use std::{convert::TryInto, os::raw::c_int, ptr};

fn main() -> Result<()> {
    // Output parameters
    let file_name = "out_read_write_mpi.h5";
    let dset_name = "data";

    // Iniitialize mpi (better use `mpi` crate)
    let mut initialized: c_int = 1;
    unsafe { MPI_Initialized(&mut initialized) };
    if initialized == 0 {
        unsafe { MPI_Init(ptr::null_mut(), ptr::null_mut()) };
    }
    let world_comm = unsafe { RSMPI_COMM_WORLD };
    let mut rank: c_int = -1;
    let mut size: c_int = -1;
    unsafe { MPI_Comm_rank(world_comm, &mut rank) };
    unsafe { MPI_Comm_size(world_comm, &mut size) };

    assert!(size == 2, "Example must be run with 2 processors, got {}", size);

    // Create data (global data and local data held by each processor)
    let glob_size = 5;
    let data = Array1::<f64>::zeros(glob_size);
    let slice = if rank == 0 { s![..3] } else { s![3..] };
    let data_slice = Array1::from_elem(data.slice(&slice).raw_dim(), f64::from(rank + 1));
    let selection: Selection = slice.try_into()?;

    // Write dataset
    {
        // Create file with mpi access
        let file = FileBuilder::new().with_fapl(|p| p.mpio(world_comm, None)).append(file_name)?;

        // Create or overwrite dataset
        // It is better to use `unwrap_or_else` which is lazily evaluated
        // than `unwrap_or`.
        let dset = file.dataset(dset_name).unwrap_or_else(|_| {
            file.new_dataset::<f64>()
                .no_chunk()
                .shape(data.shape())
                .create(dset_name)
                .expect("Failed to create dataset.")
        });

        // Write slice
        dset.write_slice(&data_slice, &selection)?;
    }

    //Read dataset
    let (data_slice_read, data_read) = {
        // Open for read
        let file = FileBuilder::new().with_fapl(|p| p.mpio(world_comm, None)).open(file_name)?;

        // Read local held data and global array
        let dset = file.dataset(dset_name)?;
        let arr_slice_dyn: ArrayD<f64> = dset.read_slice(selection)?;
        let arr_glob_dyn: ArrayD<f64> = dset.read_dyn::<f64>()?;

        // Dyn to static
        (arr_slice_dyn.into_dimensionality::<Ix1>()?, arr_glob_dyn.into_dimensionality::<Ix1>()?)
    };

    assert_eq!(data_slice_read, data_slice);
    assert_eq!(data_read, array![1., 1., 1., 2., 2.]);

    unsafe {
        MPI_Finalize();
    }

    Ok(())
}
