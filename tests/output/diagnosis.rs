// (c) Meta Platforms, Inc. and affiliates.
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

use anyhow::Result;
use futures::FutureExt;
use serde_json::json;

use ocptv::output::{Diagnosis, DiagnosisType, Subcomponent};

use super::fixture::*;

#[tokio::test]
async fn test_step_with_diagnosis() -> Result<()> {
    let expected = [
        json_schema_version(),
        json_run_default_start(),
        json_step_default_start(),
        json!({
            "testStepArtifact": {
                "testStepId": "step0",
                "diagnosis": {
                    "verdict": "verdict",
                    "type": "PASS"
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
            s.diagnosis("verdict", DiagnosisType::Pass).await?;

            Ok(())
        }
        .boxed()
    })
    .await
}

#[tokio::test]
async fn test_step_with_diagnosis_builder() -> Result<()> {
    let expected = [
        json_schema_version(),
        json_run_default_start(),
        json_step_default_start(),
        json!({
            "testStepArtifact": {
                "testStepId": "step0",
                "diagnosis": {
                    "verdict": "verdict",
                    "type": "PASS",
                    "message": "message",
                    "hardwareInfoId": "hw0",
                    "subcomponent": {
                        "name": "name"
                    },
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
            let diagnosis = Diagnosis::builder("verdict", DiagnosisType::Pass)
                .hardware_info(dut.hardware_info("hw0").unwrap()) // must exist
                .subcomponent(&Subcomponent::builder("name").build())
                .message("message")
                .build();
            s.diagnosis_with_details(&diagnosis).await?;

            Ok(())
        }
        .boxed()
    })
    .await
}
