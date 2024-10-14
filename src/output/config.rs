// (c) Meta Platforms, Inc. and affiliates.
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

use std::path::Path;
use std::sync::Arc;

use tokio::sync::Mutex;

use crate::output as tv;
use crate::output::writer::{self, BufferWriter, FileWriter, StdoutWriter, WriterType};

/// The configuration repository for the TestRun.
pub struct Config {
    // All fields are readable for any impl inside the crate.
    pub(crate) timestamp_provider: Box<dyn TimestampProvider + Send + Sync + 'static>,
    pub(crate) writer: WriterType,
}

impl Config {
    /// Creates a new [`ConfigBuilder`]
    ///
    /// # Examples
    /// ```rust
    /// # use ocptv::output::*;
    /// let builder = Config::builder();
    /// ```
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::new()
    }
}

/// The builder for the [`Config`] object.
pub struct ConfigBuilder {
    timestamp_provider: Box<dyn TimestampProvider + Send + Sync + 'static>,
    writer: Option<WriterType>,
}

impl ConfigBuilder {
    fn new() -> Self {
        Self {
            timestamp_provider: Box::new(ConfiguredTzProvider { tz: chrono_tz::UTC }),
            writer: Some(WriterType::Stdout(StdoutWriter::new())),
        }
    }

    /// TODO: docs for all these
    pub fn timezone(mut self, timezone: chrono_tz::Tz) -> Self {
        self.timestamp_provider = Box::new(ConfiguredTzProvider { tz: timezone });
        self
    }

    pub fn with_timestamp_provider(
        mut self,
        timestamp_provider: Box<dyn TimestampProvider + Send + Sync + 'static>,
    ) -> Self {
        self.timestamp_provider = timestamp_provider;
        self
    }

    pub fn with_buffer_output(mut self, buffer: Arc<Mutex<Vec<String>>>) -> Self {
        self.writer = Some(WriterType::Buffer(BufferWriter::new(buffer)));
        self
    }

    pub async fn with_file_output<P: AsRef<Path>>(
        mut self,
        path: P,
    ) -> Result<Self, tv::OcptvError> {
        self.writer = Some(WriterType::File(FileWriter::new(path).await?));
        Ok(self)
    }

    pub fn with_custom_output(
        mut self,
        custom: Box<dyn writer::Writer + Send + Sync + 'static>,
    ) -> Self {
        self.writer = Some(WriterType::Custom(custom));
        self
    }

    pub fn build(self) -> Config {
        Config {
            timestamp_provider: self.timestamp_provider,
            writer: self
                .writer
                .unwrap_or(WriterType::Stdout(StdoutWriter::new())),
        }
    }
}

/// TODO: docs
pub trait TimestampProvider {
    fn now(&self) -> chrono::DateTime<chrono_tz::Tz>;
}

struct ConfiguredTzProvider {
    tz: chrono_tz::Tz,
}

impl TimestampProvider for ConfiguredTzProvider {
    fn now(&self) -> chrono::DateTime<chrono_tz::Tz> {
        chrono::Local::now().with_timezone(&self.tz)
    }
}
