use std::fmt::{self, Debug};

use crate::internal_prelude::*;

/// Any HDF5 object that can be referenced through an identifier.
#[repr(transparent)]
#[derive(Clone)]
pub struct Object(Handle);

impl ObjectClass for Object {
    const NAME: &'static str = "object";
    const VALID_TYPES: &'static [H5I_type_t] = &[];

    fn from_handle(handle: Handle) -> Self {
        Self(handle)
    }

    fn handle(&self) -> &Handle {
        &self.0
    }

    // TODO: short_repr()
}

impl Debug for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.debug_fmt(f)
    }
}

impl Object {
    pub fn id(&self) -> hid_t {
        self.0.id()
    }

    /// Returns reference count if the handle is valid and 0 otherwise.
    pub fn refcount(&self) -> u32 {
        self.handle().refcount()
    }

    /// Returns `true` if the object has a valid unlocked identifier (`false` for pre-defined
    /// locked identifiers like property list classes).
    pub fn is_valid(&self) -> bool {
        self.handle().is_valid_user_id()
    }

    /// Returns type of the object.
    pub fn id_type(&self) -> H5I_type_t {
        self.handle().id_type()
    }

    pub(crate) fn try_borrow(&self) -> Result<Handle> {
        Handle::try_borrow(self.id())
    }
}

macro_rules! impl_downcast {
    ($func:ident, $tp:ty) => {
        impl Object {
            #[doc = "Downcast the object into $tp if possible."]
            pub fn $func(&self) -> Result<$tp> {
                self.clone().cast()
            }
        }
    };
}

impl_downcast!(as_file, File);
impl_downcast!(as_group, Group);
impl_downcast!(as_dataset, Dataset);
impl_downcast!(as_location, Location);
impl_downcast!(as_attr, Attribute);
impl_downcast!(as_container, Container);
impl_downcast!(as_datatype, Datatype);
impl_downcast!(as_dataspace, Dataspace);
impl_downcast!(as_plist, PropertyList);

#[cfg(test)]
pub mod tests {
    use std::ops::Deref;

    use hdf5_sys::{h5i::H5I_type_t, h5p::H5Pcreate};

    use crate::globals::H5P_FILE_ACCESS;
    use crate::internal_prelude::*;

    pub struct TestObject(Handle);

    impl ObjectClass for TestObject {
        const NAME: &'static str = "test object";
        const VALID_TYPES: &'static [H5I_type_t] = &[];

        fn from_handle(handle: Handle) -> Self {
            Self(handle)
        }

        fn handle(&self) -> &Handle {
            &self.0
        }
    }

    impl Deref for TestObject {
        type Target = Object;

        fn deref(&self) -> &Object {
            unsafe { self.transmute() }
        }
    }

    impl TestObject {
        fn incref(&self) {
            self.0.incref()
        }

        fn decref(&self) {
            self.0.decref()
        }
    }

    #[test]
    pub fn test_not_a_valid_user_id() {
        assert_err!(TestObject::from_id(H5I_INVALID_HID), "Invalid handle id");
        assert_err!(TestObject::from_id(H5P_DEFAULT), "Invalid handle id");
    }

    #[test]
    pub fn test_new_user_id() {
        let obj = TestObject::from_id(h5call!(H5Pcreate(*H5P_FILE_ACCESS)).unwrap()).unwrap();
        assert!(obj.id() > 0);
        assert!(obj.is_valid());
        assert!(obj.handle().is_valid_id());
        assert_eq!(obj.id_type(), H5I_type_t::H5I_GENPROP_LST);

        assert_eq!(obj.refcount(), 1);
        obj.incref();
        assert_eq!(obj.refcount(), 2);
        obj.decref();
        assert_eq!(obj.refcount(), 1);
        obj.decref();
        h5lock!({
            obj.decref();
            assert_eq!(obj.refcount(), 0);
            assert!(!obj.is_valid());
            assert!(!obj.handle().is_valid_id());
            drop(obj);
        });
    }

    #[test]
    pub fn test_incref_decref_drop() {
        use std::mem::ManuallyDrop;
        let mut obj = TestObject::from_id(h5call!(H5Pcreate(*H5P_FILE_ACCESS)).unwrap()).unwrap();
        let obj_id = obj.id();
        obj = TestObject::from_id(h5call!(H5Pcreate(*H5P_FILE_ACCESS)).unwrap()).unwrap();
        assert_ne!(obj_id, obj.id());
        assert!(obj.id() > 0);
        assert!(obj.is_valid());
        assert!(obj.handle().is_valid_id());
        assert_eq!(obj.refcount(), 1);

        let obj2 = TestObject::from_id(obj.id()).unwrap();
        obj2.incref();
        assert_eq!(obj.refcount(), 2);
        assert_eq!(obj2.refcount(), 2);

        drop(obj2);
        assert!(obj.is_valid());
        assert_eq!(obj.refcount(), 1);

        // obj is already owned, we must ensure we do not call drop on this without
        // an incref
        let mut obj2 = ManuallyDrop::new(TestObject::from_id(obj.id()).unwrap());
        assert_eq!(obj.refcount(), 1);

        obj2.incref();
        // We can now take, as we have exactly two handles
        let obj2 = unsafe { ManuallyDrop::take(&mut obj2) };

        h5lock!({
            // We must hold a lock here to prevent another thread creating an object
            // with the same identifier as the one we just owned. Failing to do this
            // might lead to the wrong object being dropped.
            obj.decref();
            obj.decref();
            // We here have to dangling identifiers stored in obj and obj2. As this part
            // is locked we know some other object is not going to created with these
            // identifiers
            assert!(!obj.is_valid());
            assert!(!obj2.is_valid());
            // By manually dropping we don't close some other unrelated objects.
            // Dropping/closing an invalid object is allowed
            drop(obj);
            drop(obj2);
        });
    }
}
