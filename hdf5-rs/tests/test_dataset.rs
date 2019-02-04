use std::fmt;

use ndarray::{s, Array1, Array2, ArrayD, IxDyn, SliceInfo};
use rand::prelude::{Rng, SeedableRng, SmallRng};

use hdf5_types::TypeDescriptor;

mod common;

use self::common::gen::{gen_arr, gen_slice, Enum, FixedStruct, Gen, TupleStruct, VarLenStruct};
use self::common::util::new_in_memory_file;

fn test_write_slice<T, R>(
    rng: &mut R, ds: &h5::Dataset, arr: &ArrayD<T>, default_value: &T, _ndim: usize,
) -> h5::Result<()>
where
    T: h5::H5Type + fmt::Debug + PartialEq + Gen + Clone,
    R: Rng + ?Sized,
{
    let shape = arr.shape();
    let slice = gen_slice(rng, shape);

    // Take a random slice of the dataset, and convert it to a standard dense layout
    let sliced_array_view = arr.slice(slice.as_ref());
    let mut sliced_array_copy = ArrayD::from_elem(sliced_array_view.shape(), default_value.clone());
    sliced_array_copy.assign(&sliced_array_view);

    // Write these elements into their 'correct' places in the matrix
    {
        let dsw = ds.as_writer();
        dsw.write_slice(&sliced_array_copy, &slice)?;
    }

    // Read back out the random from the full dataset
    let full_ds = ds.read_dyn::<T>()?;
    let read_slice = full_ds.slice(slice.as_ref());

    assert_eq!(sliced_array_view, read_slice);
    Ok(())
}

fn test_read_slice<T, R>(
    rng: &mut R, ds: &h5::Dataset, arr: &ArrayD<T>, ndim: usize,
) -> h5::Result<()>
where
    T: h5::H5Type + fmt::Debug + PartialEq + Gen,
    R: Rng + ?Sized,
{
    ds.write(arr)?;

    // Test various sliced reads
    let shape = arr.shape();

    let out_dyn = ds.read_dyn::<T>();
    assert_eq!(arr, &out_dyn?.into_dimensionality().unwrap());

    let dsr = ds.as_reader();

    for _ in 0..10 {
        let slice = gen_slice(rng, shape);

        // Do a sliced HDF5 read
        let sliced_read = dsr.read_slice(&slice).unwrap();

        // Slice the full dataset
        let sliced_dataset = arr.slice(slice.as_ref());

        // Ensure that the H5 sliced read matches the ndarray slice of the original array.
        if sliced_read != sliced_dataset {
            println!("{:?}", slice);
        }
        assert_eq!(sliced_read, sliced_dataset);
    }

    // Test that we get an error if we use the wrong dimensionality when slicing.
    let mut bad_shape = Vec::from(shape);
    bad_shape.push(1);
    let bad_slice = gen_slice(rng, &bad_shape);
    let bad_slice: SliceInfo<_, IxDyn> = ndarray::SliceInfo::new(bad_slice.as_slice()).unwrap();

    let bad_sliced_read: h5::Result<ArrayD<T>> = dsr.read_slice(&bad_slice);
    assert!(bad_sliced_read.is_err());

    // Tests for dimension-dropping slices with static dimensionality.
    if ndim == 2 && shape[0] > 0 && shape[1] > 0 {
        let v: Array1<T> = dsr.read_slice_1d(s![0, ..])?;
        assert_eq!(shape[1], v.shape()[0]);

        let v: Array1<T> = dsr.read_slice_1d(s![.., 0])?;
        assert_eq!(shape[0], v.shape()[0]);
    } 

    if ndim == 3 && shape[0] > 0 && shape[1] > 0 && shape[2] > 0 {
        let v: Array2<T> = dsr.read_slice_2d(s![0, .., ..])?;
        assert_eq!(shape[1], v.shape()[0]);
        assert_eq!(shape[2], v.shape()[1]);

        let v: Array1<T> = dsr.read_slice_1d(s![0, 0, ..])?;
        assert_eq!(shape[2], v.shape()[0]);
    } 

    Ok(())
}

fn test_read<T>(ds: &h5::Dataset, arr: &ArrayD<T>, ndim: usize) -> h5::Result<()>
where
    T: h5::H5Type + fmt::Debug + PartialEq + Gen,
{
    ds.write(arr)?;

    // read_raw()
    let out_vec = ds.read_raw::<T>();
    assert_eq!(arr.as_slice().unwrap(), out_vec?.as_slice());

    // read_dyn()
    let out_dyn = ds.read_dyn::<T>();
    assert_eq!(arr, &out_dyn?.into_dimensionality().unwrap());

    // read_scalar()
    let out_scalar = ds.read_scalar::<T>();
    if ndim == 0 {
        assert_eq!(arr.as_slice().unwrap()[0], out_scalar?);
    } else {
        assert!(out_scalar.is_err());
    }

    // read_1d()
    let out_1d = ds.read_1d::<T>();
    if ndim == 1 {
        assert_eq!(arr, &out_1d?.into_dimensionality().unwrap());
    } else {
        assert!(out_1d.is_err());
    }

    // read_2d()
    let out_2d = ds.read_2d::<T>();
    if ndim == 2 {
        assert_eq!(arr, &out_2d?.into_dimensionality().unwrap());
    } else {
        assert!(out_2d.is_err());
    }

    Ok(())
}

fn test_write<T>(ds: &h5::Dataset, arr: &ArrayD<T>, ndim: usize) -> h5::Result<()>
where
    T: h5::H5Type + fmt::Debug + PartialEq + Gen,
{
    // .write()
    ds.write(arr)?;
    assert_eq!(&ds.read_dyn::<T>()?, arr);

    // .write_scalar()
    if ndim == 0 {
        ds.write_scalar(&arr.as_slice().unwrap()[0])?;
        assert_eq!(&ds.read_dyn::<T>()?, arr);
    } else if arr.len() > 0 {
        assert!(ds.write_scalar(&arr.as_slice().unwrap()[0]).is_err());
    }

    // .write_raw()
    ds.write_raw(arr.as_slice().unwrap())?;
    assert_eq!(&ds.read_dyn::<T>()?, arr);

    Ok(())
}

fn test_read_write<T>() -> h5::Result<()>
where
    T: h5::H5Type + fmt::Debug + PartialEq + Gen + Clone,
{
    let td = T::type_descriptor();
    let mut packed = vec![false];
    if let TypeDescriptor::Compound(_) = td {
        packed.push(true);
    }

    let mut rng = SmallRng::seed_from_u64(42);
    let file = new_in_memory_file()?;

    for packed in &packed {
        for ndim in 0..=4 {
            for _ in 0..=20 {
                for mode in 0..4 {
                    let arr: ArrayD<T> = gen_arr(&mut rng, ndim);

                    let ds: h5::Dataset = file
                        .new_dataset::<T>()
                        .packed(*packed)
                        .create("x", arr.shape().to_vec())?;
                    let ds = scopeguard::guard(ds, |ds| {
                        drop(ds);
                        drop(file.unlink("x"));
                    });

                    if mode == 0 {
                        test_read(&ds, &arr, ndim)?;
                    } else if mode == 1 {
                        test_write(&ds, &arr, ndim)?;
                    } else if mode == 2 {
                        test_read_slice(&mut rng, &ds, &arr, ndim)?;
                    } else if mode == 3 {
                        let default_value = T::gen(&mut rng);
                        test_write_slice(&mut rng, &ds, &arr, &default_value, ndim)?;
                    }
                }
            }
        }
    }

    Ok(())
}

#[test]
fn test_read_write_primitive() -> h5::Result<()> {
    test_read_write::<i8>()?;
    test_read_write::<i16>()?;
    test_read_write::<i32>()?;
    test_read_write::<i64>()?;
    test_read_write::<u8>()?;
    test_read_write::<u16>()?;
    test_read_write::<u32>()?;
    test_read_write::<u64>()?;
    test_read_write::<isize>()?;
    test_read_write::<usize>()?;
    test_read_write::<bool>()?;
    test_read_write::<f32>()?;
    test_read_write::<f64>()?;
    Ok(())
}

#[test]
fn test_read_write_enum() -> h5::Result<()> {
    test_read_write::<Enum>()
}

#[test]
fn test_read_write_tuple_struct() -> h5::Result<()> {
    test_read_write::<TupleStruct>()
}

#[test]
fn test_read_write_fixed_struct() -> h5::Result<()> {
    test_read_write::<FixedStruct>()
}

#[test]
fn test_read_write_varlen_struct() -> h5::Result<()> {
    test_read_write::<VarLenStruct>()
}

#[test]
fn test_read_write_tuples() -> h5::Result<()> {
    test_read_write::<(u8,)>()?;
    test_read_write::<(u64, f32)>()?;
    test_read_write::<(i8, u64, f32)>()?;
    Ok(())
}
