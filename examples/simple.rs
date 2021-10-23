#[cfg(feature = "blosc")]
use hdf5::filters::{blosc_set_nthreads, Blosc};
use hdf5::{File, H5Type, Result};
use ndarray::{arr2, s};

#[derive(H5Type, Clone, PartialEq, Debug)] // map the HDF5 type for this enum
#[repr(u8)]
pub enum Color {
    R = 1,
    G = 2,
    B = 3,
}

#[derive(H5Type, Clone, PartialEq, Debug)] // register this struct with HDF5
#[repr(C)]
pub struct Pixel {
    xy: (i64, i64),
    color: Color,
}

fn main() -> Result<()> {
    {
        let file = File::create("pixels.h5")?; // open the file for writing
        let group = file.create_group("dir")?; // create a group
        #[cfg(feature = "blosc")]
        blosc_set_nthreads(2); // set number of threads for compressing/decompressing chunks
        let builder = group.new_dataset_builder();
        #[cfg(feature = "blosc")]
        let builder = builder.blosc(Blosc::ZStd, 9, true); // enable zstd compression with shuffling
        let ds = builder
            .with_data(&arr2(&[
                // write a 2-D array of data
                [Pixel { xy: (1, 2), color: Color::R }, Pixel { xy: (2, 3), color: Color::B }],
                [Pixel { xy: (3, 4), color: Color::G }, Pixel { xy: (4, 5), color: Color::R }],
                [Pixel { xy: (5, 6), color: Color::B }, Pixel { xy: (6, 7), color: Color::G }],
            ]))
            .create("pixels")?; // finalize and write the dataset
        let attr = ds.new_attr::<Color>().shape([3]).create("colors")?; // create an attribute
        attr.write(&[Color::R, Color::G, Color::B])?;
    }
    {
        let file = File::open("pixels.h5")?; // open the file for reading
        let ds = file.dataset("dir/pixels")?; // open the dataset object
        assert_eq!(
            ds.read_slice::<Pixel, _, _>(s![1.., ..])?, // read a slice of the 2-D dataset
            arr2(&[
                [Pixel { xy: (3, 4), color: Color::G }, Pixel { xy: (4, 5), color: Color::R }],
                [Pixel { xy: (5, 6), color: Color::B }, Pixel { xy: (6, 7), color: Color::G }],
            ])
        );
        let attr = ds.attr("colors")?; // open the attribute
        assert_eq!(attr.read_1d::<Color>()?.as_slice().unwrap(), &[Color::R, Color::G, Color::B]);
    }
    Ok(())
}
