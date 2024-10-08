// (c) Meta Platforms, Inc. and affiliates.
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

use crate::output as tv;
use crate::spec;
use tv::dut;

pub struct Error {
    symptom: String,
    message: Option<String>,
    software_infos: Option<Vec<spec::SoftwareInfo>>,
    source_location: Option<spec::SourceLocation>,
}

impl Error {
    pub fn builder(symptom: &str) -> ErrorBuilder {
        ErrorBuilder::new(symptom)
    }

    pub fn to_artifact(&self) -> spec::Error {
        spec::Error {
            symptom: self.symptom.clone(),
            message: self.message.clone(),
            software_infos: self.software_infos.clone(),
            source_location: self.source_location.clone(),
        }
    }
}

#[derive(Debug)]
pub struct ErrorBuilder {
    symptom: String,
    message: Option<String>,
    software_infos: Option<Vec<spec::SoftwareInfo>>,
    source_location: Option<spec::SourceLocation>,
}

impl ErrorBuilder {
    fn new(symptom: &str) -> Self {
        ErrorBuilder {
            symptom: symptom.to_string(),
            message: None,
            source_location: None,
            software_infos: None,
        }
    }
    pub fn message(mut self, value: &str) -> ErrorBuilder {
        self.message = Some(value.to_string());
        self
    }
    pub fn source(mut self, file: &str, line: i32) -> ErrorBuilder {
        self.source_location = Some(spec::SourceLocation {
            file: file.to_string(),
            line,
        });
        self
    }
    pub fn add_software_info(mut self, software_info: &dut::SoftwareInfo) -> ErrorBuilder {
        self.software_infos = match self.software_infos {
            Some(mut software_infos) => {
                software_infos.push(software_info.to_spec());
                Some(software_infos)
            }
            None => Some(vec![software_info.to_spec()]),
        };
        self
    }

    pub fn build(self) -> Error {
        Error {
            symptom: self.symptom,
            message: self.message,
            source_location: self.source_location,
            software_infos: self.software_infos,
        }
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use assert_json_diff::assert_json_include;

    use super::*;
    use crate::output as tv;
    use crate::spec;
    use tv::dut;

    #[test]
    fn test_error_output_as_test_run_descendant_to_artifact() -> Result<()> {
        let error = Error::builder("symptom")
            .message("")
            .add_software_info(&dut::SoftwareInfo::builder("id", "name").build())
            .source("", 1)
            .build();

        let artifact = error.to_artifact();
        assert_eq!(
            artifact,
            spec::Error {
                symptom: error.symptom.clone(),
                message: error.message.clone(),
                software_infos: error.software_infos.clone(),
                source_location: error.source_location.clone(),
            }
        );

        Ok(())
    }

    #[test]
    fn test_error_output_as_test_step_descendant_to_artifact() -> Result<()> {
        let error = Error::builder("symptom")
            .message("")
            .add_software_info(&dut::SoftwareInfo::builder("id", "name").build())
            .source("", 1)
            .build();

        let artifact = error.to_artifact();
        assert_eq!(
            artifact,
            spec::Error {
                symptom: error.symptom.clone(),
                message: error.message.clone(),
                software_infos: error.software_infos.clone(),
                source_location: error.source_location.clone(),
            }
        );

        Ok(())
    }

    #[test]
    fn test_error() -> Result<()> {
        let expected_run = serde_json::json!({
            "message": "message",
            "softwareInfoIds": [
                {
                    "computerSystem": null,
                    "name": "name",
                    "revision": null,
                    "softwareInfoId":
                    "software_id",
                    "softwareType": null,
                    "version": null
                },
                {
                    "computerSystem": null,
                    "name": "name",
                    "revision": null,
                    "softwareInfoId":
                    "software_id",
                    "softwareType": null,
                    "version": null
                }
            ],
            "sourceLocation": {"file": "file.rs", "line": 1},
            "symptom": "symptom"
        });
        let expected_step = serde_json::json!({
            "message": "message",
            "softwareInfoIds": [
                {
                    "computerSystem": null,
                    "name": "name",
                    "revision": null,
                    "softwareInfoId": "software_id",
                    "softwareType": null,
                    "version": null
                },
                {
                    "computerSystem": null,
                    "name": "name",
                    "revision": null,
                    "softwareInfoId": "software_id",
                    "softwareType": null,
                    "version": null
                }
            ],
            "sourceLocation": {"file":"file.rs","line":1},
            "symptom":"symptom"
        });

        let software = dut::SoftwareInfo::builder("software_id", "name").build();
        let error = ErrorBuilder::new("symptom")
            .message("message")
            .source("file.rs", 1)
            .add_software_info(&software)
            .add_software_info(&software)
            .build();

        let spec_error = error.to_artifact();
        let actual = serde_json::json!(spec_error);
        assert_json_include!(actual: actual, expected: &expected_run);

        let spec_error = error.to_artifact();
        let actual = serde_json::json!(spec_error);
        assert_json_include!(actual: actual, expected: &expected_step);

        Ok(())
    }
}
