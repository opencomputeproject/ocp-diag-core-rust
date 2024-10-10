// (c) Meta Platforms, Inc. and affiliates.
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
#![allow(warnings)]

use anyhow::Result;
use async_trait::async_trait;
use futures::FutureExt;
use std::io;
use tokio::sync::mpsc;

use ocptv::ocptv_log_debug;
use ocptv::output as tv;
use tv::{Config, DutInfo, TestResult, TestRun, TestRunOutcome, TestStatus, Writer};

struct Channel {
    tx: mpsc::Sender<String>,
}

#[async_trait]
impl Writer for Channel {
    async fn write(&self, s: &str) -> Result<(), io::Error> {
        self.tx.send(s.to_owned()).await.map_err(io::Error::other)?;
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    #[cfg(feature = "boxed-scopes")]
    {
        let (tx, mut rx) = mpsc::channel::<String>(1);
        let task = tokio::spawn(async move {
            while let Some(s) = rx.recv().await {
                println!("{}", s);
            }
        });

        let config = Config::builder()
            .with_custom_output(Box::new(Channel { tx }))
            .build();

        let dut = DutInfo::builder("dut0").build();

        TestRun::builder("extensions", "1.0")
            .config(config)
            .build()
            .scope(dut, |r| {
                async move {
                    ocptv_log_debug!(r, "log debug").await?;

                    Ok(TestRunOutcome {
                        status: TestStatus::Complete,
                        result: TestResult::Pass,
                    })
                }
                .boxed()
            })
            .await?;

        task.await?;
    }
    Ok(())
}
