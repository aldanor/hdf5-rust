use ffi::h5::herr_t;
use ffi::h5e::{
    H5Ewalk2, H5Eget_msg, H5E_error2_t, H5Eset_auto2, H5Eget_current_stack,
    H5Eclose_stack, H5E_WALK_DOWNWARD, H5E_DEFAULT
};

use util::{get_h5_str, string_from_cstr};

use std::ptr;
use std::fmt;
use std::ops::Index;
use std::error::Error as BaseError;

use libc::{c_uint, c_void};
use num::{Integer, Zero, Bounded};

#[derive(Clone)]
pub struct ErrorFrame {
    desc: String,
    func: String,
    major: String,
    minor: String,
    description: String,
}

impl ErrorFrame {
    pub fn new<S1, S2, S3, S4>(desc: S1, func: S2, major: S3, minor: S4) -> ErrorFrame
    where S1: Into<String>, S2: Into<String>, S3: Into<String>, S4: Into<String> {
        let (s_func, s_desc) = (func.into(), desc.into());
        let description = format!("{}(): {}", s_func.clone(), s_desc.clone());
        ErrorFrame {
            desc: s_desc,
            func: s_func,
            major: major.into(),
            minor: minor.into(),
            description: description,
        }
    }

    pub fn desc(&self) -> &str {
        self.desc.as_ref()
    }

    pub fn description(&self) -> &str {
        self.description.as_ref()
    }

    pub fn detail(&self) -> Option<String> {
        Some(format!(
            "Error in {}(): {} [{}: {}]", self.func, self.desc, self.major, self.minor
        ).clone())
    }
}

pub fn silence_errors() {
    h5lock!(H5Eset_auto2(H5E_DEFAULT, None, ptr::null_mut()));
}

#[derive(Clone)]
pub struct ErrorStack {
    frames: Vec<ErrorFrame>,
    description: Option<String>,
}

impl Index<usize> for ErrorStack {
    type Output = ErrorFrame;

    fn index<'a>(&'a self, index: usize) -> &'a ErrorFrame {
        &self.frames[index]
    }
}

impl ErrorStack {
    // This low-level function is not thread-safe and has to be synchronized by the user
    pub fn query() -> Result<Option<ErrorStack>> {
        struct CallbackData {
            stack: ErrorStack,
            err: Option<Error>,
        }

        extern fn callback(_: c_uint, err_desc: *const H5E_error2_t, data: *mut c_void) -> herr_t {
            unsafe {
                let data = &mut *(data as *mut CallbackData);
                if data.err.is_some() {
                    return 0;
                }
                let closure = |e: H5E_error2_t| -> Result<ErrorFrame> {
                    let (desc, func) = (string_from_cstr(e.desc), string_from_cstr(e.func_name));
                    let major = try!(get_h5_str(|m, s| {
                        H5Eget_msg(e.maj_num, ptr::null_mut(), m, s)
                    }));
                    let minor = try!(get_h5_str(|m, s| {
                        H5Eget_msg(e.min_num, ptr::null_mut(), m, s)
                    }));
                    Ok(ErrorFrame::new(desc, func, major, minor))
                };
                match closure(*err_desc) {
                    Ok(frame) => { data.stack.push(frame); 0 },
                    Err(err)  => { data.err = Some(From::from(err)); 0 }
                }
            }
        }

        let mut data = CallbackData { stack: ErrorStack::new(), err: None };
        let data_ptr: *mut c_void = &mut data as *mut _ as *mut c_void;

        // known HDF5 bug: H5Eget_msg() may corrupt the current stack, so we copy it first
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

    pub fn new() -> ErrorStack {
        ErrorStack { frames: Vec::new(), description: None }
    }

    pub fn len(&self) -> usize {
        self.frames.len()
    }

    pub fn push(&mut self, frame: ErrorFrame) {
        self.frames.push(frame);
        if self.len() >= 1 {
            let top_desc = self.frames[0].description().to_string();
            if self.len() == 1 {
                self.description = Some(top_desc);
            } else {
                self.description = Some(format!(
                    "{}: {}", top_desc, self.frames[self.len() - 1].desc()
                ));
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn top(&self) -> Option<&ErrorFrame> {
        match !self.is_empty() {
            false => None,
            true  => Some(&self.frames[0]),
        }
    }

    pub fn description(&self) -> &str {
        match self.description {
            None           => "unknown library error",
            Some(ref desc) => desc.as_ref(), //frame.description(),
        }
    }

    pub fn detail(&self) -> Option<String> {
        self.top().and_then(|frame| frame.detail())
    }
}

/// The error type for HDF5-related functions.
#[derive(Clone)]
pub enum Error {
    /// An error occurred in the C API of the HDF5 library. Full error stack is captured.
    LibraryError(ErrorStack),
    /// A user error occurred in the high-level Rust API (e.g., invalid user input).
    InternalError(String),
}

/// A type for results generated by HDF5-related functions where the `Err` type is
/// set to `hdf5::Error`.
pub type Result<T> = ::std::result::Result<T, Error>;

impl Error {
    pub fn query() -> Option<Error> {
        match ErrorStack::query() {
            Err(err)        => Some(From::from(err)),
            Ok(Some(stack)) => Some(Error::LibraryError(stack)),
            Ok(None)        => None,
        }
    }

    pub fn description(&self) -> &str {
        match *self {
            Error::InternalError(ref desc) => desc.as_ref(),
            Error::LibraryError(ref stack) => stack.description(),
        }
    }
}

impl<S> From<S> for Error where S: Into<String> {
    fn from(desc: S) -> Error {
        Error::InternalError(desc.into())
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> ::std::result::Result<(), fmt::Error> {
        match *self {
            Error::InternalError(ref desc) => desc.fmt(formatter),
            Error::LibraryError(ref stack) => stack.description().fmt(formatter),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> ::std::result::Result<(), fmt::Error> {
        self.description().fmt(formatter)
    }
}

impl ::std::error::Error for Error {
    fn description(&self) -> &str {
        self.description()
    }
}

pub fn h5check<T>(value: T) -> Result<T> where T: Integer + Zero + Bounded,
{
    let maybe_error = if T::min_value() < T::zero() {
        value < T::zero()
    } else {
        value == T::zero()
    };

    match maybe_error {
        false => Ok(value),
        true  => match Error::query() {
            None       => Ok(value),
            Some(err)  => Err(From::from(err)),
        },
    }
}

#[cfg(test)]
mod tests {
    use ffi::h5::herr_t;
    use ffi::h5p::{H5Pcreate, H5Pclose};
    use globals::H5P_ROOT;
    use super::{ErrorStack, Result, silence_errors};

    #[test]
    pub fn test_error_stack() {
        silence_errors();

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
        assert_eq!(&stack.detail().unwrap(),
            "Error in H5Pclose(): can't close [Property lists: Unable to free object]");

        assert!(stack.len() >= 2 && stack.len() <= 3); // depending on HDF5 version
        assert!(!stack.is_empty());

        assert_eq!(stack[0].description(), "H5Pclose(): can't close");
        assert_eq!(&stack[0].detail().unwrap(),
            "Error in H5Pclose(): can't close \
            [Property lists: Unable to free object]");

        assert_eq!(stack[stack.len() - 1].description(), "H5I_dec_ref(): can't locate ID");
        assert_eq!(&stack[stack.len() - 1].detail().unwrap(),
            "Error in H5I_dec_ref(): can't locate ID \
            [Object atom: Unable to find atom information (already closed?)]");

        let empty_stack = ErrorStack::new();
        assert!(empty_stack.is_empty());
        assert_eq!(empty_stack.len(), 0);
    }

    #[test]
    pub fn test_h5call() {
        silence_errors();

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
        silence_errors();

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
