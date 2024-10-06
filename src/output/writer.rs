// (c) Meta Platforms, Inc. and affiliates.
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

use std::convert::Infallible;
use std::io::{self, Write};
use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;

#[async_trait]
pub trait Writer {
    async fn write(&self, s: &str) -> Result<(), io::Error>;
}

pub enum WriterType {
    // optimization: static dispatch for these known types
    Stdout(StdoutWriter),
    File(FileWriter),
    Buffer(BufferWriter),

    Custom(Box<dyn Writer + Send + Sync + 'static>),
}

pub struct FileWriter {
    file: Arc<Mutex<fs::File>>,
}

impl FileWriter {
    pub async fn new<P: AsRef<Path>>(path: P) -> Result<Self, io::Error> {
        let file = fs::File::create(path).await?;
        Ok(FileWriter {
            file: Arc::new(Mutex::new(file)),
        })
    }

    pub async fn write(&self, s: &str) -> Result<(), io::Error> {
        let mut handle = self.file.lock().await;

        let mut buf = Vec::<u8>::new();
        writeln!(buf, "{}", s)?;

        handle.write_all(&buf).await?;
        handle.flush().await?;

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

    pub async fn write(&self, s: &str) -> Result<(), Infallible> {
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

    pub async fn write(&self, s: &str) -> Result<(), Infallible> {
        println!("{}", s);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::*;
    use anyhow::Result;

    struct ErrorWriter {}

    #[async_trait]
    impl Writer for ErrorWriter {
        async fn write(&self, _s: &str) -> Result<(), io::Error> {
            Err(io::Error::other("err"))
        }
    }

    #[tokio::test]
    async fn test_ocptv_error_has_public_source() -> Result<()> {
        let dut = DutInfo::builder("dut_id").build();
        let run_builder = TestRun::builder("run_name", &dut, "1.0").config(
            Config::builder()
                .with_custom_output(Box::new(ErrorWriter {}))
                .build(),
        );

        let actual = run_builder.build().start().await;
        assert!(actual.is_err());

        match &actual {
            Err(OcptvError::IoError(ioe)) => {
                assert_eq!(ioe.kind(), io::ErrorKind::Other);
            }
            _ => panic!("unknown error"),
        }

        Ok(())
    }
}
