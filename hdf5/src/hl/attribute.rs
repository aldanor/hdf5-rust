use std::fmt::{self, Debug};
use std::ops::Deref;
use std::ptr::addr_of_mut;

use hdf5_sys::{
    h5::{H5_index_t, H5_iter_order_t},
    h5a::{H5A_info_t, H5A_operator2_t, H5Acreate2, H5Adelete, H5Aiterate2},
};
use hdf5_types::TypeDescriptor;
use ndarray::ArrayView;

use crate::internal_prelude::*;

/// Represents the HDF5 attribute object.
#[repr(transparent)]
#[derive(Clone)]
pub struct Attribute(Handle);

impl ObjectClass for Attribute {
    const NAME: &'static str = "attribute";
    const VALID_TYPES: &'static [H5I_type_t] = &[H5I_ATTR];

    fn from_handle(handle: Handle) -> Self {
        Self(handle)
    }

    fn handle(&self) -> &Handle {
        &self.0
    }

    // TODO: short_repr()
}

impl Debug for Attribute {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.debug_fmt(f)
    }
}

impl Deref for Attribute {
    type Target = Container;

    fn deref(&self) -> &Container {
        unsafe { self.transmute() }
    }
}

impl Attribute {
    /// Returns names of all the members in the group, non-recursively.
    pub fn attr_names(obj: &Location) -> Result<Vec<String>> {
        extern "C" fn attributes_callback(
            _id: hid_t, attr_name: *const c_char, _info: *const H5A_info_t, op_data: *mut c_void,
        ) -> herr_t {
            std::panic::catch_unwind(|| {
                let other_data: &mut Vec<String> =
                    unsafe { &mut *(op_data.cast::<std::vec::Vec<std::string::String>>()) };
                other_data.push(string_from_cstr(attr_name));
                0 // Continue iteration
            })
            .unwrap_or(-1)
        }

        let callback_fn: H5A_operator2_t = Some(attributes_callback);
        let iteration_position: *mut hsize_t = &mut { 0_u64 };
        let mut result: Vec<String> = Vec::new();
        let other_data: *mut c_void = addr_of_mut!(result).cast();

        h5call!(H5Aiterate2(
            obj.handle().id(),
            H5_index_t::H5_INDEX_NAME,
            H5_iter_order_t::H5_ITER_INC,
            iteration_position,
            callback_fn,
            other_data
        ))?;

        Ok(result)
    }
}

#[derive(Clone)]
/// An attribute builder
pub struct AttributeBuilder {
    builder: AttributeBuilderInner,
}

impl AttributeBuilder {
    pub fn new(parent: &Location) -> Self {
        Self { builder: AttributeBuilderInner::new(parent) }
    }

    pub fn empty<T: H5Type>(self) -> AttributeBuilderEmpty {
        self.empty_as(&T::type_descriptor())
    }

    pub fn empty_as(self, type_desc: &TypeDescriptor) -> AttributeBuilderEmpty {
        AttributeBuilderEmpty { builder: self.builder, type_desc: type_desc.clone() }
    }

    pub fn with_data<'d, A, T, D>(self, data: A) -> AttributeBuilderData<'d, T, D>
    where
        A: Into<ArrayView<'d, T, D>>,
        T: H5Type,
        D: ndarray::Dimension,
    {
        self.with_data_as::<A, T, D>(data, &T::type_descriptor())
    }

    pub fn with_data_as<'d, A, T, D>(
        self, data: A, type_desc: &TypeDescriptor,
    ) -> AttributeBuilderData<'d, T, D>
    where
        A: Into<ArrayView<'d, T, D>>,
        T: H5Type,
        D: ndarray::Dimension,
    {
        AttributeBuilderData {
            builder: self.builder,
            data: data.into(),
            type_desc: type_desc.clone(),
            conv: Conversion::Soft,
        }
    }

    #[inline]
    #[must_use]
    pub fn packed(mut self, packed: bool) -> Self {
        self.builder.packed(packed);
        self
    }
}

#[derive(Clone)]
/// An attribute builder with the type known
pub struct AttributeBuilderEmpty {
    builder: AttributeBuilderInner,
    type_desc: TypeDescriptor,
}

impl AttributeBuilderEmpty {
    pub fn shape<S: Into<Extents>>(self, extents: S) -> AttributeBuilderEmptyShape {
        AttributeBuilderEmptyShape {
            builder: self.builder,
            type_desc: self.type_desc,
            extents: extents.into(),
        }
    }
    pub fn create<'n, T: Into<&'n str>>(self, name: T) -> Result<Attribute> {
        self.shape(()).create(name)
    }

    #[inline]
    #[must_use]
    pub fn packed(mut self, packed: bool) -> Self {
        self.builder.packed(packed);
        self
    }
}

#[derive(Clone)]
/// An attribute builder with type and shape known
pub struct AttributeBuilderEmptyShape {
    builder: AttributeBuilderInner,
    type_desc: TypeDescriptor,
    extents: Extents,
}

impl AttributeBuilderEmptyShape {
    pub fn create<'n, T: Into<&'n str>>(&self, name: T) -> Result<Attribute> {
        h5lock!(self.builder.create(&self.type_desc, name.into(), &self.extents))
    }

    #[inline]
    #[must_use]
    pub fn packed(mut self, packed: bool) -> Self {
        self.builder.packed(packed);
        self
    }
}

#[derive(Clone)]
/// An attribute builder with type, shape, and data known
pub struct AttributeBuilderData<'d, T, D> {
    builder: AttributeBuilderInner,
    data: ArrayView<'d, T, D>,
    type_desc: TypeDescriptor,
    conv: Conversion,
}

impl<'d, T, D> AttributeBuilderData<'d, T, D>
where
    T: H5Type,
    D: ndarray::Dimension,
{
    /// Set maximum allowed conversion level.
    pub fn conversion(mut self, conv: Conversion) -> Self {
        self.conv = conv;
        self
    }

    /// Disallow all conversions.
    pub fn no_convert(mut self) -> Self {
        self.conv = Conversion::NoOp;
        self
    }

    pub fn create<'n, N: Into<&'n str>>(&self, name: N) -> Result<Attribute> {
        ensure!(
            self.data.is_standard_layout(),
            "input array is not in standard layout or is not contiguous"
        ); // TODO: relax this when it's supported in the writer
        let extents = Extents::from(self.data.shape());
        let name = name.into();

        h5lock!({
            let dtype_src = Datatype::from_type::<T>()?;
            let dtype_dst = Datatype::from_descriptor(&self.type_desc)?;
            dtype_src.ensure_convertible(&dtype_dst, self.conv)?;
            let ds = self.builder.create(&self.type_desc, name, &extents)?;
            if let Err(err) = ds.write(self.data.view()) {
                self.builder.try_unlink(name);
                Err(err)
            } else {
                Ok(ds)
            }
        })
    }

    #[inline]
    #[must_use]
    pub fn packed(mut self, packed: bool) -> Self {
        self.builder.packed(packed);
        self
    }
}

#[derive(Clone)]
/// The true internal dataset builder
struct AttributeBuilderInner {
    parent: Result<Handle>,
    packed: bool,
}

impl AttributeBuilderInner {
    pub fn new(parent: &Location) -> Self {
        Self { parent: parent.try_borrow(), packed: false }
    }

    pub fn packed(&mut self, packed: bool) {
        self.packed = packed;
    }

    unsafe fn create(
        &self, desc: &TypeDescriptor, name: &str, extents: &Extents,
    ) -> Result<Attribute> {
        // construct in-file type descriptor; convert to packed representation if needed
        let desc = if self.packed { desc.to_packed_repr() } else { desc.to_c_repr() };

        let datatype = Datatype::from_descriptor(&desc)?;
        let parent = try_ref_clone!(self.parent);

        let dataspace = Dataspace::try_new(extents)?;

        let name = to_cstring(name)?;
        Attribute::from_id(h5try!(H5Acreate2(
            parent.id(),
            name.as_ptr(),
            datatype.id(),
            dataspace.id(),
            // these args are currently unused as if HDF5 1.12
            // see details: https://portal.hdfgroup.org/display/HDF5/H5A_CREATE2
            H5P_DEFAULT,
            H5P_DEFAULT,
        )))
    }

    fn try_unlink(&self, name: &str) {
        let name = to_cstring(name).unwrap();
        if let Ok(parent) = &self.parent {
            h5lock!(H5Adelete(parent.id(), name.as_ptr()));
        }
    }
}

#[cfg(test)]
pub mod attribute_tests {
    use crate::internal_prelude::*;
    use ndarray::{arr2, Array2};
    use std::str::FromStr;
    use types::VarLenUnicode;

    #[test]
    pub fn test_shape_ndim_size() {
        with_tmp_file(|file| {
            let d = file.new_attr::<f32>().shape((2, 3)).create("name1").unwrap();
            assert_eq!(d.shape(), vec![2, 3]);
            assert_eq!(d.size(), 6);
            assert_eq!(d.ndim(), 2);
            assert_eq!(d.is_scalar(), false);

            let d = file.new_attr::<u8>().shape(()).create("name2").unwrap();
            assert_eq!(d.shape(), vec![]);
            assert_eq!(d.size(), 1);
            assert_eq!(d.ndim(), 0);
            assert_eq!(d.is_scalar(), true);
        })
    }

    #[test]
    pub fn test_get_file_attr_names() {
        with_tmp_file(|file| {
            let _ = file.new_attr::<f32>().shape((2, 3)).create("name1").unwrap();
            let _ = file.new_attr::<u8>().shape(()).create("name2").unwrap();

            let attr_names = file.attr_names().unwrap();
            assert_eq!(attr_names.len(), 2);
            assert!(attr_names.contains(&"name1".to_string()));
            assert!(attr_names.contains(&"name2".to_string()));
        })
    }

    #[test]
    pub fn test_get_dataset_attr_names() {
        with_tmp_file(|file| {
            let ds = file.new_dataset::<u32>().shape((10, 10)).create("d1").unwrap();

            let _ = ds.new_attr::<f32>().shape((2, 3)).create("name1").unwrap();
            let _ = ds.new_attr::<u8>().shape(()).create("name2").unwrap();

            let attr_names = ds.attr_names().unwrap();
            assert_eq!(attr_names.len(), 2);
            assert!(attr_names.contains(&"name1".to_string()));
            assert!(attr_names.contains(&"name2".to_string()));
        })
    }

    #[test]
    pub fn test_datatype() {
        with_tmp_file(|file| {
            assert_eq!(
                file.new_attr::<f32>().shape(1).create("name").unwrap().dtype().unwrap(),
                Datatype::from_type::<f32>().unwrap()
            );
        })
    }

    #[test]
    pub fn test_read_write() {
        with_tmp_file(|file| {
            let arr = arr2(&[[1, 2, 3], [4, 5, 6]]);

            let attr = file.new_attr::<f32>().shape((2, 3)).create("foo").unwrap();
            attr.as_writer().write(&arr).unwrap();

            let read_attr = file.attr("foo").unwrap();
            assert_eq!(read_attr.shape(), vec![2, 3]);

            let arr_dyn: Array2<_> = read_attr.as_reader().read().unwrap();

            assert_eq!(arr, arr_dyn.into_dimensionality().unwrap());
        })
    }

    #[test]
    pub fn test_create() {
        with_tmp_file(|file| {
            let attr = file.new_attr::<u32>().shape((1, 2)).create("foo").unwrap();
            assert!(attr.is_valid());
            assert_eq!(attr.shape(), vec![1, 2]);
            // FIXME - attr.name() returns "/" here, which is the name the attribute is connected to,
            // not the name of the attribute.
            //assert_eq!(attr.name(), "foo");
            assert_eq!(file.attr("foo").unwrap().shape(), vec![1, 2]);
        })
    }

    #[test]
    pub fn test_create_with_data() {
        with_tmp_file(|file| {
            let arr = arr2(&[[1, 2, 3], [4, 5, 6]]);

            let attr = file.new_attr_builder().with_data(&arr).create("foo").unwrap();
            assert!(attr.is_valid());
            assert_eq!(attr.shape(), vec![2, 3]);
            // FIXME - attr.name() returns "/" here, which is the name the attribute is connected to,
            // not the name of the attribute.
            //assert_eq!(attr.name(), "foo");
            assert_eq!(file.attr("foo").unwrap().shape(), vec![2, 3]);

            let read_attr = file.attr("foo").unwrap();
            assert_eq!(read_attr.shape(), vec![2, 3]);
            let arr_dyn: Array2<_> = read_attr.as_reader().read().unwrap();
            assert_eq!(arr, arr_dyn.into_dimensionality().unwrap());
        })
    }

    #[test]
    pub fn test_missing() {
        with_tmp_file(|file| {
            let _ = file.new_attr::<u32>().shape((1, 2)).create("foo").unwrap();
            let missing_result = file.attr("bar");
            assert!(missing_result.is_err());
        })
    }

    #[test]
    pub fn test_write_read_str() {
        with_tmp_file(|file| {
            let s = VarLenUnicode::from_str("var len foo").unwrap();
            let attr = file.new_attr::<VarLenUnicode>().shape(()).create("foo").unwrap();
            attr.as_writer().write_scalar(&s).unwrap();
            let read_attr = file.attr("foo").unwrap();
            assert_eq!(read_attr.shape(), []);
            let r: VarLenUnicode = read_attr.as_reader().read_scalar().unwrap();
            assert_eq!(r, s);
        })
    }

    #[test]
    pub fn test_list_names() {
        with_tmp_file(|file| {
            let arr1 = arr2(&[[123], [456]]);
            let _attr1 = file.new_attr_builder().with_data(&arr1).create("foo").unwrap();
            let _attr2 = file.new_attr_builder().with_data("string").create("bar").unwrap();
            let attr_names = file.attr_names().unwrap();
            assert_eq!(attr_names.len(), 2);
            assert!(attr_names.contains(&"foo".to_string()));
            assert!(attr_names.contains(&"bar".to_string()));
        })
    }
}
