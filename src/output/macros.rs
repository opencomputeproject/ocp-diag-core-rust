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
/// let test_run = TestRun::new("run_name", "my_dut", "1.0");
/// test_run.start().await?;
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
/// let test_run = TestRun::new("run_name", "my_dut", "1.0");
/// test_run.start().await?;
/// ocptv_error!(test_run, "symptom", "Error message");
/// test_run.end(TestStatus::Complete, TestResult::Pass).await?;
///
/// # Ok::<(), WriterError>(())
/// # });
/// ```
#[macro_export]
macro_rules! ocptv_error {
    ($runner:expr , $symptom:expr, $msg:expr) => {
        async {
            $runner
                .error_with_details(
                    &$crate::output::Error::builder($symptom)
                        .message($msg)
                        .source(file!(), line!() as i32)
                        .build(),
                )
                .await
        }
    };
    ($runner:expr, $symptom:expr) => {
        async {
            $runner
                .error_with_details(
                    &$crate::output::Error::builder($symptom)
                        .source(file!(), line!() as i32)
                        .build(),
                )
                .await
        }
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
/// let test_run = TestRun::new("run_name", "my_dut", "1.0");
/// test_run.start().await?;
/// ocptv_log_debug!(test_run, "Log message");
/// test_run.end(TestStatus::Complete, TestResult::Pass).await?;
///
/// # Ok::<(), WriterError>(())
/// # });
/// ```

#[macro_export]
macro_rules! ocptv_log_debug {
    ($runner:expr, $msg:expr) => {
        async {
            $runner
                .log_with_details(
                    &$crate::output::Log::builder($msg)
                        .severity($crate::output::LogSeverity::Debug)
                        .source(file!(), line!() as i32)
                        .build(),
                )
                .await
        }
    };
}

#[macro_export]
macro_rules! ocptv_log_info {
    ($runner:expr, $msg:expr) => {
        async {
            $runner
                .log_with_details(
                    &$crate::output::Log::builder($msg)
                        .severity($crate::output::LogSeverity::Info)
                        .source(file!(), line!() as i32)
                        .build(),
                )
                .await
        }
    };
}

#[macro_export]
macro_rules! ocptv_log_warning {
    ($runner:expr, $msg:expr) => {
        async {
            $runner
                .log_with_details(
                    &$crate::output::Log::builder($msg)
                        .severity($crate::output::LogSeverity::Warning)
                        .source(file!(), line!() as i32)
                        .build(),
                )
                .await
        }
    };
}

#[macro_export]
macro_rules! ocptv_log_error {
    ($runner:expr, $msg:expr) => {
        async {
            $runner
                .log_with_details(
                    &$crate::output::Log::builder($msg)
                        .severity($crate::output::LogSeverity::Error)
                        .source(file!(), line!() as i32)
                        .build(),
                )
                .await
        }
    };
}

#[macro_export]
macro_rules! ocptv_log_fatal {
    ($runner:expr, $msg:expr) => {
        async {
            $runner
                .log_with_details(
                    &$crate::output::Log::builder($msg)
                        .severity($crate::output::LogSeverity::Fatal)
                        .source(file!(), line!() as i32)
                        .build(),
                )
                .await
        }
    };
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use anyhow::anyhow;
    use anyhow::Result;
    use assert_json_diff::assert_json_include;
    use serde_json::json;
    use tokio::sync::Mutex;

    use crate::output::objects::*;
    use crate::output::runner::*;

    #[tokio::test]
    async fn test_ocptv_error_macro_with_symptom_and_message() -> Result<()> {
        let expected = json!({
            "testRunArtifact":{
                "error": {
                    "message": "Error message",
                    "symptom": "symptom"
                }
            },
            "sequenceNumber": 1
        });

        let buffer: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));
        let dut = DutInfo::builder("dut_id").build();
        let test_run = TestRun::builder("run_name", &dut, "1.0")
            .config(Config::builder().with_buffer_output(buffer.clone()).build())
            .build();

        ocptv_error!(test_run, "symptom", "Error message").await?;

        let actual = serde_json::from_str::<serde_json::Value>(
            buffer
                .lock()
                .await
                .first()
                .ok_or(anyhow!("Buffer is empty"))?,
        )?;
        assert_json_include!(actual: actual.clone(), expected: &expected);

        let source = actual
            .get("testRunArtifact")
            .ok_or(anyhow!("testRunArtifact key does not exist"))?
            .get("error")
            .ok_or(anyhow!("error key does not exist"))?;
        assert_ne!(
            source.get("sourceLocation"),
            None,
            "sourceLocation is not present in the serialized object"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_ocptv_error_macro_with_symptom() -> Result<()> {
        let expected = json!({
            "testRunArtifact": {
                "error": {
                    "symptom": "symptom"
                }
            },
            "sequenceNumber": 1
        });

        let dut = DutInfo::builder("dut_id").build();
        let buffer: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));
        let test_run = TestRun::builder("run_name", &dut, "1.0")
            .config(Config::builder().with_buffer_output(buffer.clone()).build())
            .build();

        ocptv_error!(test_run, "symptom").await?;

        let actual = serde_json::from_str::<serde_json::Value>(
            buffer
                .lock()
                .await
                .first()
                .ok_or(anyhow!("Buffer is empty"))?,
        )?;
        assert_json_include!(actual: actual.clone(), expected: &expected);

        let source = actual
            .get("testRunArtifact")
            .ok_or(anyhow!("testRunArtifact key does not exist"))?
            .get("error")
            .ok_or(anyhow!("error key does not exist"))?;
        assert_ne!(
            source.get("sourceLocation"),
            None,
            "sourceLocation is not present in the serialized object"
        );

        Ok(())
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
            "sequenceNumber":1
        });

        let dut = DutInfo::builder("dut_id").build();
        let buffer: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));
        let test_run = TestRun::builder("run_name", &dut, "1.0")
            .config(Config::builder().with_buffer_output(buffer.clone()).build())
            .build();

        ocptv_log_debug!(test_run, "log message").await?;

        let actual = serde_json::from_str::<serde_json::Value>(
            buffer
                .lock()
                .await
                .first()
                .ok_or(anyhow!("Buffer is empty"))?,
        )?;
        assert_json_include!(actual: actual.clone(), expected: &expected);

        let source = actual
            .get("testRunArtifact")
            .ok_or(anyhow!("testRunArtifact key does not exist"))?
            .get("log")
            .ok_or(anyhow!("log key does not exist"))?;
        assert_ne!(
            source.get("sourceLocation"),
            None,
            "sourceLocation is not present in the serialized object"
        );

        Ok(())
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
            "sequenceNumber": 1
        });

        let dut = DutInfo::builder("dut_id").build();
        let buffer: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));
        let test_run = TestRun::builder("run_name", &dut, "1.0")
            .config(Config::builder().with_buffer_output(buffer.clone()).build())
            .build();

        ocptv_log_info!(test_run, "log message").await?;

        let actual = serde_json::from_str::<serde_json::Value>(
            buffer
                .lock()
                .await
                .first()
                .ok_or(anyhow!("Buffer is empty"))?,
        )?;
        assert_json_include!(actual: actual.clone(), expected: &expected);

        let source = actual
            .get("testRunArtifact")
            .ok_or(anyhow!("testRunArtifact key does not exist"))?
            .get("log")
            .ok_or(anyhow!("log key does not exist"))?;

        assert_ne!(
            source.get("sourceLocation"),
            None,
            "sourceLocation is not present in the serialized object"
        );

        Ok(())
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
            "sequenceNumber": 1
        });

        let dut = DutInfo::builder("dut_id").build();
        let buffer: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));
        let test_run = TestRun::builder("run_name", &dut, "1.0")
            .config(Config::builder().with_buffer_output(buffer.clone()).build())
            .build();

        ocptv_log_warning!(test_run, "log message").await?;

        let actual = serde_json::from_str::<serde_json::Value>(
            buffer
                .lock()
                .await
                .first()
                .ok_or(anyhow!("Buffer is empty"))?,
        )?;
        assert_json_include!(actual: actual.clone(), expected: &expected);

        let source = actual
            .get("testRunArtifact")
            .ok_or(anyhow!("testRunArtifact key does not exist"))?
            .get("log")
            .ok_or(anyhow!("log key does not exist"))?;
        assert_ne!(
            source.get("sourceLocation"),
            None,
            "sourceLocation is not present in the serialized object"
        );

        Ok(())
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
            "sequenceNumber":1
        });

        let dut = DutInfo::builder("dut_id").build();
        let buffer: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));
        let test_run = TestRun::builder("run_name", &dut, "1.0")
            .config(Config::builder().with_buffer_output(buffer.clone()).build())
            .build();

        ocptv_log_error!(test_run, "log message").await?;

        let actual = serde_json::from_str::<serde_json::Value>(
            buffer
                .lock()
                .await
                .first()
                .ok_or(anyhow!("Buffer is empty"))?,
        )?;
        assert_json_include!(actual: actual.clone(), expected: &expected);

        let source = actual
            .get("testRunArtifact")
            .ok_or(anyhow!("testRunArtifact key does not exist"))?
            .get("log")
            .ok_or(anyhow!("log key does not exist"))?;
        assert_ne!(
            source.get("sourceLocation"),
            None,
            "sourceLocation is not present in the serialized object"
        );

        Ok(())
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
            "sequenceNumber": 1
        });

        let dut = DutInfo::builder("dut_id").build();
        let buffer: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));
        let test_run = TestRun::builder("run_name", &dut, "1.0")
            .config(Config::builder().with_buffer_output(buffer.clone()).build())
            .build();

        ocptv_log_fatal!(test_run, "log message").await?;

        let actual = serde_json::from_str::<serde_json::Value>(
            buffer
                .lock()
                .await
                .first()
                .ok_or(anyhow!("Buffer is empty"))?,
        )?;
        assert_json_include!(actual: actual.clone(), expected: &expected);

        let source = actual
            .get("testRunArtifact")
            .ok_or(anyhow!("testRunArtifact key does not exist"))?
            .get("log")
            .ok_or(anyhow!("log key does not exist"))?;
        assert_ne!(
            source.get("sourceLocation"),
            None,
            "sourceLocation is not present in the serialized object"
        );

        Ok(())
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
            "sequenceNumber": 1
        });

        let dut = DutInfo::builder("dut_id").build();
        let buffer: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));
        let test_run = TestRun::builder("run_name", &dut, "1.0")
            .config(Config::builder().with_buffer_output(buffer.clone()).build())
            .build();

        let step = test_run.step("step_name")?;

        ocptv_error!(step, "symptom", "Error message").await?;

        let actual = serde_json::from_str::<serde_json::Value>(
            buffer
                .lock()
                .await
                .first()
                .ok_or(anyhow!("Buffer is empty"))?,
        )?;
        assert_json_include!(actual: actual.clone(), expected: &expected);

        let source = actual
            .get("testStepArtifact")
            .ok_or(anyhow!("testStepArtifact key does not exist"))?
            .get("error")
            .ok_or(anyhow!("error key does not exist"))?;
        assert_ne!(
            source.get("sourceLocation"),
            None,
            "sourceLocation is not present in the serialized object"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_ocptv_error_macro_with_symptom_in_step() -> Result<()> {
        let expected = json!({
            "testStepArtifact": {
                "error": {
                    "symptom": "symptom"
                }
            },
            "sequenceNumber": 1
        });

        let dut = DutInfo::builder("dut_id").build();
        let buffer: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));
        let test_run = TestRun::builder("run_name", &dut, "1.0")
            .config(Config::builder().with_buffer_output(buffer.clone()).build())
            .build();

        let step = test_run.step("step_name")?;

        ocptv_error!(step, "symptom").await?;

        let actual = serde_json::from_str::<serde_json::Value>(
            buffer
                .lock()
                .await
                .first()
                .ok_or(anyhow!("Buffer is empty"))?,
        )?;
        assert_json_include!(actual: actual.clone(), expected: &expected);

        let source = actual
            .get("testStepArtifact")
            .ok_or(anyhow!("testStepArtifact key does not exist"))?
            .get("error")
            .ok_or(anyhow!("error key does not exist"))?;
        assert_ne!(
            source.get("sourceLocation"),
            None,
            "sourceLocation is not present in the serialized object"
        );

        Ok(())
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
            "sequenceNumber": 1
        });

        let dut = DutInfo::builder("dut_id").build();
        let buffer: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));
        let test_run = TestRun::builder("run_name", &dut, "1.0")
            .config(Config::builder().with_buffer_output(buffer.clone()).build())
            .build();

        let step = test_run.step("step_name")?;
        ocptv_log_debug!(step, "log message").await?;

        let actual = serde_json::from_str::<serde_json::Value>(
            buffer
                .lock()
                .await
                .first()
                .ok_or(anyhow!("Buffer is empty"))?,
        )?;
        assert_json_include!(actual: actual.clone(), expected: &expected);

        let source = actual
            .get("testStepArtifact")
            .ok_or(anyhow!("testStepArtifact key does not exist"))?
            .get("log")
            .ok_or(anyhow!("log key does not exist"))?;
        assert_ne!(
            source.get("sourceLocation"),
            None,
            "sourceLocation is not present in the serialized object"
        );

        Ok(())
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
            "sequenceNumber": 1
        });

        let dut = DutInfo::builder("dut_id").build();
        let buffer: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));
        let test_run = TestRun::builder("run_name", &dut, "1.0")
            .config(Config::builder().with_buffer_output(buffer.clone()).build())
            .build();

        let step = test_run.step("step_name")?;
        ocptv_log_info!(step, "log message").await?;

        let actual = serde_json::from_str::<serde_json::Value>(
            buffer
                .lock()
                .await
                .first()
                .ok_or(anyhow!("Buffer is empty"))?,
        )?;
        assert_json_include!(actual: actual.clone(), expected: &expected);

        let source = actual
            .get("testStepArtifact")
            .ok_or(anyhow!("testStepArtifact key does not exist"))?
            .get("log")
            .ok_or(anyhow!("log key does not exist"))?;
        assert_ne!(
            source.get("sourceLocation"),
            None,
            "sourceLocation is not present in the serialized object"
        );

        Ok(())
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
            "sequenceNumber": 1
        });

        let dut = DutInfo::builder("dut_id").build();
        let buffer: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));
        let test_run = TestRun::builder("run_name", &dut, "1.0")
            .config(Config::builder().with_buffer_output(buffer.clone()).build())
            .build();

        let step = test_run.step("step_name")?;
        ocptv_log_warning!(step, "log message").await?;

        let actual = serde_json::from_str::<serde_json::Value>(
            buffer
                .lock()
                .await
                .first()
                .ok_or(anyhow!("Buffer is empty"))?,
        )?;
        assert_json_include!(actual: actual.clone(), expected: &expected);

        let source = actual
            .get("testStepArtifact")
            .ok_or(anyhow!("testStepArtifact key does not exist"))?
            .get("log")
            .ok_or(anyhow!("log key does not exist"))?;
        assert_ne!(
            source.get("sourceLocation"),
            None,
            "sourceLocation is not present in the serialized object"
        );

        Ok(())
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
            "sequenceNumber": 1
        });

        let dut = DutInfo::builder("dut_id").build();
        let buffer: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));
        let test_run = TestRun::builder("run_name", &dut, "1.0")
            .config(Config::builder().with_buffer_output(buffer.clone()).build())
            .build();

        let step = test_run.step("step_name")?;
        ocptv_log_error!(step, "log message").await?;

        let actual = serde_json::from_str::<serde_json::Value>(
            buffer
                .lock()
                .await
                .first()
                .ok_or(anyhow!("Buffer is empty"))?,
        )?;
        assert_json_include!(actual: actual.clone(), expected: &expected);

        let source = actual
            .get("testStepArtifact")
            .ok_or(anyhow!("testStepArtifact key does not exist"))?
            .get("log")
            .ok_or(anyhow!("log key does not exist"))?;
        assert_ne!(
            source.get("sourceLocation"),
            None,
            "sourceLocation is not present in the serialized object"
        );

        Ok(())
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
            "sequenceNumber": 1
        });

        let dut = DutInfo::builder("dut_id").build();
        let buffer: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));
        let test_run = TestRun::builder("run_name", &dut, "1.0")
            .config(Config::builder().with_buffer_output(buffer.clone()).build())
            .build();

        let step = test_run.step("step_name")?;
        ocptv_log_fatal!(step, "log message").await?;

        let actual = serde_json::from_str::<serde_json::Value>(
            buffer
                .lock()
                .await
                .first()
                .ok_or(anyhow!("Buffer is empty"))?,
        )?;
        assert_json_include!(actual: actual.clone(), expected: &expected);

        let source = actual
            .get("testStepArtifact")
            .ok_or(anyhow!("testStepArtifact key does not exist"))?
            .get("log")
            .ok_or(anyhow!("log key does not exist"))?;
        assert_ne!(
            source.get("sourceLocation"),
            None,
            "sourceLocation is not present in the serialized object"
        );

        Ok(())
    }
}
