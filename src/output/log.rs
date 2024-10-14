// (c) Meta Platforms, Inc. and affiliates.
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

use crate::spec;

/// TODO: docs
pub struct Log {
    severity: spec::LogSeverity,
    message: String,
    source_location: Option<spec::SourceLocation>,
}

impl Log {
    pub fn builder(message: &str) -> LogBuilder {
        LogBuilder::new(message)
    }

    pub fn to_artifact(&self) -> spec::Log {
        spec::Log {
            severity: self.severity.clone(),
            message: self.message.clone(),
            source_location: self.source_location.clone(),
        }
    }
}

/// TODO: docs
#[derive(Debug)]
pub struct LogBuilder {
    severity: spec::LogSeverity,
    message: String,
    source_location: Option<spec::SourceLocation>,
}

impl LogBuilder {
    fn new(message: &str) -> Self {
        LogBuilder {
            severity: spec::LogSeverity::Info,
            message: message.to_string(),
            source_location: None,
        }
    }

    pub fn severity(mut self, value: spec::LogSeverity) -> Self {
        self.severity = value;
        self
    }

    pub fn source(mut self, file: &str, line: i32) -> Self {
        self.source_location = Some(spec::SourceLocation {
            file: file.to_string(),
            line,
        });
        self
    }

    pub fn build(self) -> Log {
        Log {
            severity: self.severity,
            message: self.message,
            source_location: self.source_location,
        }
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::*;
    use crate::spec;

    #[test]
    fn test_log_output_as_test_run_descendant_to_artifact() -> Result<()> {
        let log = Log::builder("test")
            .severity(spec::LogSeverity::Info)
            .build();

        let artifact = log.to_artifact();
        assert_eq!(
            artifact,
            spec::Log {
                severity: log.severity.clone(),
                message: log.message.clone(),
                source_location: log.source_location.clone(),
            },
        );

        Ok(())
    }

    #[test]
    fn test_log_output_as_test_step_descendant_to_artifact() -> Result<()> {
        let log = Log::builder("test")
            .severity(spec::LogSeverity::Info)
            .build();

        let artifact = log.to_artifact();
        assert_eq!(
            artifact,
            spec::Log {
                severity: log.severity.clone(),
                message: log.message.clone(),
                source_location: log.source_location.clone(),
            }
        );

        Ok(())
    }
}
