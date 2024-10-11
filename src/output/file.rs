// (c) Meta Platforms, Inc. and affiliates.
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

use std::collections::BTreeMap;

use mime;

use crate::output::{self as tv, trait_ext::MapExt};
use crate::spec;

/// This structure represents a File message.
///
/// ref: <https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#file>
///
/// # Examples
///
/// ## Create a File object with the `new` method
///
/// ```
/// # use ocptv::output::*;
/// let uri = Uri::parse("file:///tmp/foo").unwrap();
/// let file = File::new("name", uri);
/// ```
///
/// ## Create a File object with the `builder` method
///
/// ```
/// # use ocptv::output::*;
/// # use std::str::FromStr;
/// let uri = Uri::parse("file:///tmp/foo").unwrap();
/// let file = File::builder("name", uri)
///     .is_snapshot(true)
///     .description("description")
///     .content_type(mime::TEXT_PLAIN)
///     .add_metadata("key", "value".into())
///     .build();
/// ```
pub struct File {
    name: String,
    uri: tv::Uri,
    is_snapshot: bool,
    description: Option<String>,
    content_type: Option<mime::Mime>,
    metadata: BTreeMap<String, tv::Value>,
}

impl File {
    /// Builds a new File object.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ocptv::output::*;
    /// let uri = Uri::parse("file:///tmp/foo").unwrap();
    /// let file = File::new("name", uri);
    /// ```
    pub fn new(name: &str, uri: tv::Uri) -> Self {
        File {
            name: name.to_owned(),
            uri,
            is_snapshot: false,
            description: None,
            content_type: None,
            metadata: BTreeMap::new(),
        }
    }

    /// Builds a new File object using [`FileBuilder`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use ocptv::output::*;
    /// # use std::str::FromStr;
    /// let uri = Uri::parse("file:///tmp/foo").unwrap();
    /// let file = File::builder("name", uri)
    ///     .description("description")
    ///     .content_type(mime::TEXT_PLAIN)
    ///     .add_metadata("key", "value".into())
    ///     .build();
    /// ```
    pub fn builder(name: &str, uri: tv::Uri) -> FileBuilder {
        FileBuilder::new(name, uri)
    }

    /// Creates an artifact from a File object.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ocptv::output::*;
    /// let uri = Uri::parse("file:///tmp/foo").unwrap();
    /// let file = File::new("name", uri);
    /// let _ = file.to_artifact();
    /// ```
    pub fn to_artifact(&self) -> spec::File {
        spec::File {
            name: self.name.clone(),
            uri: self.uri.as_str().to_owned(),
            is_snapshot: self.is_snapshot,
            description: self.description.clone(),
            content_type: self.content_type.as_ref().map(|ct| ct.to_string()),
            metadata: self.metadata.option(),
        }
    }
}

/// This structure builds a [`File`] object.
///
/// # Examples
///
/// ```
/// # use ocptv::output::*;
/// # use std::str::FromStr;
/// let uri = Uri::parse("file:///tmp/foo").unwrap();
/// let builder = File::builder("name", uri)
///     .description("description")
///     .content_type(mime::TEXT_PLAIN)
///     .add_metadata("key", "value".into());
/// let file = builder.build();
/// ```
pub struct FileBuilder {
    name: String,
    uri: tv::Uri,
    is_snapshot: bool,
    description: Option<String>,
    content_type: Option<mime::Mime>,

    metadata: BTreeMap<String, tv::Value>,
}

impl FileBuilder {
    fn new(name: &str, uri: tv::Uri) -> Self {
        FileBuilder {
            name: name.to_string(),
            uri,
            is_snapshot: false,
            description: None,
            content_type: None,
            metadata: BTreeMap::new(),
        }
    }

    /// Set the is_snapshot attribute in a [`FileBuilder`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use ocptv::output::*;
    /// let uri = Uri::parse("file:///tmp/foo").unwrap();
    /// let builder = File::builder("name", uri)
    ///     .is_snapshot(true);
    /// ```
    pub fn is_snapshot(mut self, value: bool) -> FileBuilder {
        self.is_snapshot = value;
        self
    }

    /// Add a description to a [`FileBuilder`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use ocptv::output::*;
    /// let uri = Uri::parse("file:///tmp/foo").unwrap();
    /// let builder = File::builder("name", uri)
    ///     .description("description");
    /// ```
    pub fn description(mut self, description: &str) -> FileBuilder {
        self.description = Some(description.to_owned());
        self
    }

    /// Add a content_type to a [`FileBuilder`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use ocptv::output::*;
    /// # use std::str::FromStr;
    /// let uri = Uri::parse("file:///tmp/foo").unwrap();
    /// let builder = File::builder("name", uri)
    ///     .content_type(mime::TEXT_PLAIN);
    /// ```
    pub fn content_type(mut self, content_type: mime::Mime) -> FileBuilder {
        self.content_type = Some(content_type);
        self
    }

    /// Add custom metadata to a [`FileBuilder`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use ocptv::output::*;
    ///
    /// let uri = Uri::parse("file:///tmp/foo").unwrap();
    /// let builder = File::builder("name", uri)
    ///     .add_metadata("key", "value".into());
    /// ```
    pub fn add_metadata(mut self, key: &str, value: tv::Value) -> FileBuilder {
        self.metadata.insert(key.to_string(), value);
        self
    }

    /// Builds a [`File`] object from a [`FileBuilder`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use ocptv::output::*;
    ///
    /// let uri = Uri::parse("file:///tmp/foo").unwrap();
    /// let builder = File::builder("name", uri);
    /// let file = builder.build();
    /// ```
    pub fn build(self) -> File {
        File {
            name: self.name,
            uri: self.uri,
            is_snapshot: self.is_snapshot,
            description: self.description,
            content_type: self.content_type,
            metadata: self.metadata,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output as tv;
    use crate::spec;
    use anyhow::Result;
    use maplit::btreemap;
    use maplit::convert_args;

    #[test]
    fn test_file_as_test_step_descendant_to_artifact() -> Result<()> {
        let name = "name".to_owned();
        let uri = tv::Uri::parse("file:///tmp/foo")?;
        let is_snapshot = false;
        let file = File::new(&name, uri.clone());

        let artifact = file.to_artifact();

        assert_eq!(
            artifact,
            spec::File {
                name,
                uri: uri.as_str().to_owned(),
                is_snapshot,
                description: None,
                content_type: None,
                metadata: None,
            }
        );

        Ok(())
    }

    #[test]
    fn test_file_builder_as_test_step_descendant_to_artifact() -> Result<()> {
        let name = "name".to_owned();
        let uri = tv::Uri::parse("file:///tmp/foo")?;
        let is_snapshot = false;
        let description = "description".to_owned();
        let content_type = mime::TEXT_PLAIN;
        let meta_key = "key";
        let meta_value = tv::Value::from("value");
        let metadata = convert_args!(btreemap!(
            meta_key => meta_value.clone(),
        ));

        let file = File::builder(&name, uri.clone())
            .is_snapshot(is_snapshot)
            .description(&description)
            .content_type(content_type.clone())
            .add_metadata(meta_key, meta_value.clone())
            .add_metadata(meta_key, meta_value.clone())
            .build();

        let artifact = file.to_artifact();
        assert_eq!(
            artifact,
            spec::File {
                name,
                uri: uri.as_str().to_owned(),
                is_snapshot,
                description: Some(description),
                content_type: Some(content_type.to_string()),
                metadata: Some(metadata),
            }
        );

        Ok(())
    }
}
