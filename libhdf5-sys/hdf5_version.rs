use std::os::raw::{c_int, c_uint};

extern {
    pub fn H5open() -> c_int;
    pub fn H5get_libversion(majnum: *mut c_uint, minnum: *mut c_uint,
                            relnum: *mut c_uint) -> c_int;
}

fn main() {
    let mut v: (c_uint, c_uint, c_uint) = (0, 0, 0);
    unsafe {
        assert_eq!(H5open(), 0);
        assert_eq!(H5get_libversion(&mut v.0, &mut v.1, &mut v.2), 0);
    }
    println!("{} {} {}", v.0, v.1, v.2);
}
