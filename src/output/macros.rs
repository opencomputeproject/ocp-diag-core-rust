// (c) Meta Platforms, Inc. and affiliates.
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

//! OCPTV library macros
//!
//! This module contains a set of macros which are exported from the ocptv
//! library.

/// Emits an artifact of type Error.
/// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#error
///
/// Equivalent to the crate::runner::TestRun::error_with_details method.
///
/// It accepts both a symptom and a message, or just a symptom.
/// Information about the source file and line number is automatically added.
///
/// # Examples
///
/// ## Passing only symptom
///
/// ```rust
/// # tokio_test::block_on(async {
/// # use ocptv::output::*;
///
/// use ocptv::ocptv_error;
///
/// let test_run = TestRun::new("run_name", "my_dut", "1.0").start().await?;
/// ocptv_error!(test_run, "symptom");
/// test_run.end(TestStatus::Complete, TestResult::Pass).await?;
///
/// # Ok::<(), WriterError>(())
/// # });
/// ```
///
/// ## Passing both symptom and message
///
/// ```rust
/// # tokio_test::block_on(async {
/// # use ocptv::output::*;
///
/// use ocptv::ocptv_error;
///
/// let test_run = TestRun::new("run_name", "my_dut", "1.0").start().await?;
/// ocptv_error!(test_run, "symptom", "Error message");
/// test_run.end(TestStatus::Complete, TestResult::Pass).await?;
///
/// # Ok::<(), WriterError>(())
/// # });
/// ```
#[macro_export]
macro_rules! ocptv_error {
    ($runner:expr, $symptom:expr, $msg:expr) => {
        $runner.error_with_details(
            &$crate::output::Error::builder($symptom)
                .message($msg)
                .source(file!(), line!() as i32)
                .build(),
        )
    };

    ($runner:expr, $symptom:expr) => {
        $runner.error_with_details(
            &$crate::output::Error::builder($symptom)
                .source(file!(), line!() as i32)
                .build(),
        )
    };
}

/// The following macros emit an artifact of type Log.
/// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#log
///
/// Equivalent to the crate::runner::TestRun::log_with_details method.
///
/// They accept message as only parameter.
/// Information about the source file and line number is automatically added.
///
/// There is one macro for each severity level: DEBUG, INFO, WARNING, ERROR, and FATAL.
///
/// # Examples
///
/// ## DEBUG
///
/// ```rust
/// # tokio_test::block_on(async {
/// # use ocptv::output::*;
///
/// use ocptv::ocptv_log_debug;
///
/// let run = TestRun::new("run_name", "my_dut", "1.0").start().await?;
/// ocptv_log_debug!(run, "Log message");
/// run.end(TestStatus::Complete, TestResult::Pass).await?;
///
/// # Ok::<(), WriterError>(())
/// # });
/// ```

macro_rules! ocptv_log {
    ($name:ident, $severity:ident) => {
        #[macro_export]
        macro_rules! $name {
            ($artifact:expr, $msg:expr) => {
                $artifact.log_with_details(
                    &$crate::output::Log::builder($msg)
                        .severity($crate::output::LogSeverity::$severity)
                        .source(file!(), line!() as i32)
                        .build(),
                )
            };
        }
    };
}

ocptv_log!(ocptv_log_debug, Debug);
ocptv_log!(ocptv_log_info, Info);
ocptv_log!(ocptv_log_warning, Warning);
ocptv_log!(ocptv_log_error, Error);
ocptv_log!(ocptv_log_fatal, Fatal);

#[cfg(test)]
mod tests {
    use std::future::Future;
    use std::sync::Arc;

    use anyhow::anyhow;
    use anyhow::Result;
    use assert_json_diff::assert_json_include;
    use serde_json::json;
    use tokio::sync::Mutex;

    use crate::output::objects::*;
    use crate::output::runner::*;

    async fn check_output<F, R>(expected: &serde_json::Value, func: F) -> Result<serde_json::Value>
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
                .first_chunk::<3>()
                .ok_or(anyhow!("buffer is missing macro output item"))?[2],
        )?;
        assert_json_include!(actual: actual.clone(), expected: expected);

        Ok(actual)
    }

    async fn check_output_run<F, R>(expected: &serde_json::Value, key: &str, func: F) -> Result<()>
    where
        R: Future<Output = Result<()>>,
        F: FnOnce(StartedTestRun) -> R,
    {
        let actual = check_output(expected, func).await?;

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
        F: FnOnce(TestStep) -> R,
    {
        let actual = check_output(expected, |run| async move {
            let step = run.step("step_name")?;
            // TODO: missing step start here

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
            "sequenceNumber": 3
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
            "sequenceNumber": 3
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
            "sequenceNumber": 3
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
            "sequenceNumber": 3
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
            "sequenceNumber": 3
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
            "sequenceNumber": 3
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
            "sequenceNumber": 3
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
}
