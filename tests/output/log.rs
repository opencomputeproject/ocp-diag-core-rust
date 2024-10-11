// (c) Meta Platforms, Inc. and affiliates.
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

use anyhow::Result;
use futures::FutureExt;
use serde_json::json;

use ocptv::output::{Log, LogSeverity};

use super::fixture::*;

#[tokio::test]
async fn test_testrun_with_log() -> Result<()> {
    let expected = [
        json_schema_version(),
        json_run_default_start(),
        json!({
            "testRunArtifact": {
                "log": {
                    "message": "This is a log message with INFO severity",
                    "severity": "INFO"
                }
            },
            "sequenceNumber": 2,
            "timestamp": DATETIME_FORMATTED
        }),
        json_run_pass(3),
    ];

    check_output_run(&expected, |r, _| {
        async {
            r.add_log(
                LogSeverity::Info,
                "This is a log message with INFO severity",
            )
            .await
        }
        .boxed()
    })
    .await
}

#[tokio::test]
async fn test_testrun_with_log_with_details() -> Result<()> {
    let expected = [
        json_schema_version(),
        json_run_default_start(),
        json!({
            "testRunArtifact": {
                "log": {
                    "message": "This is a log message with INFO severity",
                    "severity": "INFO",
                    "sourceLocation": {
                        "file": "file",
                        "line": 1
                    }
                }
            },
            "sequenceNumber": 2,
            "timestamp": DATETIME_FORMATTED
        }),
        json_run_pass(3),
    ];

    check_output_run(&expected, |r, _| {
        async {
            r.add_log_with_details(
                &Log::builder("This is a log message with INFO severity")
                    .severity(LogSeverity::Info)
                    .source("file", 1)
                    .build(),
            )
            .await
        }
        .boxed()
    })
    .await
}

#[tokio::test]
async fn test_testrun_step_log() -> Result<()> {
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

    check_output_step(&expected, |s, _| {
        async {
            s.add_log(
                LogSeverity::Info,
                "This is a log message with INFO severity",
            )
            .await?;

            Ok(())
        }
        .boxed()
    })
    .await
}

#[tokio::test]
async fn test_testrun_step_log_with_details() -> Result<()> {
    let expected = [
        json_schema_version(),
        json_run_default_start(),
        json_step_default_start(),
        json!({
            "testStepArtifact": {
                "testStepId": "step0",
                "log": {
                    "message": "This is a log message with INFO severity",
                    "severity": "INFO",
                    "sourceLocation": {
                        "file": "file",
                        "line": 1
                    }
                }
            },
            "sequenceNumber": 3,
            "timestamp": DATETIME_FORMATTED
        }),
        json_step_complete(4),
        json_run_pass(5),
    ];

    check_output_step(&expected, |s, _| {
        async {
            s.add_log_with_details(
                &Log::builder("This is a log message with INFO severity")
                    .severity(LogSeverity::Info)
                    .source("file", 1)
                    .build(),
            )
            .await?;

            Ok(())
        }
        .boxed()
    })
    .await
}
