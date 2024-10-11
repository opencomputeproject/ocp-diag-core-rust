// (c) Meta Platforms, Inc. and affiliates.
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
// #![allow(warnings)]

use anyhow::Result;
use futures::FutureExt;

use ocptv::output as tv;
use tv::{TestResult, TestStatus, ValidatorType};

async fn run_measure_step(step: &tv::StartedTestStep) -> Result<TestStatus, tv::OcptvError> {
    step.add_measurement_with_details(
        tv::Measurement::builder("temp", 40.into())
            .add_validator(
                tv::Validator::builder(ValidatorType::GreaterThan, 30.into())
                    .name("gt_30")
                    .build(),
            )
            .build(),
    )
    .await?;

    step.add_measurement_series_with_details(
        tv::MeasurementSeriesInfo::builder("fan_speed")
            .unit("rpm")
            .add_validator(
                tv::Validator::builder(ValidatorType::LessThanOrEqual, 3000.into()).build(),
            )
            .build(),
    )
    .scope(|s| {
        async move {
            s.add_measurement(1000.into()).await?;

            Ok(())
        }
        .boxed()
    })
    .await?;

    step.add_measurement_with_details(
        tv::Measurement::builder("fan_speed", 1200.into())
            .unit("rpm")
            .build(),
    )
    .await?;

    Ok(TestStatus::Complete)
}

/// Showcase a measurement item and series, both using validators to document
/// what the diagnostic package actually validated.
#[tokio::main]
async fn main() -> Result<()> {
    let dut = tv::DutInfo::builder("dut0").build();

    tv::TestRun::builder("simple measurement", "1.0")
        .build()
        .scope(dut, |r| {
            async move {
                r.add_step("step0")
                    .scope(|s| run_measure_step(s).boxed())
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
