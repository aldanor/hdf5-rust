use self::common::gen::{Enum, FixedStruct, Gen, TupleStruct, VarLenStruct};
use self::common::util::new_in_memory_file;
use rand::prelude::{SeedableRng, SmallRng};
use std::fmt;

mod common;

/// Tests attribute creation, writing, reading.
fn test_read_write<T>() -> hdf5::Result<()>
where
    T: hdf5::H5Type + fmt::Debug + PartialEq + Gen + Clone,
{
    let mut rng = SmallRng::seed_from_u64(42);
    let file = new_in_memory_file()?;
    let ds = file.new_dataset::<T>().create("asdf/1234/dataset", 0)?;

    for i in 0..1000 {
        let value = T::gen(&mut rng);
        let attr = ds.create_attr::<T>(format!("attr_{}", i).as_str())?;
        attr.write_scalar(&value)?;
        assert_eq!(value, attr.read_scalar::<T>()?);
    }
    Ok(())
}

#[test]
pub fn test_read_write_primitive() -> hdf5::Result<()> {
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
fn test_read_write_enum() -> hdf5::Result<()> {
    test_read_write::<Enum>()
}

#[test]
fn test_read_write_tuple_struct() -> hdf5::Result<()> {
    test_read_write::<TupleStruct>()
}

#[test]
fn test_read_write_fixed_struct() -> hdf5::Result<()> {
    test_read_write::<FixedStruct>()
}

#[test]
fn test_read_write_varlen_struct() -> hdf5::Result<()> {
    test_read_write::<VarLenStruct>()
}

#[test]
fn test_read_write_tuples() -> hdf5::Result<()> {
    test_read_write::<(u8,)>()?;
    test_read_write::<(u64, f32)>()?;
    test_read_write::<(i8, u64, f32)>()?;
    Ok(())
}
