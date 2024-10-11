// (c) Meta Platforms, Inc. and affiliates.
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

use anyhow::Result;
use futures::FutureExt;

use ocptv::output as tv;
use rand::Rng;
use tv::{TestResult, TestStatus};

fn get_fan_speed() -> i32 {
    let mut rng = rand::thread_rng();
    rng.gen_range(1500..1700)
}

async fn run_diagnosis_step(step: &tv::StartedTestStep) -> Result<TestStatus, tv::OcptvError> {
    let fan_speed = get_fan_speed();

    if fan_speed >= 1600 {
        step.diagnosis("fan_ok", tv::DiagnosisType::Pass).await?;
    } else {
        step.diagnosis("fan_low", tv::DiagnosisType::Fail).await?;
    }

    Ok(TestStatus::Complete)
}

/// Simple demo with diagnosis.
#[tokio::main]
async fn main() -> Result<()> {
    let dut = tv::DutInfo::builder("dut0").build();

    tv::TestRun::builder("simple measurement", "1.0")
        .build()
        .scope(dut, |r| {
            async move {
                r.add_step("step0")
                    .scope(|s| { s.file("output") }.boxed())
                    .await?;

                Ok(tv::TestRunOutcome {
                    status: TestStatus::Complete,
                    result: TestResult::Pass,
                })
            }
            .boxed()
        })
        .await?;

    Ok(())
}
