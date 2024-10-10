// (c) Meta Platforms, Inc. and affiliates.
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

use anyhow::Result;
use futures::FutureExt;

use ocptv::ocptv_log_info;
use ocptv::output as tv;
use tv::{TestResult, TestStatus};

/// Show a scoped run with scoped steps, everything starts at "with" time and
/// ends automatically when the block ends (regardless of unhandled exceptions).
#[tokio::main]
async fn main() -> Result<()> {
    let dut = tv::DutInfo::builder("dut0").build();

    tv::TestRun::builder("step fail", "1.0")
        .build()
        .scope(dut, |r| {
            async move {
                r.add_step("step0")
                    .scope(|s| {
                        async move {
                            ocptv_log_info!(s, "info log").await?;
                            Ok(TestStatus::Complete)
                        }
                        .boxed()
                    })
                    .await?;

                r.add_step("step1")
                    .scope(|_s| async move { Ok(TestStatus::Error) }.boxed())
                    .await?;

                Ok(tv::TestRunOutcome {
                    status: TestStatus::Complete,
                    result: TestResult::Fail,
                })
            }
            .boxed()
        })
        .await?;

    Ok(())
}
