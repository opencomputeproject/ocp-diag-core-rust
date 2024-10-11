// (c) Meta Platforms, Inc. and affiliates.
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

//! OCPTV library macros
//!
//! This module contains a set of macros which are exported from the ocptv
//! library.

/// Emit an artifact of type Error.
///
/// ref: <https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#error>
///
/// Equivalent to the [`$crate::StartedTestRun::error_with_details`] method.
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

macro_rules! ocptv_log {
    ($name:ident, $severity:path) => {
        /// Emit an artifact of type Log.
        ///
        /// ref: <https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#log>
        ///
        /// Equivalent to the [`$crate::StartedTestRun::log_with_details`] method.
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
        #[macro_export]
        macro_rules! $name {
            ($artifact:expr, $msg:expr) => {
                $artifact.add_log_with_details(
                    &$crate::output::Log::builder($msg)
                        .severity($severity)
                        .source(file!(), line!() as i32)
                        .build(),
                )
            };
        }
    };
}

ocptv_log!(ocptv_log_debug, ocptv::output::LogSeverity::Debug);
ocptv_log!(ocptv_log_info, ocptv::output::LogSeverity::Info);
ocptv_log!(ocptv_log_warning, ocptv::output::LogSeverity::Warning);
ocptv_log!(ocptv_log_error, ocptv::output::LogSeverity::Error);
ocptv_log!(ocptv_log_fatal, ocptv::output::LogSeverity::Fatal);

macro_rules! ocptv_diagnosis {
    ($name:ident, $diagnosis_type:path) => {
        /// Emit an artifact of type Diagnosis.
        ///
        /// ref: <https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#diagnosis>
        ///
        /// Equivalent to the [`$crate::StartedTestStep::diagnosis_with_details`] method.
        ///
        /// They accept verdict as only parameter.
        /// Information about the source file and line number is automatically added.
        ///
        /// There is one macro for each DiagnosisType variant: Pass, Fail, Unknown.
        ///
        /// # Examples
        ///
        /// ## DEBUG
        ///
        /// ```rust
        /// # tokio_test::block_on(async {
        /// # use ocptv::output::*;
        /// use ocptv::ocptv_diagnosis_pass;
        ///
        /// let dut = DutInfo::new("my dut");
        /// let run = TestRun::new("diagnostic_name", "1.0").start(dut).await?;
        ///
        /// let step = run.add_step("step_name").start().await?;
        /// ocptv_diagnosis_pass!(step, "verdict");
        /// step.end(TestStatus::Complete).await?;
        ///
        /// run.end(TestStatus::Complete, TestResult::Pass).await?;
        ///
        /// # Ok::<(), OcptvError>(())
        /// # });
        /// ```
        #[macro_export]
        macro_rules! $name {
            ($artifact:expr, $verdict:expr) => {
                $artifact.diagnosis_with_details(
                    &$crate::output::Diagnosis::builder($verdict, $diagnosis_type)
                        .source(file!(), line!() as i32)
                        .build(),
                )
            };
        }
    };
}

ocptv_diagnosis!(ocptv_diagnosis_pass, ocptv::output::DiagnosisType::Pass);
ocptv_diagnosis!(ocptv_diagnosis_fail, ocptv::output::DiagnosisType::Fail);
ocptv_diagnosis!(
    ocptv_diagnosis_unknown,
    ocptv::output::DiagnosisType::Unknown
);
