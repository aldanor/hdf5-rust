use datatype::{Datatype, AnyDatatype};
use error::Result;
use handle::{ID, FromID};
use plist::PropertyList;
use space::{Dimension, Ix};

use ffi::h5::hsize_t;
use ffi::h5d::H5D_FILL_TIME_ALLOC;
use ffi::h5p::{H5Pcreate, H5Pset_chunk, H5Pset_fletcher32, H5Pset_scaleoffset, H5Pset_shuffle,
               H5Pset_deflate, H5Pset_szip, H5Pset_fill_time, H5Pget_nfilters, H5Pget_filter2};
use ffi::h5z::{H5Z_SO_INT, H5Z_SO_FLOAT_DSCALE, H5_SZIP_EC_OPTION_MASK, H5_SZIP_NN_OPTION_MASK,
               H5Z_FILTER_DEFLATE, H5Z_FILTER_SZIP, H5Z_FILTER_SHUFFLE, H5Z_FILTER_FLETCHER32,
               H5Z_FILTER_SCALEOFFSET, H5Z_FILTER_CONFIG_ENCODE_ENABLED,
               H5Z_FILTER_CONFIG_DECODE_ENABLED, H5Zfilter_avail, H5Zget_filter_info,
               H5Z_filter_t};
use globals::H5P_DATASET_CREATE;

use libc::{c_int, c_uint, size_t, c_char};
use num::Bounded;
use num::integer::div_floor;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Szip {
    EntropyCoding,
    NearestNeighbor,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Chunk<T: Sized + Copy + Dimension> {
    None,
    Auto,
    Manual(T),
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Filters {
    gzip: Option<u8>,
    szip: Option<(Szip, u8)>,
    shuffle: bool,
    fletcher32: bool,
    scale_offset: Option<u32>,
}

impl Default for Filters {
    fn default() -> Filters {
        Filters {
            gzip: None,
            szip: None,
            shuffle: false,
            fletcher32: false,
            scale_offset: None,
        }
    }
}

impl Filters {
    pub fn new() -> Filters {
        Filters::default()
    }

    /// Enable gzip compression with a specified level (0-9).
    pub fn gzip(&mut self, level: u8) -> &mut Filters {
        self.gzip = Some(level); self
    }

    /// Disable gzip compression.
    pub fn no_gzip(&mut self) -> &mut Filters {
        self.gzip = None; self
    }

    /// Get the current settings for gzip filter.
    pub fn get_gzip(&self) -> Option<u8> {
        self.gzip
    }

    /// Enable szip compression with a specified method (EC, NN) and level (0-32).
    pub fn szip(&mut self, method: Szip, level: u8) -> &mut Filters {
        self.szip = Some((method, level)); self
    }

    /// Disable szip compression.
    pub fn no_szip(&mut self) -> &mut Filters {
        self.szip = None; self
    }

    /// Get the current settings for szip filter.
    pub fn get_szip(&self) -> Option<(Szip, u8)> {
        self.szip
    }

    /// Enable or disable shuffle filter.
    pub fn shuffle(&mut self, shuffle: bool) -> &mut Filters {
        self.shuffle = shuffle; self
    }

    /// Get the current settings for shuffle filter.
    pub fn get_shuffle(&self) -> bool {
        self.shuffle
    }

    /// Enable or disable fletcher32 filter.
    pub fn fletcher32(&mut self, fletcher32: bool) -> &mut Filters {
        self.fletcher32 = fletcher32; self
    }

    /// Get the current settings for fletcher32 filter.
    pub fn get_fletcher32(&self) -> bool {
        self.fletcher32
    }

    /// Enable scale-offset filter with a specified factor (0 means automatic).
    pub fn scale_offset(&mut self, scale_offset: u32) -> &mut Filters {
        self.scale_offset = Some(scale_offset); self
    }

    /// Disable scale_offset compression.
    pub fn no_scale_offset(&mut self) -> &mut Filters {
        self.scale_offset = None; self
    }

    /// Get the current settings for scale_offset filter.
    pub fn get_scale_offset(&self) -> Option<u32> {
        self.scale_offset
    }

    /// Enable gzip filter with default settings (compression level 4).
    pub fn gzip_default(&mut self) -> &mut Filters {
        self.gzip = Some(4); self
    }

    /// Enable szip filter with default settings (NN method, compression level 8).
    pub fn szip_default(&mut self) -> &mut Filters {
        self.szip = Some((Szip::NearestNeighbor, 8)); self
    }

    pub fn validate(&self) -> Result<()> {
        if self.gzip.is_some() && self.szip.is_some() {
            fail!("Cannot specify two compression options at once.")
        }
        if let Some(level) = self.gzip {
            ensure!(level <= 9, "Invalid level for gzip compression, expected 0-9 integer.");
        }
        if let Some((_, pixels_per_block)) = self.szip {
            ensure!(pixels_per_block <= 32 && pixels_per_block % 2 == 0,
                    "Invalid pixels per block for szip compression, expected even 0-32 integer.");
        }
        if let Some(offset) = self.scale_offset {
            ensure!(offset <= c_int::max_value() as u32,
                    "Scale-offset factor too large, maximum is {}.", c_int::max_value());
        }
        if self.scale_offset.is_some() && self.fletcher32 {
            fail!("Cannot use lossy scale-offset filter with fletcher32.");
        }
        Ok(())
    }

    pub fn from_dcpl(dcpl: &PropertyList) -> Result<Filters> {
        let mut filters = Filters::default();
        h5lock!({
            let id = dcpl.id();
            let n_filters: c_int = h5try_s!(H5Pget_nfilters(id));

            for idx in 0..n_filters {
                let flags: *mut c_uint = &mut 0;
                let n_elements: *mut size_t = &mut 16;
                let mut values: Vec<c_uint> = Vec::with_capacity(16);
                values.set_len(16);
                let mut name: Vec<c_char> = Vec::with_capacity(256);
                name.set_len(256);
                let filter_config: *mut c_uint = &mut 0;

                let code = H5Pget_filter2(id, idx as c_uint, flags, n_elements, values.as_mut_ptr(),
                                          256, name.as_mut_ptr(), filter_config);
                name.push(0);

                match code {
                    H5Z_FILTER_DEFLATE => {
                        filters.gzip(values[0] as u8);
                    },
                    H5Z_FILTER_SZIP => {
                        let method = match values[0] {
                            v if v & H5_SZIP_EC_OPTION_MASK != 0 => Szip::EntropyCoding,
                            v if v & H5_SZIP_NN_OPTION_MASK != 0 => Szip::NearestNeighbor,
                            _ => fail!("Unknown szip method: {:?}", values[0]),
                        };
                        filters.szip(method, values[1] as u8);
                    },
                    H5Z_FILTER_SHUFFLE => {
                        filters.shuffle(true);
                    },
                    H5Z_FILTER_FLETCHER32 => {
                        filters.fletcher32(true);
                    },
                    H5Z_FILTER_SCALEOFFSET => {
                        filters.scale_offset(values[1]);
                    },
                    _ => fail!("Unsupported filter: {:?}", code),
                };
            }

            Ok(())
        }).and(filters.validate().and(Ok(filters)))
    }

    fn ensure_available(&self, name: &str, code: H5Z_filter_t) -> Result<()> {
        ensure!(h5lock!(H5Zfilter_avail(code) == 1), "Filter not available: {}", name);
        let flags: *mut c_uint = &mut 0;
        h5try!(H5Zget_filter_info(code, flags));
        ensure!(unsafe { *flags & H5Z_FILTER_CONFIG_ENCODE_ENABLED != 0 },
                "Encoding is not enabled for filter: {}", name);
        ensure!(unsafe { *flags & H5Z_FILTER_CONFIG_DECODE_ENABLED != 0 },
                "Decoding is not enabled for filter: {}", name);
        Ok(())
    }

    pub fn to_dcpl<D1: Dimension, D2: Dimension>(&self, datatype: &Datatype, shape: D1,
                                                   chunks: Option<D2>) -> Result<PropertyList> {
        try!(self.validate());

        h5lock!({
            let plist = try!(PropertyList::from_id(H5Pcreate(*H5P_DATASET_CREATE)));
            let id = plist.id();

            // chunks
            let mut chunks: Option<Vec<Ix>> = chunks.map(|x| x.dims());
            if chunks.is_none() && (self.gzip.is_some() || self.szip.is_some() || self.shuffle ||
                                    self.fletcher32 || self.scale_offset.is_some()) {
                chunks = Some(vec![]); // enable chunking automatically if needed
            }
            if let Some(chunks) = chunks {
                ensure!(shape.ndim() > 0, "Chunking cannot be enabled for scalar datasets.");
                let chunks = if chunks.ndim() == 0 {
                    infer_chunk_size(shape.clone(), datatype.size()) // automatic
                } else {
                    chunks
                };
                ensure!(chunks.ndim() == shape.ndim(),
                        "Invalid chunks ndim: expected {}, got {}", chunks.ndim(), shape.ndim());
                let dims: Vec<hsize_t> = chunks.iter().map(|&x| x as hsize_t).collect();
                h5try_s!(H5Pset_chunk(id, chunks.ndim() as c_int, dims.as_ptr()));
                h5try_s!(H5Pset_fill_time(id, H5D_FILL_TIME_ALLOC));
            }

            // fletcher32
            if self.fletcher32 {
                try!(self.ensure_available("fletcher32", H5Z_FILTER_FLETCHER32));
                H5Pset_fletcher32(id);
            }

            // scale-offset
            if let &Some(offset) = &self.scale_offset {
                try!(self.ensure_available("scaleoffset", H5Z_FILTER_SCALEOFFSET));
                match datatype {
                    &Datatype::Integer(_) => {
                        H5Pset_scaleoffset(id, H5Z_SO_INT, offset as c_int);
                    },
                    &Datatype::Float(_) => {
                        ensure!(offset > 0,
                                "Can only use positive scale-offset factor with floats.");
                        H5Pset_scaleoffset(id, H5Z_SO_FLOAT_DSCALE, offset as c_int);
                    },
                    // _ => {
                    //     fail!("Can only use scale/offset with integer/float datatypes.");
                    // }
                }
            }

            // shuffle
            if self.shuffle {
                try!(self.ensure_available("shuffle", H5Z_FILTER_SHUFFLE));
                h5try_s!(H5Pset_shuffle(id));
            }

            // compression
            if let Some(level) = self.gzip {
                try!(self.ensure_available("gzip", H5Z_FILTER_DEFLATE));
                h5try_s!(H5Pset_deflate(id, level as c_uint));
            } else if let Some((method, pixels_per_block)) = self.szip {
                try!(self.ensure_available("szip", H5Z_FILTER_SZIP));
                let options = match method {
                    Szip::EntropyCoding   => H5_SZIP_EC_OPTION_MASK,
                    Szip::NearestNeighbor => H5_SZIP_NN_OPTION_MASK,
                };
                h5try_s!(H5Pset_szip(id, options, pixels_per_block as c_uint));
            }

            Ok(plist)
        })
    }
}

pub fn infer_chunk_size<D: Dimension>(shape: D, typesize: usize) -> Vec<Ix> {
    // This algorithm is borrowed from h5py, though the idea originally comes from PyTables.

    const CHUNK_BASE: f64 = (16 * 1024) as f64;
    const CHUNK_MIN:  f64 = (8 * 1024) as f64;
    const CHUNK_MAX:  f64 = (1024 * 1024) as f64;

    let ndim = shape.ndim();
    if ndim == 0 {
        return vec![];
    }

    let mut chunks = shape.dims();
    let total = (typesize * shape.size()) as f64;
    let mut target: f64 = CHUNK_BASE * 2.0_f64.powf((total / (1024.0 * 1024.0)).log10());

    if target > CHUNK_MAX {
        target = CHUNK_MAX;
    } else if target < CHUNK_MIN {
        target = CHUNK_MIN;
    }

    // Loop over axes, dividing them by 2, stop when all of the following is true:
    // - chunk size is smaller than the target chunk size or is within 50% of target chunk size
    // - chunk size is smaller than the maximum chunk size
    for i in 0.. {
        let size = chunks.iter().fold(1, |acc, &el| acc * el);
        let bytes = (size * typesize) as f64;
        if (bytes < target * 1.5 && bytes < CHUNK_MAX) || size == 1 {
            break;
        }
        let axis = i % ndim;
        chunks[axis] = div_floor(chunks[axis] + 1, 2);
    }

    chunks
}

#[cfg(test)]
mod tests {
    use super::{Filters, Szip};
    use datatype::ToDatatype;
    use error::{Result, silence_errors};
    use ffi::h5z::{H5Z_FILTER_SZIP, H5Zfilter_avail};
    use num::Bounded;

    fn szip_available() -> bool {
        h5lock!(H5Zfilter_avail(H5Z_FILTER_SZIP) == 1)
    }

    fn make_filters<T: ToDatatype>(filters: &Filters) -> Result<Filters> {
        let datatype = T::to_datatype().unwrap();
        let shape = (10, 20);
        let chunks = Some(());
        let dcpl = try!(filters.to_dcpl(&datatype, &shape, chunks));
        Filters::from_dcpl(&dcpl)
    }

    fn check_roundtrip<T: ToDatatype>(filters: &Filters) {
        assert_eq!(make_filters::<T>(filters).unwrap(), *filters);
    }

    #[test]
    pub fn test_szip() {
        silence_errors();

        if !szip_available() {
            assert_err!(make_filters::<u32>(&Filters::new().szip_default()),
                        "Filter not available: szip");
        } else {
            assert!(Filters::new().get_szip().is_none());
            assert_eq!(Filters::new().szip(Szip::EntropyCoding, 4).get_szip(),
                       Some((Szip::EntropyCoding, 4)));
            assert!(Filters::new().szip(Szip::EntropyCoding, 4).no_szip().get_szip().is_none());
            assert_eq!(Filters::new().szip_default().get_szip(),
                       Some((Szip::NearestNeighbor, 8)));

            check_roundtrip::<u32>(Filters::new().no_szip());
            check_roundtrip::<u32>(Filters::new().szip(Szip::EntropyCoding, 4));
            check_roundtrip::<u32>(Filters::new().szip(Szip::NearestNeighbor, 4));

            check_roundtrip::<f32>(Filters::new().no_szip());
            check_roundtrip::<f32>(Filters::new().szip(Szip::EntropyCoding, 4));
            check_roundtrip::<f32>(Filters::new().szip(Szip::NearestNeighbor, 4));

            assert_err!(make_filters::<u32>(&Filters::new().szip(Szip::EntropyCoding, 1)),
                        "Invalid pixels per block for szip compression");
            assert_err!(make_filters::<u32>(&Filters::new().szip(Szip::NearestNeighbor, 34)),
                        "Invalid pixels per block for szip compression");
        }
    }

    #[test]
    pub fn test_gzip() {
        silence_errors();

        assert!(Filters::new().get_gzip().is_none());
        assert_eq!(Filters::new().gzip(7).get_gzip(), Some(7));
        assert!(Filters::new().gzip(7).no_gzip().get_gzip().is_none());
        assert_eq!(Filters::new().gzip_default().get_gzip(), Some(4));

        check_roundtrip::<u32>(Filters::new().no_gzip());
        check_roundtrip::<u32>(Filters::new().gzip(7));

        check_roundtrip::<f32>(Filters::new().no_gzip());
        check_roundtrip::<f32>(Filters::new().gzip(7));

        assert_err!(make_filters::<u32>(&Filters::new().gzip_default().szip_default()),
                    "Cannot specify two compression options at once");
        assert_err!(make_filters::<u32>(&Filters::new().gzip(42)),
                    "Invalid level for gzip compression");
    }

    #[test]
    pub fn test_shuffle() {
        assert!(!Filters::new().get_shuffle());
        assert!(Filters::new().shuffle(true).get_shuffle());
        assert!(!Filters::new().shuffle(true).shuffle(false).get_shuffle());

        check_roundtrip::<u32>(Filters::new().shuffle(false));
        check_roundtrip::<u32>(Filters::new().shuffle(true));

        check_roundtrip::<f32>(Filters::new().shuffle(false));
        check_roundtrip::<f32>(Filters::new().shuffle(true));
    }

    #[test]
    pub fn test_fletcher32() {
        assert!(!Filters::new().get_fletcher32());
        assert!(Filters::new().fletcher32(true).get_fletcher32());
        assert!(!Filters::new().fletcher32(true).fletcher32(false).get_fletcher32());

        check_roundtrip::<u32>(Filters::new().fletcher32(false));
        check_roundtrip::<u32>(Filters::new().fletcher32(true));

        check_roundtrip::<f32>(Filters::new().fletcher32(false));
        check_roundtrip::<f32>(Filters::new().fletcher32(true));
    }

    #[test]
    pub fn test_scale_offset() {
        silence_errors();

        assert!(Filters::new().get_scale_offset().is_none());
        assert_eq!(Filters::new().scale_offset(8).get_scale_offset(), Some(8));
        assert!(Filters::new().scale_offset(8).no_scale_offset().get_scale_offset().is_none());

        check_roundtrip::<u32>(Filters::new().no_scale_offset());
        check_roundtrip::<u32>(Filters::new().scale_offset(0));
        check_roundtrip::<u32>(Filters::new().scale_offset(8));

        check_roundtrip::<f32>(Filters::new().no_scale_offset());
        assert_err!(make_filters::<f32>(&Filters::new().scale_offset(0)),
                    "Can only use positive scale-offset factor with floats");
        check_roundtrip::<f32>(Filters::new().scale_offset(8));

        assert_err!(make_filters::<u32>(&Filters::new().scale_offset(u32::max_value())),
                    "Scale-offset factor too large");
        assert_err!(make_filters::<u32>(&Filters::new().scale_offset(0).fletcher32(true)),
                    "Cannot use lossy scale-offset filter with fletcher32");
    }

    #[test]
    pub fn test_filters() {
        let mut filters = Filters::new();
        filters.shuffle(true).gzip_default();
        let datatype = u32::to_datatype().unwrap();
        let dcpl = filters.to_dcpl(&datatype, (10, 20), Some(())).unwrap();
        let filters2 = Filters::from_dcpl(&dcpl).unwrap();
        assert_eq!(filters2, filters);
    }
}
