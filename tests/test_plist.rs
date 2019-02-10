#[macro_use]
extern crate mashup;

use std::mem;

use h5::file::*;
use h5::plist::*;

macro_rules! test_pl {
    ($ty:ident, $field:ident ($($arg:expr),+): $($name:ident=$value:expr),+) => (
        test_pl!($ty, $field ($($arg,)+): $($name=$value,)+)
    );

    ($ty:ident, $field:ident ($($arg:expr,)+): $($name:ident=$value:expr,)+) => ({
        let mut b = $ty::build();
        mashup! { m["get" $field] = get_ $field; }
        b.$field($($arg,)+);
        let fapl = b.finish()?;
        $(assert_eq!(fapl.$field().$name, $value);)+
        m! { $(assert_eq!(fapl."get" $field()?.$name, $value);)+ }
    });

    ($ty:ident, $field:ident: $($name:ident=$value:expr),+) => (
        test_pl!($ty, $field: $($name=$value,)+)
    );

    ($ty:ident, $field:ident: $($name:ident=$value:expr,)+) => ({
        test_pl!($ty, $field ($($value,)+): $($name=$value,)+)
    });

    ($ty:ident, $field:ident ($arg:expr): $value:expr) => ({
        let mut b = $ty::build();
        mashup! { m["get" $field] = get_ $field; }
        b.$field($arg);
        let fapl = b.finish()?;
        assert_eq!(fapl.$field(), $value);
        m! { assert_eq!(fapl."get" $field()?, $value); }
    });

    ($ty:ident, $field:ident: $value:expr) => ({
        test_pl!($ty, $field ($value): $value)
    });
}

macro_rules! test_pl_common {
    ($cls:ident, $plc:expr, $func:expr) => {
        let pl_default = $cls::try_new()?;
        assert_eq!(pl_default.class()?, $plc);
        assert_eq!(pl_default, pl_default);

        let mut b = $cls::build();
        let pl = $func(&mut b)?;
        assert_eq!(pl.class()?, $plc);
        assert_eq!(pl, pl);
        assert_ne!(pl, pl_default);

        let pl2 = pl.copy();
        assert_eq!(pl.class()?, $plc);
        assert_eq!(pl2, pl);
        assert_ne!(pl2, pl_default);
    };
}

macro_rules! check_matches {
    ($e:expr, $o:expr, $($p:tt)+) => (
        match $e {
            $($p)+ => $o,
            ref e => panic!("assertion failed: `{:?}` does not match `{}`", e, stringify!($($p)+)),
        }
    )
}

type FC = FileCreate;
type FCB = FileCreateBuilder;

#[test]
fn test_fcpl_common() -> h5::Result<()> {
    test_pl_common!(FC, PropertyListClass::FileCreate, |b: &mut FCB| b.userblock(2048).finish());
    Ok(())
}

#[test]
fn test_fcpl_sizes() -> h5::Result<()> {
    use libhdf5_sys::h5::hsize_t;
    let fcpl = FileCreate::try_new()?;
    assert_eq!(fcpl.sizes().sizeof_addr, mem::size_of::<hsize_t>());
    assert_eq!(fcpl.sizes().sizeof_size, mem::size_of::<hsize_t>());
    Ok(())
}

#[test]
fn test_fcpl_set_userblock() -> h5::Result<()> {
    test_pl!(FC, userblock: 0);
    test_pl!(FC, userblock: 4096);
    Ok(())
}

#[test]
fn test_fcpl_set_sym_k() -> h5::Result<()> {
    test_pl!(FC, sym_k: tree_rank = 17, node_size = 5);
    test_pl!(FC, sym_k: tree_rank = 18, node_size = 6);
    Ok(())
}

#[test]
fn test_fcpl_set_istore_k() -> h5::Result<()> {
    test_pl!(FC, istore_k: 33);
    test_pl!(FC, istore_k: 123);
    Ok(())
}

#[test]
fn test_fcpl_set_shared_mesg_change() -> h5::Result<()> {
    test_pl!(FC, shared_mesg_phase_change: max_list = 51, min_btree = 41);
    test_pl!(FC, shared_mesg_phase_change: max_list = 52, min_btree = 42);
    Ok(())
}

#[test]
fn test_fcpl_set_shared_mesg_indexes() -> h5::Result<()> {
    let idx = vec![SharedMessageIndex {
        message_types: SharedMessageType::ATTRIBUTE,
        min_message_size: 16,
    }];
    test_pl!(FC, shared_mesg_indexes(&idx): idx);
    let idx = vec![];
    test_pl!(FC, shared_mesg_indexes(&idx): idx);
    Ok(())
}

#[test]
#[cfg(hdf5_1_10_1)]
fn test_fcpl_set_file_space_page_size() -> h5::Result<()> {
    test_pl!(FC, file_space_page_size: 512);
    test_pl!(FC, file_space_page_size: 999);
    Ok(())
}

#[test]
#[cfg(hdf5_1_10_1)]
fn test_fcpl_set_file_space_strategy() -> h5::Result<()> {
    test_pl!(FC, file_space_strategy: FileSpaceStrategy::PageAggregation);
    test_pl!(FC, file_space_strategy: FileSpaceStrategy::None);
    let fsm = FileSpaceStrategy::FreeSpaceManager { paged: true, persist: true, threshold: 123 };
    test_pl!(FC, file_space_strategy: fsm);
    Ok(())
}
