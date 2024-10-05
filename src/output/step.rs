// (c) Meta Platforms, Inc. and affiliates.
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

use serde_json::Value;
use std::sync::atomic;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::output as tv;
use crate::spec;
use tv::measurement::MeasurementSeries;
use tv::{emitter, error, log, measurement, state, step};

/// A single test step in the scope of a [`TestRun`].
///
/// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#test-step-artifacts
pub struct TestStep {
    name: String,
    state: Arc<Mutex<state::TestState>>,
}

impl TestStep {
    pub(crate) fn new(name: &str, state: Arc<Mutex<state::TestState>>) -> TestStep {
        TestStep {
            name: name.to_string(),
            state,
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
        let start = step::TestStepStart::new(&self.name);
        self.state
            .lock()
            .await
            .emitter
            .emit(&start.to_artifact())
            .await?;

        Ok(StartedTestStep {
            step: self,
            measurement_id_no: Arc::new(atomic::AtomicU64::new(0)),
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
    measurement_id_no: Arc<atomic::AtomicU64>,
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
        let end = step::TestStepEnd::new(status);
        self.step
            .state
            .lock()
            .await
            .emitter
            .emit(&end.to_artifact())
            .await?;
        Ok(())
    }

    /// Eemits Log message.
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
        let emitter = &self.step.state.lock().await.emitter;

        let artifact = spec::TestStepArtifact {
            descendant: spec::TestStepArtifactDescendant::Log(log.to_artifact()),
        };
        emitter
            .emit(&spec::RootArtifact::TestStepArtifact(artifact))
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
        let emitter = &self.step.state.lock().await.emitter;

        let artifact = spec::TestStepArtifact {
            descendant: spec::TestStepArtifactDescendant::Log(log.to_artifact()),
        };
        emitter
            .emit(&spec::RootArtifact::TestStepArtifact(artifact))
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
        let emitter = &self.step.state.lock().await.emitter;

        let artifact = spec::TestStepArtifact {
            descendant: spec::TestStepArtifactDescendant::Error(error.to_artifact()),
        };
        emitter
            .emit(&spec::RootArtifact::TestStepArtifact(artifact))
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
        let emitter = &self.step.state.lock().await.emitter;

        let artifact = spec::TestStepArtifact {
            descendant: spec::TestStepArtifactDescendant::Error(error.to_artifact()),
        };
        emitter
            .emit(&spec::RootArtifact::TestStepArtifact(artifact))
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
        let emitter = &self.step.state.lock().await.emitter;

        let artifact = spec::TestStepArtifact {
            descendant: spec::TestStepArtifactDescendant::Error(error.to_artifact()),
        };
        emitter
            .emit(&spec::RootArtifact::TestStepArtifact(artifact))
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
        let measurement = measurement::Measurement::new(name, value);
        self.step
            .state
            .lock()
            .await
            .emitter
            .emit(&measurement.to_artifact())
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
        measurement: &measurement::Measurement,
    ) -> Result<(), emitter::WriterError> {
        self.step
            .state
            .lock()
            .await
            .emitter
            .emit(&measurement.to_artifact())
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
        self.measurement_id_no
            .fetch_add(1, atomic::Ordering::SeqCst);
        let series_id: String = format!(
            "series_{}",
            self.measurement_id_no.load(atomic::Ordering::SeqCst)
        );

        MeasurementSeries::new(&series_id, name, self.step.state.clone())
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
        start: measurement::MeasurementSeriesStart,
    ) -> MeasurementSeries {
        MeasurementSeries::new_with_details(start, self.step.state.clone())
    }
}

pub struct TestStepStart {
    name: String,
}

impl TestStepStart {
    pub fn new(name: &str) -> TestStepStart {
        TestStepStart {
            name: name.to_string(),
        }
    }

    pub fn to_artifact(&self) -> spec::RootArtifact {
        spec::RootArtifact::TestStepArtifact(spec::TestStepArtifact {
            descendant: spec::TestStepArtifactDescendant::TestStepStart(spec::TestStepStart {
                name: self.name.clone(),
            }),
        })
    }
}

pub struct TestStepEnd {
    status: spec::TestStatus,
}

impl TestStepEnd {
    pub fn new(status: spec::TestStatus) -> TestStepEnd {
        TestStepEnd { status }
    }

    pub fn to_artifact(&self) -> spec::RootArtifact {
        spec::RootArtifact::TestStepArtifact(spec::TestStepArtifact {
            descendant: spec::TestStepArtifactDescendant::TestStepEnd(spec::TestStepEnd {
                status: self.status.clone(),
            }),
        })
    }
}

#[cfg(test)]
mod tests {}
