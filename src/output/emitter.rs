// (c) Meta Platforms, Inc. and affiliates.
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

use std::io;
use std::sync::atomic::{self, Ordering};
use std::sync::Arc;

use unwrap_infallible::UnwrapInfallible;

use crate::output::{
    config,
    writer::{self, WriterType},
};
use crate::spec;

pub struct JsonEmitter {
    timestamp_provider: Box<dyn config::TimestampProvider + Send + Sync + 'static>,
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

    fn incr_seqno(&self) -> u64 {
        self.seqno.fetch_add(1, Ordering::AcqRel)
    }

    fn serialize_artifact(&self, object: &spec::RootImpl) -> String {
        let root = spec::Root {
            artifact: object.clone(),
            timestamp: self.timestamp_provider.now(),
            seqno: self.incr_seqno(),
        };

        serde_json::json!(root).to_string()
    }

    pub fn timestamp_provider(&self) -> &(dyn config::TimestampProvider + Send + Sync + 'static) {
        &*self.timestamp_provider
    }

    pub async fn emit(&self, object: &spec::RootImpl) -> Result<(), io::Error> {
        let s = self.serialize_artifact(object);

        match &self.writer {
            WriterType::File(file) => file.write(&s).await?,
            WriterType::Stdout(stdout) => stdout.write(&s).await.unwrap_infallible(),
            WriterType::Buffer(buffer) => buffer.write(&s).await.unwrap_infallible(),

            WriterType::Custom(custom) => custom.write(&s).await?,
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use anyhow::{anyhow, Result};
    use assert_json_diff::assert_json_eq;
    use serde_json::json;
    use tokio::sync::Mutex;

    use super::*;

    pub struct NullTimestampProvider {}

    impl NullTimestampProvider {
        // warn: linter is wrong here, this is used in a serde_json::json! block
        #[allow(dead_code)]
        pub const FORMATTED: &str = "1970-01-01T00:00:00.000Z";
    }

    impl config::TimestampProvider for NullTimestampProvider {
        fn now(&self) -> chrono::DateTime<chrono_tz::Tz> {
            chrono::DateTime::from_timestamp_nanos(0).with_timezone(&chrono_tz::UTC)
        }
    }

    #[tokio::test]
    async fn test_emit_using_buffer_writer() -> Result<()> {
        let expected = json!({
            "schemaVersion": {
                "major": spec::SPEC_VERSION.0,
                "minor": spec::SPEC_VERSION.1,
            },
            "sequenceNumber": 0,
            "timestamp": NullTimestampProvider::FORMATTED,
        });

        let buffer = Arc::new(Mutex::new(vec![]));
        let writer = writer::BufferWriter::new(buffer.clone());
        let emitter = JsonEmitter::new(
            Box::new(NullTimestampProvider {}),
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
        assert_json_eq!(deserialized, expected);

        Ok(())
    }

    #[tokio::test]
    async fn test_sequence_number_increments_at_each_call() -> Result<()> {
        let expected_1 = json!({
            "schemaVersion": {
                "major": spec::SPEC_VERSION.0,
                "minor": spec::SPEC_VERSION.1,
            },
            "sequenceNumber": 0,
            "timestamp": NullTimestampProvider::FORMATTED,
        });
        let expected_2 = json!({
            "schemaVersion": {
                "major": spec::SPEC_VERSION.0,
                "minor": spec::SPEC_VERSION.1,
            },
            "sequenceNumber": 1,
            "timestamp": NullTimestampProvider::FORMATTED,
        });

        let buffer = Arc::new(Mutex::new(vec![]));
        let writer = writer::BufferWriter::new(buffer.clone());
        let emitter = JsonEmitter::new(
            Box::new(NullTimestampProvider {}),
            writer::WriterType::Buffer(writer),
        );

        let version = spec::RootImpl::SchemaVersion(spec::SchemaVersion::default());
        emitter.emit(&version).await?;
        emitter.emit(&version).await?;

        let deserialized = serde_json::from_str::<serde_json::Value>(
            buffer.lock().await.first().ok_or(anyhow!("no outputs"))?,
        )?;
        assert_json_eq!(deserialized, expected_1);

        let deserialized = serde_json::from_str::<serde_json::Value>(
            buffer.lock().await.get(1).ok_or(anyhow!("no outputs"))?,
        )?;
        assert_json_eq!(deserialized, expected_2);

        Ok(())
    }
}
