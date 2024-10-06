// (c) Meta Platforms, Inc. and affiliates.
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
#![deny(warnings)]

mod config;
mod dut;
mod emitter;
mod error;
mod log;
mod macros;
mod measure;
mod run;
mod step;
mod writer;

pub use crate::spec::LogSeverity;
pub use crate::spec::TestResult;
pub use crate::spec::TestStatus;
pub use crate::spec::ValidatorType;
pub use crate::spec::SPEC_VERSION;
pub use config::*;
pub use dut::*;
pub use emitter::*;
pub use error::*;
pub use log::*;
pub use measure::*;
pub use run::*;
pub use step::*;
pub use writer::*;

// re-export this as a public type we present
pub use serde_json::Value;

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum OcptvError {
    #[error("failed to write to output stream")]
    IoError(#[from] std::io::Error),
    // other?
}
