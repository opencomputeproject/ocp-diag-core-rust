// (c) Meta Platforms, Inc. and affiliates.
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::output::emitter;

/// The configuration repository for the TestRun.
pub struct Config {
    pub(crate) timezone: chrono_tz::Tz,
    pub(crate) writer: emitter::WriterType,
}

impl Config {
    /// Creates a new [`ConfigBuilder`]
    ///
    /// # Examples
    /// ```rust
    /// # use ocptv::output::*;
    ///
    /// let builder = Config::builder();
    /// ```
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::new()
    }
}

/// The builder for the [`Config`] object.
pub struct ConfigBuilder {
    timezone: Option<chrono_tz::Tz>,
    writer: Option<emitter::WriterType>,
}

impl ConfigBuilder {
    fn new() -> Self {
        Self {
            timezone: None,
            writer: Some(emitter::WriterType::Stdout(emitter::StdoutWriter::new())),
        }
    }

    pub fn timezone(mut self, timezone: chrono_tz::Tz) -> Self {
        self.timezone = Some(timezone);
        self
    }

    pub fn with_buffer_output(mut self, buffer: Arc<Mutex<Vec<String>>>) -> Self {
        self.writer = Some(emitter::WriterType::Buffer(emitter::BufferWriter::new(
            buffer,
        )));
        self
    }

    pub async fn with_file_output<P: AsRef<Path>>(
        mut self,
        path: P,
    ) -> Result<Self, emitter::WriterError> {
        self.writer = Some(emitter::WriterType::File(
            emitter::FileWriter::new(path).await?,
        ));
        Ok(self)
    }

    pub fn build(self) -> Config {
        Config {
            timezone: self.timezone.unwrap_or(chrono_tz::UTC),
            writer: self
                .writer
                .unwrap_or(emitter::WriterType::Stdout(emitter::StdoutWriter::new())),
        }
    }
}
