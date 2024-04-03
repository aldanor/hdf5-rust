use hdf5::*;

fn main() {
    let file = File::create("file.h5").unwrap();
    let _g = file.create_group("g").unwrap();
    let _gg = file.create_group("gg").unwrap();
    let refs =
        [file.reference("g").unwrap(), file.reference("gg").unwrap(), file.reference("g").unwrap()];

    let ds = file.new_dataset_builder().with_data(&refs).create("refs").unwrap();
    ds.write_slice(&refs[1..2], 1..2).unwrap();
    ds.write_slice(&refs[2..3], 2..3).unwrap();

    let refs: ndarray::Array1<ObjectReference> = ds.read().unwrap();
    let g = file.dereference(&refs[1]);
    println!("{g:?}");
}
