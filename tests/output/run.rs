// (c) Meta Platforms, Inc. and affiliates.
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

use std::sync::Arc;

use anyhow::Result;
use assert_json_diff::assert_json_include;
use futures::FutureExt;
use serde_json::json;
use tokio::sync::Mutex;

use ocptv::output::{DutInfo, TestResult, TestRun, TestStatus};

use super::fixture::*;

#[tokio::test]
async fn test_testrun_start_and_end() -> Result<()> {
    let expected = [
        json_schema_version(),
        json_run_default_start(),
        json_run_pass(2),
    ];

    check_output_run(&expected, |_, _| async { Ok(()) }.boxed()).await
}

#[cfg(feature = "boxed-scopes")]
#[tokio::test]
async fn test_testrun_with_scope() -> Result<()> {
    use ocptv::output::{LogSeverity, TestResult, TestRunOutcome, TestStatus};

    let expected = [
        json_schema_version(),
        json_run_default_start(),
        json!({
            "testRunArtifact": {
                "log": {
                    "message": "First message",
                    "severity": "INFO"
                }
            },
            "sequenceNumber": 2,
            "timestamp": DATETIME_FORMATTED
        }),
        json_run_pass(3),
    ];

    check_output(&expected, |run_builder, dut| async {
        let run = run_builder.build();

        run.scope(dut, |r| {
            async move {
                r.add_log(LogSeverity::Info, "First message").await?;

                Ok(TestRunOutcome {
                    status: TestStatus::Complete,
                    result: TestResult::Pass,
                })
            }
            .boxed()
        })
        .await?;

        Ok(())
    })
    .await
}

#[tokio::test]
async fn test_testrun_instantiation_with_new() -> Result<()> {
    let expected = [
        json_schema_version(),
        json_run_default_start(),
        json_run_pass(2),
    ];
    let buffer: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));

    let dut = DutInfo::builder("dut_id").build();
    let run = TestRun::new("run_name", "1.0").start(dut).await?;
    run.end(TestStatus::Complete, TestResult::Pass).await?;

    for (idx, entry) in buffer.lock().await.iter().enumerate() {
        let value = serde_json::from_str::<serde_json::Value>(entry)?;
        assert_json_include!(actual: value, expected: &expected[idx]);
    }

    Ok(())
}

#[tokio::test]
async fn test_testrun_metadata() -> Result<()> {
    let expected = [
        json_schema_version(),
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
                    "metadata": {"key": "value"},
                    "name": "run_name",
                    "parameters": {},
                    "version": "1.0",

                    "commandLine": "",
                }
            },
            "sequenceNumber": 1,
            "timestamp": DATETIME_FORMATTED
        }),
        json_run_pass(2),
    ];

    check_output(&expected, |run_builder, dut| async {
        let run = run_builder
            .add_metadata("key", "value".into())
            .build()
            .start(dut)
            .await?;

        run.end(TestStatus::Complete, TestResult::Pass).await?;
        Ok(())
    })
    .await
}

#[tokio::test]
async fn test_testrun_builder() -> Result<()> {
    let expected = [
        json_schema_version(),
        json!({
            "testRunArtifact": {
                "testRunStart": {
                    "commandLine": "cmd_line",
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
                    "metadata": {
                        "key": "value",
                        "key2": "value2"
                    },
                    "name": "run_name",
                    "parameters": {
                        "key": "value"
                    },
                    "version": "1.0"
                }
            },
            "sequenceNumber": 1,
            "timestamp": DATETIME_FORMATTED
        }),
        json_run_pass(2),
    ];

    check_output(&expected, |run_builder, dut| async {
        let run = run_builder
            .add_metadata("key", "value".into())
            .add_metadata("key2", "value2".into())
            .add_parameter("key", "value".into())
            .command_line("cmd_line")
            .build()
            .start(dut)
            .await?;

        run.end(TestStatus::Complete, TestResult::Pass).await?;
        Ok(())
    })
    .await
}
