// (c) Meta Platforms, Inc. and affiliates.
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

use std::io::{self, Write};
use std::path::Path;
use std::sync::Arc;

use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;

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
    file: Arc<Mutex<fs::File>>,
}

impl FileWriter {
    pub async fn new<P: AsRef<Path>>(path: P) -> Result<Self, WriterError> {
        let file = fs::File::create(path).await.map_err(WriterError::IoError)?;
        Ok(FileWriter {
            file: Arc::new(Mutex::new(file)),
        })
    }

    pub async fn write(&self, s: &str) -> Result<(), WriterError> {
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

    pub async fn write(&self, s: &str) -> Result<(), WriterError> {
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

    pub async fn write(&self, s: &str) -> Result<(), WriterError> {
        println!("{}", s);
        Ok(())
    }
}
