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
/// ```
/// use std::io::Write;
/// use std::io::{self};
///
/// use ocptv_formatter::ocptv_error;
/// use ocptv_formatter::Error;
/// use ocptv_formatter::TestResult;
/// use ocptv_formatter::TestRun;
/// use ocptv_formatter::TestStatus;
///
/// let test_run = TestRun::new("run_name", "my_dut", "1.0");
/// test_run.start().unwrap();
/// ocptv_error!(test_run, "symptom");
/// test_run
///     .end(TestStatus::Complete, TestResult::Pass)
///     .unwrap();
/// ```
///
/// ## Passing both symptom and message
///
/// ```
/// use std::io::Write;
/// use std::io::{self};
///
/// use ocptv_formatter::ocptv_error;
/// use ocptv_formatter::Error;
/// use ocptv_formatter::TestResult;
/// use ocptv_formatter::TestRun;
/// use ocptv_formatter::TestStatus;
///
/// let test_run = TestRun::new("run_name", "my_dut", "1.0");
/// test_run.start().unwrap();
/// ocptv_error!(test_run, "symptom", "Error message");
/// test_run
///     .end(TestStatus::Complete, TestResult::Pass)
///     .unwrap();
/// ```
#[macro_export]
macro_rules! ocptv_error {
    ($runner:expr , $symptom:expr, $msg:expr) => {
        async {
            $runner
                .error_with_details(
                    &$crate::Error::builder($symptom)
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
                    &$crate::Error::builder($symptom)
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
/// ```
/// use std::io::Write;
/// use std::io::{self};
///
/// use ocptv_formatter::ocptv_log_debug;
/// use ocptv_formatter::ocptv_log_error;
/// use ocptv_formatter::ocptv_log_fatal;
/// use ocptv_formatter::ocptv_log_info;
/// use ocptv_formatter::ocptv_log_warning;
/// use ocptv_formatter::LogSeverity;
/// use ocptv_formatter::TestResult;
/// use ocptv_formatter::TestRun;
/// use ocptv_formatter::TestStatus;
///
/// let test_run = TestRun::new("run_name", "my_dut", "1.0");
/// test_run.start().unwrap();
/// ocptv_log_debug!(test_run, "Log message");
/// test_run
///     .end(TestStatus::Complete, TestResult::Pass)
///     .unwrap();
/// ```

#[macro_export]
macro_rules! ocptv_log_debug {
    ($runner:expr, $msg:expr) => {
        async {
            $runner
                .log_with_details(
                    &$crate::Log::builder($msg)
                        .severity($crate::LogSeverity::Debug)
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
                    &$crate::Log::builder($msg)
                        .severity($crate::LogSeverity::Info)
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
                    &$crate::Log::builder($msg)
                        .severity($crate::LogSeverity::Warning)
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
                    &$crate::Log::builder($msg)
                        .severity($crate::LogSeverity::Error)
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
                    &$crate::Log::builder($msg)
                        .severity($crate::LogSeverity::Fatal)
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

    use anyhow::Result;
    use assert_json_diff::assert_json_include;
    use serde_json::json;
    use tokio::sync::Mutex;

    use crate::objects::*;
    use crate::runner::*;

    #[tokio::test]
    async fn test_ocptv_error_macro_with_symptom_and_message() -> Result<()> {
        let expected = json!({"sequenceNumber":1,"testRunArtifact":{"error":{"message":"Error message","softwareInfoIds":null,"symptom":"symptom"}}});

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
                .ok_or(anyhow::Error::msg("Buffer is empty"))?,
        )?;
        assert_json_include!(actual: actual.clone(), expected: &expected);
        let source = actual
            .get("testRunArtifact")
            .ok_or(anyhow::Error::msg("testRunArtifact key does not exist"))?
            .get("error")
            .ok_or(anyhow::Error::msg("error key does not exist"))?;
        assert_ne!(
            source.get("sourceLocation"),
            None,
            "sourceLocation is not present in the serialized object"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_ocptv_error_macro_with_symptom() -> Result<()> {
        let expected = json!({"sequenceNumber":1,"testRunArtifact":{"error":{"message":null,"softwareInfoIds":null,"symptom":"symptom"}}});

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
                .ok_or(anyhow::Error::msg("Buffer is empty"))?,
        )?;
        assert_json_include!(actual: actual.clone(), expected: &expected);
        let source = actual
            .get("testRunArtifact")
            .ok_or(anyhow::Error::msg("testRunArtifact key does not exist"))?
            .get("error")
            .ok_or(anyhow::Error::msg("error key does not exist"))?;
        assert_ne!(
            source.get("sourceLocation"),
            None,
            "sourceLocation is not present in the serialized object"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_ocptv_log_debug() -> Result<()> {
        let expected = json!({"sequenceNumber":1,"testRunArtifact":{"log":{"message":"log message","severity":"DEBUG"}}});

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
                .ok_or(anyhow::Error::msg("Buffer is empty"))?,
        )?;
        assert_json_include!(actual: actual.clone(), expected: &expected);
        let source = actual
            .get("testRunArtifact")
            .ok_or(anyhow::Error::msg("testRunArtifact key does not exist"))?
            .get("log")
            .ok_or(anyhow::Error::msg("log key does not exist"))?;
        assert_ne!(
            source.get("sourceLocation"),
            None,
            "sourceLocation is not present in the serialized object"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_ocptv_log_info() -> Result<()> {
        let expected = json!({"sequenceNumber":1,"testRunArtifact":{"log":{"message":"log message","severity":"INFO"}}});

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
                .ok_or(anyhow::Error::msg("Buffer is empty"))?,
        )?;
        assert_json_include!(actual: actual.clone(), expected: &expected);
        let source = actual
            .get("testRunArtifact")
            .ok_or(anyhow::Error::msg("testRunArtifact key does not exist"))?
            .get("log")
            .ok_or(anyhow::Error::msg("log key does not exist"))?;
        assert_ne!(
            source.get("sourceLocation"),
            None,
            "sourceLocation is not present in the serialized object"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_ocptv_log_warning() -> Result<()> {
        let expected = json!({"sequenceNumber":1,"testRunArtifact":{"log":{"message":"log message","severity":"WARNING"}}});

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
                .ok_or(anyhow::Error::msg("Buffer is empty"))?,
        )?;
        assert_json_include!(actual: actual.clone(), expected: &expected);
        let source = actual
            .get("testRunArtifact")
            .ok_or(anyhow::Error::msg("testRunArtifact key does not exist"))?
            .get("log")
            .ok_or(anyhow::Error::msg("log key does not exist"))?;
        assert_ne!(
            source.get("sourceLocation"),
            None,
            "sourceLocation is not present in the serialized object"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_ocptv_log_error() -> Result<()> {
        let expected = json!({"sequenceNumber":1,"testRunArtifact":{"log":{"message":"log message","severity":"ERROR"}}});

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
                .ok_or(anyhow::Error::msg("Buffer is empty"))?,
        )?;
        assert_json_include!(actual: actual.clone(), expected: &expected);
        let source = actual
            .get("testRunArtifact")
            .ok_or(anyhow::Error::msg("testRunArtifact key does not exist"))?
            .get("log")
            .ok_or(anyhow::Error::msg("log key does not exist"))?;
        assert_ne!(
            source.get("sourceLocation"),
            None,
            "sourceLocation is not present in the serialized object"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_ocptv_log_fatal() -> Result<()> {
        let expected = json!({"sequenceNumber":1,"testRunArtifact":{"log":{"message":"log message","severity":"FATAL"}}});

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
                .ok_or(anyhow::Error::msg("Buffer is empty"))?,
        )?;
        assert_json_include!(actual: actual.clone(), expected: &expected);
        let source = actual
            .get("testRunArtifact")
            .ok_or(anyhow::Error::msg("testRunArtifact key does not exist"))?
            .get("log")
            .ok_or(anyhow::Error::msg("log key does not exist"))?;
        assert_ne!(
            source.get("sourceLocation"),
            None,
            "sourceLocation is not present in the serialized object"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_ocptv_error_macro_with_symptom_and_message_in_step() -> Result<()> {
        let expected = json!({"sequenceNumber":1,"testStepArtifact":{"error":{"message":"Error message","softwareInfoIds":null,"symptom":"symptom"}}});

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
                .ok_or(anyhow::Error::msg("Buffer is empty"))?,
        )?;
        assert_json_include!(actual: actual.clone(), expected: &expected);
        let source = actual
            .get("testStepArtifact")
            .ok_or(anyhow::Error::msg("testStepArtifact key does not exist"))?
            .get("error")
            .ok_or(anyhow::Error::msg("error key does not exist"))?;
        assert_ne!(
            source.get("sourceLocation"),
            None,
            "sourceLocation is not present in the serialized object"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_ocptv_error_macro_with_symptom_in_step() -> Result<()> {
        let expected = json!({"sequenceNumber":1,"testStepArtifact":{"error":{"message":null,"softwareInfoIds":null,"symptom":"symptom"}}});

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
                .ok_or(anyhow::Error::msg("Buffer is empty"))?,
        )?;
        assert_json_include!(actual: actual.clone(), expected: &expected);
        let source = actual
            .get("testStepArtifact")
            .ok_or(anyhow::Error::msg("testStepArtifact key does not exist"))?
            .get("error")
            .ok_or(anyhow::Error::msg("error key does not exist"))?;
        assert_ne!(
            source.get("sourceLocation"),
            None,
            "sourceLocation is not present in the serialized object"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_ocptv_log_debug_in_step() -> Result<()> {
        let expected = json!({"sequenceNumber":1,"testStepArtifact":{"log":{"message":"log message","severity":"DEBUG"}}});

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
                .ok_or(anyhow::Error::msg("Buffer is empty"))?,
        )?;
        assert_json_include!(actual: actual.clone(), expected: &expected);
        let source = actual
            .get("testStepArtifact")
            .ok_or(anyhow::Error::msg("testStepArtifact key does not exist"))?
            .get("log")
            .ok_or(anyhow::Error::msg("log key does not exist"))?;
        assert_ne!(
            source.get("sourceLocation"),
            None,
            "sourceLocation is not present in the serialized object"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_ocptv_log_info_in_step() -> Result<()> {
        let expected = json!({"sequenceNumber":1,"testStepArtifact":{"log":{"message":"log message","severity":"INFO"}}});

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
                .ok_or(anyhow::Error::msg("Buffer is empty"))?,
        )?;
        assert_json_include!(actual: actual.clone(), expected: &expected);
        let source = actual
            .get("testStepArtifact")
            .ok_or(anyhow::Error::msg("testStepArtifact key does not exist"))?
            .get("log")
            .ok_or(anyhow::Error::msg("log key does not exist"))?;
        assert_ne!(
            source.get("sourceLocation"),
            None,
            "sourceLocation is not present in the serialized object"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_ocptv_log_warning_in_step() -> Result<()> {
        let expected = json!({"sequenceNumber":1,"testStepArtifact":{"log":{"message":"log message","severity":"WARNING"}}});

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
                .ok_or(anyhow::Error::msg("Buffer is empty"))?,
        )?;
        assert_json_include!(actual: actual.clone(), expected: &expected);
        let source = actual
            .get("testStepArtifact")
            .ok_or(anyhow::Error::msg("testStepArtifact key does not exist"))?
            .get("log")
            .ok_or(anyhow::Error::msg("log key does not exist"))?;
        assert_ne!(
            source.get("sourceLocation"),
            None,
            "sourceLocation is not present in the serialized object"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_ocptv_log_error_in_step() -> Result<()> {
        let expected = json!({"sequenceNumber":1,"testStepArtifact":{"log":{"message":"log message","severity":"ERROR"}}});

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
                .ok_or(anyhow::Error::msg("Buffer is empty"))?,
        )?;
        assert_json_include!(actual: actual.clone(), expected: &expected);
        let source = actual
            .get("testStepArtifact")
            .ok_or(anyhow::Error::msg("testStepArtifact key does not exist"))?
            .get("log")
            .ok_or(anyhow::Error::msg("log key does not exist"))?;
        assert_ne!(
            source.get("sourceLocation"),
            None,
            "sourceLocation is not present in the serialized object"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_ocptv_log_fatal_in_step() -> Result<()> {
        let expected = json!({"sequenceNumber":1,"testStepArtifact":{"log":{"message":"log message","severity":"FATAL"}}});

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
                .ok_or(anyhow::Error::msg("Buffer is empty"))?,
        )?;
        assert_json_include!(actual: actual.clone(), expected: &expected);
        let source = actual
            .get("testStepArtifact")
            .ok_or(anyhow::Error::msg("testStepArtifact key does not exist"))?
            .get("log")
            .ok_or(anyhow::Error::msg("log key does not exist"))?;
        assert_ne!(
            source.get("sourceLocation"),
            None,
            "sourceLocation is not present in the serialized object"
        );
        Ok(())
    }
}
