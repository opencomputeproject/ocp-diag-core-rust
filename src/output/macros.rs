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
/// let dut = DutInfo::new("my_dut");
/// let test_run = TestRun::new("run_name", "1.0").start(dut).await?;
/// ocptv_error!(test_run, "symptom");
/// test_run.end(TestStatus::Complete, TestResult::Pass).await?;
///
/// # Ok::<(), OcptvError>(())
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
/// let dut = DutInfo::new("my_dut");
/// let test_run = TestRun::new("run_name", "1.0").start(dut).await?;
/// ocptv_error!(test_run, "symptom", "Error message");
/// test_run.end(TestStatus::Complete, TestResult::Pass).await?;
///
/// # Ok::<(), OcptvError>(())
/// # });
/// ```
#[macro_export]
macro_rules! ocptv_error {
    ($runner:expr, $symptom:expr, $msg:expr) => {
        $runner.add_error_with_details(
            &$crate::output::Error::builder($symptom)
                .message($msg)
                .source(file!(), line!() as i32)
                .build(),
        )
    };

    ($runner:expr, $symptom:expr) => {
        $runner.add_error_with_details(
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
/// let dut = DutInfo::new("my_dut");
/// let run = TestRun::new("run_name", "1.0").start(dut).await?;
/// ocptv_log_debug!(run, "Log message");
/// run.end(TestStatus::Complete, TestResult::Pass).await?;
///
/// # Ok::<(), OcptvError>(())
/// # });
/// ```

macro_rules! ocptv_log {
    ($name:ident, $severity:ident) => {
        #[macro_export]
        macro_rules! $name {
            ($artifact:expr, $msg:expr) => {
                $artifact.add_log_with_details(
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
