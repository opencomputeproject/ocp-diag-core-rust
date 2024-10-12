// (c) Meta Platforms, Inc. and affiliates.
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

use std::str::FromStr;

use anyhow::Result;
use futures::FutureExt;

use ocptv::output as tv;
use tv::{TestResult, TestStatus};

async fn run_file_step(step: &tv::StartedTestStep) -> Result<TestStatus, tv::OcptvError> {
    let uri = tv::Uri::from_str("file:///root/mem_cfg_log").unwrap();
    step.add_file("mem_cfg_log", uri).await?;

    Ok(TestStatus::Complete)
}

/// Simple demo with file.
#[tokio::main]
async fn main() -> Result<()> {
    let dut = tv::DutInfo::builder("dut0").build();

    tv::TestRun::builder("simple measurement", "1.0")
        .build()
        .scope(dut, |r| async move {
            r.add_step("step0")
                .scope(|s| run_file_step(s).boxed())
                .await?;

            Ok(tv::TestRunOutcome {
                status: TestStatus::Complete,
                result: TestResult::Pass,
            })
        })
        .await?;

    Ok(())
}
