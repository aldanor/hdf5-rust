use std::collections::HashMap;
use std::ptr::{self, addr_of_mut};

use hdf5_sys::h5p::{
    H5Pget_filter2, H5Pget_nfilters, H5Pset_deflate, H5Pset_filter, H5Pset_fletcher32, H5Pset_nbit,
    H5Pset_scaleoffset, H5Pset_shuffle, H5Pset_szip,
};
use hdf5_sys::h5t::H5T_class_t;
use hdf5_sys::h5z::{
    H5Zfilter_avail, H5Zget_filter_info, H5Z_FILTER_CONFIG_DECODE_ENABLED,
    H5Z_FILTER_CONFIG_ENCODE_ENABLED, H5Z_FILTER_DEFLATE, H5Z_FILTER_FLETCHER32, H5Z_FILTER_NBIT,
    H5Z_FILTER_SCALEOFFSET, H5Z_FILTER_SHUFFLE, H5Z_FILTER_SZIP, H5Z_FLAG_OPTIONAL,
    H5Z_SO_FLOAT_DSCALE, H5Z_SO_INT, H5_SZIP_EC_OPTION_MASK, H5_SZIP_MAX_PIXELS_PER_BLOCK,
    H5_SZIP_NN_OPTION_MASK,
};

pub use hdf5_sys::h5z::H5Z_filter_t;

use crate::internal_prelude::*;

#[cfg(feature = "blosc")]
mod blosc;
#[cfg(feature = "lzf")]
mod lzf;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SZip {
    Entropy,
    NearestNeighbor,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScaleOffset {
    Integer(u16),
    FloatDScale(u8),
}

#[cfg(feature = "blosc")]
mod blosc_impl {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    #[cfg(feature = "blosc")]
    pub enum Blosc {
        BloscLZ,
        LZ4,
        LZ4HC,
        Snappy,
        ZLib,
        ZStd,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    #[cfg(feature = "blosc")]
    pub enum BloscShuffle {
        None,
        Byte,
        Bit,
    }

    #[cfg(feature = "blosc")]
    impl Default for BloscShuffle {
        fn default() -> Self {
            Self::Byte
        }
    }

    #[cfg(feature = "blosc")]
    impl From<bool> for BloscShuffle {
        fn from(shuffle: bool) -> Self {
            if shuffle {
                Self::Byte
            } else {
                Self::None
            }
        }
    }

    #[cfg(feature = "blosc")]
    impl Default for Blosc {
        fn default() -> Self {
            Self::BloscLZ
        }
    }

    #[cfg(feature = "blosc")]
    pub fn blosc_get_nthreads() -> u8 {
        h5lock!(super::blosc::blosc_get_nthreads()).max(0).min(255) as _
    }

    #[cfg(feature = "blosc")]
    pub fn blosc_set_nthreads(num_threads: u8) -> u8 {
        use std::os::raw::c_int;
        let nthreads = h5lock!(super::blosc::blosc_set_nthreads(c_int::from(num_threads)));
        nthreads.max(0).min(255) as _
    }
}

#[cfg(feature = "blosc")]
pub use blosc_impl::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Filter {
    Deflate(u8),
    Shuffle,
    Fletcher32,
    SZip(SZip, u8),
    NBit,
    ScaleOffset(ScaleOffset),
    #[cfg(feature = "lzf")]
    LZF,
    #[cfg(feature = "blosc")]
    Blosc(Blosc, u8, BloscShuffle),
    User(H5Z_filter_t, Vec<c_uint>),
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct FilterInfo {
    pub is_available: bool,
    pub encode_enabled: bool,
    pub decode_enabled: bool,
}

/// This function requires a synchronisation with other calls to `hdf5`
pub(crate) fn register_filters() {
    #[cfg(feature = "lzf")]
    if let Err(e) = lzf::register_lzf() {
        eprintln!("Error while registering LZF filter: {}", e);
    }
    #[cfg(feature = "blosc")]
    if let Err(e) = blosc::register_blosc() {
        eprintln!("Error while registering Blosc filter: {}", e);
    }
}

/// Returns `true` if deflate filter is available.
pub fn deflate_available() -> bool {
    h5lock!(H5Zfilter_avail(H5Z_FILTER_DEFLATE) == 1)
}

/// Returns `true` if deflate filter is available.
#[doc(hidden)]
#[deprecated(note = "deprecated; use deflate_available()")]
pub fn gzip_available() -> bool {
    deflate_available()
}

/// Returns `true` if szip filter is available.
pub fn szip_available() -> bool {
    h5lock!(H5Zfilter_avail(H5Z_FILTER_SZIP) == 1)
}

/// Returns `true` if LZF filter is available.
pub fn lzf_available() -> bool {
    h5lock!(H5Zfilter_avail(32000) == 1)
}

/// Returns `true` if Blosc filter is available.
pub fn blosc_available() -> bool {
    h5lock!(H5Zfilter_avail(32001) == 1)
}

impl Filter {
    pub fn id(&self) -> H5Z_filter_t {
        match self {
            Self::Deflate(_) => H5Z_FILTER_DEFLATE,
            Self::Shuffle => H5Z_FILTER_SHUFFLE,
            Self::Fletcher32 => H5Z_FILTER_FLETCHER32,
            Self::SZip(_, _) => H5Z_FILTER_SZIP,
            Self::NBit => H5Z_FILTER_NBIT,
            Self::ScaleOffset(_) => H5Z_FILTER_SCALEOFFSET,
            #[cfg(feature = "lzf")]
            Self::LZF => lzf::LZF_FILTER_ID,
            #[cfg(feature = "blosc")]
            Self::Blosc(_, _, _) => blosc::BLOSC_FILTER_ID,
            Self::User(id, _) => *id,
        }
    }

    pub fn get_info(filter_id: H5Z_filter_t) -> FilterInfo {
        if !h5call!(H5Zfilter_avail(filter_id)).map(|x| x > 0).unwrap_or_default() {
            return FilterInfo::default();
        }
        let mut flags: c_uint = 0;
        h5lock!(H5Zget_filter_info(filter_id, addr_of_mut!(flags)));
        FilterInfo {
            is_available: true,
            encode_enabled: flags & H5Z_FILTER_CONFIG_ENCODE_ENABLED != 0,
            decode_enabled: flags & H5Z_FILTER_CONFIG_DECODE_ENABLED != 0,
        }
    }

    pub fn is_available(&self) -> bool {
        Self::get_info(self.id()).is_available
    }

    pub fn encode_enabled(&self) -> bool {
        Self::get_info(self.id()).encode_enabled
    }

    pub fn decode_enabled(&self) -> bool {
        Self::get_info(self.id()).decode_enabled
    }

    pub fn deflate(level: u8) -> Self {
        Self::Deflate(level)
    }

    pub fn shuffle() -> Self {
        Self::Shuffle
    }

    pub fn fletcher32() -> Self {
        Self::Fletcher32
    }

    pub fn szip(coding: SZip, px_per_block: u8) -> Self {
        Self::SZip(coding, px_per_block)
    }

    pub fn nbit() -> Self {
        Self::NBit
    }

    pub fn scale_offset(mode: ScaleOffset) -> Self {
        Self::ScaleOffset(mode)
    }

    #[cfg(feature = "lzf")]
    pub fn lzf() -> Self {
        Self::LZF
    }

    #[cfg(feature = "blosc")]
    pub fn blosc<T>(complib: Blosc, clevel: u8, shuffle: T) -> Self
    where
        T: Into<BloscShuffle>,
    {
        Self::Blosc(complib, clevel, shuffle.into())
    }

    #[cfg(feature = "blosc")]
    pub fn blosc_blosclz<T>(clevel: u8, shuffle: T) -> Self
    where
        T: Into<BloscShuffle>,
    {
        Self::blosc(Blosc::BloscLZ, clevel, shuffle)
    }

    #[cfg(feature = "blosc")]
    pub fn blosc_lz4<T>(clevel: u8, shuffle: T) -> Self
    where
        T: Into<BloscShuffle>,
    {
        Self::blosc(Blosc::LZ4, clevel, shuffle)
    }

    #[cfg(feature = "blosc")]
    pub fn blosc_lz4hc<T>(clevel: u8, shuffle: T) -> Self
    where
        T: Into<BloscShuffle>,
    {
        Self::blosc(Blosc::LZ4HC, clevel, shuffle)
    }

    #[cfg(feature = "blosc")]
    pub fn blosc_snappy<T>(clevel: u8, shuffle: T) -> Self
    where
        T: Into<BloscShuffle>,
    {
        Self::blosc(Blosc::Snappy, clevel, shuffle)
    }

    #[cfg(feature = "blosc")]
    pub fn blosc_zlib<T>(clevel: u8, shuffle: T) -> Self
    where
        T: Into<BloscShuffle>,
    {
        Self::blosc(Blosc::ZLib, clevel, shuffle)
    }

    #[cfg(feature = "blosc")]
    pub fn blosc_zstd<T>(clevel: u8, shuffle: T) -> Self
    where
        T: Into<BloscShuffle>,
    {
        Self::blosc(Blosc::ZStd, clevel, shuffle)
    }

    pub fn user(id: H5Z_filter_t, cdata: &[c_uint]) -> Self {
        Self::User(id, cdata.to_vec())
    }

    fn parse_deflate(cdata: &[c_uint]) -> Result<Self> {
        ensure!(!cdata.is_empty(), "expected cdata.len() >= 1 for deflate filter");
        ensure!(cdata[0] <= 9, "invalid deflate level: {}", cdata[0]);
        Ok(Self::deflate(cdata[0] as _))
    }

    fn parse_shuffle(_cdata: &[c_uint]) -> Result<Self> {
        Ok(Self::shuffle())
    }

    fn parse_fletcher32(_cdata: &[c_uint]) -> Result<Self> {
        Ok(Self::fletcher32())
    }

    fn parse_nbit(_cdata: &[c_uint]) -> Result<Self> {
        Ok(Self::nbit())
    }

    fn parse_szip(cdata: &[c_uint]) -> Result<Self> {
        ensure!(cdata.len() >= 2, "expected cdata.len() >= 2 for szip filter");
        let m = cdata[0];
        ensure!(
            (m & H5_SZIP_EC_OPTION_MASK != 0) != (m & H5_SZIP_NN_OPTION_MASK != 0),
            "invalid szip mask: {}: expected EC or NN to be set",
            m
        );
        let szip_coding =
            if m & H5_SZIP_EC_OPTION_MASK == 0 { SZip::NearestNeighbor } else { SZip::Entropy };
        let px_per_block = cdata[1];
        ensure!(
            px_per_block <= H5_SZIP_MAX_PIXELS_PER_BLOCK,
            "invalid pixels per block for szip filter: {}",
            px_per_block
        );
        Ok(Self::szip(szip_coding, px_per_block as _))
    }

    fn parse_scaleoffset(cdata: &[c_uint]) -> Result<Self> {
        ensure!(cdata.len() >= 2, "expected cdata.len() >= 2 for scaleoffset filter");
        let scale_type = cdata[0];
        let mode = if scale_type == (H5Z_SO_INT as c_uint) {
            ensure!(
                cdata[1] <= c_uint::from(u16::max_value()),
                "invalid int scale-offset: {}",
                cdata[1]
            );
            ScaleOffset::Integer(cdata[1] as _)
        } else if scale_type == (H5Z_SO_FLOAT_DSCALE as c_uint) {
            ensure!(
                cdata[1] <= c_uint::from(u8::max_value()),
                "invalid float scale-offset: {}",
                cdata[1]
            );
            ScaleOffset::FloatDScale(cdata[1] as _)
        } else {
            fail!("invalid scale type for scaleoffset filter: {}", cdata[0])
        };
        Ok(Self::scale_offset(mode))
    }

    #[cfg(feature = "lzf")]
    fn parse_lzf(_cdata: &[c_uint]) -> Result<Self> {
        Ok(Self::lzf())
    }

    #[cfg(feature = "blosc")]
    fn parse_blosc(cdata: &[c_uint]) -> Result<Self> {
        ensure!(cdata.len() >= 5, "expected at least length 5 cdata for blosc filter");
        ensure!(cdata.len() <= 7, "expected at most length 7 cdata for blosc filter");
        ensure!(cdata[4] <= 9, "invalid blosc clevel: {}", cdata[4]);
        let clevel = cdata[4] as u8;
        let shuffle = if cdata.len() >= 6 {
            match cdata[5] {
                blosc::BLOSC_NOSHUFFLE => BloscShuffle::None,
                blosc::BLOSC_SHUFFLE => BloscShuffle::Byte,
                blosc::BLOSC_BITSHUFFLE => BloscShuffle::Bit,
                _ => fail!("invalid blosc shuffle: {}", cdata[5]),
            }
        } else {
            BloscShuffle::Byte
        };
        let complib = if cdata.len() >= 7 {
            match cdata[6] {
                blosc::BLOSC_BLOSCLZ => Blosc::BloscLZ,
                blosc::BLOSC_LZ4 => Blosc::LZ4,
                blosc::BLOSC_LZ4HC => Blosc::LZ4HC,
                blosc::BLOSC_SNAPPY => Blosc::Snappy,
                blosc::BLOSC_ZLIB => Blosc::ZLib,
                blosc::BLOSC_ZSTD => Blosc::ZStd,
                _ => fail!("invalid blosc complib: {}", cdata[6]),
            }
        } else {
            Blosc::BloscLZ
        };
        Ok(Self::blosc(complib, clevel, shuffle))
    }

    pub fn from_raw(filter_id: H5Z_filter_t, cdata: &[c_uint]) -> Result<Self> {
        ensure!(filter_id > 0, "invalid filter id: {}", filter_id);
        match filter_id {
            H5Z_FILTER_DEFLATE => Self::parse_deflate(cdata),
            H5Z_FILTER_SHUFFLE => Self::parse_shuffle(cdata),
            H5Z_FILTER_FLETCHER32 => Self::parse_fletcher32(cdata),
            H5Z_FILTER_SZIP => Self::parse_szip(cdata),
            H5Z_FILTER_NBIT => Self::parse_nbit(cdata),
            H5Z_FILTER_SCALEOFFSET => Self::parse_scaleoffset(cdata),
            #[cfg(feature = "lzf")]
            lzf::LZF_FILTER_ID => Self::parse_lzf(cdata),
            #[cfg(feature = "blosc")]
            blosc::BLOSC_FILTER_ID => Self::parse_blosc(cdata),
            _ => Ok(Self::user(filter_id, cdata)),
        }
    }

    unsafe fn apply_deflate(plist_id: hid_t, level: u8) -> herr_t {
        H5Pset_deflate(plist_id, c_uint::from(level))
    }

    unsafe fn apply_shuffle(plist_id: hid_t) -> herr_t {
        H5Pset_shuffle(plist_id)
    }

    unsafe fn apply_fletcher32(plist_id: hid_t) -> herr_t {
        H5Pset_fletcher32(plist_id)
    }

    unsafe fn apply_szip(plist_id: hid_t, coding: SZip, px_per_block: u8) -> herr_t {
        let mask = match coding {
            SZip::Entropy => H5_SZIP_EC_OPTION_MASK,
            SZip::NearestNeighbor => H5_SZIP_NN_OPTION_MASK,
        };
        H5Pset_szip(plist_id, mask, c_uint::from(px_per_block))
    }

    unsafe fn apply_nbit(plist_id: hid_t) -> herr_t {
        H5Pset_nbit(plist_id)
    }

    unsafe fn apply_scaleoffset(plist_id: hid_t, mode: ScaleOffset) -> herr_t {
        let (scale_type, factor) = match mode {
            ScaleOffset::Integer(bits) => (H5Z_SO_INT, c_int::from(bits)),
            ScaleOffset::FloatDScale(factor) => (H5Z_SO_FLOAT_DSCALE, c_int::from(factor)),
        };
        H5Pset_scaleoffset(plist_id, scale_type, factor)
    }

    #[cfg(feature = "lzf")]
    unsafe fn apply_lzf(plist_id: hid_t) -> herr_t {
        Self::apply_user(plist_id, lzf::LZF_FILTER_ID, &[])
    }

    #[cfg(feature = "blosc")]
    unsafe fn apply_blosc(
        plist_id: hid_t, complib: Blosc, clevel: u8, shuffle: BloscShuffle,
    ) -> herr_t {
        let mut cdata: Vec<c_uint> = vec![0; 7];
        cdata[4] = c_uint::from(clevel);
        cdata[5] = match shuffle {
            BloscShuffle::None => blosc::BLOSC_NOSHUFFLE,
            BloscShuffle::Byte => blosc::BLOSC_SHUFFLE,
            BloscShuffle::Bit => blosc::BLOSC_BITSHUFFLE,
        };
        cdata[6] = match complib {
            Blosc::BloscLZ => blosc::BLOSC_BLOSCLZ,
            Blosc::LZ4 => blosc::BLOSC_LZ4,
            Blosc::LZ4HC => blosc::BLOSC_LZ4HC,
            Blosc::Snappy => blosc::BLOSC_SNAPPY,
            Blosc::ZLib => blosc::BLOSC_ZLIB,
            Blosc::ZStd => blosc::BLOSC_ZSTD,
        };
        Self::apply_user(plist_id, blosc::BLOSC_FILTER_ID, &cdata)
    }

    unsafe fn apply_user(plist_id: hid_t, filter_id: H5Z_filter_t, cdata: &[c_uint]) -> herr_t {
        // We're setting custom filters to optional, same way h5py does it, since
        // the only mention of H5Z_FLAG_MANDATORY in the HDF5 source itself is
        // in H5Pset_fletcher32() in H5Pocpl.c; for all other purposes than
        // verifying checksums optional filter makes more sense than mandatory.
        let cd_nelmts = cdata.len() as _;
        let cd_values = if cd_nelmts == 0 { ptr::null() } else { cdata.as_ptr() };
        H5Pset_filter(plist_id, filter_id, H5Z_FLAG_OPTIONAL, cd_nelmts, cd_values)
    }

    pub(crate) fn apply_to_plist(&self, id: hid_t) -> Result<()> {
        h5try!(match self {
            Self::Deflate(level) => Self::apply_deflate(id, *level),
            Self::Shuffle => Self::apply_shuffle(id),
            Self::Fletcher32 => Self::apply_fletcher32(id),
            Self::SZip(coding, px_per_block) => Self::apply_szip(id, *coding, *px_per_block),
            Self::NBit => Self::apply_nbit(id),
            Self::ScaleOffset(mode) => Self::apply_scaleoffset(id, *mode),
            #[cfg(feature = "lzf")]
            Self::LZF => Self::apply_lzf(id),
            #[cfg(feature = "blosc")]
            Self::Blosc(complib, clevel, shuffle) => {
                Self::apply_blosc(id, *complib, *clevel, *shuffle)
            }
            Self::User(filter_id, ref cdata) => Self::apply_user(id, *filter_id, cdata),
        });
        Ok(())
    }

    pub(crate) fn extract_pipeline(plist_id: hid_t) -> Result<Vec<Self>> {
        let mut filters = Vec::new();
        let mut name: Vec<c_char> = vec![0; 257];
        let mut cd_values: Vec<c_uint> = vec![0; 32];
        h5lock!({
            let n_filters = h5try!(H5Pget_nfilters(plist_id));
            for idx in 0..n_filters {
                let mut flags: c_uint = 0;
                let mut cd_nelmts: size_t = cd_values.len() as _;
                let filter_id = h5try!(H5Pget_filter2(
                    plist_id,
                    idx as _,
                    addr_of_mut!(flags),
                    addr_of_mut!(cd_nelmts),
                    cd_values.as_mut_ptr(),
                    name.len() as _,
                    name.as_mut_ptr(),
                    ptr::null_mut(),
                ));
                let cdata = &cd_values[..(cd_nelmts as _)];
                let flt = Self::from_raw(filter_id, cdata)?;
                filters.push(flt);
            }
            Ok(filters)
        })
    }
}

const COMP_FILTER_IDS: &[H5Z_filter_t] = &[H5Z_FILTER_DEFLATE, H5Z_FILTER_SZIP, 32000, 32001];

pub(crate) fn validate_filters(filters: &[Filter], type_class: H5T_class_t) -> Result<()> {
    let mut map: HashMap<H5Z_filter_t, &Filter> = HashMap::new();
    let mut comp_filter: Option<&Filter> = None;

    for filter in filters {
        ensure!(filter.is_available(), "Filter not available: {:?}", filter);

        let id = filter.id();

        if let Some(f) = map.get(&id) {
            fail!("Duplicate filters: {:?} and {:?}", f, filter);
        } else if COMP_FILTER_IDS.contains(&id) {
            if let Some(comp_filter) = comp_filter {
                fail!("Multiple compression filters: {:?} and {:?}", comp_filter, filter);
            }
            comp_filter = Some(filter);
        } else if id == H5Z_FILTER_FLETCHER32 && map.contains_key(&H5Z_FILTER_SCALEOFFSET) {
            fail!("Lossy scale-offset filter before fletcher2 checksum filter");
        } else if let Filter::ScaleOffset(mode) = filter {
            match type_class {
                H5T_class_t::H5T_INTEGER | H5T_class_t::H5T_ENUM => {
                    if let ScaleOffset::FloatDScale(_) = mode {
                        fail!("Invalid scale-offset mode for integer type: {:?}", mode);
                    }
                }
                H5T_class_t::H5T_FLOAT => {
                    if let ScaleOffset::Integer(_) = mode {
                        fail!("Invalid scale-offset mode for float type: {:?}", mode);
                    }
                }
                _ => fail!("Can only use scale-offset with ints/floats, got: {:?}", type_class),
            }
        } else if matches!(filter, Filter::SZip(_, _)) {
            // https://github.com/h5py/h5py/issues/953
            if map.contains_key(&H5Z_FILTER_FLETCHER32) {
                fail!("Fletcher32 filter must be placed after szip filter");
            }
        } else if matches!(filter, Filter::Shuffle) {
            if let Some(comp_filter) = comp_filter {
                fail!("Shuffle filter placed after compression filter: {:?}", comp_filter);
            }
        }
        map.insert(id, filter);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use hdf5_sys::h5t::H5T_class_t;

    use super::{
        blosc_available, deflate_available, lzf_available, szip_available, validate_filters,
        Filter, FilterInfo, SZip, ScaleOffset,
    };
    use crate::test::with_tmp_file;
    use crate::{plist::DatasetCreate, Result};

    #[test]
    fn test_filter_pipeline() -> Result<()> {
        let mut comp_filters = vec![];
        if deflate_available() {
            comp_filters.push(Filter::deflate(3));
        }
        if szip_available() {
            comp_filters.push(Filter::szip(SZip::Entropy, 8));
        }
        assert_eq!(cfg!(feature = "lzf"), lzf_available());
        #[cfg(feature = "lzf")]
        {
            comp_filters.push(Filter::lzf());
        }
        assert_eq!(cfg!(feature = "blosc"), blosc_available());
        #[cfg(feature = "blosc")]
        {
            use super::BloscShuffle;
            comp_filters.push(Filter::blosc_blosclz(1, false));
            comp_filters.push(Filter::blosc_lz4(3, true));
            comp_filters.push(Filter::blosc_lz4hc(5, BloscShuffle::Bit));
            comp_filters.push(Filter::blosc_zlib(7, BloscShuffle::None));
            comp_filters.push(Filter::blosc_zstd(9, BloscShuffle::Byte));
            comp_filters.push(Filter::blosc_snappy(0, BloscShuffle::Bit));
        }
        for c in &comp_filters {
            assert!(c.is_available());
            assert!(c.encode_enabled());
            assert!(c.decode_enabled());

            let pipeline = vec![
                Filter::nbit(),
                Filter::shuffle(),
                c.clone(),
                Filter::fletcher32(),
                Filter::scale_offset(ScaleOffset::Integer(3)),
            ];
            validate_filters(&pipeline, H5T_class_t::H5T_INTEGER)?;

            let plist = DatasetCreate::try_new()?;
            for flt in &pipeline {
                flt.apply_to_plist(plist.id())?;
            }
            assert_eq!(Filter::extract_pipeline(plist.id())?, pipeline);

            let mut b = DatasetCreate::build();
            b.set_filters(&pipeline);
            b.chunk(10);
            let plist = b.finish()?;
            assert_eq!(Filter::extract_pipeline(plist.id())?, pipeline);

            let res = with_tmp_file(|file| {
                file.new_dataset_builder()
                    .empty::<i32>()
                    .shape((10_000, 20))
                    .with_dcpl(|p| p.set_filters(&pipeline))
                    .create("foo")
                    .unwrap();
                let plist = file.dataset("foo").unwrap().dcpl().unwrap();
                Filter::extract_pipeline(plist.id()).unwrap()
            });
            assert_eq!(res, pipeline);
        }

        let bad_filter = Filter::user(12_345, &[1, 2, 3, 4, 5]);
        assert_eq!(Filter::get_info(bad_filter.id()), FilterInfo::default());
        assert!(!bad_filter.is_available());
        assert!(!bad_filter.encode_enabled());
        assert!(!bad_filter.decode_enabled());
        assert_err!(
            validate_filters(&[bad_filter], H5T_class_t::H5T_INTEGER),
            "Filter not available"
        );

        Ok(())
    }
}
