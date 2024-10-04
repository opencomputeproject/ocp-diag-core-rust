// (c) Meta Platforms, Inc. and affiliates.
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

use crate::output as tv;
use tv::models;

pub struct Log {
    severity: models::LogSeverity,
    message: String,
    source_location: Option<models::SourceLocationSpec>,
}

impl Log {
    pub fn builder(message: &str) -> LogBuilder {
        LogBuilder::new(message)
    }

    pub fn to_artifact(&self) -> models::LogSpec {
        models::LogSpec {
            severity: self.severity.clone(),
            message: self.message.clone(),
            source_location: self.source_location.clone(),
        }
    }
}

#[derive(Debug)]
pub struct LogBuilder {
    severity: models::LogSeverity,
    message: String,
    source_location: Option<models::SourceLocationSpec>,
}

impl LogBuilder {
    fn new(message: &str) -> Self {
        LogBuilder {
            severity: models::LogSeverity::Info,
            message: message.to_string(),
            source_location: None,
        }
    }
    pub fn severity(mut self, value: models::LogSeverity) -> LogBuilder {
        self.severity = value;
        self
    }
    pub fn source(mut self, file: &str, line: i32) -> LogBuilder {
        self.source_location = Some(models::SourceLocationSpec {
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
    use crate::output as tv;
    use tv::models;

    #[test]
    fn test_log_output_as_test_run_descendant_to_artifact() -> Result<()> {
        let log = Log::builder("test")
            .severity(models::LogSeverity::Info)
            .build();

        let artifact = log.to_artifact();
        assert_eq!(
            artifact,
            models::LogSpec {
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
            .severity(models::LogSeverity::Info)
            .build();

        let artifact = log.to_artifact();
        assert_eq!(
            artifact,
            models::LogSpec {
                severity: log.severity.clone(),
                message: log.message.clone(),
                source_location: log.source_location.clone(),
            }
        );

        Ok(())
    }
}
