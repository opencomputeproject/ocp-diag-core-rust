// (c) Meta Platforms, Inc. and affiliates.
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

use std::io;
use std::sync::atomic::{self, Ordering};
use std::sync::Arc;

#[cfg(feature = "boxed-scopes")]
use futures::future::BoxFuture;

use crate::output as tv;
use crate::spec::{self, TestStepArtifactImpl};
use tv::OcptvError;
use tv::{config, diagnosis, emitter, error, file, log, measure, Ident};

/// A single test step in the scope of a [`TestRun`].
///
/// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#test-step-artifacts
pub struct TestStep {
    name: String,

    emitter: Arc<StepEmitter>,
}

impl TestStep {
    // note: this object is crate public but users should only construct
    // instances through the `StartedTestRun.add_step` api
    pub(crate) fn new(id: &str, name: &str, run_emitter: Arc<emitter::JsonEmitter>) -> Self {
        TestStep {
            name: name.to_owned(),
            emitter: Arc::new(StepEmitter {
                step_id: id.to_owned(),
                emitter: run_emitter,
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
    /// let dut = DutInfo::new("my_dut");
    /// let run = TestRun::new("diagnostic_name", "1.0").start(dut).await?;
    /// let step = run.add_step("step_name").start().await?;
    ///
    /// # Ok::<(), OcptvError>(())
    /// # });
    /// ```
    pub async fn start(self) -> Result<StartedTestStep, tv::OcptvError> {
        self.emitter
            .emit(&TestStepArtifactImpl::TestStepStart(spec::TestStepStart {
                name: self.name.clone(),
            }))
            .await?;

        Ok(StartedTestStep {
            step: self,
            measurement_seqno: Arc::new(atomic::AtomicU64::new(0)),
        })
    }

    /// Builds a scope in the [`TestStep`] object, taking care of starting and
    /// ending it. View [`TestStep::start`] and [`TestStep::end`] methods.
    /// After the scope is constructed, additional objects may be added to it.
    /// This is the preferred usage for the [`TestStep`], since it guarantees
    /// all the messages are emitted between the start and end messages, the order
    /// is respected and no messages is lost.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # tokio_test::block_on(async {
    /// # use futures::FutureExt;
    /// # use ocptv::output::*;
    ///
    /// let dut = DutInfo::new("my_dut");
    /// let run = TestRun::new("diagnostic_name", "1.0").start(dut).await?;
    ///
    /// let step = run.add_step("first step");
    /// step.scope(|s| {
    ///     async move {
    ///         s.add_log(
    ///             LogSeverity::Info,
    ///             "This is a log message with INFO severity",
    ///         ).await?;
    ///         Ok(TestStatus::Complete)
    ///     }.boxed()
    /// }).await?;
    ///
    /// # Ok::<(), OcptvError>(())
    /// # });
    /// ```
    #[cfg(feature = "boxed-scopes")]
    pub async fn scope<F>(self, func: F) -> Result<(), tv::OcptvError>
    where
        F: FnOnce(&StartedTestStep) -> BoxFuture<'_, Result<tv::TestStatus, tv::OcptvError>>,
    {
        let step = self.start().await?;
        let status = func(&step).await?;
        step.end(status).await?;

        Ok(())
    }
}

pub struct StartedTestStep {
    step: TestStep,
    measurement_seqno: Arc<atomic::AtomicU64>,
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
    /// let dut = DutInfo::new("my_dut");
    /// let run = TestRun::new("diagnostic_name", "1.0").start(dut).await?;
    ///
    /// let step = run.add_step("step_name").start().await?;
    /// step.end(TestStatus::Complete).await?;
    ///
    /// # Ok::<(), OcptvError>(())
    /// # });
    /// ```
    pub async fn end(self, status: spec::TestStatus) -> Result<(), tv::OcptvError> {
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
    /// let dut = DutInfo::new("my_dut");
    /// let run = TestRun::new("diagnostic_name", "1.0").start(dut).await?;
    ///
    /// let step = run.add_step("step_name").start().await?;
    /// step.add_log(
    ///     LogSeverity::Info,
    ///     "This is a log message with INFO severity",
    /// ).await?;
    /// step.end(TestStatus::Complete).await?;
    ///
    /// # Ok::<(), OcptvError>(())
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
    /// let dut = DutInfo::new("my_dut");
    /// let run = TestRun::new("diagnostic_name", "1.0").start(dut).await?;
    ///
    /// let step = run.add_step("step_name").start().await?;
    /// ocptv_log_info!(step, "This is a log message with INFO severity").await?;
    /// step.end(TestStatus::Complete).await?;
    ///
    /// # Ok::<(), OcptvError>(())
    /// # });
    /// ```
    pub async fn add_log(
        &self,
        severity: spec::LogSeverity,
        msg: &str,
    ) -> Result<(), tv::OcptvError> {
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
    /// let dut = DutInfo::new("my_dut");
    /// let run = TestRun::new("diagnostic_name", "1.0").start(dut).await?;
    ///
    /// let step = run.add_step("step_name").start().await?;
    /// step.add_log_with_details(
    ///     &Log::builder("This is a log message with INFO severity")
    ///         .severity(LogSeverity::Info)
    ///         .source("file", 1)
    ///         .build(),
    /// ).await?;
    /// step.end(TestStatus::Complete).await?;
    ///
    /// # Ok::<(), OcptvError>(())
    /// # });
    /// ```
    pub async fn add_log_with_details(&self, log: &log::Log) -> Result<(), tv::OcptvError> {
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
    /// let dut = DutInfo::new("my_dut");
    /// let run = TestRun::new("diagnostic_name", "1.0").start(dut).await?;
    ///
    /// let step = run.add_step("step_name").start().await?;
    /// step.add_error("symptom").await?;
    /// step.end(TestStatus::Complete).await?;
    ///
    /// # Ok::<(), OcptvError>(())
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
    /// let dut = DutInfo::new("my_dut");
    /// let run = TestRun::new("diagnostic_name", "1.0").start(dut).await?;
    ///
    /// let step = run.add_step("step_name").start().await?;
    /// ocptv_error!(step, "symptom").await?;
    /// step.end(TestStatus::Complete).await?;
    ///
    /// # Ok::<(), OcptvError>(())
    /// # });
    /// ```
    pub async fn add_error(&self, symptom: &str) -> Result<(), tv::OcptvError> {
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
    /// let dut = DutInfo::new("my_dut");
    /// let run = TestRun::new("diagnostic_name", "1.0").start(dut).await?;
    ///
    /// let step = run.add_step("step_name").start().await?;
    /// step.add_error_with_msg("symptom", "error message").await?;
    /// step.end(TestStatus::Complete).await?;
    ///
    /// # Ok::<(), OcptvError>(())
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
    /// let dut = DutInfo::new("my_dut");
    /// let run = TestRun::new("diagnostic_name", "1.0").start(dut).await?;
    ///
    /// let step = run.add_step("step_name").start().await?;
    /// ocptv_error!(step, "symptom", "error message").await?;
    /// step.end(TestStatus::Complete).await?;
    ///
    /// # Ok::<(), OcptvError>(())
    /// # });
    /// ```
    pub async fn add_error_with_msg(&self, symptom: &str, msg: &str) -> Result<(), tv::OcptvError> {
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
    /// let mut dut = DutInfo::new("my_dut");
    /// let sw_info = dut.add_software_info(SoftwareInfo::builder("name").build());
    /// let run = TestRun::builder("diagnostic_name", "1.0").build().start(dut).await?;
    ///
    /// let step = run.add_step("step_name").start().await?;
    /// step.add_error_with_details(
    ///     &Error::builder("symptom")
    ///         .message("Error message")
    ///         .source("file", 1)
    ///         .add_software_info(&sw_info)
    ///         .build(),
    /// ).await?;
    /// step.end(TestStatus::Complete).await?;
    ///
    /// # Ok::<(), OcptvError>(())
    /// # });
    /// ```
    pub async fn add_error_with_details(&self, error: &error::Error) -> Result<(), tv::OcptvError> {
        self.step
            .emitter
            .emit(&TestStepArtifactImpl::Error(error.to_artifact()))
            .await?;

        Ok(())
    }

    /// Emits an extension message;
    ///
    /// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#extension
    ///
    /// # Examples
    ///
    /// ```rust
    /// # tokio_test::block_on(async {
    /// # use ocptv::output::*;
    ///
    /// let dut = DutInfo::new("my_dut");
    /// let run = TestRun::new("diagnostic_name", "1.0").start(dut).await?;
    /// let step = run.add_step("step_name").start().await?;
    ///
    /// #[derive(serde::Serialize)]
    /// struct Ext { i: u32 }
    ///
    /// step.add_extension("ext_name", Ext { i: 42 }).await?;
    ///
    /// # Ok::<(), OcptvError>(())
    /// # });
    /// ```
    pub async fn add_extension<S: serde::Serialize>(
        &self,
        name: &str,
        any: S,
    ) -> Result<(), tv::OcptvError> {
        let ext = TestStepArtifactImpl::Extension(spec::Extension {
            name: name.to_owned(),
            content: serde_json::to_value(&any).map_err(|e| OcptvError::Format(Box::new(e)))?,
        });

        self.step.emitter.emit(&ext).await?;
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
    /// let dut = DutInfo::new("my_dut");
    /// let run = TestRun::new("diagnostic_name", "1.0").start(dut).await?;
    ///
    /// let step = run.add_step("step_name").start().await?;
    /// step.add_measurement("name", 50.into()).await?;
    /// step.end(TestStatus::Complete).await?;
    ///
    /// # Ok::<(), OcptvError>(())
    /// # });
    /// ```
    pub async fn add_measurement(
        &self,
        name: &str,
        value: tv::Value,
    ) -> Result<(), tv::OcptvError> {
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
    /// let mut dut = DutInfo::new("my_dut");
    /// let hw_info = dut.add_hardware_info(HardwareInfo::builder("fan").build());
    /// let run = TestRun::builder("diagnostic_name", "1.0").build().start(dut).await?;
    /// let step = run.add_step("step_name").start().await?;
    ///
    /// let measurement = Measurement::builder("name", 5000.into())
    ///     .add_validator(&Validator::builder(ValidatorType::Equal, 30.into()).build())
    ///     .add_metadata("key", "value".into())
    ///     .hardware_info(&hw_info)
    ///     .subcomponent(&Subcomponent::builder("name").build())
    ///     .build();
    /// step.add_measurement_with_details(&measurement).await?;
    ///
    /// step.end(TestStatus::Complete).await?;
    ///
    /// # Ok::<(), OcptvError>(())
    /// # });
    /// ```
    pub async fn add_measurement_with_details(
        &self,
        measurement: &measure::Measurement,
    ) -> Result<(), tv::OcptvError> {
        self.step
            .emitter
            .emit(&spec::TestStepArtifactImpl::Measurement(
                measurement.to_artifact(),
            ))
            .await?;

        Ok(())
    }

    /// Create a Measurement Series (a time-series list of measurements).
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
    /// let dut = DutInfo::new("my_dut");
    /// let run = TestRun::new("diagnostic_name", "1.0").start(dut).await?;
    /// let step = run.add_step("step_name").start().await?;
    /// let series = step.add_measurement_series("name");
    ///
    /// # Ok::<(), OcptvError>(())
    /// # });
    /// ```
    pub fn add_measurement_series(&self, name: &str) -> tv::MeasurementSeries {
        self.add_measurement_series_with_details(tv::MeasurementSeriesInfo::new(name))
    }

    /// Create a Measurement Series (a time-series list of measurements).
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
    /// let dut = DutInfo::new("my_dut");
    /// let run = TestRun::new("diagnostic_name", "1.0").start(dut).await?;
    /// let step = run.add_step("step_name").start().await?;
    /// let series =
    ///     step.add_measurement_series_with_details(MeasurementSeriesInfo::new("name"));
    ///
    /// # Ok::<(), OcptvError>(())
    /// # });
    /// ```
    pub fn add_measurement_series_with_details(
        &self,
        info: measure::MeasurementSeriesInfo,
    ) -> tv::MeasurementSeries {
        // spec says this identifier is unique in the scope of the test run, so create it from
        // the step identifier and a counter
        // ref: https://github.com/opencomputeproject/ocp-diag-core/blob/main/json_spec/README.md#measurementseriesstart
        let series_id = match &info.id {
            Ident::Auto => format!(
                "{}_series{}",
                self.step.emitter.step_id,
                self.measurement_seqno.fetch_add(1, Ordering::AcqRel)
            ),
            Ident::Exact(value) => value.to_owned(),
        };

        tv::MeasurementSeries::new(&series_id, info, Arc::clone(&self.step.emitter))
    }

    /// Emits a Diagnosis message.
    ///
    /// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#diagnosis
    ///
    /// # Examples
    ///
    /// ```rust
    /// # tokio_test::block_on(async {
    /// # use ocptv::output::*;
    ///
    /// let dut = DutInfo::new("my dut");
    /// let run = TestRun::new("diagnostic_name", "1.0").start(dut).await?;
    ///
    /// let step = run.add_step("step_name").start().await?;
    /// step.diagnosis("verdict", DiagnosisType::Pass).await?;
    /// step.end(TestStatus::Complete).await?;
    ///
    /// # Ok::<(), OcptvError>(())
    /// # });
    /// ```
    pub async fn diagnosis(
        &self,
        verdict: &str,
        diagnosis_type: spec::DiagnosisType,
    ) -> Result<(), tv::OcptvError> {
        let diagnosis = diagnosis::Diagnosis::new(verdict, diagnosis_type);

        self.step
            .emitter
            .emit(&TestStepArtifactImpl::Diagnosis(diagnosis.to_artifact()))
            .await?;

        Ok(())
    }

    /// Emits a Diagnosis message.
    /// This method accepts a [`objects::Error`] object.
    ///
    /// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#diagnosis
    ///
    /// # Examples
    ///
    /// ```rust
    /// # tokio_test::block_on(async {
    /// # use ocptv::output::*;
    ///
    /// let mut dut = DutInfo::new("my_dut");
    /// let hw_info = dut.add_hardware_info(HardwareInfo::builder("fan").build());
    /// let run = TestRun::new("diagnostic_name", "1.0").start(dut).await?;
    /// let step = run.add_step("step_name").start().await?;
    ///
    /// let diagnosis = Diagnosis::builder("verdict", DiagnosisType::Pass)
    ///     .hardware_info(&hw_info)
    ///     .message("message")
    ///     .subcomponent(&Subcomponent::builder("name").build())
    ///     .source("file.rs", 1)
    ///     .build();
    /// step.diagnosis_with_details(&diagnosis).await?;
    ///
    /// step.end(TestStatus::Complete).await?;
    ///
    /// # Ok::<(), OcptvError>(())
    /// # });
    /// ```
    pub async fn diagnosis_with_details(
        &self,
        diagnosis: &diagnosis::Diagnosis,
    ) -> Result<(), tv::OcptvError> {
        self.step
            .emitter
            .emit(&spec::TestStepArtifactImpl::Diagnosis(
                diagnosis.to_artifact(),
            ))
            .await?;

        Ok(())
    }

    /// Emits a File message.
    ///
    /// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#file
    ///
    /// # Examples
    ///
    /// ```rust
    /// # tokio_test::block_on(async {
    /// # use ocptv::output::*;
    ///
    /// let dut = DutInfo::new("my_dut");
    /// let run = TestRun::new("diagnostic_name", "1.0").start(dut).await?;
    ///
    /// let step = run.add_step("step_name").start().await?;
    /// let uri = Uri::parse("file:///tmp/foo").unwrap();
    /// step.file("name", uri).await?;
    /// step.end(TestStatus::Complete).await?;
    ///
    /// # Ok::<(), OcptvError>(())
    /// # });
    /// ```
    pub async fn file(&self, name: &str, uri: tv::Uri) -> Result<(), tv::OcptvError> {
        let file = file::File::new(name, uri);

        self.step
            .emitter
            .emit(&TestStepArtifactImpl::File(file.to_artifact()))
            .await?;

        Ok(())
    }

    /// Emits a File message.
    /// This method accepts a [`objects::Error`] object.
    ///
    /// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#file
    ///
    /// # Examples
    ///
    /// ```rust
    /// # tokio_test::block_on(async {
    /// # use ocptv::output::*;
    /// # use std::str::FromStr;
    ///
    /// let dut = DutInfo::new("my_dut");
    /// let run = TestRun::new("diagnostic_name", "1.0").start(dut).await?;
    /// let uri = Uri::parse("file:///tmp/foo").unwrap();
    ///
    /// let step = run.add_step("step_name").start().await?;
    ///
    /// let uri = Uri::parse("file:///tmp/foo").unwrap();
    /// let file = File::builder("name", uri)
    ///     .description("description")
    ///     .content_type(Mime::from_str("text/plain").unwrap())
    ///     .add_metadata("key", "value".into())
    ///     .build();
    /// step.file_with_details(&file).await?;
    /// step.end(TestStatus::Complete).await?;
    ///
    /// # Ok::<(), OcptvError>(())
    /// # });
    /// ```
    pub async fn file_with_details(&self, file: &file::File) -> Result<(), tv::OcptvError> {
        self.step
            .emitter
            .emit(&spec::TestStepArtifactImpl::File(file.to_artifact()))
            .await?;

        Ok(())
    }
}

pub struct StepEmitter {
    step_id: String,
    // root emitter
    emitter: Arc<emitter::JsonEmitter>,
}

impl StepEmitter {
    pub async fn emit(&self, object: &spec::TestStepArtifactImpl) -> Result<(), io::Error> {
        let root = spec::RootImpl::TestStepArtifact(spec::TestStepArtifact {
            id: self.step_id.clone(),
            // TODO: can these copies be avoided?
            artifact: object.clone(),
        });
        self.emitter.emit(&root).await?;

        Ok(())
    }

    pub fn timestamp_provider(&self) -> &(dyn config::TimestampProvider + Send + Sync + 'static) {
        self.emitter.timestamp_provider()
    }
}
