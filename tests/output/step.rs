// (c) Meta Platforms, Inc. and affiliates.
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

use std::sync::Arc;

use anyhow::Result;
use futures::FutureExt;
use serde_json::json;
use tokio::sync::Mutex;

use ocptv::output::{Config, DutInfo, OcptvError, TestRun};

use super::fixture::*;

#[tokio::test]
async fn test_testrun_with_step() -> Result<()> {
    let expected = [
        json_schema_version(),
        json_run_default_start(),
        json_step_default_start(),
        json_step_complete(3),
        json_run_pass(4),
    ];

    check_output_step(&expected, |_, _| async { Ok(()) }.boxed()).await
}

#[cfg(feature = "boxed-scopes")]
#[tokio::test]
async fn test_testrun_step_scope_log() -> Result<()> {
    use ocptv::output::{LogSeverity, TestStatus};

    let expected = [
        json_schema_version(),
        json_run_default_start(),
        json_step_default_start(),
        json!({
            "testStepArtifact": {
                "testStepId": "step0",
                "log": {
                    "message": "This is a log message with INFO severity",
                    "severity": "INFO"
                }
            },
            "sequenceNumber": 3,
            "timestamp": DATETIME_FORMATTED
        }),
        json_step_complete(4),
        json_run_pass(5),
    ];

    check_output_run(&expected, |r, _| {
        async move {
            r.add_step("first step")
                .scope(|s| {
                    async move {
                        s.add_log(
                            LogSeverity::Info,
                            "This is a log message with INFO severity",
                        )
                        .await?;

                        Ok(TestStatus::Complete)
                    }
                    .boxed()
                })
                .await
        }
        .boxed()
    })
    .await
}

#[tokio::test]
async fn test_step_with_extension() -> Result<()> {
    let expected = [
        json_schema_version(),
        json_run_default_start(),
        json_step_default_start(),
        json!({
            "testStepArtifact": {
                "testStepId": "step0",
                "extension": {
                    "name": "extension",
                    "content": {
                        "@type": "TestExtension",
                        "stringField": "string",
                        "numberField": 42
                    }
                }
            },
            "sequenceNumber": 3,
            "timestamp": DATETIME_FORMATTED
        }),
        json_step_complete(4),
        json_run_pass(5),
    ];

    #[derive(serde::Serialize)]
    struct Ext {
        #[serde(rename = "@type")]
        r#type: String,
        #[serde(rename = "stringField")]
        string_field: String,
        #[serde(rename = "numberField")]
        number_field: u32,
    }

    check_output_step(&expected, |s, _| {
        async {
            s.add_extension(
                "extension",
                Ext {
                    r#type: "TestExtension".to_owned(),
                    string_field: "string".to_owned(),
                    number_field: 42,
                },
            )
            .await?;

            Ok(())
        }
        .boxed()
    })
    .await
}

#[tokio::test]
async fn test_step_with_extension_which_fails() -> Result<()> {
    #[derive(thiserror::Error, Debug, PartialEq)]
    enum TestError {
        #[error("test_error_fail")]
        Fail,
    }

    fn fail_serialize<S>(_: &u32, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        Err(serde::ser::Error::custom(TestError::Fail))
    }

    #[derive(serde::Serialize)]
    struct Ext {
        #[serde(serialize_with = "fail_serialize")]
        i: u32,
    }

    let buffer: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));
    let dut = DutInfo::builder("dut_id").build();
    let run = TestRun::builder("run_name", "1.0")
        .config(
            Config::builder()
                .with_buffer_output(Arc::clone(&buffer))
                .with_timestamp_provider(Box::new(FixedTsProvider {}))
                .build(),
        )
        .build()
        .start(dut)
        .await?;
    let step = run.add_step("first step").start().await?;

    let result = step.add_extension("extension", Ext { i: 0 }).await;

    match result {
        Err(OcptvError::Format(e)) => {
            // `to_string` is the only way to check this error. `serde_json::Error` only
            // implements source/cause for io errors, and this is a string
            assert_eq!(e.to_string(), "test_error_fail");
        }
        _ => panic!("unexpected ocptv error type"),
    }

    Ok(())
}
