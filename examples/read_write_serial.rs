//! Read and write dynamically sized arrays.
//!
//! ``` ignore
//! cargo run --example read_write_serial
//! ```
use hdf5::{Dataset, Error, File, Result, Selection};
use ndarray::{array, s, Array, Array2, ArrayView, Dimension};
use std::convert::TryInto;

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

fn main() -> Result<()> {
    // Output parameters
    let file_name = "out_read_write_serial.h5";
    let dset_name = "data";

    // Create data and data slice
    let shape = [3, 4];
    let data = Array2::<f64>::from_elem(shape, 2.);
    let slice = s![1..3, 1..3];
    let mut data_slice = data.slice(&slice).to_owned();
    data_slice.fill(4.0);

    // Write dataset
    {
        let file = File::create(file_name)?;
        let dset = file
            .new_dataset::<f64>()
            .shape(data.shape())
            .create(dset_name)
            .expect("Failed to create dataset.");

        // Write all the data
        write(&dset, &data, ..)?;

        // Write a slice of the data
        write(&dset, &data_slice, slice)?;
    }

    // Read dataset
    let data_read: Array2<f64>;
    {
        let file = File::open(file_name)?;
        let dset = file.dataset(dset_name)?;

        // Read entire dataset
        data_read = read(&dset, ..)?;
    }

    assert_eq!(data_read, array![[2., 2., 2., 2.], [2., 4., 4., 2.], [2., 4., 4., 2.]]);

    Ok(())
}
