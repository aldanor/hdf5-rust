use crate::globals::H5P_DATASET_CREATE;
use crate::internal_prelude::*;

use hdf5_sys::{
    h5p::{
        H5Pcreate, H5Pget_filter2, H5Pget_nfilters, H5Pset_deflate, H5Pset_fletcher32,
        H5Pset_scaleoffset, H5Pset_shuffle, H5Pset_szip,
    },
    h5t::{H5Tget_class, H5T_FLOAT, H5T_INTEGER},
    h5z::{
        H5Z_filter_t, H5Zfilter_avail, H5Zget_filter_info, H5Z_FILTER_CONFIG_DECODE_ENABLED,
        H5Z_FILTER_CONFIG_ENCODE_ENABLED, H5Z_FILTER_DEFLATE, H5Z_FILTER_FLETCHER32,
        H5Z_FILTER_SCALEOFFSET, H5Z_FILTER_SHUFFLE, H5Z_FILTER_SZIP, H5Z_SO_FLOAT_DSCALE,
        H5Z_SO_INT, H5_SZIP_EC_OPTION_MASK, H5_SZIP_NN_OPTION_MASK,
    },
};

/// Returns `true` if gzip filter is available.
pub fn gzip_available() -> bool {
    h5lock!(H5Zfilter_avail(H5Z_FILTER_DEFLATE) == 1)
}

/// Returns `true` if szip filter is available.
pub fn szip_available() -> bool {
    h5lock!(H5Zfilter_avail(H5Z_FILTER_SZIP) == 1)
}

/// HDF5 filters and compression options.
#[derive(Clone, PartialEq, Debug)]
pub struct Filters {
    gzip: Option<u8>,
    szip: Option<(bool, u8)>,
    shuffle: bool,
    fletcher32: bool,
    scale_offset: Option<u32>,
}

impl Default for Filters {
    fn default() -> Self {
        Self { gzip: None, szip: None, shuffle: false, fletcher32: false, scale_offset: None }
    }
}

impl Filters {
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable gzip compression with a specified level (0-9).
    pub fn gzip(&mut self, level: u8) -> &mut Self {
        self.gzip = Some(level);
        self
    }

    /// Disable gzip compression.
    pub fn no_gzip(&mut self) -> &mut Self {
        self.gzip = None;
        self
    }

    /// Get the current settings for gzip filter.
    pub fn get_gzip(&self) -> Option<u8> {
        self.gzip
    }

    /// Enable szip compression with a specified method (EC, NN) and level (0-32).
    ///
    /// If `nn` if set to `true` (default), the nearest neighbor method is used, otherwise
    /// the method is set to entropy coding.
    pub fn szip(&mut self, nn: bool, level: u8) -> &mut Self {
        self.szip = Some((nn, level));
        self
    }

    /// Disable szip compression.
    pub fn no_szip(&mut self) -> &mut Self {
        self.szip = None;
        self
    }

    /// Get the current settings for szip filter.
    ///
    /// Returns a tuple `(nn, level)`, where `nn` indicates whether the nearest neighbor
    /// method is used and `level` is the associated compression level.
    pub fn get_szip(&self) -> Option<(bool, u8)> {
        self.szip
    }

    /// Enable or disable shuffle filter.
    pub fn shuffle(&mut self, shuffle: bool) -> &mut Self {
        self.shuffle = shuffle;
        self
    }

    /// Get the current settings for shuffle filter.
    pub fn get_shuffle(&self) -> bool {
        self.shuffle
    }

    /// Enable or disable fletcher32 filter.
    pub fn fletcher32(&mut self, fletcher32: bool) -> &mut Self {
        self.fletcher32 = fletcher32;
        self
    }

    /// Get the current settings for fletcher32 filter.
    pub fn get_fletcher32(&self) -> bool {
        self.fletcher32
    }

    /// Enable scale-offset filter with a specified factor (0 means automatic).
    pub fn scale_offset(&mut self, scale_offset: u32) -> &mut Self {
        self.scale_offset = Some(scale_offset);
        self
    }

    /// Disable scale-offset compression.
    pub fn no_scale_offset(&mut self) -> &mut Self {
        self.scale_offset = None;
        self
    }

    /// Get the current settings for scale-offset filter.
    pub fn get_scale_offset(&self) -> Option<u32> {
        self.scale_offset
    }

    /// Enable gzip filter with default settings (compression level 4).
    pub fn gzip_default(&mut self) -> &mut Self {
        self.gzip = Some(4);
        self
    }

    /// Enable szip filter with default settings (NN method, compression level 8).
    pub fn szip_default(&mut self) -> &mut Self {
        self.szip = Some((true, 8));
        self
    }

    /// Returns `true` if any filters are enabled and thus chunkins is required.
    pub fn has_filters(&self) -> bool {
        self.gzip.is_some()
            || self.szip.is_some()
            || self.shuffle
            || self.fletcher32
            || self.scale_offset.is_some()
    }

    /// Verify whether the filters configuration is valid.
    pub fn validate(&self) -> Result<()> {
        if self.gzip.is_some() && self.szip.is_some() {
            fail!("Cannot specify two compression options at once.")
        }
        if let Some(level) = self.gzip {
            ensure!(level <= 9, "Invalid level for gzip compression, expected 0-9 integer.");
        }
        if let Some((_, pixels_per_block)) = self.szip {
            ensure!(
                pixels_per_block <= 32 && pixels_per_block % 2 == 0,
                "Invalid pixels per block for szip compression, expected even 0-32 integer."
            );
        }
        if let Some(offset) = self.scale_offset {
            ensure!(
                offset <= c_int::max_value() as _,
                "Scale-offset factor too large, maximum is {}.",
                c_int::max_value()
            );
        }
        if self.scale_offset.is_some() && self.fletcher32 {
            fail!("Cannot use lossy scale-offset filter with fletcher32.");
        }
        Ok(())
    }

    #[doc(hidden)]
    pub fn from_dcpl(dcpl: &PropertyList) -> Result<Self> {
        let mut filters = Self::default();
        h5lock!({
            let id = dcpl.id();
            let n_filters: c_int = h5try!(H5Pget_nfilters(id));

            for idx in 0..n_filters {
                let flags: *mut c_uint = &mut 0;
                let n_elements: *mut size_t = &mut 16;

                let mut values: Vec<c_uint> = Vec::with_capacity(16);
                values.set_len(16);

                let mut name: Vec<c_char> = Vec::with_capacity(256);
                name.set_len(256);

                let filter_config: *mut c_uint = &mut 0;

                let code = H5Pget_filter2(
                    id,
                    idx as _,
                    flags,
                    n_elements,
                    values.as_mut_ptr(),
                    256,
                    name.as_mut_ptr(),
                    filter_config,
                );
                name.push(0);

                match code {
                    H5Z_FILTER_DEFLATE => {
                        filters.gzip(values[0] as _);
                    }
                    H5Z_FILTER_SZIP => {
                        let nn = match values[0] {
                            v if v & H5_SZIP_EC_OPTION_MASK != 0 => false,
                            v if v & H5_SZIP_NN_OPTION_MASK != 0 => true,
                            _ => fail!("Unknown szip method: {:?}", values[0]),
                        };
                        filters.szip(nn, values[1] as _);
                    }
                    H5Z_FILTER_SHUFFLE => {
                        filters.shuffle(true);
                    }
                    H5Z_FILTER_FLETCHER32 => {
                        filters.fletcher32(true);
                    }
                    H5Z_FILTER_SCALEOFFSET => {
                        filters.scale_offset(values[1]);
                    }
                    _ => fail!("Unsupported filter: {:?}", code),
                };
            }

            Ok(())
        })
        .and(filters.validate().and(Ok(filters)))
    }

    fn ensure_available(name: &str, code: H5Z_filter_t) -> Result<()> {
        ensure!(h5lock!(H5Zfilter_avail(code) == 1), "Filter not available: {}", name);

        let flags: *mut c_uint = &mut 0;
        h5try!(H5Zget_filter_info(code, flags));

        ensure!(
            unsafe { *flags & H5Z_FILTER_CONFIG_ENCODE_ENABLED != 0 },
            "Encoding is not enabled for filter: {}",
            name
        );
        ensure!(
            unsafe { *flags & H5Z_FILTER_CONFIG_DECODE_ENABLED != 0 },
            "Decoding is not enabled for filter: {}",
            name
        );
        Ok(())
    }

    #[doc(hidden)]
    pub fn to_dcpl(&self, datatype: &Datatype) -> Result<PropertyList> {
        self.validate()?;

        h5lock!({
            let plist = PropertyList::from_id(H5Pcreate(*H5P_DATASET_CREATE))?;
            let id = plist.id();

            // fletcher32
            if self.fletcher32 {
                Self::ensure_available("fletcher32", H5Z_FILTER_FLETCHER32)?;
                H5Pset_fletcher32(id);
            }

            // scale-offset
            if let Some(offset) = self.scale_offset {
                Self::ensure_available("scaleoffset", H5Z_FILTER_SCALEOFFSET)?;
                match H5Tget_class(datatype.id()) {
                    H5T_INTEGER => {
                        H5Pset_scaleoffset(id, H5Z_SO_INT, offset as _);
                    }
                    H5T_FLOAT => {
                        ensure!(
                            offset > 0,
                            "Can only use positive scale-offset factor with floats"
                        );
                        H5Pset_scaleoffset(id, H5Z_SO_FLOAT_DSCALE, offset as _);
                    }
                    _ => {
                        fail!("Can only use scale/offset with integer/float datatypes.");
                    }
                }
            }

            // shuffle
            if self.shuffle {
                Self::ensure_available("shuffle", H5Z_FILTER_SHUFFLE)?;
                h5try!(H5Pset_shuffle(id));
            }

            // compression
            if let Some(level) = self.gzip {
                Self::ensure_available("gzip", H5Z_FILTER_DEFLATE)?;
                h5try!(H5Pset_deflate(id, c_uint::from(level)));
            } else if let Some((nn, pixels_per_block)) = self.szip {
                Self::ensure_available("szip", H5Z_FILTER_SZIP)?;
                let options = if nn { H5_SZIP_NN_OPTION_MASK } else { H5_SZIP_EC_OPTION_MASK };
                h5try!(H5Pset_szip(id, options, c_uint::from(pixels_per_block)));
            }

            Ok(plist)
        })
    }
}

#[cfg(test)]
pub mod tests {
    use super::{gzip_available, szip_available};
    use crate::internal_prelude::*;

    fn make_filters<T: H5Type>(filters: &Filters) -> Result<Filters> {
        let datatype = Datatype::from_type::<T>().unwrap();
        let dcpl = filters.to_dcpl(&datatype)?;
        Filters::from_dcpl(&dcpl)
    }

    fn check_roundtrip<T: H5Type>(filters: &Filters) {
        assert_eq!(make_filters::<T>(filters).unwrap(), *filters);
    }

    #[test]
    pub fn test_szip() {
        let _e = silence_errors();

        if !szip_available() {
            assert_err!(
                make_filters::<u32>(&Filters::new().szip_default()),
                "Filter not available: szip"
            );
        } else {
            assert!(Filters::new().get_szip().is_none());
            assert_eq!(Filters::new().szip(false, 4).get_szip(), Some((false, 4)));
            assert!(Filters::new().szip(false, 4).no_szip().get_szip().is_none());
            assert_eq!(Filters::new().szip_default().get_szip(), Some((true, 8)));

            check_roundtrip::<u32>(Filters::new().no_szip());
            check_roundtrip::<u32>(Filters::new().szip(false, 4));
            check_roundtrip::<u32>(Filters::new().szip(true, 4));

            check_roundtrip::<f32>(Filters::new().no_szip());
            check_roundtrip::<f32>(Filters::new().szip(false, 4));
            check_roundtrip::<f32>(Filters::new().szip(true, 4));

            assert_err!(
                make_filters::<u32>(&Filters::new().szip(false, 1)),
                "Invalid pixels per block for szip compression"
            );
            assert_err!(
                make_filters::<u32>(&Filters::new().szip(true, 34)),
                "Invalid pixels per block for szip compression"
            );
        }
    }

    #[test]
    pub fn test_gzip() {
        let _e = silence_errors();

        if !gzip_available() {
            assert_err!(
                make_filters::<u32>(&Filters::new().gzip_default()),
                "Filter not available: gzip"
            );
        } else {
            assert!(Filters::new().get_gzip().is_none());
            assert_eq!(Filters::new().gzip(7).get_gzip(), Some(7));
            assert!(Filters::new().gzip(7).no_gzip().get_gzip().is_none());
            assert_eq!(Filters::new().gzip_default().get_gzip(), Some(4));

            check_roundtrip::<u32>(Filters::new().no_gzip());
            check_roundtrip::<u32>(Filters::new().gzip(7));

            check_roundtrip::<f32>(Filters::new().no_gzip());
            check_roundtrip::<f32>(Filters::new().gzip(7));

            assert_err!(
                make_filters::<u32>(&Filters::new().gzip_default().szip_default()),
                "Cannot specify two compression options at once"
            );
            assert_err!(
                make_filters::<u32>(&Filters::new().gzip(42)),
                "Invalid level for gzip compression"
            );
        }
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
        let _e = silence_errors();

        assert!(Filters::new().get_scale_offset().is_none());
        assert_eq!(Filters::new().scale_offset(8).get_scale_offset(), Some(8));
        assert!(Filters::new().scale_offset(8).no_scale_offset().get_scale_offset().is_none());

        check_roundtrip::<u32>(Filters::new().no_scale_offset());
        check_roundtrip::<u32>(Filters::new().scale_offset(0));
        check_roundtrip::<u32>(Filters::new().scale_offset(8));

        check_roundtrip::<f32>(Filters::new().no_scale_offset());
        assert_err!(
            make_filters::<f32>(&Filters::new().scale_offset(0)),
            "Can only use positive scale-offset factor with floats"
        );
        check_roundtrip::<f32>(Filters::new().scale_offset(8));

        assert_err!(
            make_filters::<u32>(&Filters::new().scale_offset(u32::max_value())),
            "Scale-offset factor too large"
        );
        assert_err!(
            make_filters::<u32>(&Filters::new().scale_offset(0).fletcher32(true)),
            "Cannot use lossy scale-offset filter with fletcher32"
        );
    }

    #[test]
    pub fn test_filters_dcpl() {
        let mut filters = Filters::new();
        filters.shuffle(true);
        if gzip_available() {
            filters.gzip_default();
        }
        let datatype = Datatype::from_type::<u32>().unwrap();
        let dcpl = filters.to_dcpl(&datatype).unwrap();
        let filters2 = Filters::from_dcpl(&dcpl).unwrap();
        assert_eq!(filters2, filters);
    }

    #[test]
    pub fn test_has_filters() {
        assert_eq!(Filters::default().has_filters(), false);
        assert_eq!(Filters::default().gzip_default().has_filters(), true);
        assert_eq!(Filters::default().szip_default().has_filters(), true);
        assert_eq!(Filters::default().fletcher32(true).has_filters(), true);
        assert_eq!(Filters::default().shuffle(true).has_filters(), true);
        assert_eq!(Filters::default().scale_offset(2).has_filters(), true);
    }
}
