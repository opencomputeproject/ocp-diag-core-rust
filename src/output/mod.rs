// (c) Meta Platforms, Inc. and affiliates.
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
#![deny(warnings)]

mod config;
mod diagnosis;
mod dut;
mod emitter;
mod error;
mod file;
mod log;
mod macros;
mod measure;
mod run;
mod step;
mod trait_ext;
mod writer;

pub use crate::spec::{
    DiagnosisType, LogSeverity, SoftwareType, SubcomponentType, TestResult, TestStatus,
    ValidatorType, SPEC_VERSION,
};
pub use config::{Config, ConfigBuilder, TimestampProvider};
pub use diagnosis::{Diagnosis, DiagnosisBuilder};
pub use dut::{
    DutHardwareInfo, DutInfo, DutInfoBuilder, DutSoftwareInfo, HardwareInfo, HardwareInfoBuilder,
    Ident, PlatformInfo, PlatformInfoBuilder, SoftwareInfo, SoftwareInfoBuilder, Subcomponent,
    SubcomponentBuilder,
};
pub use error::{Error, ErrorBuilder};
pub use file::{File, FileBuilder};
pub use log::{Log, LogBuilder};
pub use measure::{
    Measurement, MeasurementBuilder, MeasurementElementDetail, MeasurementElementDetailBuilder,
    MeasurementSeries, MeasurementSeriesDetail, MeasurementSeriesDetailBuilder,
    StartedMeasurementSeries, Validator, ValidatorBuilder,
};
pub use run::{ScopedTestRun, StartedTestRun, TestRun, TestRunBuilder, TestRunOutcome};
pub use step::{ScopedTestStep, StartedTestStep, TestStep};
pub use writer::{BufferWriter, FileWriter, StdoutWriter, Writer};

// re-export these as a public types we present
pub use serde_json::Value;
pub use url::Url as Uri;

// TODO: docs
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum OcptvError {
    #[error("failed to write to output stream")]
    IoError(#[from] std::io::Error),

    #[error("failed to format input object")]
    Format(Box<dyn std::error::Error + Send + Sync + 'static>), // opaque type so we don't leak impl
}
