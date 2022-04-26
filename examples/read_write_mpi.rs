//! Read and write dynamically sized arrays in parallel with mpi.
//!
//! This requires `hdf5` with mpi support. Run the program with mpirun:
//! ``` ignore
//! cargo build --example read_write_mpi --features mpio
//! mpirun -np 2 ./target/debug/examples/read_write_mpi
//! ```
#[cfg(feature = "mpio")]
mod mpi {
    use hdf5::{Dataset, Error, FileBuilder, Result, Selection};
    use mpi_sys::{
        MPI_Barrier, MPI_Comm_rank, MPI_Comm_size, MPI_Finalize, MPI_Init, MPI_Initialized,
        RSMPI_COMM_WORLD,
    };
    use ndarray::{array, s, Array, Array1, ArrayView, Dimension};
    use std::{convert::TryInto, os::raw::c_int, ptr};

    fn write<'a, A, S, D>(dset: &Dataset, data: A, selection: S) -> Result<()>
    where
        A: Into<ArrayView<'a, f64, D>>,
        D: Dimension,
        S: TryInto<Selection>,
        S::Error: Into<Error>,
    {
        dset.write_slice(data.into(), selection.try_into().map_err(|err| err.into())?)
    }

    fn read<S, D>(dset: &Dataset, selection: S) -> Result<Array<f64, D>>
    where
        D: ndarray::Dimension,
        S: TryInto<Selection>,
        S::Error: Into<Error>,
    {
        dset.read_slice(selection.try_into().map_err(|err| err.into())?)
    }

    pub fn main() -> Result<()> {
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

        let mut data_slice = data.slice(slice).to_owned();
        data_slice.fill(f64::from(rank + 1));

        // Write dataset
        {
            // Create file with mpi access
            let file =
                FileBuilder::new().with_fapl(|p| p.mpio(world_comm, None)).create(file_name)?;

            let dset = file
                .new_dataset::<f64>()
                .shape(data.shape())
                .create(dset_name)
                .expect("Failed to create dataset.");

            // Write slice
            write(&dset, &data_slice, slice)?;
        }

        unsafe {
            // All processes must write the data before it is available globally
            MPI_Barrier(world_comm);
        }

        //Read dataset
        let arr_slice_dyn: Array1<f64>;
        let arr_glob_dyn: Array1<f64>;
        {
            // Open for read
            let file =
                FileBuilder::new().with_fapl(|p| p.mpio(world_comm, None)).open(file_name)?;

            let dset = file.dataset(dset_name)?;

            // Read local data
            arr_slice_dyn = read(&dset, slice)?;
            // Read global data
            arr_glob_dyn = read(&dset, ..)?;
        };

        assert_eq!(arr_slice_dyn, data_slice);
        assert_eq!(arr_glob_dyn, array![1., 1., 1., 2., 2.]);

        unsafe {
            MPI_Finalize();
        }

        Ok(())
    }
}

#[cfg(feature = "mpio")]
fn main() -> hdf5::Result<()> {
    mpi::main()
}

#[cfg(not(feature = "mpio"))]
fn main() {
    println!("This example requires the `mpio` feature");
}
