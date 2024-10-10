// (c) Meta Platforms, Inc. and affiliates.
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
// #![allow(warnings)]

use anyhow::Result;
use futures::FutureExt;

use ocptv::output::{self as tv, MeasurementSeriesInfo};
use tv::{
    DutInfo, Measurement, StartedTestStep, TestResult, TestRun, TestRunOutcome, TestStatus,
    Validator, ValidatorType,
};

async fn run_measure_step(step: &StartedTestStep) -> Result<TestStatus, tv::OcptvError> {
    step.add_measurement_with_details(
        &Measurement::builder("temp", 40.into())
            .add_validator(
                &Validator::builder(ValidatorType::GreaterThan, 30.into())
                    .name("gt_30")
                    .build(),
            )
            .build(),
    )
    .await?;

    step.add_measurement_series_with_details(
        MeasurementSeriesInfo::builder("fan_speed")
            .unit("rpm")
            .add_validator(&Validator::builder(ValidatorType::LessThanOrEqual, 3000.into()).build())
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
        &Measurement::builder("fan_speed", 1200.into())
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
    let dut = DutInfo::builder("dut0").build();

    #[cfg(feature = "boxed-scopes")]
    TestRun::builder("simple measurement", "1.0")
        .build()
        .scope(dut, |r| {
            async move {
                r.add_step("step0")
                    .scope(|s| run_measure_step(s).boxed())
                    .await?;

                Ok(TestRunOutcome {
                    status: TestStatus::Complete,
                    result: TestResult::Pass,
                })
            }
            .boxed()
        })
        .await?;

    Ok(())
}
