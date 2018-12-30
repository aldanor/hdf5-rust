use std::fmt;

use ndarray::ArrayD;
use rand::prelude::{SeedableRng, SmallRng};

use hdf5_types::TypeDescriptor;

mod common;

use self::common::gen::{gen_arr, Enum, FixedStruct, Gen, TupleStruct, VarLenStruct};
use self::common::util::new_in_memory_file;

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
    T: h5::H5Type + fmt::Debug + PartialEq + Gen,
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
                for read in &[false, true] {
                    let arr: ArrayD<T> = gen_arr(&mut rng, ndim);

                    let ds: h5::Dataset = file
                        .new_dataset::<T>()
                        .packed(*packed)
                        .create("x", arr.shape().to_vec())?;
                    let ds = scopeguard::guard(ds, |ds| {
                        drop(ds);
                        drop(file.unlink("x"));
                    });

                    if *read {
                        test_read(&ds, &arr, ndim)?;
                    } else {
                        test_write(&ds, &arr, ndim)?;
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
