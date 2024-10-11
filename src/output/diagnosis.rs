// (c) Meta Platforms, Inc. and affiliates.
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

use crate::output as tv;
use crate::spec;
use tv::dut;

/// This structure represents a Diagnosis message.
///
/// ref: <https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#diagnosis>
///
/// Information about the source file and line number are not automatically added.
/// Add them using the builder or the macros octptv_diagnosis_*
///
/// # Examples
///
/// ## Create a Diagnosis object with the `new` method
///
/// ```
/// # use ocptv::output::*;
/// let diagnosis = Diagnosis::new("verdict", DiagnosisType::Pass);
/// ```
///
/// ## Create a Diagnosis object with the `builder` method
///
/// ```
/// # use ocptv::output::*;
/// let mut dut = DutInfo::new("dut0");
/// let hw_info = dut.add_hardware_info(HardwareInfo::builder("name").build());
///
/// let diagnosis = Diagnosis::builder("verdict", DiagnosisType::Pass)
///     .message("message")
///     .hardware_info(&hw_info)
///     .subcomponent(&Subcomponent::builder("name").build())
///     .source("file.rs", 1)
///     .build();
/// ```
#[derive(Default)]
pub struct Diagnosis {
    verdict: String,
    diagnosis_type: spec::DiagnosisType,
    message: Option<String>,
    hardware_info: Option<tv::DutHardwareInfo>,
    subcomponent: Option<tv::Subcomponent>,
    source_location: Option<spec::SourceLocation>,
}

impl Diagnosis {
    /// Builds a new Diagnosis object.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ocptv::output::*;
    ///
    /// let diagnosis = Diagnosis::new("verdict", DiagnosisType::Pass);
    /// ```
    pub fn new(verdict: &str, diagnosis_type: spec::DiagnosisType) -> Self {
        Diagnosis {
            verdict: verdict.to_owned(),
            diagnosis_type,
            ..Default::default()
        }
    }

    /// Builds a new Diagnosis object using [`DiagnosisBuilder`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use ocptv::output::*;
    /// let mut dut = DutInfo::new("dut0");
    /// let hw_info = dut.add_hardware_info(HardwareInfo::builder("name").build());
    ///
    /// let diagnosis = Diagnosis::builder("verdict", DiagnosisType::Pass)
    ///     .message("message")
    ///     .hardware_info(&hw_info)
    ///     .subcomponent(&Subcomponent::builder("name").build())
    ///     .source("file.rs", 1)
    ///     .build();
    /// ```
    pub fn builder(verdict: &str, diagnosis_type: spec::DiagnosisType) -> DiagnosisBuilder {
        DiagnosisBuilder::new(verdict, diagnosis_type)
    }

    /// Creates an artifact from a Diagnosis object.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ocptv::output::*;
    /// let diagnosis = Diagnosis::new("verdict", DiagnosisType::Pass);
    /// let _ = diagnosis.to_artifact();
    /// ```
    pub fn to_artifact(&self) -> spec::Diagnosis {
        spec::Diagnosis {
            verdict: self.verdict.clone(),
            diagnosis_type: self.diagnosis_type.clone(),
            message: self.message.clone(),
            hardware_info: self
                .hardware_info
                .as_ref()
                .map(dut::DutHardwareInfo::to_spec),
            subcomponent: self
                .subcomponent
                .as_ref()
                .map(|subcomponent| subcomponent.to_spec()),
            source_location: self.source_location.clone(),
        }
    }
}

/// This structure builds a [`Diagnosis`] object.
///
/// # Examples
///
/// ```
/// # use ocptv::output::*;
/// let mut dut = DutInfo::new("dut0");
/// let hw_info = dut.add_hardware_info(HardwareInfo::builder("name").build());
///
/// let builder = Diagnosis::builder("verdict", DiagnosisType::Pass)
///     .message("message")
///     .hardware_info(&hw_info)
///     .subcomponent(&Subcomponent::builder("name").build())
///     .source("file.rs", 1);
/// let diagnosis = builder.build();
/// ```
#[derive(Default)]
pub struct DiagnosisBuilder {
    verdict: String,
    diagnosis_type: spec::DiagnosisType,
    message: Option<String>,

    hardware_info: Option<tv::DutHardwareInfo>,
    subcomponent: Option<tv::Subcomponent>,

    source_location: Option<spec::SourceLocation>,
}

impl DiagnosisBuilder {
    /// Creates a new DiagnosisBuilder.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ocptv::output::*;
    /// let builder = DiagnosisBuilder::new("verdict", DiagnosisType::Pass);
    /// ```
    pub fn new(verdict: &str, diagnosis_type: spec::DiagnosisType) -> Self {
        DiagnosisBuilder {
            verdict: verdict.to_owned(),
            diagnosis_type,
            ..Default::default()
        }
    }

    /// Add a message to a [`DiagnosisBuilder`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use ocptv::output::*;
    /// let builder = DiagnosisBuilder::new("verdict", DiagnosisType::Pass)
    ///     .message("message");
    /// ```
    pub fn message(mut self, message: &str) -> DiagnosisBuilder {
        self.message = Some(message.to_owned());
        self
    }

    /// Add a [`dut::HardwareInfo`] to a [`DiagnosisBuilder`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use ocptv::output::*;
    /// let mut dut = DutInfo::new("dut0");
    /// let hw_info = dut.add_hardware_info(HardwareInfo::builder("name").build());
    ///
    /// let builder = DiagnosisBuilder::new("verdict", DiagnosisType::Pass)
    ///     .hardware_info(&hw_info);
    /// ```
    pub fn hardware_info(mut self, hardware_info: &dut::DutHardwareInfo) -> DiagnosisBuilder {
        self.hardware_info = Some(hardware_info.clone());
        self
    }

    /// Add a [`dut::Subcomponent`] to a [`DiagnosisBuilder`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use ocptv::output::*;
    /// let builder = DiagnosisBuilder::new("verdict", DiagnosisType::Pass)
    ///     .subcomponent(&Subcomponent::builder("name").build());
    /// ```
    pub fn subcomponent(mut self, subcomponent: &dut::Subcomponent) -> DiagnosisBuilder {
        self.subcomponent = Some(subcomponent.clone());
        self
    }

    /// Add a source location to a [`DiagnosisBuilder`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use ocptv::output::*;
    /// let builder = DiagnosisBuilder::new("verdict", DiagnosisType::Pass)
    ///     .source("file.rs", 1);
    /// ```
    pub fn source(mut self, file: &str, line: i32) -> DiagnosisBuilder {
        self.source_location = Some(spec::SourceLocation {
            file: file.to_owned(),
            line,
        });
        self
    }

    /// Builds a [`Diagnosis`] object from a [`DiagnosisBuilder`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use ocptv::output::*;
    /// let builder = DiagnosisBuilder::new("verdict", DiagnosisType::Pass);
    /// let diagnosis = builder.build();
    /// ```
    pub fn build(self) -> Diagnosis {
        Diagnosis {
            verdict: self.verdict,
            diagnosis_type: self.diagnosis_type,
            message: self.message,
            hardware_info: self.hardware_info,
            subcomponent: self.subcomponent,
            source_location: self.source_location,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output as tv;
    use crate::spec;
    use anyhow::Result;
    use tv::dut::*;

    #[test]
    fn test_diagnosis_as_test_step_descendant_to_artifact() -> Result<()> {
        let verdict = "verdict".to_owned();
        let diagnosis_type = spec::DiagnosisType::Pass;
        let diagnosis = Diagnosis::new(&verdict, diagnosis_type.clone());

        let artifact = diagnosis.to_artifact();

        assert_eq!(
            artifact,
            spec::Diagnosis {
                verdict: verdict.to_owned(),
                diagnosis_type,
                message: None,
                hardware_info: None,
                subcomponent: None,
                source_location: None,
            }
        );

        Ok(())
    }

    #[test]
    fn test_diagnosis_builder_as_test_step_descendant_to_artifact() -> Result<()> {
        let mut dut = DutInfo::new("dut0");

        let verdict = "verdict".to_owned();
        let diagnosis_type = spec::DiagnosisType::Pass;
        let hardware_info = dut.add_hardware_info(HardwareInfo::builder("name").build());
        let subcomponent = Subcomponent::builder("name").build();
        let file = "file.rs".to_owned();
        let line = 1;
        let message = "message".to_owned();

        let diagnosis = Diagnosis::builder(&verdict, diagnosis_type.clone())
            .hardware_info(&hardware_info)
            .message(&message)
            .subcomponent(&subcomponent)
            .source(&file, line)
            .build();

        let artifact = diagnosis.to_artifact();
        assert_eq!(
            artifact,
            spec::Diagnosis {
                verdict,
                diagnosis_type,
                hardware_info: Some(hardware_info.to_spec()),
                subcomponent: Some(subcomponent.to_spec()),
                message: Some(message),
                source_location: Some(spec::SourceLocation { file, line })
            }
        );

        Ok(())
    }
}
