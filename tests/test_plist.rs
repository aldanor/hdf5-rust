use std::mem;

use libhdf5_sys::h5::hsize_t;

use h5::file::*;
use h5::plist::*;

macro_rules! test_plist {
    ($ty:ident, $builder:ident, $cls:expr => {$($field:ident($get:ident),)*}) => {
        test_plist!($ty, $builder, $cls => {$($field($get)),*});
    };

    ($ty:ident, $builder:ident, $cls:expr => {$($field:ident($get:ident)),*}) => {
        let pl_default = $ty::try_new()?;
        assert_eq!(pl_default.class()?, $cls);

        let mut builder = $ty::build();
        $(
        builder.$field($field.clone());
        )*

        let pl = builder.finish()?;
        assert_eq!(pl.class()?, $cls);
        $(
        assert_eq!(pl.$field(), $field);
        assert_eq!(pl.$get().unwrap(), $field);
        )*;

        assert_eq!(pl_default, pl_default);
        assert_eq!(pl, pl);
        assert_ne!(pl, pl_default);

        let pl_copy = $builder::from_plist(&pl)?.finish()?;
        assert_eq!(pl, pl_copy);
    }
}

#[test]
fn test_file_create_plist() -> h5::Result<()> {
    let fcpl = FileCreate::try_new()?;

    assert_eq!(fcpl.sizes().sizeof_addr, mem::size_of::<hsize_t>());
    assert_eq!(fcpl.sizes().sizeof_size, mem::size_of::<hsize_t>());

    let userblock = 2048;
    let sym_k = SymbolTableInfo { tree_rank: 17, node_size: 5 };
    let istore_k = 33;
    let shared_mesg_phase_change = PhaseChangeInfo { max_list: 51, min_btree: 41 };
    let shared_mesg_indexes = vec![SharedMessageIndex {
        message_types: SharedMessageType::ATTRIBUTE,
        min_message_size: 16,
    }];

    test_plist!(FileCreate, FileCreateBuilder, PropertyListClass::FileCreate => {
        userblock(get_userblock),
        sym_k(get_sym_k),
        istore_k(get_istore_k),
        shared_mesg_phase_change(get_shared_mesg_phase_change),
        shared_mesg_indexes(get_shared_mesg_indexes),
    });

    #[cfg(hdf5_1_10_1)]
    {
        let fcpl = FileCreate::try_new()?;
        assert_eq!(fcpl.file_space_strategy(), FileSpaceStrategy::default());

        let file_space_page_size = 16384;
        let file_space_strategy = FileSpaceStrategy::None;

        test_plist!(FileCreate, FileCreateBuilder, PropertyListClass::FileCreate => {
            file_space_page_size(get_file_space_page_size),
            file_space_strategy(get_file_space_strategy),
        });

        let file_space_strategy = FileSpaceStrategy::PageAggregation;
        test_plist!(FileCreate, FileCreateBuilder, PropertyListClass::FileCreate => {
            file_space_strategy(get_file_space_strategy),
        });
    }

    Ok(())
}

#[test]
fn test_file_access_plist() -> h5::Result<()> {
    let fapl = FileAccess::try_new()?;
    println!("{:#?}", fapl);
    let mut b = FileAccess::build();

    b.split();
    let fapl3 = b.finish()?;
    println!("{:#?}", fapl3);
    println!("{:#?}", fapl3.get_driver());

    Ok(())
}
