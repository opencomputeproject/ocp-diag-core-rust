// (c) Meta Platforms, Inc. and affiliates.
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
#![allow(warnings)]

use anyhow::Result;
use chrono::Duration;

use ocptv::output::{self as tv};
use tv::{TestResult, TestStatus};

async fn step0_measurements(step: tv::ScopedTestStep) -> Result<TestStatus, tv::OcptvError> {
    let fan_speed = step
        .add_measurement_series_detail(
            tv::MeasurementSeriesDetail::builder("fan_speed")
                .unit("rpm")
                .build(),
        )
        .start()
        .await?;

    fan_speed.add_measurement(1000.into()).await?;
    fan_speed.add_measurement(1200.into()).await?;
    fan_speed.add_measurement(1500.into()).await?;

    fan_speed.end().await?;
    Ok(TestStatus::Complete)
}

async fn step1_measurements(step: tv::ScopedTestStep) -> Result<TestStatus, tv::OcptvError> {
    step.add_measurement_series_detail(
        tv::MeasurementSeriesDetail::builder("temp0")
            .unit("C")
            .build(),
    )
    .scope(|s| {
        async move {
            let two_seconds_ago =
                chrono::Local::now().with_timezone(&chrono_tz::UTC) - Duration::seconds(2);
            s.add_measurement_detail(
                tv::MeasurementElementDetail::builder(42.into())
                    .timestamp(two_seconds_ago)
                    .build(),
            )
            .await?;

            s.add_measurement(43.into()).await?;
            Ok(())
        }
    })
    .await?;

    Ok(TestStatus::Complete)
}

async fn step2_measurements(step: tv::ScopedTestStep) -> Result<TestStatus, tv::OcptvError> {
    let freq0 = step
        .add_measurement_series_detail(
            tv::MeasurementSeriesDetail::builder("freq0")
                .unit("hz")
                .build(),
        )
        .start()
        .await?;

    let freq1 = step
        .add_measurement_series_detail(
            tv::MeasurementSeriesDetail::builder("freq0")
                .unit("hz")
                .build(),
        )
        .start()
        .await?;

    freq0.add_measurement(1.0.into()).await?;
    freq1.add_measurement(2.0.into()).await?;
    freq0.add_measurement(1.2.into()).await?;

    freq0.end().await?;
    freq1.end().await?;
    Ok(TestStatus::Complete)
}

/// Show various patterns of time measurement series.
///
/// Step0 has a single series, manually ended.
/// Step1 has a single series but using a scope, so series ends automatically.
/// Step2 shows multiple measurement interspersed series, they can be concurrent.
#[tokio::main]
async fn main() -> Result<()> {
    let dut = tv::DutInfo::builder("dut0").build();

    tv::TestRun::builder("simple measurement", "1.0")
        .build()
        .scope(dut, |r| async move {
            r.add_step("step0")
                .scope(step0_measurements)
                .await?;

            r.add_step("step1")
                .scope(step1_measurements)
                .await?;

            r.add_step("step2")
                .scope(step2_measurements)
                .await?;

            Ok(tv::TestRunOutcome {
                status: TestStatus::Complete,
                result: TestResult::Pass,
            })
        })
        .await?;

    Ok(())
}
