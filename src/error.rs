use libc::{c_uint, c_void};

use std::error::{Error, FromError};
use std::ops::Index;
use std::num::Int;
use std::ptr;

pub struct H5ErrorFrame {
    pub desc: String,
    pub func: String,
    pub major: String,
    pub minor: String
}

impl H5ErrorFrame {
    pub fn description(&self) -> &str {
        self.desc.as_slice()
    }

    pub fn detail(&self) -> Option<String> {
        Some(format!("Error in {}(): {} [{}: {}]",
             self.func, self.desc, self.major, self.minor).clone())
    }
}

pub fn silence_errors() {
    use ffi::h5e::{H5Eset_auto2, H5E_DEFAULT};

    h5lock! { unsafe {
        H5Eset_auto2(H5E_DEFAULT, None, ptr::null_mut::<c_void>())
    }};
}

#[test]
fn test_error_stack_query() {
    use ffi::types::hid_t;
    use ffi::h5p::{H5Pcreate, H5Pclose, H5P_CLS_ROOT_ID};

    let cls_id = *H5P_CLS_ROOT_ID;
    let mut plist_id: hid_t = 0;

    silence_errors();

    let result_no_error = h5lock! { unsafe {
        plist_id = H5Pcreate(cls_id);
        H5Pclose(plist_id);
        H5ErrorStack::query()
    }};
    assert!(result_no_error.ok().unwrap().is_none());

    let result_error = h5lock! {unsafe {
        H5Pclose(plist_id);
        H5ErrorStack::query()
    }};
    let stack = result_error.ok().unwrap().unwrap();
    assert_eq!(stack.description(), "can't close");
    assert_eq!(stack.detail().unwrap().as_slice(),
               "Error in H5Pclose(): can't close [Property lists: Unable to free object]");

    assert_eq!(stack.len(), 3);
    assert!(!stack.is_empty());

    assert_eq!(stack[0].description(), "can't close");
    assert_eq!(stack[0].detail().unwrap().as_slice(),
               "Error in H5Pclose(): can't close \
                [Property lists: Unable to free object]");

    assert_eq!(stack[1].description(), "can't decrement ID ref count");
    assert_eq!(stack[1].detail().unwrap().as_slice(),
               "Error in H5I_dec_app_ref(): can't decrement ID ref count \
                [Object atom: Unable to decrement reference count]");

    assert_eq!(stack[2].description(), "can't locate ID");
    assert_eq!(stack[2].detail().unwrap().as_slice(),
               "Error in H5I_dec_ref(): can't locate ID \
                [Object atom: Unable to find atom information (already closed?)]");

    let empty_stack = H5ErrorStack::new();
    assert!(empty_stack.is_empty());
    assert_eq!(empty_stack.len(), 0);
}

pub struct H5ErrorStack {
    frames: Vec<H5ErrorFrame>
}

impl Index<usize> for H5ErrorStack {
    type Output = H5ErrorFrame;

    fn index<'a>(&'a self, index: &usize) -> &'a H5ErrorFrame {
        &self.frames[*index]
    }
}

impl H5ErrorStack {
    // This low-level function is not thread-safe and has to be synchronized by the user
    pub fn query() -> H5Result<Option<H5ErrorStack>> {
        use ffi::types::herr_t;
        use ffi::util::{get_h5_str, str_from_c};
        use ffi::h5e::{H5Ewalk2, H5Eget_msg, H5E_error2_t, H5E_type_t, H5E_WALK_DOWNWARD,
                       H5Eget_current_stack, H5Eclose_stack};

        struct CallbackData {
            stack: H5ErrorStack,
            err: Option<H5Error>,
        }

        extern fn callback(_: c_uint, err_desc: *const H5E_error2_t, data: *mut c_void) -> herr_t {
            unsafe {
                let data = &mut *(data as *mut CallbackData);
                if data.err.is_some() {
                    return 0;
                }
                let closure = |&: e: H5E_error2_t| -> H5Result<H5ErrorFrame> {
                    let (desc, func) = (str_from_c(e.desc), str_from_c(e.func_name));
                    let major = try!(get_h5_str(|m, s| {
                        H5Eget_msg(e.maj_num, ptr::null_mut::<H5E_type_t>(), m, s)
                    }));
                    let minor = try!(get_h5_str(|m, s| {
                        H5Eget_msg(e.min_num, ptr::null_mut::<H5E_type_t>(), m, s)
                    }));
                    Ok(H5ErrorFrame { desc: desc, func: func, major: major, minor: minor })
                };
                match closure(*err_desc) {
                    Ok(frame) => { data.stack.push(frame); 0 },
                    Err(err)  => { data.err = Some(FromError::from_error(err)); 0 },
                }
            }
        }

        let mut data = CallbackData { stack: H5ErrorStack::new(), err: None };
        let data_ptr: *mut c_void = &mut data as *mut _ as *mut c_void;

        // HDF5 bug: H5Eget_msg() may corrupt the current stack, so copy it first
        unsafe {
            let stack_id = H5Eget_current_stack();
            ensure!(stack_id >= 0, "failed to copy the current error stack");
            H5Ewalk2(stack_id, H5E_WALK_DOWNWARD, Some(callback), data_ptr);
            H5Eclose_stack(stack_id)
        };

        match (data.err, data.stack.is_empty()) {
            (Some(err), _) => Err(err),
            (None, false)  => Ok(Some(data.stack)),
            (None, true)   => Ok(None),
        }
    }

    pub fn new() -> H5ErrorStack {
        H5ErrorStack { frames: Vec::new() }
    }

    pub fn len(&self) -> usize {
        self.frames.len()
    }

    pub fn push(&mut self, frame: H5ErrorFrame) {
        self.frames.push(frame)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn top(&self) -> Option<&H5ErrorFrame> {
        match !self.is_empty() {
            false => None,
            true  => Some(&self.frames[0]),
        }
    }

    pub fn description(&self) -> &str {
        match self.top() {
            None        => "unknown library error",
            Some(frame) => frame.description(),
        }
    }

    pub fn detail(&self) -> Option<String> {
        match self.top() {
            None        => None,
            Some(frame) => frame.detail(),
        }
    }
}

pub enum H5Error {
    LibraryError(H5ErrorStack),
    InternalError(&'static str),
}

pub type H5Result<T> = Result<T, H5Error>;

impl H5Error {
    pub fn query() -> Option<H5Error> {
        match H5ErrorStack::query() {
            Err(err)        => Some(FromError::from_error(err)),
            Ok(Some(stack)) => Some(H5Error::LibraryError(stack)),
            Ok(None)        => None,
        }
    }
}

impl FromError<&'static str> for H5Error {
    fn from_error(desc: &'static str) -> H5Error {
        H5Error::InternalError(desc)
    }
}

impl Error for H5Error {
    fn description(&self) -> &str {
        match *self {
            H5Error::InternalError(desc)     => desc,
            H5Error::LibraryError(ref stack) => stack.description(),
        }
    }

    fn detail(&self) -> Option<String> {
        match *self {
            H5Error::InternalError(_)        => None,
            H5Error::LibraryError(ref stack) => stack.detail()
        }
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}

#[test]
fn test_h5check() {
    use ffi::h5p::{H5Pcreate, H5Pclose, H5P_CLS_ROOT_ID};

    silence_errors();

    let plist_id = h5lock! { unsafe {
        H5Pcreate(*H5P_CLS_ROOT_ID)
    }};

    let result_no_error = h5lock! {
        h5check(|| unsafe { H5Pclose(plist_id) })
    };
    assert!(result_no_error.is_ok());

    let result_error = h5lock! {
        h5check(|| unsafe { H5Pclose(plist_id) })
    };
    assert!(result_error.is_err());
}


pub fn h5check<T, F>(func: F) -> H5Result<T> where F: FnOnce() -> T, T: Int,
{
    use sync::h5sync;

    let min_value: T = Int::min_value();
    let zero:      T = Int::zero();
    let value:     T = func();

    let maybe_error = if min_value < zero {
        value < zero
    } else {
        value == zero
    };

    match maybe_error {
        false => Ok(value),
        true  => match H5Error::query() {
            None       => Ok(value),
            Some(err)  => Err(FromError::from_error(err)),
        },
    }
}


macro_rules! h5try {
    ($expr:expr) => {
        match h5check(unsafe { $expr }) {
            Ok(value) => Ok(value),
            Err(err) => return Err(::std::error::FromError::from_error(err)),
        }
    }
}
