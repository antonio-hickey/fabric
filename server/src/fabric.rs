use crate::Error;
use serde_json::Value;
use std::collections::HashMap;

/// The data structure store.
#[derive(Default)]
pub struct Fabric {
    pub cache: HashMap<String, Value>,
}
impl Fabric {
    /// Initialize a new instance of `Fabric`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get a value from the cache.
    pub fn get(&self, keys: Vec<&str>) -> Result<Value, Error> {
        if let Some(root) = keys.first() {
            let mut current_value = self
                .cache
                .get(root.to_owned())
                .ok_or_else(|| Error::InvalidKeyPath(keys.join(".")))?;

            for key in keys.iter().skip(1) {
                current_value = current_value
                    .as_object()
                    .ok_or_else(|| Error::InvalidKeyPath(keys.join(".")))?
                    .get(key.to_owned())
                    .ok_or_else(|| Error::InvalidKeyPath(keys.join(".")))?;
            }

            Ok(current_value.clone())
        } else {
            Err(Error::InvalidKeyPath(keys.join(".")))
        }
    }

    /// Set (update or create) a value in the cache.
    pub fn set(&mut self, keys: Vec<&str>, value: &str) -> Result<(), Error> {
        let value = serde_json::from_str(value)?;

        if let Some(root) = keys.first() {
            let mut current_value = self
                .cache
                .entry(root.to_string())
                .or_insert_with(|| Value::Object(serde_json::Map::new()));

            for key in keys.iter().skip(1).take(keys.len() - 2) {
                current_value = current_value
                    .as_object_mut()
                    .ok_or_else(|| Error::InvalidKeyPath(keys.join(".")))?
                    .entry((*key).to_string())
                    .or_insert_with(|| Value::Object(serde_json::Map::new()));
            }

            let final_key = keys.last().unwrap();
            current_value
                .as_object_mut()
                .ok_or_else(|| Error::InvalidKeyPath(keys.join(".")))?
                .insert(final_key.to_string(), value);

            Ok(())
        } else {
            Err(Error::InvalidKeyPath(keys.join("")))
        }
    }
}
