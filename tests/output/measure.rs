// (c) Meta Platforms, Inc. and affiliates.
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

use anyhow::Result;
use serde_json::json;

use ocptv::output::{
    Ident, Measurement, MeasurementElementDetail, MeasurementSeriesDetail, Subcomponent, Validator,
    ValidatorType,
};

use super::fixture::*;

#[tokio::test]
async fn test_step_with_measurement() -> Result<()> {
    let expected = [
        json_schema_version(),
        json_run_default_start(),
        json_step_default_start(),
        json!({
            "testStepArtifact": {
                "testStepId": "step0",
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

    check_output_step(&expected, |s, _| async move {
        s.add_measurement("name", 50).await?;

        Ok(())
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
                "testStepId": "step0",
                "measurement": {
                    "name": "name",
                    "value": 50,
                    "validators": [{
                        "type": "EQUAL",
                        "value": 30
                    }],
                    "hardwareInfoId": "hw0",
                    "subcomponent": {
                        "name": "name"
                    },
                    "metadata": {
                        "key": "value",
                        "key2": "value2"
                    }
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
            let hw_info = dut.hardware_info("hw0").unwrap(); // must exist

            let measurement = Measurement::builder("name", 50)
                .add_validator(Validator::builder(ValidatorType::Equal, 30).build())
                .add_metadata("key", "value")
                .add_metadata("key2", "value2")
                .hardware_info(hw_info)
                .subcomponent(Subcomponent::builder("name").build())
                .build();
            s.add_measurement_detail(measurement).await?;

            Ok(())
        }
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
                "testStepId": "step0",
                "measurementSeriesStart": {
                    "measurementSeriesId": "step0_series0",
                    "name": "name"
                }
            },
            "sequenceNumber": 3,
            "timestamp": DATETIME_FORMATTED
        }),
        json!({
            "testStepArtifact": {
                "testStepId": "step0",
                "measurementSeriesEnd": {
                    "measurementSeriesId": "step0_series0",
                    "totalCount": 0
                }
            },
            "sequenceNumber": 4,
            "timestamp": DATETIME_FORMATTED
        }),
        json_step_complete(5),
        json_run_pass(6),
    ];

    check_output_step(&expected, |s, _| async move {
        let series = s.add_measurement_series("name").start().await?;
        series.end().await?;

        Ok(())
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
                "testStepId": "step0",
                "measurementSeriesStart": {
                    "measurementSeriesId": "step0_series0",
                    "name": "name"
                }
            },
            "sequenceNumber": 3,
            "timestamp": DATETIME_FORMATTED
        }),
        json!({
            "testStepArtifact": {
                "testStepId": "step0",
                "measurementSeriesEnd": {
                    "measurementSeriesId": "step0_series0",
                    "totalCount": 0
                }
            },
            "sequenceNumber": 4,
            "timestamp": DATETIME_FORMATTED
        }),
        json!({
            "testStepArtifact": {
                "testStepId": "step0",
                "measurementSeriesStart": {
                    "measurementSeriesId": "step0_series1",
                    "name": "name"
                }
            },
            "sequenceNumber": 5,
            "timestamp": DATETIME_FORMATTED
        }),
        json!({
            "testStepArtifact": {
                "testStepId": "step0",
                "measurementSeriesEnd": {
                    "measurementSeriesId": "step0_series1",
                    "totalCount": 0
                }
            },
            "sequenceNumber": 6,
            "timestamp": DATETIME_FORMATTED
        }),
        json_step_complete(7),
        json_run_pass(8),
    ];

    check_output_step(&expected, |s, _| async move {
        let series = s.add_measurement_series("name").start().await?;
        series.end().await?;

        let series_2 = s.add_measurement_series("name").start().await?;
        series_2.end().await?;

        Ok(())
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
                "testStepId": "step0",
                "measurementSeriesStart": {
                    "measurementSeriesId": "series_id",
                    "name": "name",
                    "unit": "unit",
                    "validators": [{
                        "type": "EQUAL",
                        "value": 30
                    }],
                    "hardwareInfoId": "hw0",
                    "subcomponent": {
                        "name": "name"
                    },
                    "metadata": {
                        "key": "value",
                        "key2": "value2"
                    }
                }
            },
            "sequenceNumber": 3,
            "timestamp": DATETIME_FORMATTED
        }),
        json!({
            "testStepArtifact": {
                "testStepId": "step0",
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

    check_output_step(&expected, |s, dut| {
        async move {
            let hw_info = dut.hardware_info("hw0").unwrap(); // must exist

            let series = s
                .add_measurement_series_detail(
                    MeasurementSeriesDetail::builder("name")
                        .id(Ident::Exact("series_id".to_owned()))
                        .unit("unit")
                        .add_metadata("key", "value")
                        .add_metadata("key2", "value2")
                        .add_validator(Validator::builder(ValidatorType::Equal, 30).build())
                        .hardware_info(hw_info)
                        .subcomponent(Subcomponent::builder("name").build())
                        .build(),
                )
                .start()
                .await?;
            series.end().await?;

            Ok(())
        }
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
                "testStepId": "step0",
                "measurementSeriesStart": {
                    "measurementSeriesId": "step0_series0",
                    "name": "name"
                }
            },
            "sequenceNumber": 3,
            "timestamp": DATETIME_FORMATTED
        }),
        json!({
            "testStepArtifact": {
                "testStepId": "step0",
                "measurementSeriesElement": {
                    "index": 0,
                    "measurementSeriesId": "step0_series0",
                    "value": 60,
                    "timestamp": DATETIME_FORMATTED
                }
            },
            "sequenceNumber": 4,
            "timestamp": DATETIME_FORMATTED
        }),
        json!({
            "testStepArtifact": {
                "testStepId": "step0",
                "measurementSeriesEnd": {
                    "measurementSeriesId": "step0_series0",
                    "totalCount": 1
                }
            },
            "sequenceNumber": 5,
            "timestamp": DATETIME_FORMATTED
        }),
        json_step_complete(6),
        json_run_pass(7),
    ];

    check_output_step(&expected, |s, _| async move {
        let series = s.add_measurement_series("name").start().await?;
        series.add_measurement(60).await?;
        series.end().await?;

        Ok(())
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
                "testStepId": "step0",
                "measurementSeriesStart": {
                    "measurementSeriesId": "step0_series0",
                    "name": "name"
                }
            },
            "sequenceNumber": 3,
            "timestamp": DATETIME_FORMATTED
        }),
        json!({
            "testStepArtifact": {
                "testStepId": "step0",
                "measurementSeriesElement": {
                    "index": 0,
                    "measurementSeriesId": "step0_series0",
                    "value": 60,
                    "timestamp": DATETIME_FORMATTED
                }
            },
            "sequenceNumber": 4,
            "timestamp": DATETIME_FORMATTED
        }),
        json!({
            "testStepArtifact": {
                "testStepId": "step0",
                "measurementSeriesElement": {
                    "index": 1,
                    "measurementSeriesId": "step0_series0",
                    "value": 70,
                    "timestamp": DATETIME_FORMATTED
                }
            },
            "sequenceNumber": 5,
            "timestamp": DATETIME_FORMATTED
        }),
        json!({
            "testStepArtifact": {
                "testStepId": "step0",
                "measurementSeriesElement": {
                    "index": 2,
                    "measurementSeriesId": "step0_series0",
                    "value": 80,
                    "timestamp": DATETIME_FORMATTED
                }
            },
            "sequenceNumber": 6,
            "timestamp": DATETIME_FORMATTED
        }),
        json!({
            "testStepArtifact": {
                "testStepId": "step0",
                "measurementSeriesEnd": {
                    "measurementSeriesId": "step0_series0",
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
        async move {
            let series = s.add_measurement_series("name").start().await?;
            // add more than one element to check the index increments correctly
            series.add_measurement(60).await?;
            series.add_measurement(70).await?;
            series.add_measurement(80).await?;
            series.end().await?;

            Ok(())
        }
    })
    .await
}

#[tokio::test]
async fn test_step_with_measurement_series_element_with_details() -> Result<()> {
    let expected = [
        json_schema_version(),
        json_run_default_start(),
        json_step_default_start(),
        json!({
            "testStepArtifact": {
                "testStepId": "step0",
                "measurementSeriesStart": {
                    "measurementSeriesId": "step0_series0",
                    "name": "name"
                }
            },
            "sequenceNumber": 3,
            "timestamp": DATETIME_FORMATTED
        }),
        json!({
            "testStepArtifact": {
                "testStepId": "step0",
                "measurementSeriesElement": {
                    "index": 0,
                    "measurementSeriesId": "step0_series0",
                    "metadata": {
                        "key": "value",
                        "key2": "value2"
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
                "testStepId": "step0",
                "measurementSeriesEnd": {
                    "measurementSeriesId": "step0_series0",
                    "totalCount": 1
                }
            },
            "sequenceNumber": 5,
            "timestamp": DATETIME_FORMATTED
        }),
        json_step_complete(6),
        json_run_pass(7),
    ];

    check_output_step(&expected, |s, _| async move {
        s.add_measurement_series("name")
            .scope(|s| async move {
                s.add_measurement_detail(
                    MeasurementElementDetail::builder(60)
                        .timestamp(DATETIME.with_timezone(&chrono_tz::UTC))
                        .add_metadata("key", "value")
                        .add_metadata("key2", "value2")
                        .build(),
                )
                .await?;

                Ok(())
            })
            .await?;

        Ok(())
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
                "testStepId": "step0",
                "measurementSeriesStart": {
                    "measurementSeriesId": "step0_series0",
                    "name": "name"
                }
            },
            "sequenceNumber": 3,
            "timestamp": DATETIME_FORMATTED
        }),
        json!({
            "testStepArtifact": {
                "testStepId": "step0",
                "measurementSeriesElement": {
                    "index": 0,
                    "measurementSeriesId": "step0_series0",
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
                "testStepId": "step0",
                "measurementSeriesElement": {
                    "index": 1,
                    "measurementSeriesId": "step0_series0",
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
                "testStepId": "step0",
                "measurementSeriesElement": {
                    "index": 2,
                    "measurementSeriesId": "step0_series0",
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
                "testStepId": "step0",
                "measurementSeriesEnd": {
                    "measurementSeriesId": "step0_series0",
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
        async move {
            let series = s.add_measurement_series("name").start().await?;
            // add more than one element to check the index increments correctly
            series
                .add_measurement_detail(
                    MeasurementElementDetail::builder(60)
                        .add_metadata("key", "value")
                        .build(),
                )
                .await?;
            series
                .add_measurement_detail(
                    MeasurementElementDetail::builder(70)
                        .add_metadata("key2", "value2")
                        .build(),
                )
                .await?;
            series
                .add_measurement_detail(
                    MeasurementElementDetail::builder(80)
                        .add_metadata("key3", "value3")
                        .build(),
                )
                .await?;
            series.end().await?;

            Ok(())
        }
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
            "testStepArtifact": {
                "testStepId": "step0",
                "measurementSeriesStart": {
                    "measurementSeriesId": "step0_series0",
                    "name": "name"
                }
            },
            "sequenceNumber": 3,
            "timestamp": DATETIME_FORMATTED
        }),
        json!({
            "testStepArtifact": {
                "testStepId": "step0",
                "measurementSeriesElement": {
                    "index": 0,
                    "measurementSeriesId": "step0_series0",
                    "value": 60,
                    "timestamp": DATETIME_FORMATTED
                }
            },
            "sequenceNumber": 4,
            "timestamp": DATETIME_FORMATTED
        }),
        json!({
            "testStepArtifact": {
                "testStepId": "step0",
                "measurementSeriesElement": {
                    "index": 1,
                    "measurementSeriesId": "step0_series0",
                    "value": 70,
                    "timestamp": DATETIME_FORMATTED
                }
            },
            "sequenceNumber": 5,
            "timestamp": DATETIME_FORMATTED
        }),
        json!({
            "testStepArtifact": {
                "testStepId": "step0",
                "measurementSeriesElement": {
                    "index": 2,
                    "measurementSeriesId": "step0_series0",
                    "value": 80,
                    "timestamp": DATETIME_FORMATTED
                }
            },
            "sequenceNumber": 6,
            "timestamp": DATETIME_FORMATTED
        }),
        json!({
            "testStepArtifact": {
                "testStepId": "step0",
                "measurementSeriesEnd": {
                    "measurementSeriesId": "step0_series0",
                    "totalCount": 3
                }
            },
            "sequenceNumber": 7,
            "timestamp": DATETIME_FORMATTED
        }),
        json_step_complete(8),
        json_run_pass(9),
    ];

    check_output_step(&expected, |s, _| async move {
        let series = s.add_measurement_series("name");
        series
            .scope(|s| async move {
                s.add_measurement(60).await?;
                s.add_measurement(70).await?;
                s.add_measurement(80).await?;

                Ok(())
            })
            .await?;

        Ok(())
    })
    .await
}
