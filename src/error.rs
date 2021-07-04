use std::error::Error as StdError;
use std::fmt;
use std::ops::Index;
use std::panic;
use std::ptr;

use ndarray::ShapeError;

#[cfg(not(hdf5_1_10_0))]
use hdf5_sys::h5::hssize_t;
use hdf5_sys::h5e::{
    H5E_error2_t, H5Eclose_stack, H5Eget_current_stack, H5Eget_msg, H5Ewalk2, H5E_WALK_DOWNWARD,
};

use crate::internal_prelude::*;

pub mod h5 {
    use super::*;
    #[repr(transparent)]
    #[derive(Clone)]
    pub struct ErrorStack(Handle);

    impl ObjectClass for ErrorStack {
        const NAME: &'static str = "errorstack";
        const VALID_TYPES: &'static [H5I_type_t] = &[H5I_ERROR_STACK];

        fn from_handle(handle: Handle) -> Self {
            Self(handle)
        }

        fn handle(&self) -> &Handle {
            &self.0
        }

        // TODO: short_repr()
    }

    impl ErrorStack {
        pub(crate) fn from_current() -> Result<Self> {
            let stack_id = h5lock!(H5Eget_current_stack());
            Handle::try_new(stack_id).map(Self)
        }

        pub(crate) fn into_stack(self) -> Result<super::ErrorStack> {
            struct CallbackData {
                stack: super::ErrorStack,
                err: Option<Error>,
            }
            extern "C" fn callback(
                _: c_uint, err_desc: *const H5E_error2_t, data: *mut c_void,
            ) -> herr_t {
                panic::catch_unwind(|| unsafe {
                    let data = &mut *(data.cast::<CallbackData>());
                    if data.err.is_some() {
                        return 0;
                    }
                    let closure = |e: H5E_error2_t| -> Result<ErrorFrame> {
                        let (desc, func) =
                            (string_from_cstr(e.desc), string_from_cstr(e.func_name));
                        let major =
                            get_h5_str(|m, s| H5Eget_msg(e.maj_num, ptr::null_mut(), m, s))?;
                        let minor =
                            get_h5_str(|m, s| H5Eget_msg(e.min_num, ptr::null_mut(), m, s))?;
                        Ok(ErrorFrame::new(&desc, &func, &major, &minor))
                    };
                    match closure(*err_desc) {
                        Ok(frame) => {
                            data.stack.push(frame);
                        }
                        Err(err) => {
                            data.err = Some(err);
                        }
                    }
                    0
                })
                .unwrap_or(-1)
            }

            let mut data = CallbackData { stack: super::ErrorStack::new(), err: None };
            let data_ptr: *mut c_void = (&mut data as *mut CallbackData).cast::<c_void>();

            let stack_id = self.handle().id();
            h5lock!({
                H5Ewalk2(stack_id, H5E_WALK_DOWNWARD, Some(callback), data_ptr);
            });

            if let Some(err) = data.err {
                Err(err)
            } else {
                Ok(data.stack)
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct ErrorFrame {
    desc: String,
    func: String,
    major: String,
    minor: String,
    description: String,
}

impl ErrorFrame {
    pub fn new(desc: &str, func: &str, major: &str, minor: &str) -> Self {
        Self {
            desc: desc.into(),
            func: func.into(),
            major: major.into(),
            minor: minor.into(),
            description: format!("{}(): {}", func, desc),
        }
    }

    pub fn desc(&self) -> &str {
        self.desc.as_ref()
    }

    pub fn description(&self) -> &str {
        self.description.as_ref()
    }

    pub fn detail(&self) -> Option<String> {
        Some(format!("Error in {}(): {} [{}: {}]", self.func, self.desc, self.major, self.minor))
    }
}

#[derive(Clone, Debug)]
pub struct ErrorStack {
    frames: Vec<ErrorFrame>,
    description: Option<String>,
}

impl Index<usize> for ErrorStack {
    type Output = ErrorFrame;

    fn index(&self, index: usize) -> &ErrorFrame {
        &self.frames[index]
    }
}

impl Default for ErrorStack {
    fn default() -> Self {
        Self::new()
    }
}

impl ErrorStack {
    // This low-level function is not thread-safe and has to be synchronized by the user
    pub fn query() -> Result<Option<Self>> {
        struct CallbackData {
            stack: ErrorStack,
            err: Option<Error>,
        }
        extern "C" fn callback(
            _: c_uint, err_desc: *const H5E_error2_t, data: *mut c_void,
        ) -> herr_t {
            panic::catch_unwind(|| unsafe {
                let data = &mut *(data.cast::<CallbackData>());
                if data.err.is_some() {
                    return 0;
                }
                let closure = |e: H5E_error2_t| -> Result<ErrorFrame> {
                    let (desc, func) = (string_from_cstr(e.desc), string_from_cstr(e.func_name));
                    let major = get_h5_str(|m, s| H5Eget_msg(e.maj_num, ptr::null_mut(), m, s))?;
                    let minor = get_h5_str(|m, s| H5Eget_msg(e.min_num, ptr::null_mut(), m, s))?;
                    Ok(ErrorFrame::new(&desc, &func, &major, &minor))
                };
                match closure(*err_desc) {
                    Ok(frame) => {
                        data.stack.push(frame);
                    }
                    Err(err) => {
                        data.err = Some(err);
                    }
                }
                0
            })
            .unwrap_or(-1)
        }

        let mut data = CallbackData { stack: Self::new(), err: None };
        let data_ptr: *mut c_void = (&mut data as *mut CallbackData).cast::<c_void>();

        // known HDF5 bug: H5Eget_msg() may corrupt the current stack, so we copy it first
        let stack_id = h5lock!(H5Eget_current_stack());
        ensure!(stack_id >= 0, "failed to copy the current error stack");
        h5lock!({
            H5Ewalk2(stack_id, H5E_WALK_DOWNWARD, Some(callback), data_ptr);
            H5Eclose_stack(stack_id);
        });

        match (data.err, data.stack.is_empty()) {
            (Some(err), _) => Err(err),
            (None, false) => Ok(Some(data.stack)),
            (None, true) => Ok(None),
        }
    }

    pub fn new() -> Self {
        Self { frames: Vec::new(), description: None }
    }

    pub fn len(&self) -> usize {
        self.frames.len()
    }

    pub fn push(&mut self, frame: ErrorFrame) {
        self.frames.push(frame);
        if !self.is_empty() {
            let top_desc = self.frames[0].description().to_owned();
            if self.len() == 1 {
                self.description = Some(top_desc);
            } else {
                self.description =
                    Some(format!("{}: {}", top_desc, self.frames[self.len() - 1].desc()));
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    pub fn top(&self) -> Option<&ErrorFrame> {
        if self.is_empty() {
            None
        } else {
            Some(&self.frames[0])
        }
    }

    pub fn description(&self) -> &str {
        match self.description {
            None => "unknown library error",
            Some(ref desc) => desc.as_ref(),
        }
    }

    pub fn detail(&self) -> Option<String> {
        self.top().and_then(ErrorFrame::detail)
    }
}

/// The error type for HDF5-related functions.
#[derive(Clone)]
pub enum Error {
    /// An error occurred in the C API of the HDF5 library. Full error stack is captured.
    HDF5(h5::ErrorStack),
    /// A user error occurred in the high-level Rust API (e.g., invalid user input).
    Internal(String),
}

/// A type for results generated by HDF5-related functions where the `Err` type is
/// set to `hdf5::Error`.
pub type Result<T, E = Error> = ::std::result::Result<T, E>;

impl Error {
    pub fn query() -> Result<Self> {
        if let Ok(stack) = h5::ErrorStack::from_current() {
            Ok(Self::HDF5(stack))
        } else {
            Err(Self::Internal("Could not get errorstack".to_owned()))
        }
    }
}

impl From<&str> for Error {
    fn from(desc: &str) -> Self {
        Self::Internal(desc.into())
    }
}

impl From<String> for Error {
    fn from(desc: String) -> Self {
        Self::Internal(desc)
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::Internal(ref desc) => f.write_str(desc),
            Self::HDF5(ref stack) => match stack.clone().into_stack() {
                Ok(stack) => f.write_str(stack.description()),
                Err(_) => f.write_str("Could not get error stack"),
            },
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::Internal(ref desc) => f.write_str(desc),
            Self::HDF5(ref stack) => match stack.clone().into_stack() {
                Ok(stack) => f.write_str(stack.description()),
                Err(_) => f.write_str("Could not get error stack"),
            },
        }
    }
}

impl StdError for Error {}

impl From<ShapeError> for Error {
    fn from(err: ShapeError) -> Self {
        format!("shape error: {}", err.to_string()).into()
    }
}

pub fn h5check<T: H5ErrorCode>(value: T) -> Result<T> {
    H5ErrorCode::h5check(value)
}

#[allow(unused)]
pub fn is_err_code<T: H5ErrorCode>(value: T) -> bool {
    H5ErrorCode::is_err_code(value)
}

pub trait H5ErrorCode: Copy {
    fn is_err_code(value: Self) -> bool;

    fn h5check(value: Self) -> Result<Self> {
        if Self::is_err_code(value) {
            Err(Error::query().unwrap_or_else(|e| e))
        } else {
            Ok(value)
        }
    }
}

impl H5ErrorCode for hsize_t {
    fn is_err_code(value: Self) -> bool {
        value == 0
    }
}

impl H5ErrorCode for herr_t {
    fn is_err_code(value: Self) -> bool {
        value < 0
    }
}

#[cfg(hdf5_1_10_0)]
impl H5ErrorCode for hid_t {
    fn is_err_code(value: Self) -> bool {
        value < 0
    }
}

#[cfg(not(hdf5_1_10_0))]
impl H5ErrorCode for hssize_t {
    fn is_err_code(value: Self) -> bool {
        value < 0
    }
}

impl H5ErrorCode for libc::ssize_t {
    fn is_err_code(value: Self) -> bool {
        value < 0
    }
}

#[cfg(test)]
pub mod tests {
    use hdf5_sys::h5p::{H5Pclose, H5Pcreate};

    use crate::globals::H5P_ROOT;
    use crate::internal_prelude::*;

    use super::ErrorStack;

    #[test]
    pub fn test_error_stack() {
        let result_no_error = h5lock!({
            let plist_id = H5Pcreate(*H5P_ROOT);
            H5Pclose(plist_id);
            ErrorStack::query()
        });
        assert!(result_no_error.ok().unwrap().is_none());

        let result_error = h5lock!({
            let plist_id = H5Pcreate(*H5P_ROOT);
            H5Pclose(plist_id);
            H5Pclose(plist_id);
            ErrorStack::query()
        });
        let stack = result_error.ok().unwrap().unwrap();
        assert_eq!(stack.description(), "H5Pclose(): can't close: can't locate ID");
        assert_eq!(
            &stack.detail().unwrap(),
            "Error in H5Pclose(): can't close [Property lists: Unable to free object]"
        );

        assert!(stack.len() >= 2 && stack.len() <= 3); // depending on HDF5 version
        assert!(!stack.is_empty());

        assert_eq!(stack[0].description(), "H5Pclose(): can't close");
        assert_eq!(
            &stack[0].detail().unwrap(),
            "Error in H5Pclose(): can't close \
             [Property lists: Unable to free object]"
        );

        assert_eq!(stack[stack.len() - 1].description(), "H5I_dec_ref(): can't locate ID");
        assert_eq!(
            &stack[stack.len() - 1].detail().unwrap(),
            "Error in H5I_dec_ref(): can't locate ID \
             [Object atom: Unable to find atom information (already closed?)]"
        );

        let empty_stack = ErrorStack::new();
        assert!(empty_stack.is_empty());
        assert_eq!(empty_stack.len(), 0);
    }

    #[test]
    pub fn test_h5call() {
        let result_no_error = h5call!({
            let plist_id = H5Pcreate(*H5P_ROOT);
            H5Pclose(plist_id)
        });
        assert!(result_no_error.is_ok());

        let result_error = h5call!({
            let plist_id = H5Pcreate(*H5P_ROOT);
            H5Pclose(plist_id);
            H5Pclose(plist_id)
        });
        assert!(result_error.is_err());
    }

    #[test]
    pub fn test_h5try() {
        fn f1() -> Result<herr_t> {
            h5try!(H5Pcreate(*H5P_ROOT));
            Ok(100)
        }

        assert_eq!(f1().unwrap(), 100);

        fn f2() -> Result<herr_t> {
            h5try!(H5Pcreate(123456));
            Ok(100)
        }

        assert!(f2().is_err());
    }
}
