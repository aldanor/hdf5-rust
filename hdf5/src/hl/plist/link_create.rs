//! Link create properties.

use std::fmt::{self, Debug};
use std::ops::Deref;

use hdf5_sys::h5p::{
    H5Pcreate, H5Pget_char_encoding, H5Pget_create_intermediate_group, H5Pset_char_encoding,
    H5Pset_create_intermediate_group,
};
use hdf5_sys::h5t::{H5T_cset_t, H5T_CSET_ASCII, H5T_CSET_UTF8};

use crate::globals::H5P_LINK_CREATE;
use crate::internal_prelude::*;

/// Link create properties.
#[repr(transparent)]
pub struct LinkCreate(Handle);

impl ObjectClass for LinkCreate {
    const NAME: &'static str = "link create property list";
    const VALID_TYPES: &'static [H5I_type_t] = &[H5I_GENPROP_LST];

    fn from_handle(handle: Handle) -> Self {
        Self(handle)
    }

    fn handle(&self) -> &Handle {
        &self.0
    }

    fn validate(&self) -> Result<()> {
        ensure!(
            self.is_class(PropertyListClass::LinkCreate),
            "expected link create property list, got {:?}",
            self.class()
        );
        Ok(())
    }
}

impl Debug for LinkCreate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut formatter = f.debug_struct("LinkCreate");
        formatter.field("create_intermediate_group", &self.create_intermediate_group());
        formatter.field("char_encoding", &self.char_encoding());
        formatter.finish()
    }
}

impl Deref for LinkCreate {
    type Target = PropertyList;

    fn deref(&self) -> &PropertyList {
        unsafe { self.transmute() }
    }
}

impl PartialEq for LinkCreate {
    fn eq(&self, other: &Self) -> bool {
        <PropertyList as PartialEq>::eq(self, other)
    }
}

impl Eq for LinkCreate {}

impl Clone for LinkCreate {
    fn clone(&self) -> Self {
        unsafe { self.deref().clone().cast_unchecked() }
    }
}

/// The character encoding used to create a link or attribute name.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CharEncoding {
    /// US ASCII.
    Ascii,
    /// UTF-8.
    Utf8,
}

/// Builder used to create link create property list.
#[derive(Clone, Debug, Default)]
pub struct LinkCreateBuilder {
    create_intermediate_group: Option<bool>,
    char_encoding: Option<CharEncoding>,
}

impl LinkCreateBuilder {
    /// Creates a new link create property list builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new builder from an existing property list.
    pub fn from_plist(plist: &LinkCreate) -> Result<Self> {
        let mut builder = Self::default();
        builder.create_intermediate_group(plist.get_create_intermediate_group()?);
        builder.char_encoding(plist.get_char_encoding()?);
        Ok(builder)
    }

    /// Sets whether to create intermediate groups upon creation of an object.
    pub fn create_intermediate_group(&mut self, create: bool) -> &mut Self {
        self.create_intermediate_group = Some(create);
        self
    }

    /// Sets the character encoding to use when creating links.
    pub fn char_encoding(&mut self, encoding: CharEncoding) -> &mut Self {
        self.char_encoding = Some(encoding);
        self
    }

    fn populate_plist(&self, id: hid_t) -> Result<()> {
        if let Some(create) = self.create_intermediate_group {
            h5try!(H5Pset_create_intermediate_group(id, c_uint::from(create)));
        }
        if let Some(encoding) = self.char_encoding {
            let encoding = match encoding {
                CharEncoding::Ascii => H5T_CSET_ASCII,
                CharEncoding::Utf8 => H5T_CSET_UTF8,
            };
            h5try!(H5Pset_char_encoding(id, encoding));
        }
        Ok(())
    }

    /// Copies the builder settings into a link creation property list.
    pub fn apply(&self, plist: &mut LinkCreate) -> Result<()> {
        h5lock!(self.populate_plist(plist.id()))
    }

    /// Constructs a new link creation property list.
    pub fn finish(&self) -> Result<LinkCreate> {
        h5lock!({
            let mut plist = LinkCreate::try_new()?;
            self.apply(&mut plist).map(|()| plist)
        })
    }
}

/// Link create property list.
impl LinkCreate {
    /// Creates a new link creation property list.
    pub fn try_new() -> Result<Self> {
        Self::from_id(h5try!(H5Pcreate(*H5P_LINK_CREATE)))
    }

    /// Creates a copy of the link creation property list.
    pub fn copy(&self) -> Self {
        unsafe { self.deref().copy().cast_unchecked() }
    }

    /// Returns a builder for configuring a link creation property list.
    pub fn build() -> LinkCreateBuilder {
        LinkCreateBuilder::new()
    }

    #[doc(hidden)]
    pub fn get_create_intermediate_group(&self) -> Result<bool> {
        h5get!(H5Pget_create_intermediate_group(self.id()): c_uint).map(|x| x > 0)
    }

    /// Returns `true` if intermediate groups will be created upon object creation.
    pub fn create_intermediate_group(&self) -> bool {
        self.get_create_intermediate_group().unwrap_or(false)
    }

    #[doc(hidden)]
    pub fn get_char_encoding(&self) -> Result<CharEncoding> {
        Ok(match h5get!(H5Pget_char_encoding(self.id()): H5T_cset_t)? {
            H5T_CSET_ASCII => CharEncoding::Ascii,
            H5T_CSET_UTF8 => CharEncoding::Utf8,
            encoding => fail!("Unknown char encoding: {:?}", encoding),
        })
    }

    /// Returns the character encoding used to create links.
    pub fn char_encoding(&self) -> CharEncoding {
        self.get_char_encoding().unwrap_or(CharEncoding::Ascii)
    }
}
