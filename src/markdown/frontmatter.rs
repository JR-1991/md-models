use std::collections::HashMap;

use gray_matter::{engine::YAML, Matter};
use serde::{Deserialize, Serialize};

/// Represents the front matter data of a markdown file.
#[derive(Debug, Deserialize, Serialize)]
pub struct FrontMatter {
    /// A boolean field with a default value, renamed from `id-field`.
    #[serde(default = "default_id_field", rename = "id-field")]
    pub id_field: bool,
    /// Optional hashmap of prefixes.
    pub prefixes: Option<HashMap<String, String>>,
    /// Optional namespace map.
    pub nsmap: Option<HashMap<String, String>>,
    /// A string field with a default value representing the repository URL.
    #[serde(default = "default_repo")]
    pub repo: String,
    /// A string field with a default value representing the prefix.
    #[serde(default = "default_prefix")]
    pub prefix: String,
}

impl FrontMatter {
    /// Returns the value of the `id_field`.
    ///
    /// # Returns
    /// A boolean representing the `id_field`.
    pub fn id_field(&self) -> bool {
        self.id_field
    }

    /// Returns the prefixes as an optional vector of key-value pairs.
    ///
    /// # Returns
    /// An optional vector of tuples containing the prefixes.
    pub fn prefixes(&self) -> Option<Vec<(String, String)>> {
        self.prefixes.as_ref().map(|prefixes| {
            prefixes
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect()
        })
    }

    /// Returns a reference to the namespace map.
    ///
    /// # Returns
    /// A reference to an optional hashmap of the namespace map.
    pub fn nsmap(&self) -> &Option<HashMap<String, String>> {
        &self.nsmap
    }
}

impl Default for FrontMatter {
    /// Provides default values for `FrontMatter`.
    ///
    /// # Returns
    /// A `FrontMatter` instance with default values.
    fn default() -> Self {
        FrontMatter {
            id_field: default_id_field(),
            prefixes: None,
            repo: default_repo(),
            nsmap: None,
            prefix: default_prefix(),
        }
    }
}

/// Provides the default value for the `id_field`.
///
/// # Returns
/// A boolean with the default value `true`.
fn default_id_field() -> bool {
    true
}

/// Provides the default value for the `prefix`.
///
/// # Returns
/// A string with the default value `"md"`.
fn default_prefix() -> String {
    "md".to_string()
}

/// Provides the default value for the `repo`.
///
/// # Returns
/// A string with the default value `"http://mdmodel.net/"`.
fn default_repo() -> String {
    "http://mdmodel.net/".to_string()
}

/// Parses the front matter from the given content.
///
/// # Arguments
/// * `content` - A string slice that holds the content to parse.
///
/// # Returns
/// An optional `FrontMatter` if parsing is successful, otherwise `None`.
pub fn parse_frontmatter(content: &str) -> Option<FrontMatter> {
    let matter = Matter::<YAML>::new();
    let result = matter.parse(content);

    match result.data {
        None => None,
        Some(data) => {
            let matter = data
                .deserialize()
                .expect("Could not deserialize frontmatter");
            Some(matter)
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use std::path::Path;

    use super::*;

    /// Tests the `parse_frontmatter` function.
    #[test]
    fn test_parse_frontmatter() {
        // Arrange
        let path = Path::new("tests/data/model.md");
        let content = std::fs::read_to_string(path).expect("Could not read file");

        // Act
        let frontmatter = parse_frontmatter(&content)
            .expect("Could not parse frontmatter from file. Please check the file content.");

        // Assert
        assert_eq!(frontmatter.id_field, true);
        assert_eq!(
            frontmatter.prefixes.unwrap().get("schema").unwrap(),
            "http://schema.org/"
        );
        assert_eq!(
            frontmatter.nsmap.unwrap().get("tst").unwrap(),
            "http://example.com/test/"
        );
    }
}
