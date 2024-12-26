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
        if keys.is_empty() {
            return Err(Error::InvalidKeyPath("Empty key path".to_string()));
        }

        let mut current_value = self
            .cache
            .get(keys[0])
            .cloned()
            .ok_or_else(|| Error::InvalidKeyPath(keys.join(".")))?;

        for key in keys.iter().skip(1) {
            current_value = current_value
                .as_object()
                .and_then(|obj| obj.get(*key).cloned())
                .ok_or_else(|| Error::InvalidKeyPath(keys.join(".")))?;
        }

        Ok(current_value)
    }

    /// Set a value in the cache.
    ///
    /// NOTE: A SET command can create a value or update a value by overwriting it.
    pub fn set(&mut self, keys: Vec<&str>, value: &str) -> Result<(), Error> {
        if keys.is_empty() {
            return Err(Error::InvalidKeyPath("Empty key path".to_string()));
        }

        let parsed_value: Value = serde_json::from_str(value)?;

        if keys.len() == 1 {
            self.cache.insert(keys[0].to_string(), parsed_value);
            return Ok(());
        }

        let mut current_value = self
            .cache
            .entry(keys[0].to_string())
            .or_insert_with(|| Value::Object(serde_json::Map::new()));

        for key in keys.iter().skip(1).take(keys.len().saturating_sub(2)) {
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
            .insert(final_key.to_string(), parsed_value);

        Ok(())
    }

    /// Remove a key/value pair in the cache.
    pub fn remove(&mut self, keys: Vec<&str>) -> Result<(), Error> {
        if keys.is_empty() {
            return Err(Error::InvalidKeyPath("Empty key path".to_string()));
        }

        if keys.len() == 1 {
            self.cache.remove(keys[0]);
            return Ok(());
        }

        let mut current_value = self
            .cache
            .get_mut(keys[0])
            .ok_or_else(|| Error::InvalidKeyPath(keys.join(".")))?;

        for key in keys.iter().skip(1).take(keys.len().saturating_sub(2)) {
            current_value = current_value
                .as_object_mut()
                .and_then(|obj| obj.get_mut(*key))
                .ok_or_else(|| Error::InvalidKeyPath(keys.join(".")))?;
        }

        let final_key = keys.last().unwrap();
        current_value
            .as_object_mut()
            .and_then(|obj| obj.remove(*final_key))
            .ok_or_else(|| Error::InvalidKeyPath(keys.join(".")))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_create_new_fabric_instance() {
        let fabric = Fabric::new();
        assert_eq!(fabric.cache, HashMap::new())
    }

    #[test]
    fn can_set_values() {
        let mut fabric = Fabric::new();

        fabric
            .set(["testStruct", "a"].to_vec(), "{\"x\": 5, \"y\": 10}")
            .unwrap();
        fabric.set(["testStruct", "a", "x"].to_vec(), "3").unwrap();

        fabric
            .set(
                ["otherTest"].to_vec(),
                "\"b7be9512-69d2-40c9-9a01-329ffe79e2ff\"",
            )
            .unwrap();
    }

    #[test]
    fn can_get_values() {
        let mut fabric = Fabric::new();

        fabric
            .set(["testStruct", "a"].to_vec(), "{\"x\": 5, \"y\": 10}")
            .unwrap();
        let test_struct_a_actual = fabric.get(vec!["testStruct", "a"]).unwrap();
        let test_struct_a_expected: Value = serde_json::from_str("{\"x\": 5, \"y\": 10}").unwrap();
        assert_eq!(test_struct_a_expected, test_struct_a_actual);

        fabric
            .set(["testStruct", "b"].to_vec(), "{\"x\": 3, \"y\": 10.5}")
            .unwrap();
        let test_struct_b_actual = fabric.get(vec!["testStruct", "b"]).unwrap();
        let test_struct_b_expected: Value =
            serde_json::from_str("{\"x\": 3, \"y\": 10.5}").unwrap();
        assert_eq!(test_struct_b_expected, test_struct_b_actual);

        let test_structs_actual = fabric.get(vec!["testStruct"]).unwrap();
        let test_structs_expected: Value =
            serde_json::from_str("{\"a\": {\"x\": 5, \"y\": 10}, \"b\": {\"x\": 3, \"y\": 10.5}}")
                .unwrap();
        assert_eq!(test_structs_expected, test_structs_actual);
    }

    #[test]
    fn can_remove_values() {
        let mut fabric = Fabric::new();

        fabric
            .set(
                vec!["testStruct"],
                "{\"a\": {\"x\": 5, \"y\": 10}, \"b\": {\"x\": 3, \"y\": 10.5}}",
            )
            .unwrap();

        fabric.remove(vec!["testStruct", "a"]).unwrap();

        let test_structs_actual = fabric.get(vec!["testStruct"]).unwrap();
        let test_structs_expected: Value =
            serde_json::from_str("{\"b\": {\"x\": 3, \"y\": 10.5}}").unwrap();
        assert_eq!(test_structs_expected, test_structs_actual);
    }

    #[test]
    fn handles_empty_keys() {
        let mut fabric = Fabric::new();

        assert!(fabric.set(vec![], "{\"x\": 5}").is_err());
        assert!(fabric.get(vec![]).is_err());
        assert!(fabric.remove(vec![]).is_err());
    }

    #[test]
    fn handles_invalid_key_path() {
        let mut fabric = Fabric::new();

        fabric.set(vec!["someKey"], "{\"x\": 29}").unwrap();
        assert!(fabric.get(vec!["nonexistent"]).is_err());
    }

    #[test]
    fn can_get_nested_values_with_special_keys() {
        let mut fabric = Fabric::new();

        let strategies = r#"
    {
        "b7be9512-69d2-40c9-9a01-329ffe79e2ff": {
            "id": "b7be9512-69d2-40c9-9a01-329ffe79e2ff",
            "name": "10 Year Scalper",
            "symbol": "TYH25",
            "position_size": 2,
            "account_number": "some_id",
            "tick_size": "0.015625",
            "open_trade": null
        }
    }"#;

        fabric.set(vec!["strategies"], strategies).unwrap();

        let result = fabric
            .get(vec!["strategies", "b7be9512-69d2-40c9-9a01-329ffe79e2ff"])
            .unwrap();

        let expected: Value = serde_json::from_str(
            r#"
    {
        "id": "b7be9512-69d2-40c9-9a01-329ffe79e2ff",
        "name": "10 Year Scalper",
        "symbol": "TYH25",
        "position_size": 2,
        "account_number": "some_id",
        "tick_size": "0.015625",
        "open_trade": null
    }"#,
        )
        .unwrap();

        assert_eq!(result, expected);
    }
}
