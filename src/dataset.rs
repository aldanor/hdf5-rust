use ffi::h5d::{H5Dcreate2, H5Dcreate_anon};
use ffi::h5i::{H5I_DATASET, hid_t};
use ffi::h5p::{H5Pcreate, H5Pset_create_intermediate_group, H5P_DEFAULT, H5Pset_obj_track_times};
use globals::H5P_LINK_CREATE;

use container::Container;
use datatype::{Datatype, ToDatatype};
use error::Result;
use filters::{Filters, Chunk};
use handle::{Handle, ID, FromID, get_id_type};
use location::Location;
use object::Object;
use plist::PropertyList;
use space::{Dataspace, Dimension};
use util::to_cstring;

pub struct Dataset {
    handle: Handle,
}

impl ID for Dataset {
    fn id(&self) -> hid_t {
        self.handle.id()
    }
}

impl FromID for Dataset {
    fn from_id(id: hid_t) -> Result<Dataset> {
        match get_id_type(id) {
            H5I_DATASET => Ok(Dataset { handle: try!(Handle::new(id)) }),
            _ => Err(From::from(format!("Invalid property list id: {}", id))),
        }
    }
}

impl Object for Dataset {}

impl Location for Dataset {}

impl Dataset {
}

pub struct DatasetBuilder {
    datatype: Result<Datatype>,
    filters: Filters,
    chunk: Chunk,
    parent: Result<Handle>,
    track_times: bool,
    resizable: bool,
}

impl DatasetBuilder {
    /// Create a new dataset builder, bind it to a container and set the datatype.
    pub fn new<T: ToDatatype, C: Container>(parent: &C) -> DatasetBuilder {
        // Store the reference to the parent handle, increase its reference count.
        let handle = Handle::new(parent.id());
        if let Ok(ref handle) = handle {
            handle.incref();
        }

        // Datatype and parent may contain invalid values, to be unwrapped later.
        DatasetBuilder {
            datatype: T::to_datatype(),
            filters: Filters::default(),
            chunk: Chunk::Auto,
            parent: handle,
            track_times: false,
            resizable: false,
        }
    }

    /// Disable chunking.
    pub fn no_chunk(&mut self) -> &mut DatasetBuilder {
        self.chunk = Chunk::None; self
    }

    /// Enable automatic chunking only if chunking is required (default option).
    pub fn chunk_auto(&mut self) -> &mut DatasetBuilder {
        self.chunk = Chunk::Auto; self
    }

    /// Enable chunking with automatic chunk shape.
    pub fn chunk_infer(&mut self) -> &mut DatasetBuilder {
        self.chunk = Chunk::Infer; self
    }

    /// Set chunk shape manually.
    pub fn chunk<D: Dimension>(&mut self, chunk: D) -> &mut DatasetBuilder {
        self.chunk = Chunk::Manual(chunk.dims()); self
    }

    /// Set the filters.
    pub fn filters(&mut self, filters: &Filters) -> &mut DatasetBuilder {
        self.filters = filters.clone(); self
    }

    /// Enable or disable tracking object modification time (disabled by default).
    pub fn track_times(&mut self, track_times: bool) -> &mut DatasetBuilder {
        self.track_times = track_times; self
    }

    /// Make the dataset resizable along all axes (requires chunking).
    pub fn resizable(&mut self, resizable: bool) -> &mut DatasetBuilder {
        self.resizable = resizable; self
    }

    fn finalize<D: Dimension>(&self, name: Option<String>, shape: D) -> Result<Dataset> {
        let datatype = try_ref_clone!(self.datatype);
        let parent = try_ref_clone!(self.parent);

        h5lock!({
            let dataspace = try!(Dataspace::new(&shape, self.resizable));
            let dcpl = try!(self.filters.to_dcpl(
                &datatype, &shape, self.chunk.clone(), self.resizable
            ));

            if self.track_times {
                h5try_s!(H5Pset_obj_track_times(dcpl.id(), 0));
            }

            match name.clone() {
                Some(name) => {
                    // Create intermediate groups automatically.
                    let lcpl = try!(PropertyList::from_id(h5try_s!(H5Pcreate(*H5P_LINK_CREATE))));
                    h5try_s!(H5Pset_create_intermediate_group(lcpl.id(), 1));

                    Dataset::from_id(h5try_s!(H5Dcreate2(
                        parent.id(), to_cstring(name).as_ptr(), datatype.id(),
                        dataspace.id(), lcpl.id(), dcpl.id(), H5P_DEFAULT
                    )))
                },
                _ => {
                    Dataset::from_id(h5try_s!(H5Dcreate_anon(
                        parent.id(), datatype.id(),
                        dataspace.id(), dcpl.id(), H5P_DEFAULT
                    )))
                }
            }
        })
    }

    /// Create the dataset and link it into the file structure.
    pub fn create<S: Into<String>, D: Dimension>(&self, name: S, shape: D) -> Result<Dataset> {
        self.finalize(Some(name.into()), shape)
    }

    /// Create an anonymous dataset without linking it.
    pub fn create_anon<D: Dimension>(&self, shape: D) -> Result<Dataset> {
        self.finalize(None, shape)
    }
}

#[cfg(test)]
mod tests {
}
