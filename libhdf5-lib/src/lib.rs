macro_rules! check_and_emit {
    ($flag:ident) => {
        if cfg!($flag) {
            println!("cargo:rustc-cfg={}", stringify!($flag));
        }
    };
}

pub fn emit_cfg_flags() {
    check_and_emit!(hdf5_1_8_5);
    check_and_emit!(hdf5_1_8_6);
    check_and_emit!(hdf5_1_8_7);
    check_and_emit!(hdf5_1_8_8);
    check_and_emit!(hdf5_1_8_9);
    check_and_emit!(hdf5_1_8_10);
    check_and_emit!(hdf5_1_8_11);
    check_and_emit!(hdf5_1_8_12);
    check_and_emit!(hdf5_1_8_13);
    check_and_emit!(hdf5_1_8_14);
    check_and_emit!(hdf5_1_8_15);
    check_and_emit!(hdf5_1_8_16);
    check_and_emit!(hdf5_1_8_17);
    check_and_emit!(hdf5_1_8_18);
    check_and_emit!(hdf5_1_8_19);
    check_and_emit!(hdf5_1_8_20);
    check_and_emit!(hdf5_1_8_21);
    check_and_emit!(hdf5_1_10_0);
    check_and_emit!(hdf5_1_10_1);
    check_and_emit!(hdf5_1_10_2);
    check_and_emit!(hdf5_1_10_3);
    check_and_emit!(hdf5_1_10_4);
}
