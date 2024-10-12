// (c) Meta Platforms, Inc. and affiliates.
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

use std::{sync::Arc, time::Duration};

use anyhow::Result;
use rand;

use ocptv::output as tv;
use tv::{DutInfo, TestResult, TestRun, TestRunOutcome, TestStatus};

/// While the general recommendation is to run test steps sequentially, the specification does not
/// mandate for this to happen. This example shows multiple steps running in parallel, each
/// emitting their own measurements.
#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() -> Result<()> {
    let dut = DutInfo::builder("dut0").build();

    TestRun::builder("simple measurement", "1.0")
        .build()
        .scope(dut, |r| async move {
            let run = Arc::new(r);

            let tasks = (0..5)
                .map(|i| {
                    tokio::spawn({
                        let r = Arc::clone(&run);
                        async move {
                            r.add_step(&format!("step{}", i))
                                .scope(move |s| async move {
                                    let offset = rand::random::<u64>() % 10000;
                                    tokio::time::sleep(Duration::from_micros(offset)).await;

                                    let fan_speed = 1000 + 100 * i;
                                    s.add_measurement(&format!("fan{}", i), fan_speed)
                                        .await
                                        .unwrap();

                                    Ok(TestStatus::Complete)
                                })
                                .await
                        }
                    })
                })
                .collect::<Vec<_>>();

            for t in tasks {
                t.await.map_err(|e| tv::OcptvError::Other(Box::new(e)))??;
            }

            Ok(TestRunOutcome {
                status: TestStatus::Complete,
                result: TestResult::Pass,
            })
        })
        .await?;

    Ok(())
}
