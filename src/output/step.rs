// (c) Meta Platforms, Inc. and affiliates.
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

use serde_json::Value;
use std::sync::atomic::{self, Ordering};
use std::sync::Arc;

use crate::output as tv;
use crate::spec::TestStepStart;
use crate::spec::{self, TestStepArtifactImpl};
use tv::measure::MeasurementSeries;
use tv::{emitter, error, log, measure};

use super::JsonEmitter;
use super::WriterError;

/// A single test step in the scope of a [`TestRun`].
///
/// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#test-step-artifacts
pub struct TestStep {
    name: String,

    emitter: Arc<StepEmitter>,
}

impl TestStep {
    pub(crate) fn new(id: &str, name: &str, run_emitter: Arc<JsonEmitter>) -> Self {
        TestStep {
            name: name.to_owned(),
            emitter: Arc::new(StepEmitter {
                step_id: id.to_owned(),
                run_emitter,
            }),
        }
    }

    /// Starts the test step.
    ///
    /// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#teststepstart
    ///
    /// # Examples
    ///
    /// ```rust
    /// # tokio_test::block_on(async {
    /// # use ocptv::output::*;
    ///
    /// let run = TestRun::new("diagnostic_name", "my_dut", "1.0").start().await?;
    /// let step = run.step("step_name").start().await?;
    ///
    /// # Ok::<(), WriterError>(())
    /// # });
    /// ```
    pub async fn start(self) -> Result<StartedTestStep, emitter::WriterError> {
        self.emitter
            .emit(&TestStepArtifactImpl::TestStepStart(TestStepStart {
                name: self.name.clone(),
            }))
            .await?;

        Ok(StartedTestStep {
            step: self,
            measurement_id_seqno: Arc::new(atomic::AtomicU64::new(0)),
        })
    }

    // /// Builds a scope in the [`TestStep`] object, taking care of starting and
    // /// ending it. View [`TestStep::start`] and [`TestStep::end`] methods.
    // /// After the scope is constructed, additional objects may be added to it.
    // /// This is the preferred usage for the [`TestStep`], since it guarantees
    // /// all the messages are emitted between the start and end messages, the order
    // /// is respected and no messages is lost.
    // ///
    // /// # Examples
    // ///
    // /// ```rust
    // /// # tokio_test::block_on(async {
    // /// # use ocptv::output::*;
    // ///
    // /// let run = TestRun::new("diagnostic_name", "my_dut", "1.0").start().await?;
    // ///
    // /// let step = run.step("first step")?;
    // /// step.scope(|s| async {
    // ///     s.log(
    // ///         LogSeverity::Info,
    // ///         "This is a log message with INFO severity",
    // ///     ).await?;
    // ///     Ok(TestStatus::Complete)
    // /// }).await?;
    // ///
    // /// # Ok::<(), WriterError>(())
    // /// # });
    // /// ```
    // pub async fn scope<'a, F, R>(&'a self, func: F) -> Result<(), emitters::WriterError>
    // where
    //     R: Future<Output = Result<models::TestStatus, emitters::WriterError>>,
    //     F: std::ops::FnOnce(&'a TestStep) -> R,
    // {
    //     self.start().await?;
    //     let status = func(self).await?;
    //     self.end(status).await?;
    //     Ok(())
    // }
}

pub struct StartedTestStep {
    step: TestStep,
    measurement_id_seqno: Arc<atomic::AtomicU64>,
}

impl StartedTestStep {
    /// Ends the test step.
    ///
    /// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#teststepend
    ///
    /// # Examples
    ///
    /// ```rust
    /// # tokio_test::block_on(async {
    /// # use ocptv::output::*;
    ///
    /// let run = TestRun::new("diagnostic_name", "my_dut", "1.0").start().await?;
    ///
    /// let step = run.step("step_name").start().await?;
    /// step.end(TestStatus::Complete).await?;
    ///
    /// # Ok::<(), WriterError>(())
    /// # });
    /// ```
    pub async fn end(&self, status: spec::TestStatus) -> Result<(), emitter::WriterError> {
        let end = TestStepArtifactImpl::TestStepEnd(spec::TestStepEnd { status });

        self.step.emitter.emit(&end).await?;
        Ok(())
    }

    /// Emits Log message.
    /// This method accepts a [`models::LogSeverity`] to define the severity
    /// and a [`std::string::String`] for the message.
    ///
    /// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#log
    ///
    /// # Examples
    ///
    /// ```rust
    /// # tokio_test::block_on(async {
    /// # use ocptv::output::*;
    ///
    /// let run = TestRun::new("diagnostic_name", "my_dut", "1.0").start().await?;
    ///
    /// let step = run.step("step_name").start().await?;
    /// step.log(
    ///     LogSeverity::Info,
    ///     "This is a log message with INFO severity",
    /// ).await?;
    /// step.end(TestStatus::Complete).await?;
    ///
    /// # Ok::<(), WriterError>(())
    /// # });
    /// ```
    /// ## Using macros
    ///
    /// ```rust
    /// # tokio_test::block_on(async {
    /// # use ocptv::output::*;
    ///
    /// use ocptv::ocptv_log_info;
    ///
    /// let run = TestRun::new("diagnostic_name", "my_dut", "1.0").start().await?;
    ///
    /// let step = run.step("step_name").start().await?;
    /// ocptv_log_info!(step, "This is a log message with INFO severity").await?;
    /// step.end(TestStatus::Complete).await?;
    ///
    /// # Ok::<(), WriterError>(())
    /// # });
    /// ```
    pub async fn log(
        &self,
        severity: spec::LogSeverity,
        msg: &str,
    ) -> Result<(), emitter::WriterError> {
        let log = log::Log::builder(msg).severity(severity).build();

        self.step
            .emitter
            .emit(&TestStepArtifactImpl::Log(log.to_artifact()))
            .await?;

        Ok(())
    }

    /// Emits Log message.
    /// This method accepts a [`objects::Log`] object.
    ///
    /// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#log
    ///
    /// # Examples
    ///
    /// ```rust
    /// # tokio_test::block_on(async {
    /// # use ocptv::output::*;
    ///
    /// let run = TestRun::new("diagnostic_name", "my_dut", "1.0").start().await?;
    ///
    /// let step = run.step("step_name").start().await?;
    /// step.log_with_details(
    ///     &Log::builder("This is a log message with INFO severity")
    ///         .severity(LogSeverity::Info)
    ///         .source("file", 1)
    ///         .build(),
    /// ).await?;
    /// step.end(TestStatus::Complete).await?;
    ///
    /// # Ok::<(), WriterError>(())
    /// # });
    /// ```
    pub async fn log_with_details(&self, log: &log::Log) -> Result<(), emitter::WriterError> {
        self.step
            .emitter
            .emit(&TestStepArtifactImpl::Log(log.to_artifact()))
            .await?;

        Ok(())
    }

    /// Emits an Error symptom.
    /// This method accepts a [`std::string::String`] to define the symptom.
    ///
    /// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#error
    ///
    /// # Examples
    ///
    /// ```rust
    /// # tokio_test::block_on(async {
    /// # use ocptv::output::*;
    ///
    /// let run = TestRun::new("diagnostic_name", "my_dut", "1.0").start().await?;
    ///
    /// let step = run.step("step_name").start().await?;
    /// step.error("symptom").await?;
    /// step.end(TestStatus::Complete).await?;
    ///
    /// # Ok::<(), WriterError>(())
    /// # });
    /// ```
    ///
    /// ## Using macros
    ///
    /// ```rust
    /// # tokio_test::block_on(async {
    /// # use ocptv::output::*;
    ///
    /// use ocptv::ocptv_error;
    ///
    /// let run = TestRun::new("diagnostic_name", "my_dut", "1.0").start().await?;
    ///
    /// let step = run.step("step_name").start().await?;
    /// ocptv_error!(step, "symptom").await?;
    /// step.end(TestStatus::Complete).await?;
    ///
    /// # Ok::<(), WriterError>(())
    /// # });
    /// ```
    pub async fn error(&self, symptom: &str) -> Result<(), emitter::WriterError> {
        let error = error::Error::builder(symptom).build();

        self.step
            .emitter
            .emit(&TestStepArtifactImpl::Error(error.to_artifact()))
            .await?;

        Ok(())
    }

    /// Emits an Error message.
    /// This method accepts a [`std::string::String`] to define the symptom and
    /// another [`std::string::String`] as error message.
    ///
    /// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#error
    ///
    /// # Examples
    ///
    /// ```rust
    /// # tokio_test::block_on(async {
    /// # use ocptv::output::*;
    ///
    /// let run = TestRun::new("diagnostic_name", "my_dut", "1.0").start().await?;
    ///
    /// let step = run.step("step_name").start().await?;
    /// step.error_with_msg("symptom", "error message").await?;
    /// step.end(TestStatus::Complete).await?;
    ///
    /// # Ok::<(), WriterError>(())
    /// # });
    /// ```
    ///
    /// ## Using macros
    ///  
    /// ```rust
    /// # tokio_test::block_on(async {
    /// # use ocptv::output::*;
    ///
    /// use ocptv::ocptv_error;
    ///
    /// let run = TestRun::new("diagnostic_name", "my_dut", "1.0").start().await?;
    ///
    /// let step = run.step("step_name").start().await?;
    /// ocptv_error!(step, "symptom", "error message").await?;
    /// step.end(TestStatus::Complete).await?;
    ///
    /// # Ok::<(), WriterError>(())
    /// # });
    /// ```
    pub async fn error_with_msg(
        &self,
        symptom: &str,
        msg: &str,
    ) -> Result<(), emitter::WriterError> {
        let error = error::Error::builder(symptom).message(msg).build();

        self.step
            .emitter
            .emit(&TestStepArtifactImpl::Error(error.to_artifact()))
            .await?;

        Ok(())
    }

    /// Emits a Error message.
    /// This method accepts a [`objects::Error`] object.
    ///
    /// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#error
    ///
    /// # Examples
    ///
    /// ```rust
    /// # tokio_test::block_on(async {
    /// # use ocptv::output::*;
    ///
    /// let run = TestRun::new("diagnostic_name", "my_dut", "1.0").start().await?;
    ///
    /// let step = run.step("step_name").start().await?;
    /// step.error_with_details(
    ///     &Error::builder("symptom")
    ///         .message("Error message")
    ///         .source("file", 1)
    ///         .add_software_info(&SoftwareInfo::builder("id", "name").build())
    ///         .build(),
    /// ).await?;
    /// step.end(TestStatus::Complete).await?;
    ///
    /// # Ok::<(), WriterError>(())
    /// # });
    /// ```
    pub async fn error_with_details(
        &self,
        error: &error::Error,
    ) -> Result<(), emitter::WriterError> {
        self.step
            .emitter
            .emit(&TestStepArtifactImpl::Error(error.to_artifact()))
            .await?;

        Ok(())
    }

    /// Emits a Measurement message.
    ///
    /// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#measurement
    ///
    /// # Examples
    ///
    /// ```rust
    /// # tokio_test::block_on(async {
    /// # use ocptv::output::*;
    ///
    /// let run = TestRun::new("diagnostic_name", "my_dut", "1.0").start().await?;
    ///
    /// let step = run.step("step_name").start().await?;
    /// step.add_measurement("name", 50.into()).await?;
    /// step.end(TestStatus::Complete).await?;
    ///
    /// # Ok::<(), WriterError>(())
    /// # });
    /// ```
    pub async fn add_measurement(
        &self,
        name: &str,
        value: Value,
    ) -> Result<(), emitter::WriterError> {
        let measurement = measure::Measurement::new(name, value);

        self.step
            .emitter
            .emit(&TestStepArtifactImpl::Measurement(
                measurement.to_artifact(),
            ))
            .await?;

        Ok(())
    }

    /// Emits a Measurement message.
    /// This method accepts a [`objects::Error`] object.
    ///
    /// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#measurement
    ///
    /// # Examples
    ///
    /// ```rust
    /// # tokio_test::block_on(async {
    /// # use ocptv::output::*;
    ///
    /// let hwinfo = HardwareInfo::builder("id", "fan").build();
    /// let run = TestRun::new("diagnostic_name", "my_dut", "1.0").start().await?;
    /// let step = run.step("step_name").start().await?;
    ///
    /// let measurement = Measurement::builder("name", 5000.into())
    ///     .hardware_info(&hwinfo)
    ///     .add_validator(&Validator::builder(ValidatorType::Equal, 30.into()).build())
    ///     .add_metadata("key", "value".into())
    ///     .subcomponent(&Subcomponent::builder("name").build())
    ///     .build();
    /// step.add_measurement_with_details(&measurement).await?;
    /// step.end(TestStatus::Complete).await?;
    ///
    /// # Ok::<(), WriterError>(())
    /// # });
    /// ```
    pub async fn add_measurement_with_details(
        &self,
        measurement: &measure::Measurement,
    ) -> Result<(), emitter::WriterError> {
        self.step
            .emitter
            .emit(&spec::TestStepArtifactImpl::Measurement(
                measurement.to_artifact(),
            ))
            .await?;

        Ok(())
    }

    /// Starts a Measurement Series (a time-series list of measurements).
    /// This method accepts a [`std::string::String`] as series ID and
    /// a [`std::string::String`] as series name.
    ///
    /// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#measurementseriesstart
    ///
    /// # Examples
    ///
    /// ```rust
    /// # tokio_test::block_on(async {
    /// # use ocptv::output::*;
    ///
    /// let run = TestRun::new("diagnostic_name", "my_dut", "1.0").start().await?;
    /// let step = run.step("step_name").start().await?;
    /// let series = step.measurement_series("name");
    ///
    /// # Ok::<(), WriterError>(())
    /// # });
    /// ```
    pub fn measurement_series(&self, name: &str) -> MeasurementSeries {
        let series_id: String = format!(
            "series_{}",
            self.measurement_id_seqno.fetch_add(1, Ordering::AcqRel)
        );

        MeasurementSeries::new(&series_id, name, Arc::clone(&self.step.emitter))
    }

    /// Starts a Measurement Series (a time-series list of measurements).
    /// This method accepts a [`objects::MeasurementSeriesStart`] object.
    ///
    /// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#measurementseriesstart
    ///
    /// # Examples
    ///
    /// ```rust
    /// # tokio_test::block_on(async {
    /// # use ocptv::output::*;
    ///
    /// let run = TestRun::new("diagnostic_name", "my_dut", "1.0").start().await?;
    /// let step = run.step("step_name").start().await?;
    /// let series =
    ///     step.measurement_series_with_details(MeasurementSeriesStart::new("name", "series_id"));
    ///
    /// # Ok::<(), WriterError>(())
    /// # });
    /// ```
    pub fn measurement_series_with_details(
        &self,
        start: measure::MeasurementSeriesStart,
    ) -> MeasurementSeries {
        MeasurementSeries::new_with_details(start, Arc::clone(&self.step.emitter))
    }
}

pub struct StepEmitter {
    step_id: String,
    run_emitter: Arc<JsonEmitter>,
}

impl StepEmitter {
    pub async fn emit(&self, object: &spec::TestStepArtifactImpl) -> Result<(), WriterError> {
        let root = spec::RootImpl::TestStepArtifact(spec::TestStepArtifact {
            id: self.step_id.clone(),
            // TODO: can these copies be avoided?
            artifact: object.clone(),
        });
        self.run_emitter.emit(&root).await?;

        Ok(())
    }
}
