use std::collections::HashMap;

/// A struct to manage primitive types and their corresponding JSON mappings.
pub struct PrimitiveTypes {
    types: Vec<String>,
    json_mappings: HashMap<String, String>,
}

impl Default for PrimitiveTypes {
    fn default() -> Self {
        PrimitiveTypes::new()
    }
}

impl PrimitiveTypes {
    /// Creates a new instance of `PrimitiveTypes` with predefined primitive types
    /// and their corresponding JSON mappings.
    pub fn new() -> Self {
        let mut json_mappings = HashMap::new();

        json_mappings.insert("string".to_string(), "string".to_string());
        json_mappings.insert("float".to_string(), "number".to_string());
        json_mappings.insert("integer".to_string(), "integer".to_string());
        json_mappings.insert("boolean".to_string(), "boolean".to_string());
        json_mappings.insert("bool".to_string(), "boolean".to_string());
        json_mappings.insert("null".to_string(), "null".to_string());

        PrimitiveTypes {
            types: vec![
                "string".to_string(),
                "float".to_string(),
                "integer".to_string(),
                "boolean".to_string(),
                "bool".to_string(),
                "null".to_string(),
            ],
            json_mappings,
        }
    }

    /// Filters and returns the list of non-primitive types from the given list of data types.
    ///
    /// # Arguments
    ///
    /// * `dtypes` - A reference to a vector of data types to be filtered.
    ///
    /// # Returns
    ///
    /// A vector containing only the non-primitive types from the input vector.
    pub fn filter_non_primitives(&self, dtypes: &Vec<String>) -> Vec<String> {
        let mut non_primitive_types: Vec<String> = Vec::new();
        for dtype in dtypes {
            if !self.is_primitive(dtype) {
                non_primitive_types.push(dtype.to_string());
            }
        }

        non_primitive_types
    }

    /// Filters and returns the list of primitive types from the given list of data types.
    ///
    /// # Arguments
    ///
    /// * `dtypes` - A reference to a vector of data types to be filtered.
    ///
    /// # Returns
    ///
    /// A vector containing only the primitive types from the input vector.
    pub fn filter_primitive(&self, dtypes: &Vec<String>) -> Vec<String> {
        let mut primitive_types: Vec<String> = Vec::new();
        for dtype in dtypes {
            if self.is_primitive(dtype) {
                primitive_types.push(dtype.to_string());
            }
        }

        primitive_types
    }

    /// Checks if the given data type is a primitive type.
    ///
    /// # Arguments
    ///
    /// * `dtype` - A string slice representing the data type to be checked.
    ///
    /// # Returns
    ///
    /// A boolean value indicating whether the data type is a primitive type.
    fn is_primitive(&self, dtype: &str) -> bool {
        self.types.contains(&dtype.to_string())
    }

    /// Converts a data type to its corresponding JSON representation.
    ///
    /// # Arguments
    ///
    /// * `dtype` - A reference to a string representing the data type to be converted.
    ///
    /// # Returns
    ///
    /// A string representing the JSON mapping of the data type.
    ///
    /// # Panics
    ///
    /// Panics if the data type is not a primitive type.
    pub fn dtype_to_json(&self, dtype: &String) -> String {
        if !self.json_mappings.contains_key(dtype) {
            panic!("The data type {} is not a primitive type", dtype)
        } else {
            self.json_mappings[dtype].to_string()
        }
    }
}
