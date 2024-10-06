// (c) Meta Platforms, Inc. and affiliates.
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

use std::collections::BTreeMap;
use std::env;
use std::sync::{
    atomic::{self, Ordering},
    Arc,
};

use maplit::{btreemap, convert_args};

use crate::output as tv;
use crate::spec;
use tv::step::TestStep;
use tv::{config, dut, emitter, error, log};

/// The outcome of a TestRun.
/// It's returned when the scope method of the [`TestRun`] object is used.
pub struct TestRunOutcome {
    /// Reports the execution status of the test
    pub status: spec::TestStatus,
    /// Reports the result of the test
    pub result: spec::TestResult,
}

/// The main diag test run.
///
/// This object describes a single run instance of the diag, and therefore drives the test session.
pub struct TestRun {
    name: String,
    version: String,
    parameters: BTreeMap<String, tv::Value>,
    dut: dut::DutInfo,
    command_line: String,
    metadata: Option<BTreeMap<String, tv::Value>>,

    emitter: Arc<emitter::JsonEmitter>,
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
    pub fn builder(name: &str, dut: &dut::DutInfo, version: &str) -> TestRunBuilder {
        TestRunBuilder::new(name, dut, version)
    }

    /// Creates a new [`TestRun`] object.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ocptv::output::*;
    ///
    /// let run = TestRun::new("diagnostic_name", "my_dut", "1.0");
    /// ```
    pub fn new(name: &str, dut_id: &str, version: &str) -> TestRun {
        let dut = dut::DutInfo::new(dut_id);
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
    /// let run = TestRun::new("diagnostic_name", "my_dut", "1.0");
    /// run.start().await?;
    ///
    /// # Ok::<(), OcptvError>(())
    /// # });
    /// ```
    pub async fn start(self) -> Result<StartedTestRun, tv::OcptvError> {
        // TODO: this likely will go into the emitter since it's not the run's job to emit the schema version
        self.emitter
            .emit(&spec::RootImpl::SchemaVersion(
                spec::SchemaVersion::default(),
            ))
            .await?;

        let start = spec::RootImpl::TestRunArtifact(spec::TestRunArtifact {
            artifact: spec::TestRunArtifactImpl::TestRunStart(spec::TestRunStart {
                name: self.name.clone(),
                version: self.version.clone(),
                command_line: self.command_line.clone(),
                parameters: self.parameters.clone(),
                metadata: self.metadata.clone(),
                dut_info: self.dut.to_spec(),
            }),
        });

        self.emitter.emit(&start).await?;

        Ok(StartedTestRun::new(self))
    }

    // disabling this for the moment so we don't publish api that's unusable.
    // see: https://github.com/rust-lang/rust/issues/70263
    //
    // /// Builds a scope in the [`TestRun`] object, taking care of starting and
    // /// ending it. View [`TestRun::start`] and [`TestRun::end`] methods.
    // /// After the scope is constructed, additional objects may be added to it.
    // /// This is the preferred usage for the [`TestRun`], since it guarantees
    // /// all the messages are emitted between the start and end messages, the order
    // /// is respected and no messages is lost.
    // ///
    // /// # Examples
    // ///
    // /// ```rust
    // /// # tokio_test::block_on(async {
    // /// # use ocptv::output::*;
    // ///
    // /// let run = TestRun::new("diagnostic_name", "my_dut", "1.0");
    // /// run.scope(|r| async {
    // ///     r.add_log(LogSeverity::Info, "First message").await?;
    // ///     Ok(TestRunOutcome {
    // ///         status: TestStatus::Complete,
    // ///         result: TestResult::Pass,
    // ///     })
    // /// }).await?;
    // ///
    // /// # Ok::<(), OcptvError>(())
    // /// # });
    // /// ```
    // pub async fn scope<F, R>(self, func: F) -> Result<(), emitters::WriterError>
    // where
    //     R: Future<Output = Result<TestRunOutcome, emitters::WriterError>>,
    //     for<'a> F: Fut2<'a, R>,
    // {
    //     let run = self.start().await?;
    //     let outcome = func(&run).await?;
    //     run.end(outcome.status, outcome.result).await?;

    //     Ok(())
    // }
}

/// Builder for the [`TestRun`] object.
pub struct TestRunBuilder {
    name: String,
    dut: dut::DutInfo,
    version: String,
    parameters: BTreeMap<String, tv::Value>,
    command_line: String,
    metadata: Option<BTreeMap<String, tv::Value>>,
    config: Option<config::Config>,
}

impl TestRunBuilder {
    pub fn new(name: &str, dut: &dut::DutInfo, version: &str) -> Self {
        Self {
            name: name.to_string(),
            dut: dut.clone(),
            version: version.to_string(),
            parameters: BTreeMap::new(),
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
    /// let run = TestRunBuilder::new("run_name", &dut, "1.0")
    ///     .add_parameter("param1", "value1".into())
    ///     .build();
    /// ```
    pub fn add_parameter(mut self, key: &str, value: tv::Value) -> TestRunBuilder {
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
    /// let run = TestRunBuilder::new("run_name", &dut, "1.0")
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
    /// let run = TestRunBuilder::new("run_name", &dut, "1.0")
    ///     .config(Config::builder().build())
    ///     .build();
    /// ```
    pub fn config(mut self, value: config::Config) -> TestRunBuilder {
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
    /// let run = TestRunBuilder::new("run_name", &dut, "1.0")
    ///     .add_metadata("meta1", "value1".into())
    ///     .build();
    /// ```
    pub fn add_metadata(mut self, key: &str, value: tv::Value) -> TestRunBuilder {
        self.metadata = match self.metadata {
            Some(mut metadata) => {
                metadata.insert(key.to_string(), value.clone());
                Some(metadata)
            }
            None => Some(convert_args!(btreemap!(
                key => value,
            ))),
        };
        self
    }

    pub fn build(self) -> TestRun {
        let config = self.config.unwrap_or(config::Config::builder().build());
        let emitter = emitter::JsonEmitter::new(config.timestamp_provider, config.writer);

        TestRun {
            name: self.name,
            dut: self.dut,
            version: self.version,
            parameters: self.parameters,
            command_line: self.command_line,
            metadata: self.metadata,

            emitter: Arc::new(emitter),
        }
    }
}

/// A test run that was started.
///
/// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#testrunstart
pub struct StartedTestRun {
    run: TestRun,

    step_seqno: atomic::AtomicU64,
}

impl StartedTestRun {
    fn new(run: TestRun) -> StartedTestRun {
        StartedTestRun {
            run,
            step_seqno: atomic::AtomicU64::new(0),
        }
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
    /// let run = TestRun::new("diagnostic_name", "my_dut", "1.0").start().await?;
    /// run.end(TestStatus::Complete, TestResult::Pass).await?;
    ///
    /// # Ok::<(), OcptvError>(())
    /// # });
    /// ```
    pub async fn end(
        self,
        status: spec::TestStatus,
        result: spec::TestResult,
    ) -> Result<(), tv::OcptvError> {
        let end = spec::RootImpl::TestRunArtifact(spec::TestRunArtifact {
            artifact: spec::TestRunArtifactImpl::TestRunEnd(spec::TestRunEnd { status, result }),
        });

        self.run.emitter.emit(&end).await?;
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
    /// let run = TestRun::new("diagnostic_name", "my_dut", "1.0").start().await?;
    /// run.add_log(
    ///     LogSeverity::Info,
    ///     "This is a log message with INFO severity",
    /// ).await?;
    /// run.end(TestStatus::Complete, TestResult::Pass).await?;
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

        let artifact = spec::TestRunArtifact {
            artifact: spec::TestRunArtifactImpl::Log(log.to_artifact()),
        };
        self.run
            .emitter
            .emit(&spec::RootImpl::TestRunArtifact(artifact))
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
    /// let run = TestRun::new("diagnostic_name", "my_dut", "1.0").start().await?;
    /// run.add_log_with_details(
    ///     &Log::builder("This is a log message with INFO severity")
    ///         .severity(LogSeverity::Info)
    ///         .source("file", 1)
    ///         .build(),
    /// ).await?;
    /// run.end(TestStatus::Complete, TestResult::Pass).await?;
    ///
    /// # Ok::<(), OcptvError>(())
    /// # });
    /// ```
    pub async fn add_log_with_details(&self, log: &log::Log) -> Result<(), tv::OcptvError> {
        let artifact = spec::TestRunArtifact {
            artifact: spec::TestRunArtifactImpl::Log(log.to_artifact()),
        };
        self.run
            .emitter
            .emit(&spec::RootImpl::TestRunArtifact(artifact))
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
    /// let run = TestRun::new("diagnostic_name", "my_dut", "1.0").start().await?;
    /// run.add_error("symptom").await?;
    /// run.end(TestStatus::Complete, TestResult::Pass).await?;
    ///
    /// # Ok::<(), OcptvError>(())
    /// # });
    /// ```
    pub async fn add_error(&self, symptom: &str) -> Result<(), tv::OcptvError> {
        let error = error::Error::builder(symptom).build();

        let artifact = spec::TestRunArtifact {
            artifact: spec::TestRunArtifactImpl::Error(error.to_artifact()),
        };
        self.run
            .emitter
            .emit(&spec::RootImpl::TestRunArtifact(artifact))
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
    /// let run = TestRun::new("diagnostic_name", "my_dut", "1.0").start().await?;
    /// run.add_error_with_msg("symptom", "error messasge").await?;
    /// run.end(TestStatus::Complete, TestResult::Pass).await?;
    ///
    /// # Ok::<(), OcptvError>(())
    /// # });
    /// ```
    pub async fn add_error_with_msg(&self, symptom: &str, msg: &str) -> Result<(), tv::OcptvError> {
        let error = error::Error::builder(symptom).message(msg).build();

        let artifact = spec::TestRunArtifact {
            artifact: spec::TestRunArtifactImpl::Error(error.to_artifact()),
        };
        self.run
            .emitter
            .emit(&spec::RootImpl::TestRunArtifact(artifact))
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
    /// let run = TestRun::new("diagnostic_name", "my_dut", "1.0").start().await?;
    /// run.add_error_with_details(
    ///     &Error::builder("symptom")
    ///         .message("Error message")
    ///         .source("file", 1)
    ///         .add_software_info(&SoftwareInfo::builder("id", "name").build())
    ///         .build(),
    /// ).await?;
    /// run.end(TestStatus::Complete, TestResult::Pass).await?;
    ///
    /// # Ok::<(), OcptvError>(())
    /// # });
    /// ```
    pub async fn add_error_with_details(&self, error: &error::Error) -> Result<(), tv::OcptvError> {
        let artifact = spec::TestRunArtifact {
            artifact: spec::TestRunArtifactImpl::Error(error.to_artifact()),
        };
        self.run
            .emitter
            .emit(&spec::RootImpl::TestRunArtifact(artifact))
            .await?;

        Ok(())
    }

    /// Create a new step for this test run.
    /// TODO: docs + example
    pub fn add_step(&self, name: &str) -> TestStep {
        let step_id = format!("step_{}", self.step_seqno.fetch_add(1, Ordering::AcqRel));
        TestStep::new(&step_id, name, Arc::clone(&self.run.emitter))
    }
}
