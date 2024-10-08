// (c) Meta Platforms, Inc. and affiliates.
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
#![allow(unused_imports)]

use std::fs;
use std::sync::Arc;

use anyhow::Result;
use assert_fs::prelude::*;
use assert_json_diff::assert_json_include;
use futures::future::BoxFuture;
use futures::future::Future;
use futures::FutureExt;
use predicates::prelude::*;
use serde_json::json;
use tokio::sync::Mutex;

use ocptv::output as tv;
use tv::{
    Config, DutInfo, Error, HardwareInfo, Log, LogSeverity, Measurement, MeasurementSeriesStart,
    SoftwareInfo, StartedTestRun, StartedTestStep, Subcomponent, TestResult, TestRun,
    TestRunBuilder, TestRunOutcome, TestStatus, TestStep, Validator, ValidatorType,
};

fn json_schema_version() -> serde_json::Value {
    // seqno for schemaVersion is always 1
    json!({
        "schemaVersion": {
            "major": tv::SPEC_VERSION.0,
            "minor": tv::SPEC_VERSION.1
        },
        "sequenceNumber": 1
    })
}

fn json_run_default_start() -> serde_json::Value {
    // seqno for the default test run start is always 2
    json!({
        "testRunArtifact": {
            "testRunStart": {
                "dutInfo": {
                    "dutInfoId": "dut_id"
                },
                "name": "run_name",
                "parameters": {},
                "version": "1.0"
            }
        },
        "sequenceNumber": 2
    })
}

fn json_run_pass(seqno: i32) -> serde_json::Value {
    json!({
        "testRunArtifact": {
            "testRunEnd": {
                "result": "PASS",
                "status": "COMPLETE"
            }
        },
        "sequenceNumber": seqno
    })
}

fn json_step_default_start() -> serde_json::Value {
    // seqno for the default test run start is always 3
    json!({
        "testStepArtifact": {
            "testStepStart": {
                "name": "first step"
            }
        },
        "sequenceNumber": 3
    })
}

fn json_step_complete(seqno: i32) -> serde_json::Value {
    json!({
        "testStepArtifact": {
            "testStepEnd": {
                "status": "COMPLETE"
            }
        },
        "sequenceNumber": seqno
    })
}

async fn check_output<F, R>(expected: &[serde_json::Value], test_fn: F) -> Result<()>
where
    R: Future<Output = Result<()>>,
    F: FnOnce(TestRunBuilder) -> R,
{
    let buffer: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));
    let dut = DutInfo::builder("dut_id").build();
    let run_builder = TestRun::builder("run_name", &dut, "1.0").config(
        Config::builder()
            .with_buffer_output(Arc::clone(&buffer))
            .build(),
    );

    // run the main test closure
    test_fn(run_builder).await?;

    for (idx, entry) in buffer.lock().await.iter().enumerate() {
        let value = serde_json::from_str::<serde_json::Value>(entry)?;
        assert_json_include!(actual: value, expected: &expected[idx]);
    }

    Ok(())
}

async fn check_output_run<F>(expected: &[serde_json::Value], test_fn: F) -> Result<()>
where
    F: for<'a> FnOnce(&'a StartedTestRun) -> BoxFuture<'a, Result<(), tv::WriterError>> + Send,
{
    check_output(expected, |run_builder| async {
        let run = run_builder.build();

        let run = run.start().await?;
        test_fn(&run).await?;
        run.end(TestStatus::Complete, TestResult::Pass).await?;

        Ok(())
    })
    .await
}

async fn check_output_step<F>(expected: &[serde_json::Value], test_fn: F) -> Result<()>
where
    F: for<'a> FnOnce(&'a StartedTestStep) -> BoxFuture<'a, Result<(), tv::WriterError>>,
{
    check_output(expected, |run_builder| async {
        let run = run_builder.build().start().await?;

        let step = run.step("first step").start().await?;
        test_fn(&step).await?;
        step.end(TestStatus::Complete).await?;

        run.end(TestStatus::Complete, TestResult::Pass).await?;

        Ok(())
    })
    .await
}

#[tokio::test]
async fn test_testrun_start_and_end() -> Result<()> {
    let expected = [
        json_schema_version(),
        json_run_default_start(),
        json_run_pass(3),
    ];

    check_output_run(&expected, |_| async { Ok(()) }.boxed()).await
}

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
            "sequenceNumber": 3
        }),
        json_run_pass(4),
    ];

    check_output_run(&expected, |run| {
        async {
            run.log(
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
            "sequenceNumber": 3
        }),
        json_run_pass(4),
    ];

    check_output_run(&expected, |run| {
        async {
            run.log_with_details(
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
            "sequenceNumber": 3
        }),
        json_run_pass(4),
    ];

    check_output_run(&expected, |run| {
        async { run.error("symptom").await }.boxed()
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
            "sequenceNumber": 3
        }),
        json_run_pass(4),
    ];

    check_output_run(&expected, |run| {
        async { run.error_with_msg("symptom", "Error message").await }.boxed()
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
                    "softwareInfoIds":[{
                        "name": "name",
                        "softwareInfoId": "id",
                    }],
                    "sourceLocation": {
                        "file": "file",
                        "line": 1
                    },
                    "symptom": "symptom"
                }
            },
            "sequenceNumber": 3
        }),
        json_run_pass(4),
    ];

    check_output_run(&expected, |run| {
        async {
            run.error_with_details(
                &Error::builder("symptom")
                    .message("Error message")
                    .source("file", 1)
                    .add_software_info(&SoftwareInfo::builder("id", "name").build())
                    .build(),
            )
            .await
        }
        .boxed()
    })
    .await
}

// #[tokio::test]
// async fn test_testrun_with_scope() -> Result<()> {
//     let expected = [
//         json_schema_version(),
//         json_run_default_start(),
//         json!({
//             "testRunArtifact": {
//                 "log": {
//                     "message": "First message",
//                     "severity": "INFO"
//                 }
//             },
//             "sequenceNumber": 3
//         }),
//         json_run_pass(4),
//     ];

//     check_output(&expected, |run_builder| async {
//         let run = run_builder.build();

//         run.scope(|r| async {
//             r.log(LogSeverity::Info, "First message").await?;
//             Ok(TestRunOutcome {
//                 status: TestStatus::Complete,
//                 result: TestResult::Pass,
//             })
//         })
//         .await?;

//         Ok(())
//     })
//     .await
// }

#[tokio::test]
async fn test_testrun_with_step() -> Result<()> {
    let expected = [
        json_schema_version(),
        json_run_default_start(),
        json_step_default_start(),
        json_step_complete(4),
        json_run_pass(5),
    ];

    check_output_step(&expected, |_| async { Ok(()) }.boxed()).await
}

#[tokio::test]
async fn test_testrun_step_log() -> Result<()> {
    let expected = [
        json_schema_version(),
        json_run_default_start(),
        json_step_default_start(),
        json!({
            "testStepArtifact": {
                "log": {
                    "message": "This is a log message with INFO severity",
                    "severity": "INFO"
                }
            },
            "sequenceNumber": 4
        }),
        json_step_complete(5),
        json_run_pass(6),
    ];

    check_output_step(&expected, |step| {
        async {
            step.log(
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
            "sequenceNumber": 4,
            "testStepArtifact": {
                "log": {
                    "message": "This is a log message with INFO severity",
                    "severity": "INFO",
                    "sourceLocation": {
                        "file": "file",
                        "line": 1
                    }
                }
            }
        }),
        json_step_complete(5),
        json_run_pass(6),
    ];

    check_output_step(&expected, |step| {
        async {
            step.log_with_details(
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

#[tokio::test]
async fn test_testrun_step_error() -> Result<()> {
    let expected = [
        json_schema_version(),
        json_run_default_start(),
        json_step_default_start(),
        json!({
            "sequenceNumber": 4,
            "testStepArtifact": {
                "error": {
                    "symptom": "symptom"
                }
            }
        }),
        json_step_complete(5),
        json_run_pass(6),
    ];

    check_output_step(&expected, |step| {
        async {
            step.error("symptom").await?;

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
            "sequenceNumber": 4,
            "testStepArtifact": {
                "error": {
                    "message": "Error message",
                    "symptom": "symptom"
                }
            }
        }),
        json_step_complete(5),
        json_run_pass(6),
    ];

    check_output_step(&expected, |step| {
        async {
            step.error_with_msg("symptom", "Error message").await?;

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
            "sequenceNumber": 4,
            "testStepArtifact": {
                "testStepId": "step_0",
                "error": {
                    "message": "Error message",
                    "softwareInfoIds":[{
                        "name": "name",
                        "softwareInfoId": "id"
                    }],
                    "sourceLocation": {
                        "file": "file",
                        "line": 1
                    },
                    "symptom": "symptom"
                }
            }
        }),
        json_step_complete(5),
        json_run_pass(6),
    ];

    check_output_step(&expected, |step| {
        async {
            step.error_with_details(
                &Error::builder("symptom")
                    .message("Error message")
                    .source("file", 1)
                    .add_software_info(&SoftwareInfo::builder("id", "name").build())
                    .build(),
            )
            .await?;

            Ok(())
        }
        .boxed()
    })
    .await
}

// #[tokio::test]
// async fn test_testrun_step_scope_log() -> Result<()> {
//     let expected = [
//         json_schema_version(),
//         json_run_default_start(),
//         json_step_default_start(),
//         json!({
//             "sequenceNumber": 4,
//             "testStepArtifact": {
//                 "log": {
//                     "message": "This is a log message with INFO severity",
//                     "severity": "INFO"
//                 }
//             }
//         }),
//         json_step_complete(5),
//         json_run_pass(6),
//     ];

//     check_output_run(&expected, |run| {
//         async {
//             run.step("first step")
//                 .start()
//                 .scope(|s| async {
//                     s.log(
//                         LogSeverity::Info,
//                         "This is a log message with INFO severity",
//                     )
//                     .await?;
//                     Ok(TestStatus::Complete)
//                 })
//                 .await
//         }
//         .boxed()
//     })
//     .await
// }

#[tokio::test]
async fn test_step_with_measurement() -> Result<()> {
    let expected = [
        json_schema_version(),
        json_run_default_start(),
        json_step_default_start(),
        json!({
            "testStepArtifact": {
                "testStepId": "step_0",
                "measurement": {
                    "name": "name",
                    "value": 50
                }
            },
            "sequenceNumber": 4
        }),
        json_step_complete(5),
        json_run_pass(6),
    ];

    check_output_step(&expected, |step| {
        async {
            step.add_measurement("name", 50.into()).await?;

            Ok(())
        }
        .boxed()
    })
    .await
}

// TODO: intentionally leaving these tests broken so that it's obvious later that the
// assert_json_includes was not sufficient; this case is missing `testStepId` field
#[tokio::test]
async fn test_step_with_measurement_builder() -> Result<()> {
    let expected = [
        json_schema_version(),
        json_run_default_start(),
        json_step_default_start(),
        json!({
            "sequenceNumber": 4,
            "testStepArtifact": {
                "measurement": {
                    "hardwareInfoId": "id",
                    "metadata": {
                        "key": "value"
                    },
                    "name": "name",
                    "subcomponent": {
                        "name": "name"
                    },
                    "validators":[{
                        "type": "EQUAL",
                        "value": 30
                    }],
                    "value": 50
                }
            }
        }),
        json_step_complete(5),
        json_run_pass(6),
    ];

    check_output_step(&expected, |step| {
        async {
            let measurement = Measurement::builder("name", 50.into())
                .hardware_info(&HardwareInfo::builder("id", "name").build())
                .add_validator(&Validator::builder(ValidatorType::Equal, 30.into()).build())
                .add_metadata("key", "value".into())
                .subcomponent(&Subcomponent::builder("name").build())
                .build();
            step.add_measurement_with_details(&measurement).await?;

            Ok(())
        }
        .boxed()
    })
    .await
}

#[tokio::test]
async fn test_step_with_measurement_series() -> Result<()> {
    let expected = [
        json_schema_version(),
        json_run_default_start(),
        json_step_default_start(),
        json!({
            "sequenceNumber": 4,
            "testStepArtifact": {
                "measurementSeriesStart": {
                    "measurementSeriesId": "series_1",
                    "name": "name"
                }
            }
        }),
        json!({
            "sequenceNumber": 5,
            "testStepArtifact": {
                "measurementSeriesEnd": {
                    "measurementSeriesId":
                    "series_1",
                    "totalCount": 0
                }
            }
        }),
        json_step_complete(6),
        json_run_pass(7),
    ];

    check_output_step(&expected, |step| {
        async {
            let series = step.measurement_series("name");
            series.start().await?;
            series.end().await?;

            Ok(())
        }
        .boxed()
    })
    .await
}

#[tokio::test]
async fn test_step_with_multiple_measurement_series() -> Result<()> {
    let expected = [
        json_schema_version(),
        json_run_default_start(),
        json_step_default_start(),
        json!({
            "sequenceNumber": 4,
            "testStepArtifact": {
                "measurementSeriesStart": {
                    "measurementSeriesId": "series_1",
                    "name": "name"
                }
            }
        }),
        json!({
            "sequenceNumber": 5,
            "testStepArtifact": {
                "measurementSeriesEnd": {
                    "measurementSeriesId": "series_1",
                    "totalCount": 0
                }
            }
        }),
        json!({
            "sequenceNumber": 6,
            "testStepArtifact": {
                "measurementSeriesStart": {
                    "measurementSeriesId": "series_2",
                    "name": "name"
                }
            }
        }),
        json!({
            "sequenceNumber": 7,
            "testStepArtifact": {
                "measurementSeriesEnd": {
                    "measurementSeriesId": "series_2",
                    "totalCount": 0
                }
            }
        }),
        json_step_complete(8),
        json_run_pass(9),
    ];

    check_output_step(&expected, |step| {
        async {
            let series = step.measurement_series("name");
            series.start().await?;
            series.end().await?;

            let series_2 = step.measurement_series("name");
            series_2.start().await?;
            series_2.end().await?;

            Ok(())
        }
        .boxed()
    })
    .await
}

#[tokio::test]
async fn test_step_with_measurement_series_with_details() -> Result<()> {
    let expected = [
        json_schema_version(),
        json_run_default_start(),
        json_step_default_start(),
        json!({
            "sequenceNumber": 4,
            "testStepArtifact": {
                "measurementSeriesStart": {
                    "measurementSeriesId": "series_id", "name": "name"
                }
            }
        }),
        json!({
            "sequenceNumber": 5,
            "testStepArtifact": {
                "measurementSeriesEnd": {
                    "measurementSeriesId": "series_id", "totalCount": 0
                }
            }
        }),
        json_step_complete(6),
        json_run_pass(7),
    ];

    check_output_step(&expected, |step| {
        async {
            let series = step
                .measurement_series_with_details(MeasurementSeriesStart::new("name", "series_id"));
            series.start().await?;
            series.end().await?;

            Ok(())
        }
        .boxed()
    })
    .await
}

#[tokio::test]
async fn test_step_with_measurement_series_with_details_and_start_builder() -> Result<()> {
    let expected = [
        json_schema_version(),
        json_run_default_start(),
        json_step_default_start(),
        json!({
            "sequenceNumber": 4,
            "testStepArtifact": {
                "measurementSeriesStart": {
                    "hardwareInfoId": {
                        "hardwareInfoId": "id",
                        "name": "name"
                    },
                    "measurementSeriesId": "series_id",
                    "metadata": {
                        "key": "value"
                    },
                    "name": "name",
                    "subComponent": {
                        "name": "name"
                    },
                    "validators":[{
                        "type": "EQUAL",
                        "value": 30
                    }]
                }
            }
        }),
        json!({
            "sequenceNumber": 5,
            "testStepArtifact": {
                "measurementSeriesEnd": {
                    "measurementSeriesId": "series_id",
                    "totalCount": 0
                }
            }
        }),
        json_step_complete(6),
        json_run_pass(7),
    ];

    check_output_step(&expected, |step| {
        async {
            let series = step.measurement_series_with_details(
                MeasurementSeriesStart::builder("name", "series_id")
                    .add_metadata("key", "value".into())
                    .add_validator(&Validator::builder(ValidatorType::Equal, 30.into()).build())
                    .hardware_info(&HardwareInfo::builder("id", "name").build())
                    .subcomponent(&Subcomponent::builder("name").build())
                    .build(),
            );
            series.start().await?;
            series.end().await?;

            Ok(())
        }
        .boxed()
    })
    .await
}

#[tokio::test]
async fn test_step_with_measurement_series_element() -> Result<()> {
    let expected = [
        json_schema_version(),
        json_run_default_start(),
        json_step_default_start(),
        json!({
            "sequenceNumber": 4,
            "testStepArtifact": {
                "measurementSeriesStart": {
                    "measurementSeriesId": "series_1",
                    "name": "name"
                }
            }
        }),
        json!({
            "sequenceNumber": 5,
            "testStepArtifact": {
                "measurementSeriesElement": {
                    "index": 0,
                    "measurementSeriesId": "series_1",
                    "value": 60
                }
            }
        }),
        json!({
            "sequenceNumber": 6,
            "testStepArtifact": {
                "measurementSeriesEnd": {
                    "measurementSeriesId": "series_1",
                    "totalCount": 1
                }
            }
        }),
        json_step_complete(7),
        json_run_pass(8),
    ];

    check_output_step(&expected, |step| {
        async {
            let series = step.measurement_series("name");
            series.start().await?;
            series.add_measurement(60.into()).await?;
            series.end().await?;

            Ok(())
        }
        .boxed()
    })
    .await
}

#[tokio::test]
async fn test_step_with_measurement_series_element_index_no() -> Result<()> {
    let expected = [
        json_schema_version(),
        json_run_default_start(),
        json_step_default_start(),
        json!({
            "sequenceNumber": 4,
            "testStepArtifact": {
                "measurementSeriesStart": {
                    "measurementSeriesId": "series_1",
                    "name": "name"
                }
            }
        }),
        json!({
            "sequenceNumber": 5,
            "testStepArtifact": {
                "measurementSeriesElement": {
                    "index": 0,
                    "measurementSeriesId": "series_1",
                    "value": 60
                }
            }
        }),
        json!({
            "sequenceNumber": 6,
            "testStepArtifact": {
                "measurementSeriesElement": {
                    "index": 1,
                    "measurementSeriesId": "series_1",
                    "value": 70
                }
            }
        }),
        json!({
            "sequenceNumber": 7,
            "testStepArtifact": {
                "measurementSeriesElement": {
                    "index": 2,
                    "measurementSeriesId": "series_1",
                    "value": 80
                }
            }
        }),
        json!({
            "sequenceNumber": 8,
            "testStepArtifact": {
                "measurementSeriesEnd": {
                    "measurementSeriesId": "series_1",
                    "totalCount": 3
                }
            }
        }),
        json_step_complete(9),
        json_run_pass(10),
    ];

    check_output_step(&expected, |step| {
        async {
            let series = step.measurement_series("name");
            series.start().await?;
            // add more than one element to check the index increments correctly
            series.add_measurement(60.into()).await?;
            series.add_measurement(70.into()).await?;
            series.add_measurement(80.into()).await?;
            series.end().await?;

            Ok(())
        }
        .boxed()
    })
    .await
}

#[tokio::test]
async fn test_step_with_measurement_series_element_with_metadata() -> Result<()> {
    let expected = [
        json_schema_version(),
        json_run_default_start(),
        json_step_default_start(),
        json!({
            "sequenceNumber": 4,
            "testStepArtifact": {
                "measurementSeriesStart": {
                    "measurementSeriesId": "series_1",
                    "name": "name"
                }
            }
        }),
        json!({
            "sequenceNumber": 5,
            "testStepArtifact": {
                "measurementSeriesElement": {
                    "index": 0,
                    "measurementSeriesId": "series_1",
                    "metadata": {
                        "key": "value"
                    },
                    "value": 60
                }
            }
        }),
        json!({
            "sequenceNumber": 6,
            "testStepArtifact": {
                "measurementSeriesEnd": {
                    "measurementSeriesId": "series_1",
                    "totalCount": 1
                }
            }
        }),
        json_step_complete(7),
        json_run_pass(8),
    ];

    check_output_step(&expected, |step| {
        async {
            let series = step.measurement_series("name");
            series.start().await?;
            series
                .add_measurement_with_metadata(60.into(), vec![("key", "value".into())])
                .await?;
            series.end().await?;

            Ok(())
        }
        .boxed()
    })
    .await
}

#[tokio::test]
async fn test_step_with_measurement_series_element_with_metadata_index_no() -> Result<()> {
    let expected = [
        json_schema_version(),
        json_run_default_start(),
        json_step_default_start(),
        json!({
            "sequenceNumber": 4,
            "testStepArtifact": {
                "measurementSeriesStart": {
                    "measurementSeriesId": "series_1",
                    "name": "name"
                }
            }
        }),
        json!({
            "sequenceNumber": 5,
            "testStepArtifact": {
                "measurementSeriesElement": {
                    "index": 0,
                    "measurementSeriesId": "series_1",
                    "metadata": {"key": "value"},
                    "value": 60
                }
            }
        }),
        json!({
            "sequenceNumber": 6,
            "testStepArtifact": {
                "measurementSeriesElement": {
                    "index": 1,
                    "measurementSeriesId": "series_1",
                    "metadata": {"key2": "value2"},
                    "value": 70
                }
            }
        }),
        json!({
            "sequenceNumber": 7,
            "testStepArtifact": {
                "measurementSeriesElement": {
                    "index": 2,
                    "measurementSeriesId": "series_1",
                    "metadata": {"key3": "value3"},
                    "value": 80
                }
            }
        }),
        json!({
            "sequenceNumber": 8,
            "testStepArtifact": {
                "measurementSeriesEnd": {
                    "measurementSeriesId": "series_1",
                    "totalCount": 3
                }
            }
        }),
        json_step_complete(9),
        json_run_pass(10),
    ];

    check_output_step(&expected, |step| {
        async {
            let series = step.measurement_series("name");
            series.start().await?;
            // add more than one element to check the index increments correctly
            series
                .add_measurement_with_metadata(60.into(), vec![("key", "value".into())])
                .await?;
            series
                .add_measurement_with_metadata(70.into(), vec![("key2", "value2".into())])
                .await?;
            series
                .add_measurement_with_metadata(80.into(), vec![("key3", "value3".into())])
                .await?;
            series.end().await?;

            Ok(())
        }
        .boxed()
    })
    .await
}

#[tokio::test]
async fn test_step_with_measurement_series_scope() -> Result<()> {
    let expected = [
        json_schema_version(),
        json_run_default_start(),
        json_step_default_start(),
        json!({
            "sequenceNumber": 4,
            "testStepArtifact": {
                "measurementSeriesStart": {
                    "measurementSeriesId": "series_1",
                    "name": "name"
                }
            }
        }),
        json!({
            "sequenceNumber": 5,
            "testStepArtifact": {
                "measurementSeriesElement": {
                    "index": 0,
                    "measurementSeriesId": "series_1",
                    "value": 60
                }
            }
        }),
        json!({
            "sequenceNumber": 6,
            "testStepArtifact": {
                "measurementSeriesElement": {
                    "index": 1,
                    "measurementSeriesId": "series_1",
                    "value": 70
                }
            }
        }),
        json!({
            "sequenceNumber": 7,
            "testStepArtifact": {
                "measurementSeriesElement": {
                    "index": 2,
                    "measurementSeriesId": "series_1",
                    "value": 80
                }
            }
        }),
        json!({
            "sequenceNumber": 8,
            "testStepArtifact": {
                "measurementSeriesEnd": {
                    "measurementSeriesId": "series_1",
                    "totalCount": 3
                }
            }
        }),
        json_step_complete(9),
        json_run_pass(10),
    ];

    check_output_step(&expected, |step| {
        async {
            let series = step.measurement_series("name");
            series
                .scope(|s| async {
                    s.add_measurement(60.into()).await?;
                    s.add_measurement(70.into()).await?;
                    s.add_measurement(80.into()).await?;

                    Ok(())
                })
                .await?;

            Ok(())
        }
        .boxed()
    })
    .await
}

// reasoning: the coverage(off) attribute is experimental in llvm-cov, so because we cannot
// disable the coverage itself, only run this test when in coverage mode because assert_fs
// does ultimately assume there's a real filesystem somewhere
#[cfg(coverage)]
#[tokio::test]
async fn test_config_builder_with_file() -> Result<()> {
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
            "sequenceNumber": 3
        }),
        json_run_pass(4),
    ];

    let fs = assert_fs::TempDir::new()?;
    let output_file = fs.child("output.jsonl");

    let dut = DutInfo::builder("dut_id").build();

    let run = TestRun::builder("run_name", &dut, "1.0")
        .config(
            Config::builder()
                .timezone(chrono_tz::Europe::Rome)
                .with_file_output(output_file.path())
                .await?
                .build(),
        )
        .build()
        .start()
        .await?;

    run.error_with_msg("symptom", "Error message").await?;

    run.end(TestStatus::Complete, TestResult::Pass).await?;

    output_file.assert(predicate::path::exists());
    let content = fs::read_to_string(output_file.path())?;

    for (idx, entry) in content.lines().enumerate() {
        let value = serde_json::from_str::<serde_json::Value>(entry).unwrap();
        assert_json_include!(actual: value, expected: &expected[idx]);
    }

    Ok(())
}

#[tokio::test]
async fn test_testrun_instantiation_with_new() -> Result<()> {
    let expected = [
        json_schema_version(),
        json_run_default_start(),
        json_run_pass(3),
    ];
    let buffer: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));

    let run = TestRun::new("run_name", "dut_id", "1.0").start().await?;
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
            "sequenceNumber": 2,
            "testRunArtifact": {
                "testRunStart": {
                    "dutInfo": {
                        "dutInfoId": "dut_id"
                    },
                    "metadata": {"key": "value"},
                    "name": "run_name",
                    "parameters": {},
                    "version": "1.0"
                }
            }
        }),
        json_run_pass(3),
    ];

    check_output(&expected, |run_builder| async {
        let run = run_builder
            .add_metadata("key", "value".into())
            .build()
            .start()
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
                        "dutInfoId": "dut_id"
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
            "sequenceNumber": 2
        }),
        json_run_pass(3),
    ];

    check_output(&expected, |run_builder| async {
        let run = run_builder
            .add_metadata("key", "value".into())
            .add_metadata("key2", "value2".into())
            .add_parameter("key", "value".into())
            .command_line("cmd_line")
            .build()
            .start()
            .await?;

        run.end(TestStatus::Complete, TestResult::Pass).await?;
        Ok(())
    })
    .await
}
