//! Read and write dynamically sized arrays.
//!
//! ``` ignore
//! cargo run --example read_write_serial
//! ```
use hdf5::{File, Result, Selection};
use ndarray::{array, s, Array2, ArrayD, Ix2};
use std::convert::TryInto;

fn main() -> Result<()> {
    // Output parameters
    let file_name = "out_read_write_serial.h5";
    let dset_name = "data";

    // Create data and data slice
    let shape = [3, 4];
    let data = Array2::<f64>::from_elem(shape, 2.);
    let slice = s![1..3, 1..3];
    let data_slice = Array2::from_elem(data.slice(&slice).raw_dim(), 4.);

    // Write dataset
    {
        // Open file (create or append)
        let file = File::append(file_name)?;

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

        // Write complete data
        dset.write(&data)?;

        // Write slice
        let selection: Selection = slice.try_into()?;
        dset.write_slice(&data_slice, selection)?;

        // Close
        file.close()?;
    }

    //Read dataset
    let data_read: Array2<f64> = {
        // Open for read
        let file = File::open(file_name)?;

        // Read dynamically sized array
        let dset = file.dataset(dset_name)?;
        let arr_dyn: ArrayD<f64> = dset.read_dyn::<f64>()?;

        // Close
        file.close()?;

        // Dyn to static
        arr_dyn.into_dimensionality::<Ix2>()?
    };

    assert_eq!(data_read, array![[2., 2., 2., 2.], [2., 4., 4., 2.], [2., 4., 4., 2.]]);

    Ok(())
}
