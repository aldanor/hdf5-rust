/// Attribute storage phase change thresholds.
///
/// These thresholds determine the point at which attribute storage changes from
/// compact storage (i.e., storage in the object header) to dense storage (i.e.,
/// storage in a heap and indexed with a B-tree).
///
/// In the general case, attributes are initially kept in compact storage. When
/// the number of attributes exceeds `max_compact`, attribute storage switches to
/// dense storage. If the number of attributes subsequently falls below `min_dense`,
/// the attributes are returned to compact storage.
///
/// If `max_compact` is set to 0 (zero), dense storage always used.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AttrPhaseChange {
    /// Maximum number of attributes to be stored in compact storage (default: 8).
    pub max_compact: u32,
    /// Minimum number of attributes to be stored in dense storage (default: 6).
    pub min_dense: u32,
}

impl Default for AttrPhaseChange {
    fn default() -> Self {
        Self { max_compact: 8, min_dense: 6 }
    }
}
