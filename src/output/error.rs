// (c) Meta Platforms, Inc. and affiliates.
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

use crate::output as tv;
use crate::spec;
use tv::{dut, trait_ext::VecExt, DutSoftwareInfo};

/// TODO: docs
#[derive(Clone)]
pub struct Error {
    symptom: String,
    message: Option<String>,
    software_infos: Vec<dut::DutSoftwareInfo>,
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
            software_infos: self.software_infos.map_option(DutSoftwareInfo::to_spec),
            source_location: self.source_location.clone(),
        }
    }
}

/// TODO: docs
#[derive(Debug, Default)]
pub struct ErrorBuilder {
    symptom: String,
    message: Option<String>,
    software_infos: Vec<dut::DutSoftwareInfo>,
    source_location: Option<spec::SourceLocation>,
}

impl ErrorBuilder {
    fn new(symptom: &str) -> Self {
        ErrorBuilder {
            symptom: symptom.to_string(),
            ..Default::default()
        }
    }

    pub fn message(mut self, value: &str) -> Self {
        self.message = Some(value.to_string());
        self
    }

    pub fn source(mut self, file: &str, line: i32) -> Self {
        self.source_location = Some(spec::SourceLocation {
            file: file.to_string(),
            line,
        });
        self
    }

    pub fn add_software_info(mut self, software_info: &dut::DutSoftwareInfo) -> Self {
        self.software_infos.push(software_info.clone());
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
    use assert_json_diff::assert_json_eq;
    use serde_json::json;

    use super::*;
    use crate::output as tv;
    use crate::spec;
    use tv::dut;
    use tv::Ident;

    #[test]
    fn test_error_output_as_test_run_descendant_to_artifact() -> Result<()> {
        let mut dut = dut::DutInfo::new("dut0");
        let sw_info = dut.add_software_info(dut::SoftwareInfo::builder("name").build());

        let error = Error::builder("symptom")
            .message("")
            .add_software_info(&sw_info)
            .source("", 1)
            .build();

        let artifact = error.to_artifact();
        assert_eq!(
            artifact,
            spec::Error {
                symptom: error.symptom.clone(),
                message: error.message.clone(),
                software_infos: Some(vec![sw_info.to_spec()]),
                source_location: error.source_location.clone(),
            }
        );

        Ok(())
    }

    #[test]
    fn test_error_output_as_test_step_descendant_to_artifact() -> Result<()> {
        let mut dut = dut::DutInfo::new("dut0");
        let sw_info = dut.add_software_info(dut::SoftwareInfo::builder("name").build());

        let error = Error::builder("symptom")
            .message("")
            .add_software_info(&sw_info)
            .source("", 1)
            .build();

        let artifact = error.to_artifact();
        assert_eq!(
            artifact,
            spec::Error {
                symptom: error.symptom.clone(),
                message: error.message.clone(),
                software_infos: Some(vec![sw_info.to_spec()]),
                source_location: error.source_location.clone(),
            }
        );

        Ok(())
    }

    #[test]
    fn test_error_with_multiple_software() -> Result<()> {
        let expected_run = json!({
            "message": "message",
            "softwareInfoIds": [
                "software_id",
                "software_id"
            ],
            "sourceLocation": {
                "file": "file.rs",
                "line": 1
            },
            "symptom": "symptom"
        });
        let expected_step = json!({
            "message": "message",
            "softwareInfoIds": [
                "software_id",
                "software_id"
            ],
            "sourceLocation": {"file":"file.rs","line":1},
            "symptom":"symptom"
        });

        let mut dut = dut::DutInfo::new("dut0");
        let sw_info = dut.add_software_info(
            dut::SoftwareInfo::builder("name")
                .id(Ident::Exact("software_id".to_owned()))
                .build(),
        );

        let error = ErrorBuilder::new("symptom")
            .message("message")
            .source("file.rs", 1)
            .add_software_info(&sw_info)
            .add_software_info(&sw_info)
            .build();

        let spec_error = error.to_artifact();
        let actual = json!(spec_error);
        assert_json_eq!(actual, expected_run);

        let spec_error = error.to_artifact();
        let actual = json!(spec_error);
        assert_json_eq!(actual, expected_step);

        Ok(())
    }
}
