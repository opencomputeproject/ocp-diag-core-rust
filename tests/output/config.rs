// (c) Meta Platforms, Inc. and affiliates.
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

#[cfg(coverage)]
use anyhow::Result;

// reasoning: the coverage(off) attribute is experimental in llvm-cov, so because we cannot
// disable the coverage itself, only run this test when in coverage mode because assert_fs
// does ultimately assume there's a real filesystem somewhere
#[cfg(coverage)]
#[tokio::test]
async fn test_config_builder_with_file() -> Result<()> {
    use std::fs;

    use assert_fs::prelude::*;
    use assert_json_diff::assert_json_include;
    use predicates::prelude::*;
    use serde_json::json;

    use ocptv::output::{Config, DutInfo, TestResult, TestRun, TestStatus};

    use super::fixture::*;

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

    let run = TestRun::builder("run_name", "1.0")
        .config(
            Config::builder()
                .timezone(chrono_tz::Europe::Rome)
                .with_timestamp_provider(Box::new(FixedTsProvider {}))
                .with_file_output(output_file.path())
                .await?
                .build(),
        )
        .build()
        .start(dut)
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
