#[macro_use]
extern crate mashup;

use std::mem;

use hdf5::dataset::*;
use hdf5::file::*;
use hdf5::plist::*;

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

        assert!(format!("{:?}", pl_default).starts_with(&format!("{:?}", $plc)));

        let mut b = $cls::build();
        let pl = $func(&mut b)?;
        assert_eq!(pl.class()?, $plc);
        assert_eq!(pl, pl);
        assert_ne!(pl, pl_default);

        let pl2 = pl.copy();
        assert_eq!(pl2.class()?, $plc);
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
fn test_fcpl_common() -> hdf5::Result<()> {
    test_pl_common!(FC, PropertyListClass::FileCreate, |b: &mut FCB| b.userblock(2048).finish());
    Ok(())
}

#[test]
fn test_fcpl_sizes() -> hdf5::Result<()> {
    use hdf5_sys::h5::hsize_t;
    let fcpl = FileCreate::try_new()?;
    assert_eq!(fcpl.sizes().sizeof_addr, mem::size_of::<hsize_t>());
    assert_eq!(fcpl.sizes().sizeof_size, mem::size_of::<hsize_t>());
    Ok(())
}

#[test]
fn test_fcpl_set_userblock() -> hdf5::Result<()> {
    test_pl!(FC, userblock: 0);
    test_pl!(FC, userblock: 4096);
    Ok(())
}

#[test]
fn test_fcpl_set_sym_k() -> hdf5::Result<()> {
    test_pl!(FC, sym_k: tree_rank = 17, node_size = 5);
    test_pl!(FC, sym_k: tree_rank = 18, node_size = 6);
    Ok(())
}

#[test]
fn test_fcpl_set_istore_k() -> hdf5::Result<()> {
    test_pl!(FC, istore_k: 33);
    test_pl!(FC, istore_k: 123);
    Ok(())
}

#[test]
fn test_fcpl_set_shared_mesg_change() -> hdf5::Result<()> {
    test_pl!(FC, shared_mesg_phase_change: max_list = 51, min_btree = 41);
    test_pl!(FC, shared_mesg_phase_change: max_list = 52, min_btree = 42);
    Ok(())
}

#[test]
fn test_fcpl_set_shared_mesg_indexes() -> hdf5::Result<()> {
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
fn test_fcpl_set_file_space_page_size() -> hdf5::Result<()> {
    test_pl!(FC, file_space_page_size: 512);
    test_pl!(FC, file_space_page_size: 999);
    Ok(())
}

#[test]
#[cfg(hdf5_1_10_1)]
fn test_fcpl_set_file_space_strategy() -> hdf5::Result<()> {
    test_pl!(FC, file_space_strategy: FileSpaceStrategy::PageAggregation);
    test_pl!(FC, file_space_strategy: FileSpaceStrategy::None);
    let fsm = FileSpaceStrategy::FreeSpaceManager { paged: true, persist: true, threshold: 123 };
    test_pl!(FC, file_space_strategy: fsm);
    Ok(())
}

type FA = FileAccess;
type FAB = FileAccessBuilder;

#[test]
fn test_fapl_common() -> hdf5::Result<()> {
    test_pl_common!(FA, PropertyListClass::FileAccess, |b: &mut FAB| b.sieve_buf_size(8).finish());
    Ok(())
}

#[test]
fn test_fapl_driver_sec2() -> hdf5::Result<()> {
    let mut b = FileAccess::build();
    b.sec2();
    check_matches!(b.finish()?.get_driver()?, (), FileDriver::Sec2);
    Ok(())
}

#[test]
fn test_fapl_driver_stdio() -> hdf5::Result<()> {
    let mut b = FileAccess::build();
    b.stdio();
    check_matches!(b.finish()?.get_driver()?, (), FileDriver::Stdio);
    Ok(())
}

#[test]
fn test_fapl_driver_log() -> hdf5::Result<()> {
    let mut b = FileAccess::build();

    b.log();
    check_matches!(b.finish()?.get_driver()?, (), FileDriver::Log);

    b.log_options(Some("abc"), LogFlags::TRUNCATE, 123);
    check_matches!(b.finish()?.get_driver()?, (), FileDriver::Log);

    Ok(())
}

#[test]
fn test_fapl_driver_core() -> hdf5::Result<()> {
    let mut b = FileAccess::build();

    b.core();
    let d = check_matches!(b.finish()?.get_driver()?, d, FileDriver::Core(d));
    assert_eq!(d.increment, 1024 * 1024);
    assert_eq!(d.filebacked, false);
    #[cfg(hdf5_1_8_13)]
    assert_eq!(d.write_tracking, 0);

    b.core_options(123, true);
    #[cfg(hdf5_1_8_13)]
    b.write_tracking(456);
    let d = check_matches!(b.finish()?.get_driver()?, d, FileDriver::Core(d));
    assert_eq!(d.increment, 123);
    assert_eq!(d.filebacked, true);
    #[cfg(hdf5_1_8_13)]
    assert_eq!(d.write_tracking, 456);

    b.core_filebacked(false);
    let d = check_matches!(b.finish()?.get_driver()?, d, FileDriver::Core(d));
    assert_eq!(d.increment, CoreDriver::default().increment);
    assert_eq!(d.filebacked, false);

    b.core_filebacked(true);
    let d = check_matches!(b.finish()?.get_driver()?, d, FileDriver::Core(d));
    assert_eq!(d.increment, CoreDriver::default().increment);
    assert_eq!(d.filebacked, true);

    Ok(())
}

#[test]
fn test_fapl_driver_family() -> hdf5::Result<()> {
    let mut b = FileAccess::build();

    b.family();
    let d = check_matches!(b.finish()?.get_driver()?, d, FileDriver::Family(d));
    assert_eq!(d.member_size, 0);

    b.family_options(123);
    let d = check_matches!(b.finish()?.get_driver()?, d, FileDriver::Family(d));
    assert_eq!(d.member_size, 123);

    Ok(())
}

#[test]
fn test_fapl_driver_multi() -> hdf5::Result<()> {
    let mut b = FileAccess::build();

    b.multi();
    let d = check_matches!(b.finish()?.get_driver()?, d, FileDriver::Multi(d));
    assert_eq!(d, MultiDriver::default());

    let files = vec![
        MultiFile::new("foo", 1 << 20),
        MultiFile::new("bar", 1 << 30),
        MultiFile::new("baz", 1 << 40),
        MultiFile::new("qwe", 1 << 50),
    ];
    let layout = MultiLayout {
        mem_super: 0,
        mem_btree: 1,
        mem_draw: 2,
        mem_gheap: 3,
        mem_lheap: 3,
        mem_object: 2,
    };
    b.multi_options(&files, &layout, true);
    let d = check_matches!(b.finish()?.get_driver()?, d, FileDriver::Multi(d));
    assert_eq!(d.files, files);
    assert_eq!(d.layout, layout);
    assert_eq!(d.relax, true);

    Ok(())
}

#[test]
fn test_fapl_driver_split() -> hdf5::Result<()> {
    let mut b = FileAccess::build();

    b.split();
    let d = check_matches!(b.finish()?.get_driver()?, d, FileDriver::Split(d));
    assert_eq!(d, SplitDriver::default());

    b.split_options(".foo", ".bar");
    let d = check_matches!(b.finish()?.get_driver()?, d, FileDriver::Split(d));
    assert_eq!(&d.meta_ext, ".foo");
    assert_eq!(&d.raw_ext, ".bar");

    Ok(())
}

#[test]
#[cfg(feature = "mpio")]
fn test_fapl_driver_mpio() -> hdf5::Result<()> {
    use std::os::raw::c_int;
    use std::ptr;

    use mpi_sys::{MPI_Comm_compare, MPI_Init, MPI_Initialized, MPI_CONGRUENT, RSMPI_COMM_WORLD};

    let mut initialized: c_int = 1;
    unsafe { MPI_Initialized(&mut initialized) };
    if initialized == 0 {
        unsafe { MPI_Init(ptr::null_mut(), ptr::null_mut()) };
    }
    let world_comm = unsafe { RSMPI_COMM_WORLD };

    let mut b = FileAccess::build();
    b.mpio(world_comm, None);

    let d = check_matches!(b.finish()?.get_driver()?, d, FileDriver::Mpio(d));
    let mut cmp = mem::MaybeUninit::uninit();
    unsafe { MPI_Comm_compare(d.comm, world_comm, cmp.as_mut_ptr() as &mut _) };
    assert_eq!(unsafe { cmp.assume_init() }, MPI_CONGRUENT as _);

    Ok(())
}

#[test]
#[cfg(h5_have_direct)]
fn test_fapl_driver_direct() -> hdf5::Result<()> {
    let mut b = FileAccess::build();

    b.direct();
    let d = check_matches!(b.finish()?.get_driver()?, d, FileDriver::Direct(d));
    assert_eq!(d, DirectDriver::default());

    b.direct_options(100, 200, 400);
    let d = check_matches!(b.finish()?.get_driver()?, d, FileDriver::Direct(d));
    assert_eq!(d.alignment, 100);
    assert_eq!(d.block_size, 200);
    assert_eq!(d.cbuf_size, 400);

    Ok(())
}

#[test]
fn test_fapl_set_alignment() -> hdf5::Result<()> {
    test_pl!(FA, alignment: threshold = 1, alignment = 1);
    test_pl!(FA, alignment: threshold = 0, alignment = 32);
    Ok(())
}

#[test]
fn test_fapl_set_fclose_degree() -> hdf5::Result<()> {
    test_pl!(FA, fclose_degree: FileCloseDegree::Default);
    test_pl!(FA, fclose_degree: FileCloseDegree::Weak);
    test_pl!(FA, fclose_degree: FileCloseDegree::Semi);
    test_pl!(FA, fclose_degree: FileCloseDegree::Strong);
    Ok(())
}

#[test]
fn test_fapl_set_chunk_cache() -> hdf5::Result<()> {
    test_pl!(FA, chunk_cache: nslots = 1, nbytes = 100, w0 = 0.0);
    test_pl!(FA, chunk_cache: nslots = 10, nbytes = 200, w0 = 0.5);
    test_pl!(FA, chunk_cache: nslots = 20, nbytes = 300, w0 = 1.0);
    Ok(())
}

#[test]
fn test_fapl_set_meta_block_size() -> hdf5::Result<()> {
    test_pl!(FA, meta_block_size: 0);
    test_pl!(FA, meta_block_size: 123);
    Ok(())
}

#[test]
fn test_fapl_set_sieve_buf_size() -> hdf5::Result<()> {
    test_pl!(FA, sieve_buf_size: 42);
    test_pl!(FA, sieve_buf_size: 4096);
    Ok(())
}

#[test]
fn test_fapl_set_gc_references() -> hdf5::Result<()> {
    test_pl!(FA, gc_references: true);
    test_pl!(FA, gc_references: false);
    Ok(())
}

#[test]
fn test_fapl_set_small_data_block_size() -> hdf5::Result<()> {
    test_pl!(FA, small_data_block_size: 0);
    test_pl!(FA, small_data_block_size: 123);
    Ok(())
}

#[test]
fn test_fapl_set_mdc_config() -> hdf5::Result<()> {
    let mdc_config_1 = MetadataCacheConfig {
        rpt_fcn_enabled: false,
        open_trace_file: false,
        close_trace_file: false,
        trace_file_name: "".into(),
        evictions_enabled: true,
        set_initial_size: true,
        initial_size: 1 << 22,
        min_clean_fraction: 0.30000001192092890,
        max_size: 1 << 26,
        min_size: 1 << 21,
        epoch_length: 60_000,
        incr_mode: CacheIncreaseMode::Threshold,
        lower_hr_threshold: 0.8999999761581420,
        increment: 3.0,
        apply_max_increment: true,
        max_increment: 1 << 23,
        flash_incr_mode: FlashIncreaseMode::AddSpace,
        flash_multiple: 2.0,
        flash_threshold: 0.5,
        decr_mode: CacheDecreaseMode::AgeOutWithThreshold,
        upper_hr_threshold: 0.9990000128746030,
        decrement: 0.8999999761581420,
        apply_max_decrement: true,
        max_decrement: 1 << 21,
        epochs_before_eviction: 4,
        apply_empty_reserve: true,
        empty_reserve: 0.10000000149011610,
        dirty_bytes_threshold: 1 << 19,
        metadata_write_strategy: MetadataWriteStrategy::Distributed,
    };

    let mdc_config_2 = MetadataCacheConfig {
        rpt_fcn_enabled: true,
        open_trace_file: true,
        close_trace_file: true,
        trace_file_name: "abc".into(),
        evictions_enabled: false,
        set_initial_size: false,
        initial_size: 1 << 23,
        min_clean_fraction: 0.30000001192092899,
        max_size: 1 << 27,
        min_size: 1 << 22,
        epoch_length: 70_000,
        incr_mode: CacheIncreaseMode::Off,
        lower_hr_threshold: 0.8999999761581499,
        increment: 4.0,
        apply_max_increment: false,
        max_increment: 1 << 24,
        flash_incr_mode: FlashIncreaseMode::Off,
        flash_multiple: 3.0,
        flash_threshold: 0.6,
        decr_mode: CacheDecreaseMode::Off,
        upper_hr_threshold: 0.9990000128746099,
        decrement: 0.8999999761581499,
        apply_max_decrement: false,
        max_decrement: 1 << 22,
        epochs_before_eviction: 5,
        apply_empty_reserve: false,
        empty_reserve: 0.10000000149011699,
        dirty_bytes_threshold: 1 << 20,
        metadata_write_strategy: MetadataWriteStrategy::ProcessZeroOnly,
    };

    test_pl!(FA, mdc_config(&mdc_config_1): mdc_config_1);
    test_pl!(FA, mdc_config(&mdc_config_2): mdc_config_2);

    Ok(())
}

#[test]
#[cfg(hdf5_1_8_7)]
fn test_fapl_set_elink_file_cache_size() -> hdf5::Result<()> {
    test_pl!(FA, elink_file_cache_size: 0);
    test_pl!(FA, elink_file_cache_size: 17);
    Ok(())
}

#[test]
#[cfg(hdf5_1_10_0)]
fn test_fapl_set_metadata_read_attempts() -> hdf5::Result<()> {
    test_pl!(FA, metadata_read_attempts: 1);
    test_pl!(FA, metadata_read_attempts: 17);
    Ok(())
}

#[test]
#[cfg(hdf5_1_10_0)]
fn test_fapl_set_mdc_log_options() -> hdf5::Result<()> {
    test_pl!(FA, mdc_log_options: is_enabled = true, location = "abc", start_on_access = false,);
    test_pl!(FA, mdc_log_options: is_enabled = false, location = "", start_on_access = true,);
    Ok(())
}

#[test]
#[cfg(all(hdf5_1_10_0, feature = "mpio"))]
fn test_fapl_set_all_coll_metadata_ops() -> hdf5::Result<()> {
    test_pl!(FA, all_coll_metadata_ops: true);
    test_pl!(FA, all_coll_metadata_ops: false);
    Ok(())
}

#[test]
#[cfg(all(hdf5_1_10_0, feature = "mpio"))]
fn test_fapl_set_coll_metadata_write() -> hdf5::Result<()> {
    test_pl!(FA, coll_metadata_write: true);
    test_pl!(FA, coll_metadata_write: false);
    Ok(())
}

#[test]
#[cfg(hdf5_1_10_2)]
fn test_fapl_set_libver_bounds() -> hdf5::Result<()> {
    test_pl!(FA, libver_bounds: low = LibraryVersion::Earliest, high = LibraryVersion::V18);
    test_pl!(FA, libver_bounds: low = LibraryVersion::Earliest, high = LibraryVersion::V110);
    test_pl!(FA, libver_bounds: low = LibraryVersion::V18, high = LibraryVersion::V18);
    test_pl!(FA, libver_bounds: low = LibraryVersion::V18, high = LibraryVersion::V110);
    test_pl!(FA, libver_bounds: low = LibraryVersion::V110, high = LibraryVersion::V110);
    Ok(())
}

#[test]
#[cfg(hdf5_1_10_1)]
fn test_fapl_set_page_buffer_size() -> hdf5::Result<()> {
    test_pl!(FA, page_buffer_size: buf_size = 0, min_meta_perc = 0, min_raw_perc = 0);
    test_pl!(FA, page_buffer_size: buf_size = 0, min_meta_perc = 7, min_raw_perc = 9);
    test_pl!(FA, page_buffer_size: buf_size = 3, min_meta_perc = 0, min_raw_perc = 5);
    Ok(())
}

#[test]
#[cfg(all(hdf5_1_10_1, not(h5_have_parallel)))]
fn test_fapl_set_evict_on_close() -> hdf5::Result<()> {
    test_pl!(FA, evict_on_close: true);
    test_pl!(FA, evict_on_close: false);
    Ok(())
}

#[test]
#[cfg(hdf5_1_10_1)]
fn test_fapl_set_mdc_image_config() -> hdf5::Result<()> {
    test_pl!(FA, mdc_image_config: generate_image = true);
    test_pl!(FA, mdc_image_config: generate_image = false);
    Ok(())
}

type DA = DatasetAccess;
type DAB = DatasetAccessBuilder;

#[test]
fn test_dapl_common() -> hdf5::Result<()> {
    test_pl_common!(DA, PropertyListClass::DatasetAccess, |b: &mut DAB| b
        .chunk_cache(100, 200, 0.5)
        .finish());
    Ok(())
}

#[test]
#[cfg(hdf5_1_8_17)]
fn test_dapl_set_efile_prefix() -> hdf5::Result<()> {
    assert_eq!(DA::try_new()?.get_efile_prefix().unwrap(), "".to_owned());
    assert_eq!(DA::try_new()?.efile_prefix(), "".to_owned());
    let mut b = DA::build();
    b.efile_prefix("foo");
    assert_eq!(b.finish()?.get_efile_prefix()?, "foo".to_owned());
    Ok(())
}

#[test]
fn test_dapl_set_chunk_cache() -> hdf5::Result<()> {
    test_pl!(DA, chunk_cache: nslots = 1, nbytes = 100, w0 = 0.0);
    test_pl!(DA, chunk_cache: nslots = 10, nbytes = 200, w0 = 0.5);
    test_pl!(DA, chunk_cache: nslots = 20, nbytes = 300, w0 = 1.0);
    Ok(())
}

#[test]
#[cfg(all(hdf5_1_10_0, feature = "mpio"))]
fn test_dapl_set_all_coll_metadata_ops() -> hdf5::Result<()> {
    test_pl!(DA, all_coll_metadata_ops: true);
    test_pl!(DA, all_coll_metadata_ops: false);
    Ok(())
}

#[test]
#[cfg(hdf5_1_10_0)]
fn test_dapl_set_virtual_view() -> hdf5::Result<()> {
    test_pl!(DA, virtual_view: VirtualView::FirstMissing);
    test_pl!(DA, virtual_view: VirtualView::LastAvailable);
    Ok(())
}

#[test]
#[cfg(hdf5_1_10_0)]
fn test_dapl_set_virtual_printf_gap() -> hdf5::Result<()> {
    test_pl!(DA, virtual_printf_gap: 0);
    test_pl!(DA, virtual_printf_gap: 123);
    Ok(())
}
