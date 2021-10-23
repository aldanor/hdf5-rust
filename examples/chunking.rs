//! Create, write, and read a chunked dataset
fn main() -> hdf5::Result<()> {
    let file = hdf5::File::create("chunking.h5")?;

    let ny = 100;
    let nx = 100;
    let arr = ndarray::Array2::from_shape_fn((ny, nx), |(j, i)| (1000 * j + i) as f32);

    {
        let ds = file
        .new_dataset::<f32>()
        .chunk((1, ny, nx))  // nx*ny elements will be compressed as a single chunk
        .shape((1.., ny, nx)) // Initial size of 1 on the unlimited dimension
        .deflate(3)
        .create("variable")?;

        // Writing a chunk at a time will be most efficient
        ds.write_slice(&arr, (0, .., ..))?;

        // Dataset can be resized along an unlimited dimension
        ds.resize((10, ny, nx))?;
        ds.write_slice(&arr, (1, .., ..))?;
    }

    let ds = file.dataset("variable")?;
    let chunksize = ds.chunk().unwrap();
    assert_eq!(chunksize, &[1, ny, nx]);

    let shape = ds.shape();
    assert_eq!(shape, &[10, ny, nx]);

    // Reading from a chunked dataset should be done in a chunk-wise order
    for k in 0..shape[0] {
        let _arr: ndarray::Array2<f32> = ds.read_slice((k, .., ..))?;
    }

    Ok(())
}
