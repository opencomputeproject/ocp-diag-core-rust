// (c) Meta Platforms, Inc. and affiliates.
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

use core::fmt::Debug;
use std::clone::Clone;
use std::io;
use std::io::Write;
use std::path::Path;
use std::sync::atomic;
use std::sync::Arc;

use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;

use crate::output::models;

#[derive(Debug, thiserror::Error, derive_more::Display)]
#[non_exhaustive]
pub enum WriterError {
    IoError(#[from] io::Error),
}

pub(crate) enum WriterType {
    Stdout(StdoutWriter),
    File(FileWriter),
    Buffer(BufferWriter),
}

pub struct FileWriter {
    file: Arc<Mutex<File>>,
}

impl FileWriter {
    pub async fn new<P: AsRef<Path>>(path: P) -> Result<Self, WriterError> {
        let file = File::create(path).await.map_err(WriterError::IoError)?;
        Ok(FileWriter {
            file: Arc::new(Mutex::new(file)),
        })
    }

    async fn write(&self, s: &str) -> Result<(), WriterError> {
        let mut handle = self.file.lock().await;
        let mut buf = Vec::<u8>::new();
        writeln!(buf, "{}", s)?;
        handle.write_all(&buf).await.map_err(WriterError::IoError)?;
        handle.flush().await.map_err(WriterError::IoError)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct BufferWriter {
    buffer: Arc<Mutex<Vec<String>>>,
}

impl BufferWriter {
    pub fn new(buffer: Arc<Mutex<Vec<String>>>) -> Self {
        Self { buffer }
    }

    async fn write(&self, s: &str) -> Result<(), WriterError> {
        self.buffer.lock().await.push(s.to_string());
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct StdoutWriter {}

#[allow(clippy::new_without_default)]
impl StdoutWriter {
    pub fn new() -> Self {
        StdoutWriter {}
    }

    async fn write(&self, s: &str) -> Result<(), WriterError> {
        println!("{}", s);
        Ok(())
    }
}

pub struct JsonEmitter {
    sequence_no: Arc<atomic::AtomicU64>,
    timezone: chrono_tz::Tz,
    writer: WriterType,
}

impl JsonEmitter {
    pub(crate) fn new(timezone: chrono_tz::Tz, writer: WriterType) -> Self {
        JsonEmitter {
            timezone,
            writer,
            sequence_no: Arc::new(atomic::AtomicU64::new(0)),
        }
    }

    fn serialize_artifact(&self, object: &models::RootArtifactSpec) -> serde_json::Value {
        let now = chrono::Local::now();
        let now_tz = now.with_timezone(&self.timezone);
        let out_artifact = models::RootSpec {
            artifact: object.clone(),
            timestamp: now_tz,
            seqno: self.next_sequence_no(),
        };
        serde_json::json!(out_artifact)
    }

    fn next_sequence_no(&self) -> u64 {
        self.sequence_no.fetch_add(1, atomic::Ordering::SeqCst);
        self.sequence_no.load(atomic::Ordering::SeqCst)
    }

    pub async fn emit(&self, object: &models::RootArtifactSpec) -> Result<(), WriterError> {
        let serialized = self.serialize_artifact(object);
        match self.writer {
            WriterType::File(ref file) => file.write(&serialized.to_string()).await?,
            WriterType::Stdout(ref stdout) => stdout.write(&serialized.to_string()).await?,
            WriterType::Buffer(ref buffer) => buffer.write(&serialized.to_string()).await?,
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use anyhow::{anyhow, Result};
    use assert_json_diff::assert_json_include;
    use serde_json::json;

    use super::*;
    use crate::output as tv;
    use tv::run::SchemaVersion;

    #[tokio::test]
    async fn test_emit_using_buffer_writer() -> Result<()> {
        let expected = json!({
            "schemaVersion": {
                "major": models::SPEC_VERSION.0,
                "minor": models::SPEC_VERSION.1,
            },
            "sequenceNumber": 1
        });

        let buffer = Arc::new(Mutex::new(vec![]));
        let writer = BufferWriter::new(buffer.clone());
        let emitter = JsonEmitter::new(chrono_tz::UTC, WriterType::Buffer(writer));

        let version = SchemaVersion::new();
        emitter.emit(&version.to_artifact()).await?;

        let deserialized = serde_json::from_str::<serde_json::Value>(
            buffer.lock().await.first().ok_or(anyhow!("no outputs"))?,
        )?;
        assert_json_include!(actual: deserialized, expected: expected);

        Ok(())
    }

    #[tokio::test]
    async fn test_sequence_number_increments_at_each_call() -> Result<()> {
        let expected_1 = json!({
            "schemaVersion": {
                "major": models::SPEC_VERSION.0,
                "minor": models::SPEC_VERSION.1,
            },
            "sequenceNumber": 1
        });
        let expected_2 = json!({
            "schemaVersion": {
                "major": models::SPEC_VERSION.0,
                "minor": models::SPEC_VERSION.1,
            },
            "sequenceNumber": 2
        });

        let buffer = Arc::new(Mutex::new(vec![]));
        let writer = BufferWriter::new(buffer.clone());
        let emitter = JsonEmitter::new(chrono_tz::UTC, WriterType::Buffer(writer));
        let version = SchemaVersion::new();
        emitter.emit(&version.to_artifact()).await?;
        emitter.emit(&version.to_artifact()).await?;

        let deserialized = serde_json::from_str::<serde_json::Value>(
            buffer.lock().await.first().ok_or(anyhow!("no outputs"))?,
        )?;
        assert_json_include!(actual: deserialized, expected: expected_1);

        let deserialized = serde_json::from_str::<serde_json::Value>(
            buffer.lock().await.get(1).ok_or(anyhow!("no outputs"))?,
        )?;
        assert_json_include!(actual: deserialized, expected: expected_2);

        Ok(())
    }
}
