// (c) Meta Platforms, Inc. and affiliates.
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

use std::sync::Arc;

use anyhow::Result;

use assert_json_diff::{assert_json_eq, assert_json_include};
use futures::future::BoxFuture;
use futures::future::Future;
use futures::FutureExt;
use serde_json::json;
use tokio::sync::Mutex;

use ocptv::output as tv;
use ocptv::output::OcptvError;
#[cfg(feature = "boxed-scopes")]
use tv::TestRunOutcome;
use tv::{
    Config, DutInfo, Error, HardwareInfo, Ident, Log, LogSeverity, Measurement,
    MeasurementSeriesStart, SoftwareInfo, SoftwareType, StartedTestRun, StartedTestStep,
    Subcomponent, TestResult, TestRun, TestRunBuilder, TestStatus, TimestampProvider, Validator,
    ValidatorType,
};

const DATETIME: chrono::DateTime<chrono::offset::Utc> = chrono::DateTime::from_timestamp_nanos(0);
const DATETIME_FORMATTED: &str = "1970-01-01T00:00:00.000Z";
struct FixedTsProvider {}

impl TimestampProvider for FixedTsProvider {
    fn now(&self) -> chrono::DateTime<chrono_tz::Tz> {
        // all cases will use time 0 but this is configurable
        DATETIME.with_timezone(&chrono_tz::UTC)
    }
}

fn json_schema_version() -> serde_json::Value {
    // seqno for schemaVersion is always 0
    json!({
        "schemaVersion": {
            "major": tv::SPEC_VERSION.0,
            "minor": tv::SPEC_VERSION.1
        },
        "sequenceNumber": 0,
        "timestamp": DATETIME_FORMATTED
    })
}

fn json_run_default_start() -> serde_json::Value {
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

fn json_run_pass(seqno: i32) -> serde_json::Value {
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

fn json_step_default_start() -> serde_json::Value {
    // seqno for the default test run start is always 2
    json!({
        "testStepArtifact": {
            "testStepId": "step_0",
            "testStepStart": {
                "name": "first step"
            }
        },
        "sequenceNumber": 2,
        "timestamp": DATETIME_FORMATTED
    })
}

fn json_step_complete(seqno: i32) -> serde_json::Value {
    json!({
        "testStepArtifact": {
            "testStepId": "step_0",
            "testStepEnd": {
                "status": "COMPLETE"
            }
        },
        "sequenceNumber": seqno,
        "timestamp": DATETIME_FORMATTED
    })
}

async fn check_output<F, R>(expected: &[serde_json::Value], test_fn: F) -> Result<()>
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

    let run_builder = TestRun::builder("run_name", &dut, "1.0").config(
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

async fn check_output_run<F>(expected: &[serde_json::Value], test_fn: F) -> Result<()>
where
    F: for<'a> FnOnce(&'a StartedTestRun, DutInfo) -> BoxFuture<'a, Result<(), tv::OcptvError>>,
{
    check_output(expected, |run_builder, dutinfo| async move {
        let run = run_builder.build();

        let run = run.start().await?;
        test_fn(&run, dutinfo).await?;
        run.end(TestStatus::Complete, TestResult::Pass).await?;

        Ok(())
    })
    .await
}

async fn check_output_step<F>(expected: &[serde_json::Value], test_fn: F) -> Result<()>
where
    F: for<'a> FnOnce(&'a StartedTestStep, DutInfo) -> BoxFuture<'a, Result<(), tv::OcptvError>>,
{
    check_output(expected, |run_builder, dutinfo| async move {
        let run = run_builder.build().start().await?;

        let step = run.add_step("first step").start().await?;
        test_fn(&step, dutinfo).await?;
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
        json_run_pass(2),
    ];

    check_output_run(&expected, |_, _| async { Ok(()) }.boxed()).await
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
        async { r.add_error_with_msg("symptom", "Error message").await }.boxed()
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
            r.add_error_with_details(
                &Error::builder("symptom")
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

#[cfg(feature = "boxed-scopes")]
#[tokio::test]
async fn test_testrun_with_scope() -> Result<()> {
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

    check_output(&expected, |run_builder, _| async {
        let run = run_builder.build();

        run.scope(|r| {
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

#[tokio::test]
async fn test_testrun_step_log() -> Result<()> {
    let expected = [
        json_schema_version(),
        json_run_default_start(),
        json_step_default_start(),
        json!({
            "testStepArtifact": {
                "testStepId": "step_0",
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
                "testStepId": "step_0",
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

#[tokio::test]
async fn test_testrun_step_error() -> Result<()> {
    let expected = [
        json_schema_version(),
        json_run_default_start(),
        json_step_default_start(),
        json!({
            "testStepArtifact": {
                "testStepId": "step_0",
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
                "testStepId": "step_0",
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
            s.add_error_with_msg("symptom", "Error message").await?;

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
                "testStepId": "step_0",
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
            s.add_error_with_details(
                &Error::builder("symptom")
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

#[cfg(feature = "boxed-scopes")]
#[tokio::test]
async fn test_testrun_step_scope_log() -> Result<()> {
    let expected = [
        json_schema_version(),
        json_run_default_start(),
        json_step_default_start(),
        json!({
            "testStepArtifact": {
                "testStepId": "step_0",
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
        async {
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
            "sequenceNumber": 3,
            "timestamp": DATETIME_FORMATTED
        }),
        json_step_complete(4),
        json_run_pass(5),
    ];

    check_output_step(&expected, |s, _| {
        async {
            s.add_measurement("name", 50.into()).await?;

            Ok(())
        }
        .boxed()
    })
    .await
}

#[tokio::test]
async fn test_step_with_measurement_builder() -> Result<()> {
    let expected = [
        json_schema_version(),
        json_run_default_start(),
        json_step_default_start(),
        json!({
            "testStepArtifact": {
                "testStepId": "step_0",
                "measurement": {
                    "hardwareInfoId": "id",
                    "metadata": {
                        "key": "value"
                    },
                    "name": "name",
                    "subcomponent": {
                        "name": "name"
                    },
                    "validators": [{
                        "type": "EQUAL",
                        "value": 30
                    }],
                    "value": 50
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
            let measurement = Measurement::builder("name", 50.into())
                .hardware_info(&HardwareInfo::builder("id", "name").build())
                .add_validator(&Validator::builder(ValidatorType::Equal, 30.into()).build())
                .add_metadata("key", "value".into())
                .subcomponent(&Subcomponent::builder("name").build())
                .build();
            s.add_measurement_with_details(&measurement).await?;

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
            "testStepArtifact": {
                "testStepId": "step_0",
                "measurementSeriesStart": {
                    "measurementSeriesId": "series_0",
                    "name": "name"
                }
            },
            "sequenceNumber": 3,
            "timestamp": DATETIME_FORMATTED
        }),
        json!({
            "testStepArtifact": {
                "testStepId": "step_0",
                "measurementSeriesEnd": {
                    "measurementSeriesId": "series_0",
                    "totalCount": 0
                }
            },
            "sequenceNumber": 4,
            "timestamp": DATETIME_FORMATTED
        }),
        json_step_complete(5),
        json_run_pass(6),
    ];

    check_output_step(&expected, |s, _| {
        async {
            let series = s.add_measurement_series("name").start().await?;
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
            "testStepArtifact": {
                "testStepId": "step_0",
                "measurementSeriesStart": {
                    "measurementSeriesId": "series_0",
                    "name": "name"
                }
            },
            "sequenceNumber": 3,
            "timestamp": DATETIME_FORMATTED
        }),
        json!({
            "testStepArtifact": {
                "testStepId": "step_0",
                "measurementSeriesEnd": {
                    "measurementSeriesId": "series_0",
                    "totalCount": 0
                }
            },
            "sequenceNumber": 4,
            "timestamp": DATETIME_FORMATTED
        }),
        json!({
            "testStepArtifact": {
                "testStepId": "step_0",
                "measurementSeriesStart": {
                    "measurementSeriesId": "series_1",
                    "name": "name"
                }
            },
            "sequenceNumber": 5,
            "timestamp": DATETIME_FORMATTED
        }),
        json!({
            "testStepArtifact": {
                "testStepId": "step_0",
                "measurementSeriesEnd": {
                    "measurementSeriesId": "series_1",
                    "totalCount": 0
                }
            },
            "sequenceNumber": 6,
            "timestamp": DATETIME_FORMATTED
        }),
        json_step_complete(7),
        json_run_pass(8),
    ];

    check_output_step(&expected, |s, _| {
        async {
            let series = s.add_measurement_series("name").start().await?;
            series.end().await?;

            let series_2 = s.add_measurement_series("name").start().await?;
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
            "testStepArtifact": {
                "testStepId": "step_0",
                "measurementSeriesStart": {
                    "measurementSeriesId": "series_id",
                    "name": "name"
                }
            },
            "sequenceNumber": 3,
            "timestamp": DATETIME_FORMATTED
        }),
        json!({
            "testStepArtifact": {
                "testStepId": "step_0",
                "measurementSeriesEnd": {
                    "measurementSeriesId": "series_id", "totalCount": 0
                }
            },
            "sequenceNumber": 4,
            "timestamp": DATETIME_FORMATTED
        }),
        json_step_complete(5),
        json_run_pass(6),
    ];

    check_output_step(&expected, |s, _| {
        async {
            let series = s
                .add_measurement_series_with_details(MeasurementSeriesStart::new(
                    "name",
                    "series_id",
                ))
                .start()
                .await?;
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
            "testStepArtifact": {
                "testStepId": "step_0",
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
                    "subcomponent": {
                        "name": "name"
                    },
                    "validators": [{
                        "type": "EQUAL",
                        "value": 30
                    }]
                }
            },
            "sequenceNumber": 3,
            "timestamp": DATETIME_FORMATTED
        }),
        json!({
            "testStepArtifact": {
                "testStepId": "step_0",
                "measurementSeriesEnd": {
                    "measurementSeriesId": "series_id",
                    "totalCount": 0
                }
            },
            "sequenceNumber": 4,
            "timestamp": DATETIME_FORMATTED
        }),
        json_step_complete(5),
        json_run_pass(6),
    ];

    check_output_step(&expected, |s, _| {
        async {
            let series = s
                .add_measurement_series_with_details(
                    MeasurementSeriesStart::builder("name", "series_id")
                        .add_metadata("key", "value".into())
                        .add_validator(&Validator::builder(ValidatorType::Equal, 30.into()).build())
                        .hardware_info(&HardwareInfo::builder("id", "name").build())
                        .subcomponent(&Subcomponent::builder("name").build())
                        .build(),
                )
                .start()
                .await?;
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
            "testStepArtifact": {
                "testStepId": "step_0",
                "measurementSeriesStart": {
                    "measurementSeriesId": "series_0",
                    "name": "name"
                }
            },
            "sequenceNumber": 3,
            "timestamp": DATETIME_FORMATTED
        }),
        json!({
            "testStepArtifact": {
                "testStepId": "step_0",
                "measurementSeriesElement": {
                    "index": 0,
                    "measurementSeriesId": "series_0",
                    "value": 60,
                    "timestamp": DATETIME_FORMATTED
                }
            },
            "sequenceNumber": 4,
            "timestamp": DATETIME_FORMATTED
        }),
        json!({
            "testStepArtifact": {
                "testStepId": "step_0",
                "measurementSeriesEnd": {
                    "measurementSeriesId": "series_0",
                    "totalCount": 1
                }
            },
            "sequenceNumber": 5,
            "timestamp": DATETIME_FORMATTED
        }),
        json_step_complete(6),
        json_run_pass(7),
    ];

    check_output_step(&expected, |s, _| {
        async {
            let series = s.add_measurement_series("name").start().await?;
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
            "testStepArtifact": {
                "testStepId": "step_0",
                "measurementSeriesStart": {
                    "measurementSeriesId": "series_0",
                    "name": "name"
                }
            },
            "sequenceNumber": 3,
            "timestamp": DATETIME_FORMATTED
        }),
        json!({
            "testStepArtifact": {
                "testStepId": "step_0",
                "measurementSeriesElement": {
                    "index": 0,
                    "measurementSeriesId": "series_0",
                    "value": 60,
                    "timestamp": DATETIME_FORMATTED
                }
            },
            "sequenceNumber": 4,
            "timestamp": DATETIME_FORMATTED
        }),
        json!({
            "testStepArtifact": {
                "testStepId": "step_0",
                "measurementSeriesElement": {
                    "index": 1,
                    "measurementSeriesId": "series_0",
                    "value": 70,
                    "timestamp": DATETIME_FORMATTED
                }
            },
            "sequenceNumber": 5,
            "timestamp": DATETIME_FORMATTED
        }),
        json!({
            "testStepArtifact": {
                "testStepId": "step_0",
                "measurementSeriesElement": {
                    "index": 2,
                    "measurementSeriesId": "series_0",
                    "value": 80,
                    "timestamp": DATETIME_FORMATTED
                }
            },
            "sequenceNumber": 6,
            "timestamp": DATETIME_FORMATTED
        }),
        json!({
            "testStepArtifact": {
                "testStepId": "step_0",
                "measurementSeriesEnd": {
                    "measurementSeriesId": "series_0",
                    "totalCount": 3
                }
            },
            "sequenceNumber": 7,
            "timestamp": DATETIME_FORMATTED
        }),
        json_step_complete(8),
        json_run_pass(9),
    ];

    check_output_step(&expected, |s, _| {
        async {
            let series = s.add_measurement_series("name").start().await?;
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
            "testStepArtifact": {
                "testStepId": "step_0",
                "measurementSeriesStart": {
                    "measurementSeriesId": "series_0",
                    "name": "name"
                }
            },
            "sequenceNumber": 3,
            "timestamp": DATETIME_FORMATTED
        }),
        json!({
            "testStepArtifact": {
                "testStepId": "step_0",
                "measurementSeriesElement": {
                    "index": 0,
                    "measurementSeriesId": "series_0",
                    "metadata": {
                        "key": "value"
                    },
                    "value": 60,
                    "timestamp": DATETIME_FORMATTED,
                }
            },
            "sequenceNumber": 4,
            "timestamp": DATETIME_FORMATTED
        }),
        json!({
            "testStepArtifact": {
                "testStepId": "step_0",
                "measurementSeriesEnd": {
                    "measurementSeriesId": "series_0",
                    "totalCount": 1
                }
            },
            "sequenceNumber": 5,
            "timestamp": DATETIME_FORMATTED
        }),
        json_step_complete(6),
        json_run_pass(7),
    ];

    check_output_step(&expected, |s, _| {
        async {
            let series = s.add_measurement_series("name").start().await?;
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
            "testStepArtifact": {
                "testStepId": "step_0",
                "measurementSeriesStart": {
                    "measurementSeriesId": "series_0",
                    "name": "name"
                }
            },
            "sequenceNumber": 3,
            "timestamp": DATETIME_FORMATTED
        }),
        json!({
            "testStepArtifact": {
                "testStepId": "step_0",
                "measurementSeriesElement": {
                    "index": 0,
                    "measurementSeriesId": "series_0",
                    "metadata": {"key": "value"},
                    "value": 60,
                    "timestamp": DATETIME_FORMATTED,
                }
            },
            "sequenceNumber": 4,
            "timestamp": DATETIME_FORMATTED
        }),
        json!({
            "testStepArtifact": {
                "testStepId": "step_0",
                "measurementSeriesElement": {
                    "index": 1,
                    "measurementSeriesId": "series_0",
                    "metadata": {"key2": "value2"},
                    "value": 70,
                    "timestamp": DATETIME_FORMATTED,
                }
            },
            "sequenceNumber": 5,
            "timestamp": DATETIME_FORMATTED
        }),
        json!({
            "testStepArtifact": {
                "testStepId": "step_0",
                "measurementSeriesElement": {
                    "index": 2,
                    "measurementSeriesId": "series_0",
                    "metadata": {"key3": "value3"},
                    "value": 80,
                    "timestamp": DATETIME_FORMATTED,
                }
            },
            "sequenceNumber": 6,
            "timestamp": DATETIME_FORMATTED
        }),
        json!({
            "testStepArtifact": {
                "testStepId": "step_0",
                "measurementSeriesEnd": {
                    "measurementSeriesId": "series_0",
                    "totalCount": 3
                }
            },
            "sequenceNumber": 7,
            "timestamp": DATETIME_FORMATTED
        }),
        json_step_complete(8),
        json_run_pass(9),
    ];

    check_output_step(&expected, |s, _| {
        async {
            let series = s.add_measurement_series("name").start().await?;
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

#[cfg(feature = "boxed-scopes")]
#[tokio::test]
async fn test_step_with_measurement_series_scope() -> Result<()> {
    let expected = [
        json_schema_version(),
        json_run_default_start(),
        json_step_default_start(),
        json!({
            "testStepArtifact": {
                "testStepId": "step_0",
                "measurementSeriesStart": {
                    "measurementSeriesId": "series_0",
                    "name": "name"
                }
            },
            "sequenceNumber": 3,
            "timestamp": DATETIME_FORMATTED
        }),
        json!({
            "testStepArtifact": {
                "testStepId": "step_0",
                "measurementSeriesElement": {
                    "index": 0,
                    "measurementSeriesId": "series_0",
                    "value": 60,
                    "timestamp": DATETIME_FORMATTED
                }
            },
            "sequenceNumber": 4,
            "timestamp": DATETIME_FORMATTED
        }),
        json!({
            "testStepArtifact": {
                "testStepId": "step_0",
                "measurementSeriesElement": {
                    "index": 1,
                    "measurementSeriesId": "series_0",
                    "value": 70,
                    "timestamp": DATETIME_FORMATTED
                }
            },
            "sequenceNumber": 5,
            "timestamp": DATETIME_FORMATTED
        }),
        json!({
            "testStepArtifact": {
                "testStepId": "step_0",
                "measurementSeriesElement": {
                    "index": 2,
                    "measurementSeriesId": "series_0",
                    "value": 80,
                    "timestamp": DATETIME_FORMATTED
                }
            },
            "sequenceNumber": 6,
            "timestamp": DATETIME_FORMATTED
        }),
        json!({
            "testStepArtifact": {
                "testStepId": "step_0",
                "measurementSeriesEnd": {
                    "measurementSeriesId": "series_0",
                    "totalCount": 3
                }
            },
            "sequenceNumber": 7,
            "timestamp": DATETIME_FORMATTED
        }),
        json_step_complete(8),
        json_run_pass(9),
    ];

    check_output_step(&expected, |s, _| {
        async {
            let series = s.add_measurement_series("name");
            series
                .scope(|s| {
                    async move {
                        s.add_measurement(60.into()).await?;
                        s.add_measurement(70.into()).await?;
                        s.add_measurement(80.into()).await?;

                        Ok(())
                    }
                    .boxed()
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
    use assert_fs::prelude::*;
    use predicates::prelude::*;
    use std::fs;

    let expected = [
        json_schema_version(),
        json!({
            "testRunArtifact": {
                "testRunStart": {
                    "dutInfo": {
                        "dutInfoId": "dut_id"
                    },
                    "name": "run_name",
                    "parameters": {},
                    "version": "1.0",
                    "commandLine": ""
                }
            },
            "sequenceNumber": 1,
            "timestamp": DATETIME_FORMATTED
        }),
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

    let fs = assert_fs::TempDir::new()?;
    let output_file = fs.child("output.jsonl");

    let dut = DutInfo::builder("dut_id").build();

    let run = TestRun::builder("run_name", &dut, "1.0")
        .config(
            Config::builder()
                .timezone(chrono_tz::Europe::Rome)
                .with_timestamp_provider(Box::new(FixedTsProvider {}))
                .with_file_output(output_file.path())
                .await?
                .build(),
        )
        .build()
        .start()
        .await?;

    run.add_error_with_msg("symptom", "Error message").await?;

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
async fn test_step_with_extension() -> Result<()> {
    let expected = [
        json_schema_version(),
        json_run_default_start(),
        json_step_default_start(),
        json!({
            "testStepArtifact": {
                "testStepId": "step_0",
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
            s.extension(
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
    let run = TestRun::builder("run_name", &dut, "1.0")
        .config(
            Config::builder()
                .with_buffer_output(Arc::clone(&buffer))
                .with_timestamp_provider(Box::new(FixedTsProvider {}))
                .build(),
        )
        .build()
        .start()
        .await?;
    let step = run.add_step("first step").start().await?;

    let result = step.extension("extension", Ext { i: 0 }).await;

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

#[tokio::test]
async fn test_testrun_instantiation_with_new() -> Result<()> {
    let expected = [
        json_schema_version(),
        json_run_default_start(),
        json_run_pass(2),
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

    check_output(&expected, |run_builder, _| async {
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
                        "dutInfoId": "dut_id",
                        "softwareInfos": [{
                            "softwareInfoId": "sw0",
                            "name": "ubuntu",
                            "version": "22",
                            "softwareType": "SYSTEM",
                        }],
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

    check_output(&expected, |run_builder, _| async {
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
