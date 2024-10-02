// (c) Meta Platforms, Inc. and affiliates.
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

//! OCPTV library runner
//!
//! This module contains the main entry point for the test runner. This is the
//! main object the user will interact with.

use std::env;
use std::future::Future;
use std::path::Path;
use std::sync::atomic;
use std::sync::Arc;

use serde_json::Map;
use serde_json::Value;
use tokio::sync::Mutex;

use crate::output::emitters;
use crate::output::models;
use crate::output::objects;

/// The configuration repository for the TestRun.
pub struct Config {
    timezone: chrono_tz::Tz,
    writer: emitters::WriterType,
}

impl Config {
    /// Creates a new [`ConfigBuilder`]
    ///
    /// # Examples
    /// ```rust
    /// # use ocptv::output::*;
    ///
    /// let builder = Config::builder();
    /// ```
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::new()
    }
}

/// The builder for the [`Config`] object.
pub struct ConfigBuilder {
    timezone: Option<chrono_tz::Tz>,
    writer: Option<emitters::WriterType>,
}

impl ConfigBuilder {
    fn new() -> Self {
        Self {
            timezone: None,
            writer: Some(emitters::WriterType::Stdout(emitters::StdoutWriter::new())),
        }
    }

    pub fn timezone(mut self, timezone: chrono_tz::Tz) -> Self {
        self.timezone = Some(timezone);
        self
    }

    pub fn with_buffer_output(mut self, buffer: Arc<Mutex<Vec<String>>>) -> Self {
        self.writer = Some(emitters::WriterType::Buffer(emitters::BufferWriter::new(
            buffer,
        )));
        self
    }

    pub async fn with_file_output<P: AsRef<Path>>(
        mut self,
        path: P,
    ) -> Result<Self, emitters::WriterError> {
        self.writer = Some(emitters::WriterType::File(
            emitters::FileWriter::new(path).await?,
        ));
        Ok(self)
    }

    pub fn build(self) -> Config {
        Config {
            timezone: self.timezone.unwrap_or(chrono_tz::UTC),
            writer: self
                .writer
                .unwrap_or(emitters::WriterType::Stdout(emitters::StdoutWriter::new())),
        }
    }
}

/// The outcome of a TestRun.
/// It's returned when the scope method of the [`TestRun`] object is used.
pub struct TestRunOutcome {
    /// Reports the execution status of the test
    pub status: models::TestStatus,
    /// Reports the result of the test
    pub result: models::TestResult,
}

struct TestState {
    emitter: emitters::JsonEmitter,
}

impl TestState {
    fn new(emitter: emitters::JsonEmitter) -> TestState {
        TestState { emitter }
    }
}

/// The main diag test run.
/// This object describes a single run instance of the diag, and therefore drives the test session.
///
/// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#testrunstart

pub struct TestRun {
    name: String,
    version: String,
    parameters: Map<String, Value>,
    dut: objects::DutInfo,
    command_line: String,
    metadata: Option<Map<String, Value>>,
    state: Arc<Mutex<TestState>>,
}

impl TestRun {
    /// Creates a new [`TestRunBuilder`] object.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ocptv::output::*;
    ///
    /// let dut = DutInfo::builder("my_dut").build();
    /// let builder = TestRun::builder("run_name", &dut, "1.0");
    /// ```
    pub fn builder(name: &str, dut: &objects::DutInfo, version: &str) -> TestRunBuilder {
        TestRunBuilder::new(name, dut, version)
    }

    /// Creates a new [`TestRun`] object.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ocptv::output::*;
    ///
    /// let test_run = TestRun::new("diagnostic_name", "my_dut", "1.0");
    /// ```
    pub fn new(name: &str, dut_id: &str, version: &str) -> TestRun {
        let dut = objects::DutInfo::new(dut_id);
        TestRunBuilder::new(name, &dut, version).build()
    }

    /// Starts the test run.
    ///
    /// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#schemaversion
    /// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#testrunstart
    ///
    /// # Examples
    ///
    /// ```rust
    /// # tokio_test::block_on(async {
    /// # use ocptv::output::*;
    ///
    /// let test_run = TestRun::new("diagnostic_name", "my_dut", "1.0");
    /// test_run.start().await?;
    ///
    /// # Ok::<(), WriterError>(())
    /// # });
    /// ```
    pub async fn start(&self) -> Result<(), emitters::WriterError> {
        let version = objects::SchemaVersion::new();
        self.state
            .lock()
            .await
            .emitter
            .emit(&version.to_artifact())
            .await?;

        let mut builder = objects::TestRunStart::builder(
            &self.name,
            &self.version,
            &self.command_line,
            &self.parameters,
            &self.dut,
        );

        if let Some(m) = &self.metadata {
            for m in m {
                builder = builder.add_metadata(m.0, m.1.clone())
            }
        }

        let start = builder.build();
        self.state
            .lock()
            .await
            .emitter
            .emit(&start.to_artifact())
            .await?;
        Ok(())
    }

    /// Ends the test run.
    ///
    /// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#testrunend
    ///
    /// # Examples
    ///
    /// ```rust
    /// # tokio_test::block_on(async {
    /// # use ocptv::output::*;
    ///
    /// let test_run = TestRun::new("diagnostic_name", "my_dut", "1.0");
    /// test_run.start().await?;
    /// test_run.end(TestStatus::Complete, TestResult::Pass).await?;
    ///
    /// # Ok::<(), WriterError>(())
    /// # });
    /// ```
    pub async fn end(
        &self,
        status: models::TestStatus,
        result: models::TestResult,
    ) -> Result<(), emitters::WriterError> {
        let end = objects::TestRunEnd::builder()
            .status(status)
            .result(result)
            .build();
        self.state
            .lock()
            .await
            .emitter
            .emit(&end.to_artifact())
            .await?;
        Ok(())
    }

    /// Builds a scope in the [`TestRun`] object, taking care of starting and
    /// ending it. View [`TestRun::start`] and [`TestRun::end`] methods.
    /// After the scope is constructed, additional objects may be added to it.
    /// This is the preferred usage for the [`TestRun`], since it guarantees
    /// all the messages are emitted between the start and end messages, the order
    /// is respected and no messages is lost.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # tokio_test::block_on(async {
    /// # use ocptv::output::*;
    ///
    /// let test_run = TestRun::new("diagnostic_name", "my_dut", "1.0");
    /// test_run.scope(|r| async {
    ///     r.log(LogSeverity::Info, "First message").await?;
    ///     Ok(TestRunOutcome {
    ///         status: TestStatus::Complete,
    ///         result: TestResult::Pass,
    ///     })
    /// }).await?;
    ///
    /// # Ok::<(), WriterError>(())
    /// # });
    /// ```
    pub async fn scope<'a, F, R>(&'a self, func: F) -> Result<(), emitters::WriterError>
    where
        R: Future<Output = Result<TestRunOutcome, emitters::WriterError>>,
        F: std::ops::FnOnce(&'a TestRun) -> R,
    {
        self.start().await?;
        let outcome = func(self).await?;
        self.end(outcome.status, outcome.result).await?;
        Ok(())
    }

    /// Emits a Log message.
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
    /// let test_run = TestRun::new("diagnostic_name", "my_dut", "1.0");
    /// test_run.start().await?;
    /// test_run.log(
    ///     LogSeverity::Info,
    ///     "This is a log message with INFO severity",
    /// ).await?;
    /// test_run.end(TestStatus::Complete, TestResult::Pass).await?;
    ///
    /// # Ok::<(), WriterError>(())
    /// # });
    /// ```
    pub async fn log(
        &self,
        severity: models::LogSeverity,
        msg: &str,
    ) -> Result<(), emitters::WriterError> {
        let log = objects::Log::builder(msg).severity(severity).build();
        self.state
            .lock()
            .await
            .emitter
            .emit(&log.to_artifact(objects::ArtifactContext::TestRun))
            .await?;
        Ok(())
    }

    /// Emits a Log message.
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
    /// let test_run = TestRun::new("diagnostic_name", "my_dut", "1.0");
    /// test_run.start().await?;
    /// test_run.log_with_details(
    ///     &Log::builder("This is a log message with INFO severity")
    ///         .severity(LogSeverity::Info)
    ///         .source("file", 1)
    ///         .build(),
    /// ).await?;
    /// test_run.end(TestStatus::Complete, TestResult::Pass).await?;
    ///
    /// # Ok::<(), WriterError>(())
    /// # });
    /// ```
    pub async fn log_with_details(&self, log: &objects::Log) -> Result<(), emitters::WriterError> {
        self.state
            .lock()
            .await
            .emitter
            .emit(&log.to_artifact(objects::ArtifactContext::TestRun))
            .await?;
        Ok(())
    }

    /// Emits a Error message.
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
    /// let test_run = TestRun::new("diagnostic_name", "my_dut", "1.0");
    /// test_run.start().await?;
    /// test_run.error("symptom").await?;
    /// test_run.end(TestStatus::Complete, TestResult::Pass).await?;
    ///
    /// # Ok::<(), WriterError>(())
    /// # });
    /// ```
    pub async fn error(&self, symptom: &str) -> Result<(), emitters::WriterError> {
        let error = objects::Error::builder(symptom).build();
        self.state
            .lock()
            .await
            .emitter
            .emit(&error.to_artifact(objects::ArtifactContext::TestRun))
            .await?;
        Ok(())
    }

    /// Emits a Error message.
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
    /// let test_run = TestRun::new("diagnostic_name", "my_dut", "1.0");
    /// test_run.start().await?;
    /// test_run.error_with_msg("symptom", "error messasge").await?;
    /// test_run.end(TestStatus::Complete, TestResult::Pass).await?;
    ///
    /// # Ok::<(), WriterError>(())
    /// # });
    /// ```
    pub async fn error_with_msg(
        &self,
        symptom: &str,
        msg: &str,
    ) -> Result<(), emitters::WriterError> {
        let error = objects::Error::builder(symptom).message(msg).build();
        self.state
            .lock()
            .await
            .emitter
            .emit(&error.to_artifact(objects::ArtifactContext::TestRun))
            .await?;
        Ok(())
    }

    /// Emits a Error message.
    /// This method acceps a [`objects::Error`] object.
    ///
    /// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#error
    ///
    /// # Examples
    ///
    /// ```rust
    /// # tokio_test::block_on(async {
    /// # use ocptv::output::*;
    ///
    /// let test_run = TestRun::new("diagnostic_name", "my_dut", "1.0");
    /// test_run.start().await?;
    /// test_run.error_with_details(
    ///     &Error::builder("symptom")
    ///         .message("Error message")
    ///         .source("file", 1)
    ///         .add_software_info(&SoftwareInfo::builder("id", "name").build())
    ///         .build(),
    /// ).await?;
    /// test_run.end(TestStatus::Complete, TestResult::Pass).await?;
    ///
    /// # Ok::<(), WriterError>(())
    /// # });
    /// ```
    pub async fn error_with_details(
        &self,
        error: &objects::Error,
    ) -> Result<(), emitters::WriterError> {
        self.state
            .lock()
            .await
            .emitter
            .emit(&error.to_artifact(objects::ArtifactContext::TestRun))
            .await?;
        Ok(())
    }

    pub fn step(&self, name: &str) -> Result<TestStep, emitters::WriterError> {
        Ok(TestStep::new(name, self.state.clone()))
    }
}

/// Builder for the [`TestRun`] object.
pub struct TestRunBuilder {
    name: String,
    dut: objects::DutInfo,
    version: String,
    parameters: Map<String, Value>,
    command_line: String,
    metadata: Option<Map<String, Value>>,
    config: Option<Config>,
}

impl TestRunBuilder {
    pub fn new(name: &str, dut: &objects::DutInfo, version: &str) -> Self {
        Self {
            name: name.to_string(),
            dut: dut.clone(),
            version: version.to_string(),
            parameters: Map::new(),
            command_line: env::args().collect::<Vec<_>>()[1..].join(" "),
            metadata: None,
            config: None,
        }
    }

    /// Adds a user defined parameter to the future [`TestRun`] object.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ocptv::output::*;
    ///
    /// let dut = DutInfo::builder("dut_id").build();
    /// let test_run = TestRunBuilder::new("run_name", &dut, "1.0")
    ///     .add_parameter("param1", "value1".into())
    ///     .build();
    /// ```
    pub fn add_parameter(mut self, key: &str, value: Value) -> TestRunBuilder {
        self.parameters.insert(key.to_string(), value.clone());
        self
    }

    /// Adds the command line used to run the test session  to the future
    /// [`TestRun`] object.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ocptv::output::*;
    ///
    /// let dut = DutInfo::builder("dut_id").build();
    /// let test_run = TestRunBuilder::new("run_name", &dut, "1.0")
    ///     .command_line("my_diag --arg value")
    ///     .build();
    /// ```
    pub fn command_line(mut self, cmd: &str) -> TestRunBuilder {
        self.command_line = cmd.to_string();
        self
    }

    /// Adds the configuration for the test session to the future [`TestRun`] object
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ocptv::output::{Config, TestRunBuilder, DutInfo};
    ///
    /// let dut = DutInfo::builder("dut_id").build();
    /// let test_run = TestRunBuilder::new("run_name", &dut, "1.0")
    ///     .config(Config::builder().build())
    ///     .build();
    /// ```
    pub fn config(mut self, value: Config) -> TestRunBuilder {
        self.config = Some(value);
        self
    }

    /// Adds user defined metadata to the future [`TestRun`] object
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ocptv::output::*;
    ///
    /// let dut = DutInfo::builder("dut_id").build();
    /// let test_run = TestRunBuilder::new("run_name", &dut, "1.0")
    ///     .add_metadata("meta1", "value1".into())
    ///     .build();
    /// ```
    pub fn add_metadata(mut self, key: &str, value: Value) -> TestRunBuilder {
        self.metadata = match self.metadata {
            Some(mut metadata) => {
                metadata.insert(key.to_string(), value.clone());
                Some(metadata)
            }
            None => {
                let mut metadata = Map::new();
                metadata.insert(key.to_string(), value.clone());
                Some(metadata)
            }
        };
        self
    }

    pub fn build(self) -> TestRun {
        let config = self.config.unwrap_or(Config::builder().build());
        let emitter = emitters::JsonEmitter::new(config.timezone, config.writer);
        let state = TestState::new(emitter);
        TestRun {
            name: self.name,
            dut: self.dut,
            version: self.version,
            parameters: self.parameters,
            command_line: self.command_line,
            metadata: self.metadata,
            state: Arc::new(Mutex::new(state)),
        }
    }
}

/// A single test step in the scope of a [`TestRun`].
///
/// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#test-step-artifacts
pub struct TestStep {
    name: String,
    state: Arc<Mutex<TestState>>,
    measurement_id_no: Arc<atomic::AtomicU64>,
}

impl TestStep {
    fn new(name: &str, state: Arc<Mutex<TestState>>) -> TestStep {
        TestStep {
            name: name.to_string(),
            state,
            measurement_id_no: Arc::new(atomic::AtomicU64::new(0)),
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
    /// let test_run = TestRun::new("diagnostic_name", "my_dut", "1.0");
    /// test_run.start().await?;
    ///
    /// let step = test_run.step("step_name")?;
    /// step.start().await?;
    ///
    /// # Ok::<(), WriterError>(())
    /// # });
    /// ```
    pub async fn start(&self) -> Result<(), emitters::WriterError> {
        let start = objects::TestStepStart::new(&self.name);
        self.state
            .lock()
            .await
            .emitter
            .emit(&start.to_artifact())
            .await?;
        Ok(())
    }

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
    /// let test_run = TestRun::new("diagnostic_name", "my_dut", "1.0");
    /// test_run.start().await?;
    ///
    /// let step = test_run.step("step_name")?;
    /// step.start().await?;
    /// step.end(TestStatus::Complete).await?;
    ///
    /// # Ok::<(), WriterError>(())
    /// # });
    /// ```
    pub async fn end(&self, status: models::TestStatus) -> Result<(), emitters::WriterError> {
        let end = objects::TestStepEnd::new(status);
        self.state
            .lock()
            .await
            .emitter
            .emit(&end.to_artifact())
            .await?;
        Ok(())
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
    /// # use ocptv::output::*;
    ///
    /// let test_run = TestRun::new("diagnostic_name", "my_dut", "1.0");
    /// test_run.start().await?;
    ///
    /// let step = test_run.step("first step")?;
    /// step.scope(|s| async {
    ///     s.log(
    ///         LogSeverity::Info,
    ///         "This is a log message with INFO severity",
    ///     ).await?;
    ///     Ok(TestStatus::Complete)
    /// }).await?;
    ///
    /// # Ok::<(), WriterError>(())
    /// # });
    /// ```
    pub async fn scope<'a, F, R>(&'a self, func: F) -> Result<(), emitters::WriterError>
    where
        R: Future<Output = Result<models::TestStatus, emitters::WriterError>>,
        F: std::ops::FnOnce(&'a TestStep) -> R,
    {
        self.start().await?;
        let status = func(self).await?;
        self.end(status).await?;
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
    /// let test_run = TestRun::new("diagnostic_name", "my_dut", "1.0");
    /// test_run.start().await?;
    ///
    /// let step = test_run.step("step_name")?;
    /// step.start().await?;
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
    /// let test_run = TestRun::new("diagnostic_name", "my_dut", "1.0");
    /// test_run.start().await?;
    ///
    /// let step = test_run.step("step_name")?;
    /// step.start().await?;
    /// ocptv_log_info!(step, "This is a log message with INFO severity").await?;
    /// step.end(TestStatus::Complete).await?;
    ///
    /// # Ok::<(), WriterError>(())
    /// # });
    /// ```
    pub async fn log(
        &self,
        severity: models::LogSeverity,
        msg: &str,
    ) -> Result<(), emitters::WriterError> {
        let log = objects::Log::builder(msg).severity(severity).build();
        self.state
            .lock()
            .await
            .emitter
            .emit(&log.to_artifact(objects::ArtifactContext::TestStep))
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
    /// let test_run = TestRun::new("diagnostic_name", "my_dut", "1.0");
    /// test_run.start().await?;
    ///
    /// let step = test_run.step("step_name")?;
    /// step.start().await?;
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
    pub async fn log_with_details(&self, log: &objects::Log) -> Result<(), emitters::WriterError> {
        self.state
            .lock()
            .await
            .emitter
            .emit(&log.to_artifact(objects::ArtifactContext::TestStep))
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
    /// let test_run = TestRun::new("diagnostic_name", "my_dut", "1.0");
    /// let step = test_run.step("step_name")?;
    /// step.start().await?;
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
    /// let test_run = TestRun::new("diagnostic_name", "my_dut", "1.0");
    /// test_run.start().await?;
    ///
    /// let step = test_run.step("step_name")?;
    /// step.start().await?;
    /// ocptv_error!(step, "symptom").await?;
    /// step.end(TestStatus::Complete).await?;
    ///
    /// # Ok::<(), WriterError>(())
    /// # });
    /// ```
    pub async fn error(&self, symptom: &str) -> Result<(), emitters::WriterError> {
        let error = objects::Error::builder(symptom).build();
        self.state
            .lock()
            .await
            .emitter
            .emit(&error.to_artifact(objects::ArtifactContext::TestStep))
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
    /// let test_run = TestRun::new("diagnostic_name", "my_dut", "1.0");
    /// let step = test_run.step("step_name")?;
    /// step.start().await?;
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
    /// let test_run = TestRun::new("diagnostic_name", "my_dut", "1.0");
    /// test_run.start().await?;
    /// let step = test_run.step("step_name")?;
    /// step.start().await?;
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
    ) -> Result<(), emitters::WriterError> {
        let error = objects::Error::builder(symptom).message(msg).build();
        self.state
            .lock()
            .await
            .emitter
            .emit(&error.to_artifact(objects::ArtifactContext::TestStep))
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
    /// let test_run = TestRun::new("diagnostic_name", "my_dut", "1.0");
    /// let step = test_run.step("step_name")?;
    /// step.start().await?;
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
        error: &objects::Error,
    ) -> Result<(), emitters::WriterError> {
        self.state
            .lock()
            .await
            .emitter
            .emit(&error.to_artifact(objects::ArtifactContext::TestStep))
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
    /// let test_run = TestRun::new("diagnostic_name", "my_dut", "1.0");
    /// let step = test_run.step("step_name")?;
    /// step.start().await?;
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
    ) -> Result<(), emitters::WriterError> {
        let measurement = objects::Measurement::new(name, value);
        self.state
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
    /// let test_run = TestRun::new("diagnostic_name", "my_dut", "1.0");
    /// let step = test_run.step("step_name")?;
    /// step.start().await?;
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
        measurement: &objects::Measurement,
    ) -> Result<(), emitters::WriterError> {
        self.state
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
    /// let test_run = TestRun::new("diagnostic_name", "my_dut", "1.0");
    /// let step = test_run.step("step_name")?;
    /// step.start().await?;
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

        MeasurementSeries::new(&series_id, name, self.state.clone())
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
    /// let test_run = TestRun::new("diagnostic_name", "my_dut", "1.0");
    /// let step = test_run.step("step_name")?;
    /// step.start().await?;
    /// let series =
    ///     step.measurement_series_with_details(MeasurementSeriesStart::new("name", "series_id"));
    ///
    /// # Ok::<(), WriterError>(())
    /// # });
    /// ```
    pub fn measurement_series_with_details(
        &self,
        start: objects::MeasurementSeriesStart,
    ) -> MeasurementSeries {
        MeasurementSeries::new_with_details(start, self.state.clone())
    }
}

/// The measurement series.
/// A Measurement Series is a time-series list of measurements.
///
/// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#measurementseriesstart
pub struct MeasurementSeries {
    state: Arc<Mutex<TestState>>,
    seq_no: Arc<Mutex<atomic::AtomicU64>>,
    start: objects::MeasurementSeriesStart,
}

impl MeasurementSeries {
    fn new(series_id: &str, name: &str, state: Arc<Mutex<TestState>>) -> Self {
        Self {
            state,
            seq_no: Arc::new(Mutex::new(atomic::AtomicU64::new(0))),
            start: objects::MeasurementSeriesStart::new(name, series_id),
        }
    }

    fn new_with_details(
        start: objects::MeasurementSeriesStart,
        state: Arc<Mutex<TestState>>,
    ) -> Self {
        Self {
            state,
            seq_no: Arc::new(Mutex::new(atomic::AtomicU64::new(0))),
            start,
        }
    }

    async fn current_sequence_no(&self) -> u64 {
        self.seq_no.lock().await.load(atomic::Ordering::SeqCst)
    }

    async fn increment_sequence_no(&self) {
        self.seq_no
            .lock()
            .await
            .fetch_add(1, atomic::Ordering::SeqCst);
    }

    /// Starts the measurement series.
    ///
    /// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#measurementseriesstart
    ///
    /// # Examples
    ///
    /// ```rust
    /// # tokio_test::block_on(async {
    /// # use ocptv::output::*;
    ///
    /// let test_run = TestRun::new("diagnostic_name", "my_dut", "1.0");
    /// test_run.start().await?;
    ///
    /// let step = test_run.step("step_name")?;
    /// step.start().await?;
    ///
    /// let series = step.measurement_series("name");
    /// series.start().await?;
    ///
    /// # Ok::<(), WriterError>(())
    /// # });
    /// ```
    pub async fn start(&self) -> Result<(), emitters::WriterError> {
        self.state
            .lock()
            .await
            .emitter
            .emit(&self.start.to_artifact())
            .await?;
        Ok(())
    }

    /// Ends the measurement series.
    ///
    /// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#measurementseriesend
    ///
    /// # Examples
    ///
    /// ```rust
    /// # tokio_test::block_on(async {
    /// # use ocptv::output::*;
    ///
    /// let test_run = TestRun::new("diagnostic_name", "my_dut", "1.0");
    /// test_run.start().await?;
    ///
    /// let step = test_run.step("step_name")?;
    /// step.start().await?;
    ///
    /// let series = step.measurement_series("name");
    /// series.start().await?;
    /// series.end().await?;
    ///
    /// # Ok::<(), WriterError>(())
    /// # });
    /// ```
    pub async fn end(&self) -> Result<(), emitters::WriterError> {
        let end = objects::MeasurementSeriesEnd::new(
            self.start.get_series_id(),
            self.current_sequence_no().await,
        );
        self.state
            .lock()
            .await
            .emitter
            .emit(&end.to_artifact())
            .await?;
        Ok(())
    }

    /// Adds a measurement element to the measurement series.
    ///
    /// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#measurementserieselement
    ///
    /// # Examples
    ///
    /// ```rust
    /// # tokio_test::block_on(async {
    /// # use ocptv::output::*;
    ///
    /// let test_run = TestRun::new("diagnostic_name", "my_dut", "1.0");
    /// test_run.start().await?;
    ///
    /// let step = test_run.step("step_name")?;
    /// step.start().await?;
    ///
    /// let series = step.measurement_series("name");
    /// series.start().await?;
    /// series.add_measurement(60.into()).await?;
    ///
    /// # Ok::<(), WriterError>(())
    /// # });
    /// ```
    pub async fn add_measurement(&self, value: Value) -> Result<(), emitters::WriterError> {
        let element = objects::MeasurementSeriesElement::new(
            self.current_sequence_no().await,
            value,
            &self.start,
            None,
        );
        self.increment_sequence_no().await;
        self.state
            .lock()
            .await
            .emitter
            .emit(&element.to_artifact())
            .await?;
        Ok(())
    }

    /// Adds a measurement element to the measurement series.
    /// This method accepts additional metadata to add to the element.
    ///
    /// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#measurementserieselement
    ///
    /// # Examples
    ///
    /// ```rust
    /// # tokio_test::block_on(async {
    /// # use ocptv::output::*;
    ///
    /// let test_run = TestRun::new("diagnostic_name", "my_dut", "1.0");
    /// test_run.start().await?;
    ///
    /// let step = test_run.step("step_name")?;
    /// step.start().await?;
    ///
    /// let series = step.measurement_series("name");
    /// series.start().await?;
    /// series.add_measurement_with_metadata(60.into(), vec![("key", "value".into())]).await?;
    ///
    /// # Ok::<(), WriterError>(())
    /// # });
    /// ```
    pub async fn add_measurement_with_metadata(
        &self,
        value: Value,
        metadata: Vec<(&str, Value)>,
    ) -> Result<(), emitters::WriterError> {
        let element = objects::MeasurementSeriesElement::new(
            self.current_sequence_no().await,
            value,
            &self.start,
            Some(Map::from_iter(
                metadata.iter().map(|(k, v)| (k.to_string(), v.clone())),
            )),
        );
        self.increment_sequence_no().await;
        self.state
            .lock()
            .await
            .emitter
            .emit(&element.to_artifact())
            .await?;
        Ok(())
    }

    /// Builds a scope in the [`MeasurementSeries`] object, taking care of starting and
    /// ending it. View [`MeasurementSeries::start`] and [`MeasurementSeries::end`] methods.
    /// After the scope is constructed, additional objects may be added to it.
    /// This is the preferred usage for the [`MeasurementSeries`], since it guarantees
    /// all the messages are emitted between the start and end messages, the order
    /// is respected and no messages is lost.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # tokio_test::block_on(async {
    /// # use ocptv::output::*;
    ///
    /// let test_run = TestRun::new("diagnostic_name", "my_dut", "1.0");
    /// test_run.start().await?;
    ///
    /// let step = test_run.step("step_name")?;
    /// step.start().await?;
    ///
    /// let series = step.measurement_series("name");
    /// series.start().await?;
    /// series.scope(|s| async {
    ///     s.add_measurement(60.into()).await?;
    ///     s.add_measurement(70.into()).await?;
    ///     s.add_measurement(80.into()).await?;
    ///     Ok(())
    /// }).await?;
    ///
    /// # Ok::<(), WriterError>(())
    /// # });
    /// ```
    pub async fn scope<'a, F, R>(&'a self, func: F) -> Result<(), emitters::WriterError>
    where
        R: Future<Output = Result<(), emitters::WriterError>>,
        F: std::ops::FnOnce(&'a MeasurementSeries) -> R,
    {
        self.start().await?;
        func(self).await?;
        self.end().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use anyhow::Result;
    use assert_json_diff::assert_json_include;
    use serde_json::json;
    use tokio::sync::Mutex;

    use super::*;
    use crate::output::models::*;
    use crate::output::objects::*;

    #[tokio::test]
    async fn test_testrun_start_and_end() -> Result<()> {
        let expected = [
            json!({"schemaVersion":{"major":2,"minor":0},"sequenceNumber":1}),
            json!({"sequenceNumber":2,"testRunArtifact":{"testRunStart":{"dutInfo":{"dutInfoId":"dut_id","hardwareInfos":null,"metadata":null,"name":null,"platformInfos":null,"softwareInfos":null},"metadata":null,"name":"run_name","parameters":{},"version":"1.0"}}}),
            json!({"sequenceNumber":3,"testRunArtifact":{"testRunEnd":{"result":"PASS","status":"COMPLETE"}}}),
        ];
        let buffer: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));

        let dut = DutInfo::builder("dut_id").build();

        let test_run = TestRun::builder("run_name", &dut, "1.0")
            .config(
                Config::builder()
                    .with_buffer_output(Arc::clone(&buffer))
                    .build(),
            )
            .build();
        test_run.start().await?;
        test_run.end(TestStatus::Complete, TestResult::Pass).await?;

        for (idx, entry) in buffer.lock().await.iter().enumerate() {
            let value = serde_json::from_str::<serde_json::Value>(entry)?;
            assert_json_include!(actual: value, expected: &expected[idx]);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_testrun_with_log() -> Result<()> {
        let expected = [
            json!({"schemaVersion":{"major":2,"minor":0},"sequenceNumber":1}),
            json!({"sequenceNumber":2,"testRunArtifact":{"testRunStart":{"dutInfo":{"dutInfoId":"dut_id","hardwareInfos":null,"metadata":null,"name":null,"platformInfos":null,"softwareInfos":null},"metadata":null,"name":"run_name","parameters":{},"version":"1.0"}}}),
            json!({"sequenceNumber":3,"testRunArtifact":{"log":{"message":"This is a log message with INFO severity","severity":"INFO","sourceLocation":null}}}),
            json!({"sequenceNumber":4,"testRunArtifact":{"testRunEnd":{"result":"PASS","status":"COMPLETE"}}}),
        ];
        let buffer: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));

        let dut = DutInfo::builder("dut_id").build();

        let test_run = TestRun::builder("run_name", &dut, "1.0")
            .config(
                Config::builder()
                    .with_buffer_output(Arc::clone(&buffer))
                    .build(),
            )
            .build();
        test_run.start().await?;

        test_run
            .log(
                LogSeverity::Info,
                "This is a log message with INFO severity",
            )
            .await?;

        test_run.end(TestStatus::Complete, TestResult::Pass).await?;

        for (idx, entry) in buffer.lock().await.iter().enumerate() {
            let value = serde_json::from_str::<serde_json::Value>(entry)?;
            assert_json_include!(actual: value, expected: &expected[idx]);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_testrun_with_log_with_details() -> Result<()> {
        let expected = [
            json!({"schemaVersion":{"major":2,"minor":0},"sequenceNumber":1}),
            json!({"sequenceNumber":2,"testRunArtifact":{"testRunStart":{"dutInfo":{"dutInfoId":"dut_id","hardwareInfos":null,"metadata":null,"name":null,"platformInfos":null,"softwareInfos":null},"metadata":null,"name":"run_name","parameters":{},"version":"1.0"}}}),
            json!({"sequenceNumber":3,"testRunArtifact":{"log":{"message":"This is a log message with INFO severity","severity":"INFO","sourceLocation":{
              "file": "file",
              "line": 1
            }}}}),
            json!({"sequenceNumber":4,"testRunArtifact":{"testRunEnd":{"result":"PASS","status":"COMPLETE"}}}),
        ];
        let buffer: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));

        let dut = DutInfo::builder("dut_id").build();

        let test_run = TestRun::builder("run_name", &dut, "1.0")
            .config(
                Config::builder()
                    .with_buffer_output(Arc::clone(&buffer))
                    .build(),
            )
            .build();
        test_run.start().await?;

        test_run
            .log_with_details(
                &Log::builder("This is a log message with INFO severity")
                    .severity(LogSeverity::Info)
                    .source("file", 1)
                    .build(),
            )
            .await?;

        test_run.end(TestStatus::Complete, TestResult::Pass).await?;

        for (idx, entry) in buffer.lock().await.iter().enumerate() {
            let value = serde_json::from_str::<serde_json::Value>(entry)?;
            assert_json_include!(actual: value, expected: &expected[idx]);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_testrun_with_error() -> Result<()> {
        let expected = [
            json!({"schemaVersion":{"major":2,"minor":0},"sequenceNumber":1}),
            json!({"sequenceNumber":2,"testRunArtifact":{"testRunStart":{"dutInfo":{"dutInfoId":"dut_id","hardwareInfos":null,"metadata":null,"name":null,"platformInfos":null,"softwareInfos":null},"metadata":null,"name":"run_name","parameters":{},"version":"1.0"}}}),
            json!({"sequenceNumber":3,"testRunArtifact":{"error":{"message":null,"softwareInfoIds":null,"sourceLocation":null,"symptom":"symptom"}}}),
            json!({"sequenceNumber":4,"testRunArtifact":{"testRunEnd":{"result":"PASS","status":"COMPLETE"}}}),
        ];
        let buffer: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));

        let dut = DutInfo::builder("dut_id").build();

        let test_run = TestRun::builder("run_name", &dut, "1.0")
            .config(
                Config::builder()
                    .with_buffer_output(Arc::clone(&buffer))
                    .build(),
            )
            .build();
        test_run.start().await?;

        test_run.error("symptom").await?;

        test_run.end(TestStatus::Complete, TestResult::Pass).await?;

        for (idx, entry) in buffer.lock().await.iter().enumerate() {
            let value = serde_json::from_str::<serde_json::Value>(entry)?;
            assert_json_include!(actual: value, expected: &expected[idx]);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_testrun_with_error_with_message() -> Result<()> {
        let expected = [
            json!({"schemaVersion":{"major":2,"minor":0},"sequenceNumber":1}),
            json!({"sequenceNumber":2,"testRunArtifact":{"testRunStart":{"dutInfo":{"dutInfoId":"dut_id","hardwareInfos":null,"metadata":null,"name":null,"platformInfos":null,"softwareInfos":null},"metadata":null,"name":"run_name","parameters":{},"version":"1.0"}}}),
            json!({"sequenceNumber":3,"testRunArtifact":{"error":{"message":"Error message","softwareInfoIds":null,"sourceLocation":null,"symptom":"symptom"}}}),
            json!({"sequenceNumber":4,"testRunArtifact":{"testRunEnd":{"result":"PASS","status":"COMPLETE"}}}),
        ];
        let buffer: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));

        let dut = DutInfo::builder("dut_id").build();

        let test_run = TestRun::builder("run_name", &dut, "1.0")
            .config(
                Config::builder()
                    .with_buffer_output(Arc::clone(&buffer))
                    .build(),
            )
            .build();
        test_run.start().await?;

        test_run.error_with_msg("symptom", "Error message").await?;

        test_run.end(TestStatus::Complete, TestResult::Pass).await?;

        for (idx, entry) in buffer.lock().await.iter().enumerate() {
            let value = serde_json::from_str::<serde_json::Value>(entry)?;
            assert_json_include!(actual: value, expected: &expected[idx]);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_testrun_with_error_with_details() -> Result<()> {
        let expected = [
            json!({"schemaVersion":{"major":2,"minor":0},"sequenceNumber":1}),
            json!({"sequenceNumber":2,"testRunArtifact":{"testRunStart":{"dutInfo":{"dutInfoId":"dut_id","hardwareInfos":null,"metadata":null,"name":null,"platformInfos":null,"softwareInfos":null},"metadata":null,"name":"run_name","parameters":{},"version":"1.0"}}}),
            json!({"sequenceNumber":3,"testRunArtifact":{"error":{"message":"Error message","softwareInfoIds":[
                {
                  "computerSystem": null,
                  "name": "name",
                  "revision": null,
                  "softwareInfoId": "id",
                  "softwareType": null,
                  "version": null
                }
              ],"sourceLocation":{
                "file": "file",
                "line": 1
              },"symptom":"symptom"}}}),
            json!({"sequenceNumber":4,"testRunArtifact":{"testRunEnd":{"result":"PASS","status":"COMPLETE"}}}),
        ];
        let buffer: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));

        let dut = DutInfo::builder("dut_id").build();

        let test_run = TestRun::builder("run_name", &dut, "1.0")
            .config(
                Config::builder()
                    .with_buffer_output(Arc::clone(&buffer))
                    .build(),
            )
            .build();
        test_run.start().await?;

        test_run
            .error_with_details(
                &Error::builder("symptom")
                    .message("Error message")
                    .source("file", 1)
                    .add_software_info(&SoftwareInfo::builder("id", "name").build())
                    .build(),
            )
            .await?;

        test_run.end(TestStatus::Complete, TestResult::Pass).await?;

        for (idx, entry) in buffer.lock().await.iter().enumerate() {
            let value = serde_json::from_str::<serde_json::Value>(entry)?;
            assert_json_include!(actual: value, expected: &expected[idx]);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_testrun_with_scope() -> Result<()> {
        let expected = [
            json!({"schemaVersion":{"major":2,"minor":0},"sequenceNumber":1}),
            json!({"sequenceNumber":2,"testRunArtifact":{"testRunStart":{"dutInfo":{"dutInfoId":"dut_id","hardwareInfos":null,"metadata":null,"name":null,"platformInfos":null,"softwareInfos":null},"metadata":null,"name":"run_name","parameters":{},"version":"1.0"}}}),
            json!({"sequenceNumber":3,"testRunArtifact":{"log":{"message":"First message","severity":"INFO","sourceLocation":null}}}),
            json!({"sequenceNumber":4,"testRunArtifact":{"testRunEnd":{"result":"PASS","status":"COMPLETE"}}}),
        ];
        let buffer: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));

        let dut = DutInfo::builder("dut_id").build();
        let test_run = TestRun::builder("run_name", &dut, "1.0")
            .config(
                Config::builder()
                    .with_buffer_output(Arc::clone(&buffer))
                    .build(),
            )
            .build();

        test_run
            .scope(|r| async {
                r.log(LogSeverity::Info, "First message").await?;
                Ok(TestRunOutcome {
                    status: TestStatus::Complete,
                    result: TestResult::Pass,
                })
            })
            .await?;

        for (idx, entry) in buffer.lock().await.iter().enumerate() {
            let value = serde_json::from_str::<serde_json::Value>(entry)?;
            assert_json_include!(actual: value, expected: &expected[idx]);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_testrun_with_step() -> Result<()> {
        let expected = [
            json!({"schemaVersion":{"major":2,"minor":0},"sequenceNumber":1}),
            json!({"sequenceNumber":2,"testRunArtifact":{"testRunStart":{"dutInfo":{"dutInfoId":"dut_id","hardwareInfos":null,"metadata":null,"name":null,"platformInfos":null,"softwareInfos":null},"metadata":null,"name":"run_name","parameters":{},"version":"1.0"}}}),
            json!({"sequenceNumber":3,"testStepArtifact":{"testStepStart":{"name":"first step"}}}),
            json!({"sequenceNumber":4,"testStepArtifact":{"testStepEnd":{"status":"COMPLETE"}}}),
            json!({"sequenceNumber":5,"testRunArtifact":{"testRunEnd":{"result":"PASS","status":"COMPLETE"}}}),
        ];
        let buffer: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));

        let dut = DutInfo::builder("dut_id").build();

        let test_run = TestRun::builder("run_name", &dut, "1.0")
            .config(
                Config::builder()
                    .with_buffer_output(Arc::clone(&buffer))
                    .build(),
            )
            .build();
        test_run.start().await?;

        let step = test_run.step("first step")?;
        step.start().await?;
        step.end(TestStatus::Complete).await?;

        test_run.end(TestStatus::Complete, TestResult::Pass).await?;

        for (idx, entry) in buffer.lock().await.iter().enumerate() {
            let value = serde_json::from_str::<serde_json::Value>(entry)?;
            assert_json_include!(actual: value, expected: &expected[idx]);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_testrun_step_log() -> Result<()> {
        let expected = [
            json!({"schemaVersion":{"major":2,"minor":0},"sequenceNumber":1}),
            json!({"sequenceNumber":2,"testRunArtifact":{"testRunStart":{"dutInfo":{"dutInfoId":"dut_id","hardwareInfos":null,"metadata":null,"name":null,"platformInfos":null,"softwareInfos":null},"metadata":null,"name":"run_name","parameters":{},"version":"1.0"}}}),
            json!({"sequenceNumber":3,"testStepArtifact":{"testStepStart":{"name":"first step"}}}),
            json!({"sequenceNumber":4,"testStepArtifact":{"log":{"message":"This is a log message with INFO severity","severity":"INFO","sourceLocation":null}}}),
            json!({"sequenceNumber":5,"testStepArtifact":{"testStepEnd":{"status":"COMPLETE"}}}),
            json!({"sequenceNumber":6,"testRunArtifact":{"testRunEnd":{"result":"PASS","status":"COMPLETE"}}}),
        ];
        let buffer: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));

        let dut = DutInfo::builder("dut_id").build();

        let test_run = TestRun::builder("run_name", &dut, "1.0")
            .config(
                Config::builder()
                    .with_buffer_output(Arc::clone(&buffer))
                    .build(),
            )
            .build();
        test_run.start().await?;

        let step = test_run.step("first step")?;
        step.start().await?;

        step.log(
            LogSeverity::Info,
            "This is a log message with INFO severity",
        )
        .await?;

        step.end(TestStatus::Complete).await?;

        test_run.end(TestStatus::Complete, TestResult::Pass).await?;

        for (idx, entry) in buffer.lock().await.iter().enumerate() {
            let value = serde_json::from_str::<serde_json::Value>(entry)?;
            assert_json_include!(actual: value, expected: &expected[idx]);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_testrun_step_log_with_details() -> Result<()> {
        let expected = [
            json!({"schemaVersion":{"major":2,"minor":0},"sequenceNumber":1}),
            json!({"sequenceNumber":2,"testRunArtifact":{"testRunStart":{"dutInfo":{"dutInfoId":"dut_id","hardwareInfos":null,"metadata":null,"name":null,"platformInfos":null,"softwareInfos":null},"metadata":null,"name":"run_name","parameters":{},"version":"1.0"}}}),
            json!({"sequenceNumber":3,"testStepArtifact":{"testStepStart":{"name":"first step"}}}),
            json!({"sequenceNumber":4,"testStepArtifact":{"log":{"message":"This is a log message with INFO severity","severity":"INFO","sourceLocation":{"file": "file", "line": 1}}}}),
            json!({"sequenceNumber":5,"testStepArtifact":{"testStepEnd":{"status":"COMPLETE"}}}),
            json!({"sequenceNumber":6,"testRunArtifact":{"testRunEnd":{"result":"PASS","status":"COMPLETE"}}}),
        ];
        let buffer: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));

        let dut = DutInfo::builder("dut_id").build();

        let test_run = TestRun::builder("run_name", &dut, "1.0")
            .config(
                Config::builder()
                    .with_buffer_output(Arc::clone(&buffer))
                    .build(),
            )
            .build();
        test_run.start().await?;

        let step = test_run.step("first step")?;
        step.start().await?;

        step.log_with_details(
            &Log::builder("This is a log message with INFO severity")
                .severity(LogSeverity::Info)
                .source("file", 1)
                .build(),
        )
        .await?;

        step.end(TestStatus::Complete).await?;

        test_run.end(TestStatus::Complete, TestResult::Pass).await?;

        for (idx, entry) in buffer.lock().await.iter().enumerate() {
            let value = serde_json::from_str::<serde_json::Value>(entry)?;
            assert_json_include!(actual: value, expected: &expected[idx]);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_testrun_step_error() -> Result<()> {
        let expected = [
            json!({"schemaVersion":{"major":2,"minor":0},"sequenceNumber":1}),
            json!({"sequenceNumber":2,"testRunArtifact":{"testRunStart":{"dutInfo":{"dutInfoId":"dut_id","hardwareInfos":null,"metadata":null,"name":null,"platformInfos":null,"softwareInfos":null},"metadata":null,"name":"run_name","parameters":{},"version":"1.0"}}}),
            json!({"sequenceNumber":3,"testStepArtifact":{"testStepStart":{"name":"first step"}}}),
            json!({"sequenceNumber":4,"testStepArtifact":{"error":{"message":null,"softwareInfoIds":null,"sourceLocation":null,"symptom":"symptom"}}}),
            json!({"sequenceNumber":5,"testStepArtifact":{"testStepEnd":{"status":"COMPLETE"}}}),
            json!({"sequenceNumber":6,"testRunArtifact":{"testRunEnd":{"result":"PASS","status":"COMPLETE"}}}),
        ];
        let buffer: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));

        let dut = DutInfo::builder("dut_id").build();

        let test_run = TestRun::builder("run_name", &dut, "1.0")
            .config(
                Config::builder()
                    .with_buffer_output(Arc::clone(&buffer))
                    .build(),
            )
            .build();
        test_run.start().await?;

        let step = test_run.step("first step")?;
        step.start().await?;

        step.error("symptom").await?;

        step.end(TestStatus::Complete).await?;

        test_run.end(TestStatus::Complete, TestResult::Pass).await?;

        for (idx, entry) in buffer.lock().await.iter().enumerate() {
            let value = serde_json::from_str::<serde_json::Value>(entry)?;
            assert_json_include!(actual: value, expected: &expected[idx]);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_testrun_step_error_with_message() -> Result<()> {
        let expected = [
            json!({"schemaVersion":{"major":2,"minor":0},"sequenceNumber":1}),
            json!({"sequenceNumber":2,"testRunArtifact":{"testRunStart":{"dutInfo":{"dutInfoId":"dut_id","hardwareInfos":null,"metadata":null,"name":null,"platformInfos":null,"softwareInfos":null},"metadata":null,"name":"run_name","parameters":{},"version":"1.0"}}}),
            json!({"sequenceNumber":3,"testStepArtifact":{"testStepStart":{"name":"first step"}}}),
            json!({"sequenceNumber":4,"testStepArtifact":{"error":{"message":"Error message","softwareInfoIds":null,"sourceLocation":null,"symptom":"symptom"}}}),
            json!({"sequenceNumber":5,"testStepArtifact":{"testStepEnd":{"status":"COMPLETE"}}}),
            json!({"sequenceNumber":6,"testRunArtifact":{"testRunEnd":{"result":"PASS","status":"COMPLETE"}}}),
        ];
        let buffer: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));

        let dut = DutInfo::builder("dut_id").build();

        let test_run = TestRun::builder("run_name", &dut, "1.0")
            .config(
                Config::builder()
                    .with_buffer_output(Arc::clone(&buffer))
                    .build(),
            )
            .build();
        test_run.start().await?;

        let step = test_run.step("first step")?;
        step.start().await?;

        step.error_with_msg("symptom", "Error message").await?;

        step.end(TestStatus::Complete).await?;

        test_run.end(TestStatus::Complete, TestResult::Pass).await?;

        for (idx, entry) in buffer.lock().await.iter().enumerate() {
            let value = serde_json::from_str::<serde_json::Value>(entry)?;
            assert_json_include!(actual: value, expected: &expected[idx]);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_testrun_step_error_with_details() -> Result<()> {
        let expected = [
            json!({"schemaVersion":{"major":2,"minor":0},"sequenceNumber":1}),
            json!({"sequenceNumber":2,"testRunArtifact":{"testRunStart":{"dutInfo":{"dutInfoId":"dut_id","hardwareInfos":null,"metadata":null,"name":null,"platformInfos":null,"softwareInfos":null},"metadata":null,"name":"run_name","parameters":{},"version":"1.0"}}}),
            json!({"sequenceNumber":3,"testStepArtifact":{"testStepStart":{"name":"first step"}}}),
            json!({"sequenceNumber":4,"testStepArtifact":{"error":{"message":"Error message","softwareInfoIds":[{"computerSystem": null, "name": "name", "revision": null, "softwareInfoId": "id", "softwareType": null, "version": null}],"sourceLocation":{"file": "file", "line": 1},"symptom":"symptom"}}}),
            json!({"sequenceNumber":5,"testStepArtifact":{"testStepEnd":{"status":"COMPLETE"}}}),
            json!({"sequenceNumber":6,"testRunArtifact":{"testRunEnd":{"result":"PASS","status":"COMPLETE"}}}),
        ];
        let buffer: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));

        let dut = DutInfo::builder("dut_id").build();

        let test_run = TestRun::builder("run_name", &dut, "1.0")
            .config(
                Config::builder()
                    .with_buffer_output(Arc::clone(&buffer))
                    .build(),
            )
            .build();
        test_run.start().await?;

        let step = test_run.step("first step")?;
        step.start().await?;

        step.error_with_details(
            &Error::builder("symptom")
                .message("Error message")
                .source("file", 1)
                .add_software_info(&SoftwareInfo::builder("id", "name").build())
                .build(),
        )
        .await?;

        step.end(TestStatus::Complete).await?;

        test_run.end(TestStatus::Complete, TestResult::Pass).await?;

        for (idx, entry) in buffer.lock().await.iter().enumerate() {
            let value = serde_json::from_str::<serde_json::Value>(entry)?;
            assert_json_include!(actual: value, expected: &expected[idx]);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_testrun_step_scope_log() -> Result<()> {
        let expected = [
            json!({"schemaVersion":{"major":2,"minor":0},"sequenceNumber":1}),
            json!({"sequenceNumber":2,"testRunArtifact":{"testRunStart":{"dutInfo":{"dutInfoId":"dut_id","hardwareInfos":null,"metadata":null,"name":null,"platformInfos":null,"softwareInfos":null},"metadata":null,"name":"run_name","parameters":{},"version":"1.0"}}}),
            json!({"sequenceNumber":3,"testStepArtifact":{"testStepStart":{"name":"first step"}}}),
            json!({"sequenceNumber":4,"testStepArtifact":{"log":{"message":"This is a log message with INFO severity","severity":"INFO","sourceLocation":null}}}),
            json!({"sequenceNumber":5,"testStepArtifact":{"testStepEnd":{"status":"COMPLETE"}}}),
            json!({"sequenceNumber":6,"testRunArtifact":{"testRunEnd":{"result":"PASS","status":"COMPLETE"}}}),
        ];
        let buffer: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));

        let dut = DutInfo::builder("dut_id").build();

        let test_run = TestRun::builder("run_name", &dut, "1.0")
            .config(
                Config::builder()
                    .with_buffer_output(Arc::clone(&buffer))
                    .build(),
            )
            .build();
        test_run.start().await?;

        test_run
            .step("first step")?
            .scope(|s| async {
                s.log(
                    LogSeverity::Info,
                    "This is a log message with INFO severity",
                )
                .await?;
                Ok(TestStatus::Complete)
            })
            .await?;

        test_run.end(TestStatus::Complete, TestResult::Pass).await?;

        for (idx, entry) in buffer.lock().await.iter().enumerate() {
            let value = serde_json::from_str::<serde_json::Value>(entry)?;
            assert_json_include!(actual: value, expected: &expected[idx]);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_step_with_measurement() -> Result<()> {
        let expected = [
            json!({"schemaVersion":{"major":2,"minor":0},"sequenceNumber":1}),
            json!({"sequenceNumber":2,"testRunArtifact":{"testRunStart":{"dutInfo":{"dutInfoId":"dut_id","hardwareInfos":null,"metadata":null,"name":null,"platformInfos":null,"softwareInfos":null},"metadata":null,"name":"run_name","parameters":{},"version":"1.0"}}}),
            json!({"sequenceNumber":3,"testStepArtifact":{"testStepStart":{"name":"first step"}}}),
            json!({"sequenceNumber":4,"testStepArtifact":{"measurement":{"hardwareInfoId":null,"metadata":null,"name":"name","subcomponent":null,"unit":null,"validators":null,"value":50}}}),
            json!({"sequenceNumber":5,"testStepArtifact":{"testStepEnd":{"status":"COMPLETE"}}}),
            json!({"sequenceNumber":6,"testRunArtifact":{"testRunEnd":{"result":"PASS","status":"COMPLETE"}}}),
        ];
        let buffer: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));

        let dut = DutInfo::builder("dut_id").build();

        let test_run = TestRun::builder("run_name", &dut, "1.0")
            .config(
                Config::builder()
                    .with_buffer_output(Arc::clone(&buffer))
                    .build(),
            )
            .build();
        test_run.start().await?;

        let step = test_run.step("first step")?;
        step.start().await?;

        step.add_measurement("name", 50.into()).await?;

        step.end(TestStatus::Complete).await?;

        test_run.end(TestStatus::Complete, TestResult::Pass).await?;

        for (idx, entry) in buffer.lock().await.iter().enumerate() {
            let value = serde_json::from_str::<serde_json::Value>(entry)?;
            assert_json_include!(actual: value, expected: &expected[idx]);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_step_with_measurement_builder() -> Result<()> {
        let expected = [
            json!({"schemaVersion":{"major":2,"minor":0},"sequenceNumber":1}),
            json!({"sequenceNumber":2,"testRunArtifact":{"testRunStart":{"dutInfo":{"dutInfoId":"dut_id","hardwareInfos":null,"metadata":null,"name":null,"platformInfos":null,"softwareInfos":null},"metadata":null,"name":"run_name","parameters":{},"version":"1.0"}}}),
            json!({"sequenceNumber":3,"testStepArtifact":{"testStepStart":{"name":"first step"}}}),
            json!({"sequenceNumber":4,"testStepArtifact":{"measurement":{"hardwareInfoId":"id","metadata":{"key":"value"},"name":"name","subcomponent":{"location":null,"name":"name","revision":null,"type":null,"version":null},"unit":null,"validators":[{"metadata":null,"name":null,"type":"EQUAL","value":30}],"value":50}}}),
            json!({"sequenceNumber":5,"testStepArtifact":{"testStepEnd":{"status":"COMPLETE"}}}),
            json!({"sequenceNumber":6,"testRunArtifact":{"testRunEnd":{"result":"PASS","status":"COMPLETE"}}}),
        ];
        let buffer: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));

        let dut = DutInfo::builder("dut_id").build();

        let test_run = TestRun::builder("run_name", &dut, "1.0")
            .config(
                Config::builder()
                    .with_buffer_output(Arc::clone(&buffer))
                    .build(),
            )
            .build();
        test_run.start().await?;

        let step = test_run.step("first step")?;
        step.start().await?;

        let measurement = Measurement::builder("name", 50.into())
            .hardware_info(&objects::HardwareInfo::builder("id", "name").build())
            .add_validator(
                &objects::Validator::builder(models::ValidatorType::Equal, 30.into()).build(),
            )
            .add_metadata("key", "value".into())
            .subcomponent(&objects::Subcomponent::builder("name").build())
            .build();
        step.add_measurement_with_details(&measurement).await?;

        step.end(TestStatus::Complete).await?;

        test_run.end(TestStatus::Complete, TestResult::Pass).await?;

        for (idx, entry) in buffer.lock().await.iter().enumerate() {
            let value = serde_json::from_str::<serde_json::Value>(entry)?;
            assert_json_include!(actual: value, expected: &expected[idx]);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_step_with_measurement_series() -> Result<()> {
        let expected = [
            json!({"schemaVersion":{"major":2,"minor":0},"sequenceNumber":1}),
            json!({"sequenceNumber":2,"testRunArtifact":{"testRunStart":{"dutInfo":{"dutInfoId":"dut_id","hardwareInfos":null,"metadata":null,"name":null,"platformInfos":null,"softwareInfos":null},"metadata":null,"name":"run_name","parameters":{},"version":"1.0"}}}),
            json!({"sequenceNumber":3,"testStepArtifact":{"testStepStart":{"name":"first step"}}}),
            json!({"sequenceNumber":4,"testStepArtifact":{"measurementSeriesStart":{"hardwareInfoId":null,"measurementSeriesId":"series_1","metadata":null,"name":"name","subComponent":null,"unit":null,"validators":null}}}),
            json!({"sequenceNumber":5,"testStepArtifact":{"measurementSeriesEnd":{"measurementSeriesId":"series_1","totalCount":0}}}),
            json!({"sequenceNumber":6,"testStepArtifact":{"testStepEnd":{"status":"COMPLETE"}}}),
            json!({"sequenceNumber":7,"testRunArtifact":{"testRunEnd":{"result":"PASS","status":"COMPLETE"}}}),
        ];
        let buffer: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));

        let dut = DutInfo::builder("dut_id").build();

        let test_run = TestRun::builder("run_name", &dut, "1.0")
            .config(
                Config::builder()
                    .with_buffer_output(Arc::clone(&buffer))
                    .build(),
            )
            .build();
        test_run.start().await?;

        let step = test_run.step("first step")?;
        step.start().await?;

        let series = step.measurement_series("name");
        series.start().await?;
        series.end().await?;

        step.end(TestStatus::Complete).await?;

        test_run.end(TestStatus::Complete, TestResult::Pass).await?;

        for (idx, entry) in buffer.lock().await.iter().enumerate() {
            let value = serde_json::from_str::<serde_json::Value>(entry)?;
            assert_json_include!(actual: value, expected: &expected[idx]);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_step_with_multiple_measurement_series() -> Result<()> {
        let expected = [
            json!({"schemaVersion":{"major":2,"minor":0},"sequenceNumber":1}),
            json!({"sequenceNumber":2,"testRunArtifact":{"testRunStart":{"dutInfo":{"dutInfoId":"dut_id","hardwareInfos":null,"metadata":null,"name":null,"platformInfos":null,"softwareInfos":null},"metadata":null,"name":"run_name","parameters":{},"version":"1.0"}}}),
            json!({"sequenceNumber":3,"testStepArtifact":{"testStepStart":{"name":"first step"}}}),
            json!({"sequenceNumber":4,"testStepArtifact":{"measurementSeriesStart":{"hardwareInfoId":null,"measurementSeriesId":"series_1","metadata":null,"name":"name","subComponent":null,"unit":null,"validators":null}}}),
            json!({"sequenceNumber":5,"testStepArtifact":{"measurementSeriesEnd":{"measurementSeriesId":"series_1","totalCount":0}}}),
            json!({"sequenceNumber":6,"testStepArtifact":{"measurementSeriesStart":{"hardwareInfoId":null,"measurementSeriesId":"series_2","metadata":null,"name":"name","subComponent":null,"unit":null,"validators":null}}}),
            json!({"sequenceNumber":7,"testStepArtifact":{"measurementSeriesEnd":{"measurementSeriesId":"series_2","totalCount":0}}}),
            json!({"sequenceNumber":8,"testStepArtifact":{"testStepEnd":{"status":"COMPLETE"}}}),
            json!({"sequenceNumber":9,"testRunArtifact":{"testRunEnd":{"result":"PASS","status":"COMPLETE"}}}),
        ];
        let buffer: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));

        let dut = DutInfo::builder("dut_id").build();

        let test_run = TestRun::builder("run_name", &dut, "1.0")
            .config(
                Config::builder()
                    .with_buffer_output(Arc::clone(&buffer))
                    .build(),
            )
            .build();
        test_run.start().await?;

        let step = test_run.step("first step")?;
        step.start().await?;

        let series = step.measurement_series("name");
        series.start().await?;
        series.end().await?;

        let series_2 = step.measurement_series("name");
        series_2.start().await?;
        series_2.end().await?;

        step.end(TestStatus::Complete).await?;

        test_run.end(TestStatus::Complete, TestResult::Pass).await?;

        for (idx, entry) in buffer.lock().await.iter().enumerate() {
            let value = serde_json::from_str::<serde_json::Value>(entry)?;
            assert_json_include!(actual: value, expected: &expected[idx]);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_step_with_measurement_series_with_details() -> Result<()> {
        let expected = [
            json!({"schemaVersion":{"major":2,"minor":0},"sequenceNumber":1}),
            json!({"sequenceNumber":2,"testRunArtifact":{"testRunStart":{"dutInfo":{"dutInfoId":"dut_id","hardwareInfos":null,"metadata":null,"name":null,"platformInfos":null,"softwareInfos":null},"metadata":null,"name":"run_name","parameters":{},"version":"1.0"}}}),
            json!({"sequenceNumber":3,"testStepArtifact":{"testStepStart":{"name":"first step"}}}),
            json!({"sequenceNumber":4,"testStepArtifact":{"measurementSeriesStart":{"hardwareInfoId":null,"measurementSeriesId":"series_id","metadata":null,"name":"name","subComponent":null,"unit":null,"validators":null}}}),
            json!({"sequenceNumber":5,"testStepArtifact":{"measurementSeriesEnd":{"measurementSeriesId":"series_id","totalCount":0}}}),
            json!({"sequenceNumber":6,"testStepArtifact":{"testStepEnd":{"status":"COMPLETE"}}}),
            json!({"sequenceNumber":7,"testRunArtifact":{"testRunEnd":{"result":"PASS","status":"COMPLETE"}}}),
        ];
        let buffer: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));

        let dut = DutInfo::builder("dut_id").build();

        let test_run = TestRun::builder("run_name", &dut, "1.0")
            .config(
                Config::builder()
                    .with_buffer_output(Arc::clone(&buffer))
                    .build(),
            )
            .build();
        test_run.start().await?;

        let step = test_run.step("first step")?;
        step.start().await?;

        let series =
            step.measurement_series_with_details(MeasurementSeriesStart::new("name", "series_id"));
        series.start().await?;
        series.end().await?;

        step.end(TestStatus::Complete).await?;

        test_run.end(TestStatus::Complete, TestResult::Pass).await?;

        for (idx, entry) in buffer.lock().await.iter().enumerate() {
            let value = serde_json::from_str::<serde_json::Value>(entry)?;
            assert_json_include!(actual: value, expected: &expected[idx]);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_step_with_measurement_series_with_details_and_start_builder() -> Result<()> {
        let expected = [
            json!({"schemaVersion":{"major":2,"minor":0},"sequenceNumber":1}),
            json!({"sequenceNumber":2,"testRunArtifact":{"testRunStart":{"dutInfo":{"dutInfoId":"dut_id","hardwareInfos":null,"metadata":null,"name":null,"platformInfos":null,"softwareInfos":null},"metadata":null,"name":"run_name","parameters":{},"version":"1.0"}}}),
            json!({"sequenceNumber":3,"testStepArtifact":{"testStepStart":{"name":"first step"}}}),
            json!({"sequenceNumber":4,"testStepArtifact":{"measurementSeriesStart":{"hardwareInfoId":{"computerSystem":null,"hardwareInfoId":"id","location":null,"manager":null,"manufacturer":null,"manufacturerPartNumber":null,"name":"name","odataId":null,"partNumber":null,"revision":null,"serialNumber":null,"version":null},"measurementSeriesId":"series_id","metadata":{"key":"value"},"name":"name","subComponent":{"location":null,"name":"name","revision":null,"type":null,"version":null},"unit":null,"validators":[{"metadata":null,"name":null,"type":"EQUAL","value":30}]}}}),
            json!({"sequenceNumber":5,"testStepArtifact":{"measurementSeriesEnd":{"measurementSeriesId":"series_id","totalCount":0}}}),
            json!({"sequenceNumber":6,"testStepArtifact":{"testStepEnd":{"status":"COMPLETE"}}}),
            json!({"sequenceNumber":7,"testRunArtifact":{"testRunEnd":{"result":"PASS","status":"COMPLETE"}}}),
        ];
        let buffer: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));

        let dut = DutInfo::builder("dut_id").build();

        let test_run = TestRun::builder("run_name", &dut, "1.0")
            .config(
                Config::builder()
                    .with_buffer_output(Arc::clone(&buffer))
                    .build(),
            )
            .build();
        test_run.start().await?;

        let step = test_run.step("first step")?;
        step.start().await?;

        let series = step.measurement_series_with_details(
            MeasurementSeriesStart::builder("name", "series_id")
                .add_metadata("key", "value".into())
                .add_validator(&Validator::builder(ValidatorType::Equal, 30.into()).build())
                .hardware_info(&HardwareInfo::builder("id", "name").build())
                .subcomponent(&Subcomponent::builder("name").build())
                .build(),
        );
        series.start().await?;
        series.end().await?;

        step.end(TestStatus::Complete).await?;

        test_run.end(TestStatus::Complete, TestResult::Pass).await?;

        for (idx, entry) in buffer.lock().await.iter().enumerate() {
            let value = serde_json::from_str::<serde_json::Value>(entry)?;
            assert_json_include!(actual: value, expected: &expected[idx]);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_step_with_measurement_series_element() -> Result<()> {
        let expected = [
            json!({"schemaVersion":{"major":2,"minor":0},"sequenceNumber":1}),
            json!({"sequenceNumber":2,"testRunArtifact":{"testRunStart":{"dutInfo":{"dutInfoId":"dut_id","hardwareInfos":null,"metadata":null,"name":null,"platformInfos":null,"softwareInfos":null},"metadata":null,"name":"run_name","parameters":{},"version":"1.0"}}}),
            json!({"sequenceNumber":3,"testStepArtifact":{"testStepStart":{"name":"first step"}}}),
            json!({"sequenceNumber":4,"testStepArtifact":{"measurementSeriesStart":{"hardwareInfoId":null,"measurementSeriesId":"series_1","metadata":null,"name":"name","subComponent":null,"unit":null,"validators":null}}}),
            json!({"sequenceNumber":5,"testStepArtifact":{"measurementSeriesElement":{"index":0,"measurementSeriesId":"series_1","metadata":null,"value":60}}}),
            json!({"sequenceNumber":6,"testStepArtifact":{"measurementSeriesEnd":{"measurementSeriesId":"series_1","totalCount":1}}}),
            json!({"sequenceNumber":7,"testStepArtifact":{"testStepEnd":{"status":"COMPLETE"}}}),
            json!({"sequenceNumber":8,"testRunArtifact":{"testRunEnd":{"result":"PASS","status":"COMPLETE"}}}),
        ];
        let buffer: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));

        let dut = DutInfo::builder("dut_id").build();

        let test_run = TestRun::builder("run_name", &dut, "1.0")
            .config(
                Config::builder()
                    .with_buffer_output(Arc::clone(&buffer))
                    .build(),
            )
            .build();
        test_run.start().await?;

        let step = test_run.step("first step")?;
        step.start().await?;

        let series = step.measurement_series("name");
        series.start().await?;
        series.add_measurement(60.into()).await?;
        series.end().await?;

        step.end(TestStatus::Complete).await?;

        test_run.end(TestStatus::Complete, TestResult::Pass).await?;

        for (idx, entry) in buffer.lock().await.iter().enumerate() {
            let value = serde_json::from_str::<serde_json::Value>(entry)?;
            assert_json_include!(actual: value, expected: &expected[idx]);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_step_with_measurement_series_element_index_no() -> Result<()> {
        let expected = [
            json!({"schemaVersion":{"major":2,"minor":0},"sequenceNumber":1}),
            json!({"sequenceNumber":2,"testRunArtifact":{"testRunStart":{"dutInfo":{"dutInfoId":"dut_id","hardwareInfos":null,"metadata":null,"name":null,"platformInfos":null,"softwareInfos":null},"metadata":null,"name":"run_name","parameters":{},"version":"1.0"}}}),
            json!({"sequenceNumber":3,"testStepArtifact":{"testStepStart":{"name":"first step"}}}),
            json!({"sequenceNumber":4,"testStepArtifact":{"measurementSeriesStart":{"hardwareInfoId":null,"measurementSeriesId":"series_1","metadata":null,"name":"name","subComponent":null,"unit":null,"validators":null}}}),
            json!({"sequenceNumber":5,"testStepArtifact":{"measurementSeriesElement":{"index":0,"measurementSeriesId":"series_1","metadata":null,"value":60}}}),
            json!({"sequenceNumber":6,"testStepArtifact":{"measurementSeriesElement":{"index":1,"measurementSeriesId":"series_1","metadata":null,"value":70}}}),
            json!({"sequenceNumber":7,"testStepArtifact":{"measurementSeriesElement":{"index":2,"measurementSeriesId":"series_1","metadata":null,"value":80}}}),
            json!({"sequenceNumber":8,"testStepArtifact":{"measurementSeriesEnd":{"measurementSeriesId":"series_1","totalCount":3}}}),
            json!({"sequenceNumber":9,"testStepArtifact":{"testStepEnd":{"status":"COMPLETE"}}}),
            json!({"sequenceNumber":10,"testRunArtifact":{"testRunEnd":{"result":"PASS","status":"COMPLETE"}}}),
        ];
        let buffer: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));

        let dut = DutInfo::builder("dut_id").build();

        let test_run = TestRun::builder("run_name", &dut, "1.0")
            .config(
                Config::builder()
                    .with_buffer_output(Arc::clone(&buffer))
                    .build(),
            )
            .build();
        test_run.start().await?;

        let step = test_run.step("first step")?;
        step.start().await?;
        let series = step.measurement_series("name");
        series.start().await?;
        // add more than one element to check the index increments correctly
        series.add_measurement(60.into()).await?;
        series.add_measurement(70.into()).await?;
        series.add_measurement(80.into()).await?;
        series.end().await?;
        step.end(TestStatus::Complete).await?;

        test_run.end(TestStatus::Complete, TestResult::Pass).await?;

        for (idx, entry) in buffer.lock().await.iter().enumerate() {
            let value = serde_json::from_str::<serde_json::Value>(entry)?;
            assert_json_include!(actual: value, expected: &expected[idx]);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_step_with_measurement_series_element_with_metadata() -> Result<()> {
        let expected = [
            json!({"schemaVersion":{"major":2,"minor":0},"sequenceNumber":1}),
            json!({"sequenceNumber":2,"testRunArtifact":{"testRunStart":{"dutInfo":{"dutInfoId":"dut_id","hardwareInfos":null,"metadata":null,"name":null,"platformInfos":null,"softwareInfos":null},"metadata":null,"name":"run_name","parameters":{},"version":"1.0"}}}),
            json!({"sequenceNumber":3,"testStepArtifact":{"testStepStart":{"name":"first step"}}}),
            json!({"sequenceNumber":4,"testStepArtifact":{"measurementSeriesStart":{"hardwareInfoId":null,"measurementSeriesId":"series_1","metadata":null,"name":"name","subComponent":null,"unit":null,"validators":null}}}),
            json!({"sequenceNumber":5,"testStepArtifact":{"measurementSeriesElement":{"index":0,"measurementSeriesId":"series_1","metadata":{"key": "value"},"value":60}}}),
            json!({"sequenceNumber":6,"testStepArtifact":{"measurementSeriesEnd":{"measurementSeriesId":"series_1","totalCount":1}}}),
            json!({"sequenceNumber":7,"testStepArtifact":{"testStepEnd":{"status":"COMPLETE"}}}),
            json!({"sequenceNumber":8,"testRunArtifact":{"testRunEnd":{"result":"PASS","status":"COMPLETE"}}}),
        ];
        let buffer: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));

        let dut = DutInfo::builder("dut_id").build();

        let test_run = TestRun::builder("run_name", &dut, "1.0")
            .config(
                Config::builder()
                    .with_buffer_output(Arc::clone(&buffer))
                    .build(),
            )
            .build();
        test_run.start().await?;

        let step = test_run.step("first step")?;
        step.start().await?;
        let series = step.measurement_series("name");
        series.start().await?;
        series
            .add_measurement_with_metadata(60.into(), vec![("key", "value".into())])
            .await?;
        series.end().await?;
        step.end(TestStatus::Complete).await?;

        test_run.end(TestStatus::Complete, TestResult::Pass).await?;

        for (idx, entry) in buffer.lock().await.iter().enumerate() {
            let value = serde_json::from_str::<serde_json::Value>(entry)?;
            assert_json_include!(actual: value, expected: &expected[idx]);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_step_with_measurement_series_element_with_metadata_index_no() -> Result<()> {
        let expected = [
            json!({"schemaVersion":{"major":2,"minor":0},"sequenceNumber":1}),
            json!({"sequenceNumber":2,"testRunArtifact":{"testRunStart":{"dutInfo":{"dutInfoId":"dut_id","hardwareInfos":null,"metadata":null,"name":null,"platformInfos":null,"softwareInfos":null},"metadata":null,"name":"run_name","parameters":{},"version":"1.0"}}}),
            json!({"sequenceNumber":3,"testStepArtifact":{"testStepStart":{"name":"first step"}}}),
            json!({"sequenceNumber":4,"testStepArtifact":{"measurementSeriesStart":{"hardwareInfoId":null,"measurementSeriesId":"series_1","metadata":null,"name":"name","subComponent":null,"unit":null,"validators":null}}}),
            json!({"sequenceNumber":5,"testStepArtifact":{"measurementSeriesElement":{"index":0,"measurementSeriesId":"series_1","metadata":{"key": "value"},"value":60}}}),
            json!({"sequenceNumber":6,"testStepArtifact":{"measurementSeriesElement":{"index":1,"measurementSeriesId":"series_1","metadata":{"key2": "value2"},"value":70}}}),
            json!({"sequenceNumber":7,"testStepArtifact":{"measurementSeriesElement":{"index":2,"measurementSeriesId":"series_1","metadata":{"key3": "value3"},"value":80}}}),
            json!({"sequenceNumber":8,"testStepArtifact":{"measurementSeriesEnd":{"measurementSeriesId":"series_1","totalCount":3}}}),
            json!({"sequenceNumber":9,"testStepArtifact":{"testStepEnd":{"status":"COMPLETE"}}}),
            json!({"sequenceNumber":10,"testRunArtifact":{"testRunEnd":{"result":"PASS","status":"COMPLETE"}}}),
        ];
        let buffer: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));

        let dut = DutInfo::builder("dut_id").build();

        let test_run = TestRun::builder("run_name", &dut, "1.0")
            .config(
                Config::builder()
                    .with_buffer_output(Arc::clone(&buffer))
                    .build(),
            )
            .build();
        test_run.start().await?;

        let step = test_run.step("first step")?;
        step.start().await?;
        let series = step.measurement_series("name");
        series.start().await?;
        // add more than one element to check the index increments correctly
        series
            .add_measurement_with_metadata(60.into(), vec![("key", "value".into())])
            .await?;
        series
            .add_measurement_with_metadata(70.into(), vec![("key2", "value2".into())])
            .await?;
        series
            .add_measurement_with_metadata(80.into(), vec![("key3", "value3".into())])
            .await?;
        series.end().await?;
        step.end(TestStatus::Complete).await?;

        test_run.end(TestStatus::Complete, TestResult::Pass).await?;

        for (idx, entry) in buffer.lock().await.iter().enumerate() {
            let value = serde_json::from_str::<serde_json::Value>(entry)?;
            assert_json_include!(actual: value, expected: &expected[idx]);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_step_with_measurement_series_scope() -> Result<()> {
        let expected = [
            json!({"schemaVersion":{"major":2,"minor":0},"sequenceNumber":1}),
            json!({"sequenceNumber":2,"testRunArtifact":{"testRunStart":{"dutInfo":{"dutInfoId":"dut_id","hardwareInfos":null,"metadata":null,"name":null,"platformInfos":null,"softwareInfos":null},"metadata":null,"name":"run_name","parameters":{},"version":"1.0"}}}),
            json!({"sequenceNumber":3,"testStepArtifact":{"testStepStart":{"name":"first step"}}}),
            json!({"sequenceNumber":4,"testStepArtifact":{"measurementSeriesStart":{"hardwareInfoId":null,"measurementSeriesId":"series_1","metadata":null,"name":"name","subComponent":null,"unit":null,"validators":null}}}),
            json!({"sequenceNumber":5,"testStepArtifact":{"measurementSeriesElement":{"index":0,"measurementSeriesId":"series_1","metadata":null,"value":60}}}),
            json!({"sequenceNumber":6,"testStepArtifact":{"measurementSeriesElement":{"index":1,"measurementSeriesId":"series_1","metadata":null,"value":70}}}),
            json!({"sequenceNumber":7,"testStepArtifact":{"measurementSeriesElement":{"index":2,"measurementSeriesId":"series_1","metadata":null,"value":80}}}),
            json!({"sequenceNumber":8,"testStepArtifact":{"measurementSeriesEnd":{"measurementSeriesId":"series_1","totalCount":3}}}),
            json!({"sequenceNumber":9,"testStepArtifact":{"testStepEnd":{"status":"COMPLETE"}}}),
            json!({"sequenceNumber":10,"testRunArtifact":{"testRunEnd":{"result":"PASS","status":"COMPLETE"}}}),
        ];
        let buffer: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));

        let dut = DutInfo::builder("dut_id").build();

        let test_run = TestRun::builder("run_name", &dut, "1.0")
            .config(
                Config::builder()
                    .with_buffer_output(Arc::clone(&buffer))
                    .build(),
            )
            .build();
        test_run.start().await?;

        let step = test_run.step("first step")?;
        step.start().await?;
        let series = step.measurement_series("name");
        series
            .scope(|s| async {
                s.add_measurement(60.into()).await?;
                s.add_measurement(70.into()).await?;
                s.add_measurement(80.into()).await?;

                Ok(())
            })
            .await?;
        step.end(TestStatus::Complete).await?;

        test_run.end(TestStatus::Complete, TestResult::Pass).await?;

        for (idx, entry) in buffer.lock().await.iter().enumerate() {
            let value = serde_json::from_str::<serde_json::Value>(entry)?;
            assert_json_include!(actual: value, expected: &expected[idx]);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_config_builder() -> Result<()> {
        let expected = [
            json!({"schemaVersion":{"major":2,"minor":0},"sequenceNumber":1}),
            json!({"sequenceNumber":2,"testRunArtifact":{"testRunStart":{"dutInfo":{"dutInfoId":"dut_id","hardwareInfos":null,"metadata":null,"name":null,"platformInfos":null,"softwareInfos":null},"metadata":null,"name":"run_name","parameters":{},"version":"1.0"}}}),
            json!({"sequenceNumber":3,"testRunArtifact":{"error":{"message":"Error message","softwareInfoIds":null,"sourceLocation":null,"symptom":"symptom"}}}),
            json!({"sequenceNumber":4,"testRunArtifact":{"testRunEnd":{"result":"PASS","status":"COMPLETE"}}}),
        ];
        let buffer: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));

        let dut = DutInfo::builder("dut_id").build();

        let test_run = TestRun::builder("run_name", &dut, "1.0")
            .config(
                Config::builder()
                    .with_buffer_output(Arc::clone(&buffer))
                    .timezone(chrono_tz::Europe::Rome)
                    .with_file_output(std::env::temp_dir().join("file.txt"))
                    .await?
                    .build(),
            )
            .build();
        test_run.start().await?;
        test_run.error_with_msg("symptom", "Error message").await?;
        test_run.end(TestStatus::Complete, TestResult::Pass).await?;

        for (idx, entry) in buffer.lock().await.iter().enumerate() {
            let value = serde_json::from_str::<serde_json::Value>(entry)?;
            assert_json_include!(actual: value, expected: &expected[idx]);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_testrun_instantiation_with_new() -> Result<()> {
        let test_run = TestRun::new("run_name", "dut_id", "1.0");
        test_run.start().await?;
        test_run.end(TestStatus::Complete, TestResult::Pass).await?;

        assert_eq!(test_run.dut.to_spec().id, "dut_id");
        Ok(())
    }

    #[tokio::test]
    async fn test_testrun_metadata() -> Result<()> {
        let expected = [
            json!({"schemaVersion":{"major":2,"minor":0},"sequenceNumber":1}),
            json!({"sequenceNumber":2,"testRunArtifact":{"testRunStart":{"dutInfo":{"dutInfoId":"dut_id","hardwareInfos":null,"metadata":null,"name":null,"platformInfos":null,"softwareInfos":null},"metadata":{"key": "value"},"name":"run_name","parameters":{},"version":"1.0"}}}),
            json!({"sequenceNumber":3,"testRunArtifact":{"testRunEnd":{"result":"PASS","status":"COMPLETE"}}}),
        ];
        let buffer: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));

        let dut = DutInfo::builder("dut_id").build();

        let test_run = TestRun::builder("run_name", &dut, "1.0")
            .config(
                Config::builder()
                    .with_buffer_output(Arc::clone(&buffer))
                    .build(),
            )
            .add_metadata("key", "value".into())
            .build();
        test_run.start().await?;
        test_run.end(TestStatus::Complete, TestResult::Pass).await?;

        for (idx, entry) in buffer.lock().await.iter().enumerate() {
            let value = serde_json::from_str::<serde_json::Value>(entry)?;
            assert_json_include!(actual: value, expected: &expected[idx]);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_testrun_builder() -> Result<()> {
        let expected = [
            json!({"schemaVersion":{"major":2,"minor":0},"sequenceNumber":1}),
            json!({"sequenceNumber":2,"testRunArtifact":{"testRunStart":{"commandLine":"cmd_line", "dutInfo":{"dutInfoId":"dut_id","hardwareInfos":null,"metadata":null,"name":null,"platformInfos":null,"softwareInfos":null},"metadata":{"key": "value", "key2": "value2"},"name":"run_name","parameters":{"key": "value"},"version":"1.0"}}}),
            json!({"sequenceNumber":3,"testRunArtifact":{"testRunEnd":{"result":"PASS","status":"COMPLETE"}}}),
        ];
        let buffer: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));

        let dut = DutInfo::builder("dut_id").build();

        let test_run = TestRun::builder("run_name", &dut, "1.0")
            .config(
                Config::builder()
                    .with_buffer_output(Arc::clone(&buffer))
                    .build(),
            )
            .add_metadata("key", "value".into())
            .add_metadata("key2", "value2".into())
            .add_parameter("key", "value".into())
            .command_line("cmd_line")
            .build();
        test_run.start().await?;
        test_run.end(TestStatus::Complete, TestResult::Pass).await?;

        for (idx, entry) in buffer.lock().await.iter().enumerate() {
            let value = serde_json::from_str::<serde_json::Value>(entry)?;
            assert_json_include!(actual: value, expected: &expected[idx]);
        }

        Ok(())
    }
}
