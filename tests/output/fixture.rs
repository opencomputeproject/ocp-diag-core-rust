// (c) Meta Platforms, Inc. and affiliates.
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

use std::sync::Arc;

use anyhow::Result;
use assert_json_diff::assert_json_eq;
use futures::future::Future;
use serde_json::json;
use tokio::sync::Mutex;

use ocptv::output::{
    Config, DutInfo, HardwareInfo, Ident, OcptvError, ScopedTestRun, ScopedTestStep, SoftwareInfo,
    SoftwareType, TestResult, TestRun, TestRunBuilder, TestRunOutcome, TestStatus,
    TimestampProvider, SPEC_VERSION,
};

pub const DATETIME: chrono::DateTime<chrono::offset::Utc> =
    chrono::DateTime::from_timestamp_nanos(0);
pub const DATETIME_FORMATTED: &str = "1970-01-01T00:00:00.000Z";
pub struct FixedTsProvider {}

impl TimestampProvider for FixedTsProvider {
    fn now(&self) -> chrono::DateTime<chrono_tz::Tz> {
        // all cases will use time 0 but this is configurable
        DATETIME.with_timezone(&chrono_tz::UTC)
    }
}

pub fn json_schema_version() -> serde_json::Value {
    // seqno for schemaVersion is always 0
    json!({
        "schemaVersion": {
            "major": SPEC_VERSION.0,
            "minor": SPEC_VERSION.1
        },
        "sequenceNumber": 0,
        "timestamp": DATETIME_FORMATTED
    })
}

pub fn json_run_default_start() -> serde_json::Value {
    // seqno for the default test run start is always 1
    json!({
        "testRunArtifact": {
            "testRunStart": {
                "dutInfo": {
                    "dutInfoId": "dut_id",
                    "softwareInfos": [{
                        "softwareInfoId": "sw0",
                        "name": "ubuntu",
                        "version": "22",
                        "softwareType": "SYSTEM",
                    }],
                    "hardwareInfos": [{
                        "hardwareInfoId": "hw0",
                        "name": "fan",
                        "location": "board0/fan"
                    }]
                },
                "name": "run_name",
                "parameters": {},
                "version": "1.0",
                "commandLine": ""
            }
        },
        "sequenceNumber": 1,
        "timestamp": DATETIME_FORMATTED
    })
}

pub fn json_run_pass(seqno: i32) -> serde_json::Value {
    json!({
        "testRunArtifact": {
            "testRunEnd": {
                "result": "PASS",
                "status": "COMPLETE"
            }
        },
        "sequenceNumber": seqno,
        "timestamp": DATETIME_FORMATTED
    })
}

pub fn json_step_default_start() -> serde_json::Value {
    // seqno for the default test run start is always 2
    json!({
        "testStepArtifact": {
            "testStepId": "step0",
            "testStepStart": {
                "name": "first step"
            }
        },
        "sequenceNumber": 2,
        "timestamp": DATETIME_FORMATTED
    })
}

pub fn json_step_complete(seqno: i32) -> serde_json::Value {
    json!({
        "testStepArtifact": {
            "testStepId": "step0",
            "testStepEnd": {
                "status": "COMPLETE"
            }
        },
        "sequenceNumber": seqno,
        "timestamp": DATETIME_FORMATTED
    })
}

pub async fn check_output<F, R>(expected: &[serde_json::Value], test_fn: F) -> Result<()>
where
    R: Future<Output = Result<()>>,
    F: FnOnce(TestRunBuilder, DutInfo) -> R,
{
    let buffer: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));
    let mut dut = DutInfo::builder("dut_id").build();
    dut.add_software_info(
        SoftwareInfo::builder("ubuntu")
            .id(Ident::Exact("sw0".to_owned())) // name is important as fixture
            .version("22")
            .software_type(SoftwareType::System)
            .build(),
    );
    dut.add_hardware_info(
        HardwareInfo::builder("fan")
            .id(Ident::Exact("hw0".to_owned()))
            .location("board0/fan")
            .build(),
    );

    let run_builder = TestRun::builder("run_name", "1.0").config(
        Config::builder()
            .with_buffer_output(Arc::clone(&buffer))
            .with_timestamp_provider(Box::new(FixedTsProvider {}))
            .build(),
    );

    // run the main test closure
    test_fn(run_builder, dut).await?;

    for (i, entry) in buffer.lock().await.iter().enumerate() {
        let value = serde_json::from_str::<serde_json::Value>(entry)?;
        assert_json_eq!(value, expected[i]);
    }

    Ok(())
}

pub async fn check_output_run<F, R>(expected: &[serde_json::Value], test_fn: F) -> Result<()>
where
    R: Future<Output = Result<(), OcptvError>> + Send + 'static,
    F: FnOnce(ScopedTestRun, DutInfo) -> R + Send + 'static,
{
    check_output(expected, |run_builder, dut| async move {
        run_builder
            .build()
            .scope(dut.clone(), |run| async move {
                test_fn(run, dut).await?;
                Ok(TestRunOutcome {
                    status: TestStatus::Complete,
                    result: TestResult::Pass,
                })
            })
            .await?;

        Ok(())
    })
    .await
}

pub async fn check_output_step<F, R>(expected: &[serde_json::Value], test_fn: F) -> Result<()>
where
    R: Future<Output = Result<(), OcptvError>> + Send + 'static,
    F: FnOnce(ScopedTestStep, DutInfo) -> R + Send + 'static,
{
    check_output_run(expected, |run, dut| async move {
        run.add_step("first step")
            .scope(|step| async move {
                test_fn(step, dut).await?;

                Ok(TestStatus::Complete)
            })
            .await?;

        Ok(())
    })
    .await
}
