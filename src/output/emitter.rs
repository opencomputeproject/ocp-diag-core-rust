// (c) Meta Platforms, Inc. and affiliates.
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

use std::sync::atomic::{self, Ordering};
use std::sync::Arc;

use crate::output::{config, writer};
use crate::spec;

pub struct JsonEmitter {
    // HACK: public for tests, but this should come from config directly to where needed
    pub(crate) timestamp_provider: Box<dyn config::TimestampProvider + Send + Sync + 'static>,
    writer: writer::WriterType,
    seqno: Arc<atomic::AtomicU64>,
}

impl JsonEmitter {
    pub(crate) fn new(
        timestamp_provider: Box<dyn config::TimestampProvider + Send + Sync + 'static>,
        writer: writer::WriterType,
    ) -> Self {
        JsonEmitter {
            timestamp_provider,
            writer,
            seqno: Arc::new(atomic::AtomicU64::new(0)),
        }
    }

    fn serialize_artifact(&self, object: &spec::RootImpl) -> serde_json::Value {
        let root = spec::Root {
            artifact: object.clone(),
            timestamp: self.timestamp_provider.now(),
            seqno: self.incr_seqno(),
        };
        serde_json::json!(root)
    }

    fn incr_seqno(&self) -> u64 {
        self.seqno.fetch_add(1, Ordering::AcqRel)
    }

    pub async fn emit(&self, object: &spec::RootImpl) -> Result<(), writer::WriterError> {
        let serialized = self.serialize_artifact(object);
        match self.writer {
            writer::WriterType::File(ref file) => file.write(&serialized.to_string()).await?,
            writer::WriterType::Stdout(ref stdout) => stdout.write(&serialized.to_string()).await?,
            writer::WriterType::Buffer(ref buffer) => buffer.write(&serialized.to_string()).await?,
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use anyhow::{anyhow, Result};
    use assert_json_diff::assert_json_include;
    use serde_json::json;
    use tokio::sync::Mutex;

    use super::*;

    #[tokio::test]
    async fn test_emit_using_buffer_writer() -> Result<()> {
        let expected = json!({
            "schemaVersion": {
                "major": spec::SPEC_VERSION.0,
                "minor": spec::SPEC_VERSION.1,
            },
            "sequenceNumber": 0
        });

        let buffer = Arc::new(Mutex::new(vec![]));
        let writer = writer::BufferWriter::new(buffer.clone());
        let emitter = JsonEmitter::new(
            Box::new(config::NullTimestampProvider {}),
            writer::WriterType::Buffer(writer),
        );

        emitter
            .emit(&spec::RootImpl::SchemaVersion(
                spec::SchemaVersion::default(),
            ))
            .await?;

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
                "major": spec::SPEC_VERSION.0,
                "minor": spec::SPEC_VERSION.1,
            },
            "sequenceNumber": 0
        });
        let expected_2 = json!({
            "schemaVersion": {
                "major": spec::SPEC_VERSION.0,
                "minor": spec::SPEC_VERSION.1,
            },
            "sequenceNumber": 1
        });

        let buffer = Arc::new(Mutex::new(vec![]));
        let writer = writer::BufferWriter::new(buffer.clone());
        let emitter = JsonEmitter::new(
            Box::new(config::NullTimestampProvider {}),
            writer::WriterType::Buffer(writer),
        );

        let version = spec::RootImpl::SchemaVersion(spec::SchemaVersion::default());
        emitter.emit(&version).await?;
        emitter.emit(&version).await?;

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
