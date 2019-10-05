use std::ptr;

use hdf5_sys::h5p::{
    H5Pget_filter2, H5Pget_nfilters, H5Pset_deflate, H5Pset_filter, H5Pset_fletcher32, H5Pset_nbit,
    H5Pset_scaleoffset, H5Pset_shuffle, H5Pset_szip,
};
use hdf5_sys::h5z::{
    H5Z_SO_scale_type_t, H5Z_filter_t, H5Zfilter_avail, H5Zget_filter_info,
    H5Z_FILTER_CONFIG_DECODE_ENABLED, H5Z_FILTER_CONFIG_ENCODE_ENABLED, H5Z_FILTER_DEFLATE,
    H5Z_FILTER_FLETCHER32, H5Z_FILTER_NBIT, H5Z_FILTER_SCALEOFFSET, H5Z_FILTER_SHUFFLE,
    H5Z_FILTER_SZIP, H5Z_FLAG_OPTIONAL, H5_SZIP_EC_OPTION_MASK, H5_SZIP_MAX_PIXELS_PER_BLOCK,
    H5_SZIP_NN_OPTION_MASK,
};

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
    Integer,
    FloatDScale,
}

#[cfg(feature = "blosc")]
mod blosc_impl {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Blosc {
        BloscLZ,
        LZ4,
        LZ4HC,
        Snappy,
        ZLib,
        ZStd,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum BloscShuffle {
        None,
        Byte,
        Bit,
    }

    impl Default for BloscShuffle {
        fn default() -> Self {
            BloscShuffle::Byte
        }
    }

    impl From<bool> for BloscShuffle {
        fn from(shuffle: bool) -> Self {
            if shuffle {
                BloscShuffle::Byte
            } else {
                BloscShuffle::None
            }
        }
    }

    impl Default for Blosc {
        fn default() -> Self {
            Blosc::BloscLZ
        }
    }

    pub fn blosc_get_nthreads() -> u8 {
        h5lock!(super::blosc::blosc_get_nthreads()).max(0).min(255) as _
    }

    pub fn blosc_set_nthreads(nthreads: u8) -> u8 {
        let nthreads = h5lock!(super::blosc::blosc_set_nthreads(nthreads as _));
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
    ScaleOffset(ScaleOffset, i8),
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

pub(crate) fn register_filters() -> Result<()> {
    #[cfg(feature = "lzf")]
    lzf::register_lzf()?;
    #[cfg(feature = "blosc")]
    blosc::register_blosc()?;
    Ok(())
}

/// Returns `true` if gzip filter is available.
pub fn gzip_available() -> bool {
    h5lock!(H5Zfilter_avail(H5Z_FILTER_DEFLATE) == 1)
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
            Filter::Deflate(_) => H5Z_FILTER_DEFLATE,
            Filter::Shuffle => H5Z_FILTER_SHUFFLE,
            Filter::Fletcher32 => H5Z_FILTER_FLETCHER32,
            Filter::SZip(_, _) => H5Z_FILTER_SZIP,
            Filter::NBit => H5Z_FILTER_NBIT,
            Filter::ScaleOffset(_, _) => H5Z_FILTER_SCALEOFFSET,
            #[cfg(feature = "lzf")]
            Filter::LZF => lzf::LZF_FILTER_ID,
            #[cfg(feature = "blosc")]
            Filter::Blosc(_, _, _) => blosc::BLOSC_FILTER_ID,
            Filter::User(id, _) => *id,
        }
    }

    pub fn get_info(filter_id: H5Z_filter_t) -> FilterInfo {
        if h5call!(H5Zfilter_avail(filter_id)).map(|x| x > 0).unwrap_or_default() {
            return FilterInfo::default();
        }
        let mut flags: c_uint = 0;
        h5lock!(H5Zget_filter_info(filter_id, &mut flags as *mut _));
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
        Filter::Deflate(level)
    }

    pub fn shuffle() -> Self {
        Filter::Shuffle
    }

    pub fn fletcher32() -> Self {
        Filter::Fletcher32
    }

    pub fn szip(coding: SZip, px_per_block: u8) -> Self {
        Filter::SZip(coding, px_per_block)
    }

    pub fn nbit() -> Self {
        Filter::NBit
    }

    pub fn scale_offset(mode: ScaleOffset, factor: i8) -> Self {
        Filter::ScaleOffset(mode, factor)
    }

    #[cfg(feature = "lzf")]
    pub fn lzf() -> Self {
        Filter::LZF
    }

    #[cfg(feature = "blosc")]
    pub fn blosc<T>(complib: Blosc, clevel: u8, shuffle: T) -> Self
    where
        T: Into<BloscShuffle>,
    {
        Filter::Blosc(complib, clevel, shuffle.into())
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
        Filter::User(id, cdata.to_vec())
    }

    fn parse_deflate(cdata: &[c_uint]) -> Result<Self> {
        ensure!(cdata.len() == 1, "expected length 1 cdata for deflate filter");
        ensure!(cdata[0] <= 9, "invalid deflate level: {}", cdata[0]);
        Ok(Self::deflate(cdata[0] as _))
    }

    fn parse_shuffle(cdata: &[c_uint]) -> Result<Self> {
        ensure!(cdata.len() == 0, "expected length 0 cdata for shuffle filter");
        Ok(Self::shuffle())
    }

    fn parse_fletcher32(cdata: &[c_uint]) -> Result<Self> {
        ensure!(cdata.len() == 0, "expected length 0 cdata for fletcher32 filter");
        Ok(Self::fletcher32())
    }

    fn parse_nbit(cdata: &[c_uint]) -> Result<Self> {
        ensure!(cdata.len() == 0, "expected length 0 cdata for nbit filter");
        Ok(Self::nbit())
    }

    fn parse_szip(cdata: &[c_uint]) -> Result<Self> {
        ensure!(cdata.len() == 2, "expected length 2 cdata for szip filter");
        let m = cdata[0];
        ensure!(
            (m & H5_SZIP_EC_OPTION_MASK != 0) != (m & H5_SZIP_NN_OPTION_MASK != 0),
            "invalid szip mask: {}: expected EC or NN to be set",
            m
        );
        let szip_coding =
            if m & H5_SZIP_EC_OPTION_MASK != 0 { SZip::Entropy } else { SZip::NearestNeighbor };
        let px_per_block = cdata[1];
        ensure!(
            px_per_block <= H5_SZIP_MAX_PIXELS_PER_BLOCK,
            "invalid pixels per block for szip filter: {}",
            px_per_block
        );
        Ok(Self::szip(szip_coding, px_per_block as _))
    }

    fn parse_scaleoffset(cdata: &[c_uint]) -> Result<Self> {
        ensure!(cdata.len() == 2, "expected length 2 cdata for scaleoffset filter");
        let scale_type = cdata[0];
        let scale_mode = if scale_type == (H5Z_SO_scale_type_t::H5Z_SO_INT as c_uint) {
            ScaleOffset::Integer
        } else if scale_type == (H5Z_SO_scale_type_t::H5Z_SO_FLOAT_DSCALE as c_uint) {
            ScaleOffset::FloatDScale
        } else {
            fail!("invalid scale type for scaleoffset filter: {}", cdata[0])
        };
        Ok(Self::scale_offset(scale_mode, cdata[1] as _))
    }

    #[cfg(feature = "lzf")]
    fn parse_lzf(cdata: &[c_uint]) -> Result<Self> {
        ensure!(cdata.len() == 0, "expected length 0 cdata for lzf filter");
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
        H5Pset_deflate(plist_id, level as _)
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
        H5Pset_szip(plist_id, mask, px_per_block as _)
    }

    unsafe fn apply_nbit(plist_id: hid_t) -> herr_t {
        H5Pset_nbit(plist_id)
    }

    unsafe fn apply_scaleoffset(plist_id: hid_t, mode: ScaleOffset, offset: i8) -> herr_t {
        let scale_type = match mode {
            ScaleOffset::Integer => H5Z_SO_scale_type_t::H5Z_SO_INT,
            ScaleOffset::FloatDScale => H5Z_SO_scale_type_t::H5Z_SO_FLOAT_DSCALE,
        };
        H5Pset_scaleoffset(plist_id, scale_type, offset as _)
    }

    #[cfg(feature = "lzf")]
    unsafe fn apply_lzf(plist_id: hid_t) -> herr_t {
        Self::apply_user(plist_id, lzf::LZF_FILTER_ID, &[])
    }

    #[cfg(feature = "blosc")]
    unsafe fn apply_blosc(
        plist_id: hid_t, complib: Blosc, clevel: u8, shuffle: BloscShuffle,
    ) -> herr_t {
        let mut cdata = vec![0 as c_uint; 7];
        cdata[4] = clevel as _;
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
        let cd_values = if cd_nelmts != 0 { cdata.as_ptr() } else { ptr::null() };
        H5Pset_filter(plist_id, filter_id, H5Z_FLAG_OPTIONAL, cd_nelmts, cd_values)
    }

    pub(crate) fn apply_to_plist(&self, id: hid_t) -> Result<()> {
        h5try!(match self {
            Filter::Deflate(level) => Self::apply_deflate(id, *level),
            Filter::Shuffle => Self::apply_shuffle(id),
            Filter::Fletcher32 => Self::apply_fletcher32(id),
            Filter::SZip(coding, px_per_block) => Self::apply_szip(id, *coding, *px_per_block),
            Filter::NBit => Self::apply_nbit(id),
            Filter::ScaleOffset(mode, offset) => Self::apply_scaleoffset(id, *mode, *offset),
            #[cfg(feature = "lzf")]
            Filter::LZF => Self::apply_lzf(id),
            #[cfg(feature = "blosc")]
            Filter::Blosc(complib, clevel, shuffle) => {
                Self::apply_blosc(id, *complib, *clevel, *shuffle)
            }
            Filter::User(filter_id, ref cdata) => Self::apply_user(id, *filter_id, cdata),
        });
        Ok(())
    }

    pub(crate) fn extract_pipeline(plist_id: hid_t) -> Result<Vec<Self>> {
        let mut filters = Vec::new();
        let mut name = vec![0 as c_char; 257];
        let mut cd_values = vec![0 as c_uint; 32];
        h5lock!({
            let n_filters = h5try!(H5Pget_nfilters(plist_id));
            for idx in 0..n_filters {
                let mut flags: c_uint = 0;
                let mut cd_nelmts: size_t = cd_values.len() as _;
                let filter_id = h5try!(H5Pget_filter2(
                    plist_id,
                    idx as _,
                    &mut flags as *mut _,
                    &mut cd_nelmts as *mut _,
                    cd_values.as_mut_ptr(),
                    name.len() as _,
                    name.as_mut_ptr(),
                    ptr::null_mut(),
                ));
                let cdata = &cd_values[..(cd_nelmts as _)];
                let flt = Self::from_raw(filter_id, cdata)
                    .ok()
                    .unwrap_or_else(|| Self::user(filter_id, cdata));
                filters.push(flt);
            }
            Ok(filters)
        })
    }
}
