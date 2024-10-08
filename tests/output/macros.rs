// (c) Meta Platforms, Inc. and affiliates.
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

use std::future::Future;
use std::sync::Arc;

use anyhow::anyhow;
use anyhow::Result;
use assert_json_diff::assert_json_include;
use serde_json::json;
use tokio::sync::Mutex;

use ocptv::ocptv_error;
use ocptv::output as tv;
use ocptv::{ocptv_log_debug, ocptv_log_error, ocptv_log_fatal, ocptv_log_info, ocptv_log_warning};
use tv::{Config, DutInfo, StartedTestRun, StartedTestStep, TestRun};

async fn check_output<F, R, const N: usize>(
    expected: &serde_json::Value,
    func: F,
) -> Result<serde_json::Value>
where
    R: Future<Output = Result<()>>,
    F: FnOnce(StartedTestRun) -> R,
{
    let buffer: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));

    let dut = DutInfo::builder("dut_id").build();
    let run = TestRun::builder("run_name", &dut, "1.0")
        .config(Config::builder().with_buffer_output(buffer.clone()).build())
        .build()
        .start()
        .await?;

    func(run).await?;

    let actual = serde_json::from_str::<serde_json::Value>(
        &buffer
            .lock()
            .await
            // first 2 items are schemaVersion, testRunStart
            .first_chunk::<N>()
            .ok_or(anyhow!("buffer is missing macro output item"))?[N - 1],
    )?;
    assert_json_include!(actual: actual.clone(), expected: expected);

    Ok(actual)
}

async fn check_output_run<F, R>(expected: &serde_json::Value, key: &str, func: F) -> Result<()>
where
    R: Future<Output = Result<()>>,
    F: FnOnce(StartedTestRun) -> R,
{
    let actual = check_output::<_, _, 3>(expected, func).await?;

    let source = actual
        .get("testRunArtifact")
        .ok_or(anyhow!("testRunArtifact key does not exist"))?
        .get(key)
        .ok_or(anyhow!("error key does not exist"))?;

    assert_ne!(
        source.get("sourceLocation"),
        None,
        "sourceLocation is not present in the serialized object"
    );

    Ok(())
}

async fn check_output_step<F, R>(expected: &serde_json::Value, key: &str, func: F) -> Result<()>
where
    R: Future<Output = Result<()>>,
    F: FnOnce(StartedTestStep) -> R,
{
    let actual = check_output::<_, _, 4>(expected, |run| async move {
        let step = run.step("step_name").start().await?;

        func(step).await?;
        Ok(())
    })
    .await?;

    let source = actual
        .get("testStepArtifact")
        .ok_or(anyhow!("testRunArtifact key does not exist"))?
        .get(key)
        .ok_or(anyhow!("error key does not exist"))?;

    assert_ne!(
        source.get("sourceLocation"),
        None,
        "sourceLocation is not present in the serialized object"
    );

    Ok(())
}

#[tokio::test]
async fn test_ocptv_error_macro_with_symptom_and_message() -> Result<()> {
    let expected = json!({
        "testRunArtifact": {
            "error": {
                "message": "Error message",
                "symptom": "symptom"
            }
        },
        "sequenceNumber": 2
    });

    check_output_run(&expected, "error", |run| async move {
        ocptv_error!(run, "symptom", "Error message").await?;
        Ok(())
    })
    .await
}

#[tokio::test]
async fn test_ocptv_error_macro_with_symptom() -> Result<()> {
    let expected = json!({
        "testRunArtifact": {
            "error": {
                "symptom": "symptom"
            }
        },
        "sequenceNumber": 2
    });

    check_output_run(&expected, "error", |run| async move {
        ocptv_error!(run, "symptom").await?;
        Ok(())
    })
    .await
}

#[tokio::test]
async fn test_ocptv_log_debug() -> Result<()> {
    let expected = json!({
        "testRunArtifact": {
            "log": {
                "message": "log message",
                "severity": "DEBUG"
            }
        },
        "sequenceNumber": 2
    });

    check_output_run(&expected, "log", |run| async move {
        ocptv_log_debug!(run, "log message").await?;

        Ok(())
    })
    .await
}

#[tokio::test]
async fn test_ocptv_log_info() -> Result<()> {
    let expected = json!({
        "testRunArtifact": {
            "log": {
                "message": "log message",
                "severity": "INFO"
            }
        },
        "sequenceNumber": 2
    });

    check_output_run(&expected, "log", |run| async move {
        ocptv_log_info!(run, "log message").await?;
        Ok(())
    })
    .await
}

#[tokio::test]
async fn test_ocptv_log_warning() -> Result<()> {
    let expected = json!({
        "testRunArtifact": {
            "log": {
                "message": "log message",
                "severity": "WARNING"
            }
        },
        "sequenceNumber": 2
    });

    check_output_run(&expected, "log", |run| async move {
        ocptv_log_warning!(run, "log message").await?;
        Ok(())
    })
    .await
}

#[tokio::test]
async fn test_ocptv_log_error() -> Result<()> {
    let expected = json!({
        "testRunArtifact": {
            "log": {
                "message": "log message",
                "severity": "ERROR"
            }
        },
        "sequenceNumber": 2
    });

    check_output_run(&expected, "log", |run| async move {
        ocptv_log_error!(run, "log message").await?;
        Ok(())
    })
    .await
}

#[tokio::test]
async fn test_ocptv_log_fatal() -> Result<()> {
    let expected = json!({
        "testRunArtifact": {
            "log": {
                "message": "log message",
                "severity": "FATAL"
            }
        },
        "sequenceNumber": 2
    });

    check_output_run(&expected, "log", |run| async move {
        ocptv_log_fatal!(run, "log message").await?;
        Ok(())
    })
    .await
}

#[tokio::test]
async fn test_ocptv_error_macro_with_symptom_and_message_in_step() -> Result<()> {
    let expected = json!({
        "testStepArtifact": {
            "error": {
                "message": "Error message",
                "symptom":"symptom"
            }
        },
        "sequenceNumber": 3
    });

    check_output_step(&expected, "error", |step| async move {
        ocptv_error!(step, "symptom", "Error message").await?;
        Ok(())
    })
    .await
}

#[tokio::test]
async fn test_ocptv_error_macro_with_symptom_in_step() -> Result<()> {
    let expected = json!({
        "testStepArtifact": {
            "error": {
                "symptom": "symptom"
            }
        },
        "sequenceNumber": 3
    });

    check_output_step(&expected, "error", |step| async move {
        ocptv_error!(step, "symptom").await?;
        Ok(())
    })
    .await
}

#[tokio::test]
async fn test_ocptv_log_debug_in_step() -> Result<()> {
    let expected = json!({
        "testStepArtifact": {
            "log": {
                "message": "log message",
                "severity": "DEBUG"
            }
        },
        "sequenceNumber": 3
    });

    check_output_step(&expected, "log", |step| async move {
        ocptv_log_debug!(step, "log message").await?;
        Ok(())
    })
    .await
}

#[tokio::test]
async fn test_ocptv_log_info_in_step() -> Result<()> {
    let expected = json!({
        "testStepArtifact": {
            "log": {
                "message": "log message",
                "severity": "INFO"
            }
        },
        "sequenceNumber": 3
    });

    check_output_step(&expected, "log", |step| async move {
        ocptv_log_info!(step, "log message").await?;
        Ok(())
    })
    .await
}

#[tokio::test]
async fn test_ocptv_log_warning_in_step() -> Result<()> {
    let expected = json!({
        "testStepArtifact": {
            "log": {
                "message": "log message",
                "severity":"WARNING"
            }
        },
        "sequenceNumber": 3
    });

    check_output_step(&expected, "log", |step| async move {
        ocptv_log_warning!(step, "log message").await?;
        Ok(())
    })
    .await
}

#[tokio::test]
async fn test_ocptv_log_error_in_step() -> Result<()> {
    let expected = json!({
        "testStepArtifact": {
            "log": {
                "message": "log message",
                "severity": "ERROR"
            }
        },
        "sequenceNumber": 3
    });

    check_output_step(&expected, "log", |step| async move {
        ocptv_log_error!(step, "log message").await?;
        Ok(())
    })
    .await
}

#[tokio::test]
async fn test_ocptv_log_fatal_in_step() -> Result<()> {
    let expected = json!({
        "testStepArtifact": {
            "log": {
                "message": "log message",
                "severity": "FATAL"
            }
        },
        "sequenceNumber": 3
    });

    check_output_step(&expected, "log", |step| async move {
        ocptv_log_fatal!(step, "log message").await?;
        Ok(())
    })
    .await
}
