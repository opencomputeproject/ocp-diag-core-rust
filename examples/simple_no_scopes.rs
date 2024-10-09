// (c) Meta Platforms, Inc. and affiliates.
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

use anyhow::Result;

use ocptv::ocptv_log_debug;
use ocptv::output as tv;
use tv::{DutInfo, TestResult, TestRun, TestStatus};

/// Show that a run/step can be manually started and ended.
///
/// The scope version should be preferred, as it makes it safer not to miss the end
/// artifacts in case of unhandled exceptions or code misuse.
#[tokio::main]
async fn main() -> Result<()> {
    let dut = DutInfo::builder("dut0").build();
    let run = TestRun::builder("no scopes", "1.0")
        .build()
        .start(dut)
        .await?;

    let step = run.add_step("step0").start().await?;
    ocptv_log_debug!(step, "Some interesting message.").await?;
    step.end(TestStatus::Complete).await?;

    run.end(TestStatus::Complete, TestResult::Pass).await?;
    Ok(())
}
