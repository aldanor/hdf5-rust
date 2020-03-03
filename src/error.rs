use std::cell::RefCell;
use std::error::Error as StdError;
use std::fmt;
use std::ops::Index;
use std::ptr;

use lazy_static::lazy_static;
use ndarray::ShapeError;
use num_integer::Integer;
use num_traits::{Bounded, Zero};
use parking_lot::Mutex;

use hdf5_sys::h5e::{
    H5E_error2_t, H5Eclose_stack, H5Eget_current_stack, H5Eget_msg, H5Eprint2, H5Eset_auto2,
    H5Ewalk2, H5E_DEFAULT, H5E_WALK_DOWNWARD,
};

use crate::internal_prelude::*;

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

#[must_use]
#[doc(hidden)]
pub struct SilenceErrors;

impl Default for SilenceErrors {
    fn default() -> Self {
        Self::new()
    }
}

lazy_static! {
    static ref ERROR_HANDLER: Mutex<RefCell<usize>> = Mutex::default();
}

extern "C" fn default_error_handler(estack: hid_t, _cdata: *mut c_void) -> herr_t {
    unsafe { H5Eprint2(estack, ptr::null_mut()) }
}

impl SilenceErrors {
    pub fn new() -> Self {
        Self::silence(true);
        Self
    }

    fn silence(on: bool) {
        let guard = ERROR_HANDLER.lock();
        let counter = &mut *guard.borrow_mut();
        if on {
            *counter += 1;
            if *counter == 1 {
                h5lock!(H5Eset_auto2(H5E_DEFAULT, None, ptr::null_mut()));
            }
        } else {
            if *counter > 0 {
                *counter -= 1;
            }
            if *counter == 0 {
                h5lock!(H5Eset_auto2(H5E_DEFAULT, Some(default_error_handler), ptr::null_mut()));
            }
        }
    }
}

impl Drop for SilenceErrors {
    fn drop(&mut self) {
        Self::silence(false);
    }
}

pub fn silence_errors() -> SilenceErrors {
    SilenceErrors::new()
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

struct CallbackData {
    stack: ErrorStack,
    err: Option<Error>,
}

impl ErrorStack {
    // This low-level function is not thread-safe and has to be synchronized by the user
    pub fn query() -> Result<Option<Self>> {
        extern "C" fn callback(
            _: c_uint, err_desc: *const H5E_error2_t, data: *mut c_void,
        ) -> herr_t {
            unsafe {
                let data = &mut *(data as *mut CallbackData);
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
            }
        }

        let mut data = CallbackData { stack: Self::new(), err: None };
        let data_ptr: *mut c_void = &mut data as *mut _ as *mut _;

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
    HDF5(ErrorStack),
    /// A user error occurred in the high-level Rust API (e.g., invalid user input).
    Internal(String),
}

/// A type for results generated by HDF5-related functions where the `Err` type is
/// set to `hdf5::Error`.
pub type Result<T> = ::std::result::Result<T, Error>;

impl Error {
    pub fn query() -> Option<Self> {
        match ErrorStack::query() {
            Err(err) => Some(err),
            Ok(Some(stack)) => Some(Error::HDF5(stack)),
            Ok(None) => None,
        }
    }

    pub fn description(&self) -> &str {
        match *self {
            Error::Internal(ref desc) => desc.as_ref(),
            Error::HDF5(ref stack) => stack.description(),
        }
    }
}

impl From<&str> for Error {
    fn from(desc: &str) -> Self {
        Error::Internal(desc.into())
    }
}

impl From<String> for Error {
    fn from(desc: String) -> Self {
        Error::Internal(desc)
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Internal(ref desc) => f.write_str(desc),
            Error::HDF5(ref stack) => f.write_str(stack.description()),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.description())
    }
}

impl StdError for Error {}

impl From<ShapeError> for Error {
    fn from(err: ShapeError) -> Self {
        format!("shape error: {}", err.to_string()).into()
    }
}

pub(crate) fn is_err_code<T>(value: T) -> bool
where
    T: Integer + Zero + Bounded + Copy,
{
    if T::min_value() < T::zero() {
        value < T::zero()
    } else {
        value == T::zero()
    }
}

pub fn h5check<T>(value: T) -> Result<T>
where
    T: Integer + Zero + Bounded + Copy,
{
    if is_err_code(value) {
        Error::query().map_or_else(|| Ok(value), Err)
    } else {
        Ok(value)
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
        let _e = silence_errors();

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
        let _e = silence_errors();

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
        let _e = silence_errors();

        fn f1() -> Result<herr_t> {
            let plist_id = h5try!(H5Pcreate(*H5P_ROOT));
            h5try!(H5Pclose(plist_id));
            Ok(100)
        }

        let result1 = f1();
        assert!(result1.is_ok());
        assert_eq!(result1.ok().unwrap(), 100);

        fn f2() -> Result<herr_t> {
            let plist_id = h5try!(H5Pcreate(*H5P_ROOT));
            h5try!(H5Pclose(plist_id));
            h5try!(H5Pclose(plist_id));
            Ok(100)
        }

        let result2 = f2();
        assert!(result2.is_err());
    }
}
