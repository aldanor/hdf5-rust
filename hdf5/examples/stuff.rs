use hdf5::*;

fn main() {
    let file = File::create("file.h5").unwrap();
    let _g = file.create_group("g").unwrap();
    let _gg = file.create_group("gg").unwrap();
    let refs: [ObjectReference2; 3] =
        [file.reference("g").unwrap(), file.reference("gg").unwrap(), file.reference("g").unwrap()];

    let ds = file.new_dataset_builder().with_data(&refs).create("refs").unwrap();
    ds.write_slice(&refs[1..2], 1..2).unwrap();
    ds.write_slice(&refs[2..3], 2..3).unwrap();

    let refs: ndarray::Array1<ObjectReference2> = ds.read().unwrap();
    let g = file.dereference(&refs[1]);
    // println!("{g:?}");

    #[derive(H5Type)]
    #[repr(C)]
    struct RefList {
        dataset: ObjectReference2,
        dimension: u32,
    }

    let file = File::open("dims_1d.h5").unwrap();
    let ds = file.dataset("x1").unwrap();
    let attr = ds.attr("REFERENCE_LIST").unwrap();
    let reflist = attr.read_1d::<RefList>().unwrap();
    assert_eq!(reflist.len(), 1);

    let ds = file.dataset("data").unwrap();
    let attr = ds.attr("DIMENSION_LIST").unwrap();
    let dimlist = attr.read_1d::<hdf5_types::VarLenArray<ObjectReference2>>().unwrap();
    println!("{dimlist:?}");
}
