use serde_json::Value;
use std::collections::HashMap;

/// The data structure store.
#[derive(Default)]
pub struct Fabric {
    /// Access to the cache of your data structures.
    pub structures: HashMap<String, Value>,
}
impl Fabric {
    /// Initialize a new instance of `Fabric`.
    pub fn new() -> Self {
        Self::default()
    }
}
