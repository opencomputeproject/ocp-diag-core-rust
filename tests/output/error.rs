// (c) Meta Platforms, Inc. and affiliates.
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

use anyhow::Result;
use futures::FutureExt;
use serde_json::json;

use ocptv::output::Error;

use super::fixture::*;

#[tokio::test]
async fn test_testrun_with_error() -> Result<()> {
    let expected = [
        json_schema_version(),
        json_run_default_start(),
        json!({
            "testRunArtifact": {
                "error": {
                    "symptom": "symptom"
                }
            },
            "sequenceNumber": 2,
            "timestamp": DATETIME_FORMATTED
        }),
        json_run_pass(3),
    ];

    check_output_run(&expected, |r, _| {
        async { r.add_error("symptom").await }.boxed()
    })
    .await
}

#[tokio::test]
async fn test_testrun_with_error_with_message() -> Result<()> {
    let expected = [
        json_schema_version(),
        json_run_default_start(),
        json!({
            "testRunArtifact": {
                "error": {
                    "message": "Error message",
                    "symptom": "symptom"
                }
            },
            "sequenceNumber": 2,
            "timestamp": DATETIME_FORMATTED
        }),
        json_run_pass(3),
    ];

    check_output_run(&expected, |r, _| {
        async { r.add_error_msg("symptom", "Error message").await }.boxed()
    })
    .await
}

#[tokio::test]
async fn test_testrun_with_error_with_details() -> Result<()> {
    let expected = [
        json_schema_version(),
        json_run_default_start(),
        json!({
            "testRunArtifact": {
                "error": {
                    "message": "Error message",
                    "softwareInfoIds": [
                        "sw0"
                    ],
                    "sourceLocation": {
                        "file": "file",
                        "line": 1
                    },
                    "symptom": "symptom"
                }
            },
            "sequenceNumber": 2,
            "timestamp": DATETIME_FORMATTED
        }),
        json_run_pass(3),
    ];

    check_output_run(&expected, |r, dut| {
        async move {
            r.add_error_detail(
                Error::builder("symptom")
                    .message("Error message")
                    .source("file", 1)
                    .add_software_info(dut.software_info("sw0").unwrap()) // must exist
                    .build(),
            )
            .await
        }
        .boxed()
    })
    .await
}

#[tokio::test]
async fn test_testrun_with_error_before_start() -> Result<()> {
    let expected = [
        json_schema_version(),
        json!({
            "testRunArtifact": {
                "error": {
                    "symptom": "no-dut",
                }
            },
            "sequenceNumber": 1,
            "timestamp": DATETIME_FORMATTED
        }),
    ];

    check_output(&expected, |run_builder, _| {
        async move {
            let run = run_builder.build();
            run.add_error("no-dut").await?;

            Ok(())
        }
        .boxed()
    })
    .await
}

#[tokio::test]
async fn test_testrun_with_error_with_message_before_start() -> Result<()> {
    let expected = [
        json_schema_version(),
        json!({
            "testRunArtifact": {
                "error": {
                    "symptom": "no-dut",
                    "message": "failed to find dut",
                }
            },
            "sequenceNumber": 1,
            "timestamp": DATETIME_FORMATTED
        }),
    ];

    check_output(&expected, |run_builder, _| {
        async move {
            let run = run_builder.build();
            run.add_error_msg("no-dut", "failed to find dut").await?;

            Ok(())
        }
        .boxed()
    })
    .await
}

#[tokio::test]
async fn test_testrun_with_error_with_details_before_start() -> Result<()> {
    let expected = [
        json_schema_version(),
        json!({
            "testRunArtifact": {
                "error": {
                    "message": "failed to find dut",
                    "sourceLocation": {
                        "file": "file",
                        "line": 1
                    },
                    "symptom": "no-dut"
                }
            },
            "sequenceNumber": 1,
            "timestamp": DATETIME_FORMATTED
        }),
    ];

    check_output(&expected, |run_builder, _| {
        async move {
            let run = run_builder.build();
            run.add_error_detail(
                Error::builder("no-dut")
                    .message("failed to find dut")
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

#[tokio::test]
async fn test_testrun_step_error() -> Result<()> {
    let expected = [
        json_schema_version(),
        json_run_default_start(),
        json_step_default_start(),
        json!({
            "testStepArtifact": {
                "testStepId": "step0",
                "error": {
                    "symptom": "symptom"
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
            s.add_error("symptom").await?;

            Ok(())
        }
        .boxed()
    })
    .await
}

#[tokio::test]
async fn test_testrun_step_error_with_message() -> Result<()> {
    let expected = [
        json_schema_version(),
        json_run_default_start(),
        json_step_default_start(),
        json!({
            "testStepArtifact": {
                "testStepId": "step0",
                "error": {
                    "message": "Error message",
                    "symptom": "symptom"
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
            s.add_error_msg("symptom", "Error message").await?;

            Ok(())
        }
        .boxed()
    })
    .await
}

#[tokio::test]
async fn test_testrun_step_error_with_details() -> Result<()> {
    let expected = [
        json_schema_version(),
        json_run_default_start(),
        json_step_default_start(),
        json!({
            "testStepArtifact": {
                "testStepId": "step0",
                "error": {
                    "message": "Error message",
                    "softwareInfoIds": [
                        "sw0"
                    ],
                    "sourceLocation": {
                        "file": "file",
                        "line": 1
                    },
                    "symptom": "symptom"
                }
            },
            "sequenceNumber": 3,
            "timestamp": DATETIME_FORMATTED
        }),
        json_step_complete(4),
        json_run_pass(5),
    ];

    check_output_step(&expected, |s, dut| {
        async move {
            s.add_error_detail(
                Error::builder("symptom")
                    .message("Error message")
                    .source("file", 1)
                    .add_software_info(dut.software_info("sw0").unwrap())
                    .build(),
            )
            .await?;

            Ok(())
        }
        .boxed()
    })
    .await
}
