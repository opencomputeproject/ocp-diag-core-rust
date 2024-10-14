// (c) Meta Platforms, Inc. and affiliates.
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

use anyhow::Result;

use ocptv::output as tv;
use tv::{TestResult, TestStatus};

async fn run_measure_step(step: tv::ScopedTestStep) -> Result<TestStatus, tv::OcptvError> {
    step.add_measurement("temperature", 42.5).await?;
    step.add_measurement_detail(
        tv::Measurement::builder("fan_speed", 1200)
            .unit("rpm")
            .build(),
    )
    .await?;

    Ok(TestStatus::Complete)
}

/// Simple demo with some measurements taken but not referencing DUT hardware.
#[tokio::main]
async fn main() -> Result<()> {
    let dut = tv::DutInfo::builder("dut0").build();

    tv::TestRun::builder("simple measurement", "1.0")
        .build()
        .scope(dut, |r| async move {
            r.add_step("step0").scope(run_measure_step).await?;

            Ok(tv::TestRunOutcome {
                status: TestStatus::Complete,
                result: TestResult::Pass,
            })
        })
        .await?;

    Ok(())
}
