#[allow(unused_macros)]
macro_rules! assert_err {
    ($expr:expr, $err:expr) => {
        match unsafe { $expr } {
            Ok(_) => {
                panic!("assertion failed: not an error in `{}`", stringify!($expr));
            }
            Err(ref value) => {
                let desc = value.to_string();
                if !desc.contains($err) {
                    panic!(
                        "assertion failed: error message `{}` doesn't contain `{}` in `{}`",
                        desc,
                        $err,
                        stringify!($expr)
                    );
                }
            }
        }
    };
}
