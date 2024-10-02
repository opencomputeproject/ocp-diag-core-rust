// (c) Meta Platforms, Inc. and affiliates.
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

use chrono::DateTime;
use serde_json::Map;
use serde_json::Value;

use crate::output::models;

pub enum ArtifactContext {
    TestRun,
    TestStep,
}

pub struct SchemaVersion {
    major: i8,
    minor: i8,
}

impl SchemaVersion {
    pub fn new() -> SchemaVersion {
        SchemaVersion {
            major: models::SPEC_VERSION.0,
            minor: models::SPEC_VERSION.1,
        }
    }

    pub fn to_artifact(&self) -> models::OutputArtifactDescendant {
        models::OutputArtifactDescendant::SchemaVersion(models::SchemaVersionSpec {
            major: self.major,
            minor: self.minor,
        })
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct DutInfo {
    id: String,
    name: Option<String>,
    platform_infos: Option<Vec<PlatformInfo>>,
    software_infos: Option<Vec<SoftwareInfo>>,
    hardware_infos: Option<Vec<HardwareInfo>>,
    metadata: Option<Map<String, Value>>,
}

impl DutInfo {
    pub fn builder(id: &str) -> DutInfoBuilder {
        DutInfoBuilder::new(id)
    }

    pub fn new(id: &str) -> DutInfo {
        DutInfoBuilder::new(id).build()
    }

    pub(crate) fn to_spec(&self) -> models::DutInfoSpec {
        models::DutInfoSpec {
            id: self.id.clone(),
            name: self.name.clone(),
            platform_infos: self
                .platform_infos
                .clone()
                .map(|infos| infos.iter().map(|info| info.to_spec()).collect()),
            software_infos: self
                .software_infos
                .clone()
                .map(|infos| infos.iter().map(|info| info.to_spec()).collect()),
            hardware_infos: self
                .hardware_infos
                .clone()
                .map(|infos| infos.iter().map(|info| info.to_spec()).collect()),
            metadata: self.metadata.clone(),
        }
    }
}

pub struct DutInfoBuilder {
    id: String,
    name: Option<String>,
    platform_infos: Option<Vec<PlatformInfo>>,
    software_infos: Option<Vec<SoftwareInfo>>,
    hardware_infos: Option<Vec<HardwareInfo>>,
    metadata: Option<Map<String, Value>>,
}

impl DutInfoBuilder {
    pub fn new(id: &str) -> DutInfoBuilder {
        DutInfoBuilder {
            id: id.to_string(),
            name: None,
            platform_infos: None,
            software_infos: None,
            hardware_infos: None,
            metadata: None,
        }
    }
    pub fn name(mut self, value: &str) -> DutInfoBuilder {
        self.name = Some(value.to_string());
        self
    }

    pub fn add_platform_info(mut self, platform_info: &PlatformInfo) -> DutInfoBuilder {
        self.platform_infos = match self.platform_infos {
            Some(mut platform_infos) => {
                platform_infos.push(platform_info.clone());
                Some(platform_infos)
            }
            None => Some(vec![platform_info.clone()]),
        };
        self
    }

    pub fn add_software_info(mut self, software_info: &SoftwareInfo) -> DutInfoBuilder {
        self.software_infos = match self.software_infos {
            Some(mut software_infos) => {
                software_infos.push(software_info.clone());
                Some(software_infos)
            }
            None => Some(vec![software_info.clone()]),
        };
        self
    }

    pub fn add_hardware_info(mut self, hardware_info: &HardwareInfo) -> DutInfoBuilder {
        self.hardware_infos = match self.hardware_infos {
            Some(mut hardware_infos) => {
                hardware_infos.push(hardware_info.clone());
                Some(hardware_infos)
            }
            None => Some(vec![hardware_info.clone()]),
        };
        self
    }

    pub fn add_metadata(mut self, key: &str, value: Value) -> DutInfoBuilder {
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

    pub fn build(self) -> DutInfo {
        DutInfo {
            id: self.id,
            name: self.name,
            platform_infos: self.platform_infos,
            software_infos: self.software_infos,
            hardware_infos: self.hardware_infos,
            metadata: self.metadata,
        }
    }
}

pub struct TestRunStart {
    name: String,
    version: String,
    command_line: String,
    parameters: Map<String, Value>,
    metadata: Option<Map<String, Value>>,
    dut_info: DutInfo,
}

impl TestRunStart {
    pub fn builder(
        name: &str,
        version: &str,
        command_line: &str,
        parameters: &Map<String, Value>,
        dut_info: &DutInfo,
    ) -> TestRunStartBuilder {
        TestRunStartBuilder::new(name, version, command_line, parameters, dut_info)
    }

    pub fn to_artifact(&self) -> models::OutputArtifactDescendant {
        models::OutputArtifactDescendant::TestRunArtifact(models::TestRunArtifactSpec {
            descendant: models::TestRunArtifactDescendant::TestRunStart(models::TestRunStartSpec {
                name: self.name.clone(),
                version: self.version.clone(),
                command_line: self.command_line.clone(),
                parameters: self.parameters.clone(),
                metadata: self.metadata.clone(),
                dut_info: self.dut_info.to_spec(),
            }),
        })
    }
}

pub struct TestRunStartBuilder {
    name: String,
    version: String,
    command_line: String,
    parameters: Map<String, Value>,
    metadata: Option<Map<String, Value>>,
    dut_info: DutInfo,
}

impl TestRunStartBuilder {
    pub fn new(
        name: &str,
        version: &str,
        command_line: &str,
        parameters: &Map<String, Value>,
        dut_info: &DutInfo,
    ) -> TestRunStartBuilder {
        TestRunStartBuilder {
            name: name.to_string(),
            version: version.to_string(),
            command_line: command_line.to_string(),
            parameters: parameters.clone(),
            metadata: None,
            dut_info: dut_info.clone(),
        }
    }

    pub fn add_metadata(mut self, key: &str, value: Value) -> TestRunStartBuilder {
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

    pub fn build(self) -> TestRunStart {
        TestRunStart {
            name: self.name,
            version: self.version,
            command_line: self.command_line,
            parameters: self.parameters,
            metadata: self.metadata,
            dut_info: self.dut_info,
        }
    }
}

pub struct TestRunEnd {
    status: models::TestStatus,
    result: models::TestResult,
}

impl TestRunEnd {
    pub fn builder() -> TestRunEndBuilder {
        TestRunEndBuilder::new()
    }

    pub fn to_artifact(&self) -> models::OutputArtifactDescendant {
        models::OutputArtifactDescendant::TestRunArtifact(models::TestRunArtifactSpec {
            descendant: models::TestRunArtifactDescendant::TestRunEnd(models::TestRunEndSpec {
                status: self.status.clone(),
                result: self.result.clone(),
            }),
        })
    }
}

#[derive(Debug)]
pub struct TestRunEndBuilder {
    status: models::TestStatus,
    result: models::TestResult,
}

impl TestRunEndBuilder {
    pub fn new() -> TestRunEndBuilder {
        TestRunEndBuilder {
            status: models::TestStatus::Complete,
            result: models::TestResult::Pass,
        }
    }
    pub fn status(mut self, value: models::TestStatus) -> TestRunEndBuilder {
        self.status = value;
        self
    }

    pub fn result(mut self, value: models::TestResult) -> TestRunEndBuilder {
        self.result = value;
        self
    }

    pub fn build(self) -> TestRunEnd {
        TestRunEnd {
            status: self.status,
            result: self.result,
        }
    }
}

pub struct Log {
    severity: models::LogSeverity,
    message: String,
    source_location: Option<models::SourceLocationSpec>,
}

impl Log {
    pub fn builder(message: &str) -> LogBuilder {
        LogBuilder::new(message)
    }

    pub fn to_artifact(&self, context: ArtifactContext) -> models::OutputArtifactDescendant {
        match context {
            ArtifactContext::TestRun => {
                models::OutputArtifactDescendant::TestRunArtifact(models::TestRunArtifactSpec {
                    descendant: models::TestRunArtifactDescendant::Log(models::LogSpec {
                        severity: self.severity.clone(),
                        message: self.message.clone(),
                        source_location: self.source_location.clone(),
                    }),
                })
            }
            ArtifactContext::TestStep => {
                models::OutputArtifactDescendant::TestStepArtifact(models::TestStepArtifactSpec {
                    descendant: models::TestStepArtifactDescendant::Log(models::LogSpec {
                        severity: self.severity.clone(),
                        message: self.message.clone(),
                        source_location: self.source_location.clone(),
                    }),
                })
            }
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

pub struct Error {
    symptom: String,
    message: Option<String>,
    software_infos: Option<Vec<models::SoftwareInfoSpec>>,
    source_location: Option<models::SourceLocationSpec>,
}

impl Error {
    pub fn builder(symptom: &str) -> ErrorBuilder {
        ErrorBuilder::new(symptom)
    }

    pub fn to_artifact(&self, context: ArtifactContext) -> models::OutputArtifactDescendant {
        match context {
            ArtifactContext::TestRun => {
                models::OutputArtifactDescendant::TestRunArtifact(models::TestRunArtifactSpec {
                    descendant: models::TestRunArtifactDescendant::Error(models::ErrorSpec {
                        symptom: self.symptom.clone(),
                        message: self.message.clone(),
                        software_infos: self.software_infos.clone(),
                        source_location: self.source_location.clone(),
                    }),
                })
            }
            ArtifactContext::TestStep => {
                models::OutputArtifactDescendant::TestStepArtifact(models::TestStepArtifactSpec {
                    descendant: models::TestStepArtifactDescendant::Error(models::ErrorSpec {
                        symptom: self.symptom.clone(),
                        message: self.message.clone(),
                        software_infos: self.software_infos.clone(),
                        source_location: self.source_location.clone(),
                    }),
                })
            }
        }
    }
}

#[derive(Debug)]
pub struct ErrorBuilder {
    symptom: String,
    message: Option<String>,
    software_infos: Option<Vec<models::SoftwareInfoSpec>>,
    source_location: Option<models::SourceLocationSpec>,
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
        self.source_location = Some(models::SourceLocationSpec {
            file: file.to_string(),
            line,
        });
        self
    }
    pub fn add_software_info(mut self, software_info: &SoftwareInfo) -> ErrorBuilder {
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

pub struct TestStepStart {
    name: String,
}

impl TestStepStart {
    pub fn new(name: &str) -> TestStepStart {
        TestStepStart {
            name: name.to_string(),
        }
    }

    pub fn to_artifact(&self) -> models::OutputArtifactDescendant {
        models::OutputArtifactDescendant::TestStepArtifact(models::TestStepArtifactSpec {
            descendant: models::TestStepArtifactDescendant::TestStepStart(
                models::TestStepStartSpec {
                    name: self.name.clone(),
                },
            ),
        })
    }
}

pub struct TestStepEnd {
    status: models::TestStatus,
}

impl TestStepEnd {
    pub fn new(status: models::TestStatus) -> TestStepEnd {
        TestStepEnd { status }
    }

    pub fn to_artifact(&self) -> models::OutputArtifactDescendant {
        models::OutputArtifactDescendant::TestStepArtifact(models::TestStepArtifactSpec {
            descendant: models::TestStepArtifactDescendant::TestStepEnd(models::TestStepEndSpec {
                status: self.status.clone(),
            }),
        })
    }
}

#[derive(Clone)]
pub struct Validator {
    name: Option<String>,
    validator_type: models::ValidatorType,
    value: Value,
    metadata: Option<Map<String, Value>>,
}

impl Validator {
    pub fn builder(validator_type: models::ValidatorType, value: Value) -> ValidatorBuilder {
        ValidatorBuilder::new(validator_type, value)
    }
    pub fn to_spec(&self) -> models::ValidatorSpec {
        models::ValidatorSpec {
            name: self.name.clone(),
            validator_type: self.validator_type.clone(),
            value: self.value.clone(),
            metadata: self.metadata.clone(),
        }
    }
}

#[derive(Debug)]
pub struct ValidatorBuilder {
    name: Option<String>,
    validator_type: models::ValidatorType,
    value: Value,
    metadata: Option<Map<String, Value>>,
}

impl ValidatorBuilder {
    fn new(validator_type: models::ValidatorType, value: Value) -> Self {
        ValidatorBuilder {
            validator_type,
            value: value.clone(),
            name: None,
            metadata: None,
        }
    }
    pub fn name(mut self, value: &str) -> ValidatorBuilder {
        self.name = Some(value.to_string());
        self
    }
    pub fn add_metadata(mut self, key: &str, value: Value) -> ValidatorBuilder {
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

    pub fn build(self) -> Validator {
        Validator {
            name: self.name,
            validator_type: self.validator_type,
            value: self.value,
            metadata: self.metadata,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HardwareInfo {
    id: String,
    name: String,
    version: Option<String>,
    revision: Option<String>,
    location: Option<String>,
    serial_no: Option<String>,
    part_no: Option<String>,
    manufacturer: Option<String>,
    manufacturer_part_no: Option<String>,
    odata_id: Option<String>,
    computer_system: Option<String>,
    manager: Option<String>,
}

impl HardwareInfo {
    pub fn builder(id: &str, name: &str) -> HardwareInfoBuilder {
        HardwareInfoBuilder::new(id, name)
    }
    pub fn to_spec(&self) -> models::HardwareInfoSpec {
        models::HardwareInfoSpec {
            id: self.id.clone(),
            name: self.name.clone(),
            version: self.version.clone(),
            revision: self.revision.clone(),
            location: self.location.clone(),
            serial_no: self.serial_no.clone(),
            part_no: self.part_no.clone(),
            manufacturer: self.manufacturer.clone(),
            manufacturer_part_no: self.manufacturer_part_no.clone(),
            odata_id: self.odata_id.clone(),
            computer_system: self.computer_system.clone(),
            manager: self.manager.clone(),
        }
    }
}

#[derive(Debug)]
pub struct HardwareInfoBuilder {
    id: String,
    name: String,
    version: Option<String>,
    revision: Option<String>,
    location: Option<String>,
    serial_no: Option<String>,
    part_no: Option<String>,
    manufacturer: Option<String>,
    manufacturer_part_no: Option<String>,
    odata_id: Option<String>,
    computer_system: Option<String>,
    manager: Option<String>,
}

impl HardwareInfoBuilder {
    fn new(id: &str, name: &str) -> Self {
        HardwareInfoBuilder {
            id: id.to_string(),
            name: name.to_string(),
            version: None,
            revision: None,
            location: None,
            serial_no: None,
            part_no: None,
            manufacturer: None,
            manufacturer_part_no: None,
            odata_id: None,
            computer_system: None,
            manager: None,
        }
    }
    pub fn version(mut self, value: &str) -> HardwareInfoBuilder {
        self.version = Some(value.to_string());
        self
    }
    pub fn revision(mut self, value: &str) -> HardwareInfoBuilder {
        self.revision = Some(value.to_string());
        self
    }
    pub fn location(mut self, value: &str) -> HardwareInfoBuilder {
        self.location = Some(value.to_string());
        self
    }
    pub fn serial_no(mut self, value: &str) -> HardwareInfoBuilder {
        self.serial_no = Some(value.to_string());
        self
    }
    pub fn part_no(mut self, value: &str) -> HardwareInfoBuilder {
        self.part_no = Some(value.to_string());
        self
    }
    pub fn manufacturer(mut self, value: &str) -> HardwareInfoBuilder {
        self.manufacturer = Some(value.to_string());
        self
    }
    pub fn manufacturer_part_no(mut self, value: &str) -> HardwareInfoBuilder {
        self.manufacturer_part_no = Some(value.to_string());
        self
    }
    pub fn odata_id(mut self, value: &str) -> HardwareInfoBuilder {
        self.odata_id = Some(value.to_string());
        self
    }
    pub fn computer_system(mut self, value: &str) -> HardwareInfoBuilder {
        self.computer_system = Some(value.to_string());
        self
    }
    pub fn manager(mut self, value: &str) -> HardwareInfoBuilder {
        self.manager = Some(value.to_string());
        self
    }

    pub fn build(self) -> HardwareInfo {
        HardwareInfo {
            id: self.id,
            name: self.name,
            version: self.version,
            revision: self.revision,
            location: self.location,
            serial_no: self.serial_no,
            part_no: self.part_no,
            manufacturer: self.manufacturer,
            manufacturer_part_no: self.manufacturer_part_no,
            odata_id: self.odata_id,
            computer_system: self.computer_system,
            manager: self.manager,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Subcomponent {
    subcomponent_type: Option<models::SubcomponentType>,
    name: String,
    location: Option<String>,
    version: Option<String>,
    revision: Option<String>,
}

impl Subcomponent {
    pub fn builder(name: &str) -> SubcomponentBuilder {
        SubcomponentBuilder::new(name)
    }
    pub fn to_spec(&self) -> models::SubcomponentSpec {
        models::SubcomponentSpec {
            subcomponent_type: self.subcomponent_type.clone(),
            name: self.name.clone(),
            location: self.location.clone(),
            version: self.version.clone(),
            revision: self.revision.clone(),
        }
    }
}

#[derive(Debug)]
pub struct SubcomponentBuilder {
    subcomponent_type: Option<models::SubcomponentType>,
    name: String,
    location: Option<String>,
    version: Option<String>,
    revision: Option<String>,
}

impl SubcomponentBuilder {
    fn new(name: &str) -> Self {
        SubcomponentBuilder {
            subcomponent_type: None,
            name: name.to_string(),
            location: None,
            version: None,
            revision: None,
        }
    }
    pub fn subcomponent_type(mut self, value: models::SubcomponentType) -> SubcomponentBuilder {
        self.subcomponent_type = Some(value);
        self
    }
    pub fn version(mut self, value: &str) -> SubcomponentBuilder {
        self.version = Some(value.to_string());
        self
    }
    pub fn location(mut self, value: &str) -> SubcomponentBuilder {
        self.location = Some(value.to_string());
        self
    }
    pub fn revision(mut self, value: &str) -> SubcomponentBuilder {
        self.revision = Some(value.to_string());
        self
    }

    pub fn build(self) -> Subcomponent {
        Subcomponent {
            subcomponent_type: self.subcomponent_type,
            name: self.name,
            location: self.location,
            version: self.version,
            revision: self.revision,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlatformInfo {
    info: String,
}

impl PlatformInfo {
    pub fn builder(info: &str) -> PlatformInfoBuilder {
        PlatformInfoBuilder::new(info)
    }

    pub fn to_spec(&self) -> models::PlatformInfoSpec {
        models::PlatformInfoSpec {
            info: self.info.clone(),
        }
    }
}

#[derive(Debug)]
pub struct PlatformInfoBuilder {
    info: String,
}

impl PlatformInfoBuilder {
    fn new(info: &str) -> Self {
        PlatformInfoBuilder {
            info: info.to_string(),
        }
    }

    pub fn build(self) -> PlatformInfo {
        PlatformInfo { info: self.info }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SoftwareInfo {
    id: String,
    name: String,
    version: Option<String>,
    revision: Option<String>,
    software_type: Option<models::SoftwareType>,
    computer_system: Option<String>,
}

impl SoftwareInfo {
    pub fn builder(id: &str, name: &str) -> SoftwareInfoBuilder {
        SoftwareInfoBuilder::new(id, name)
    }

    pub fn to_spec(&self) -> models::SoftwareInfoSpec {
        models::SoftwareInfoSpec {
            id: self.id.clone(),
            name: self.name.clone(),
            version: self.version.clone(),
            revision: self.revision.clone(),
            software_type: self.software_type.clone(),
            computer_system: self.computer_system.clone(),
        }
    }
}

#[derive(Debug)]
pub struct SoftwareInfoBuilder {
    id: String,
    name: String,
    version: Option<String>,
    revision: Option<String>,
    software_type: Option<models::SoftwareType>,
    computer_system: Option<String>,
}

impl SoftwareInfoBuilder {
    fn new(id: &str, name: &str) -> Self {
        SoftwareInfoBuilder {
            id: id.to_string(),
            name: name.to_string(),
            version: None,
            revision: None,
            software_type: None,
            computer_system: None,
        }
    }
    pub fn version(mut self, value: &str) -> SoftwareInfoBuilder {
        self.version = Some(value.to_string());
        self
    }
    pub fn revision(mut self, value: &str) -> SoftwareInfoBuilder {
        self.revision = Some(value.to_string());
        self
    }
    pub fn software_type(mut self, value: models::SoftwareType) -> SoftwareInfoBuilder {
        self.software_type = Some(value);
        self
    }
    pub fn computer_system(mut self, value: &str) -> SoftwareInfoBuilder {
        self.computer_system = Some(value.to_string());
        self
    }

    pub fn build(self) -> SoftwareInfo {
        SoftwareInfo {
            id: self.id,
            name: self.name,
            version: self.version,
            revision: self.revision,
            software_type: self.software_type,
            computer_system: self.computer_system,
        }
    }
}

/// This structure represents a Measurement message.
/// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#measurement
///
/// # Examples
///
/// ## Create a Measurement object with the `new` method
///
/// ```
/// use ocptv::output::Measurement;
/// use ocptv::output::Value;
///
/// let measurement = Measurement::new("name", Value::from(50));
/// ```
///
/// ## Create a Measurement object with the `builder` method
///
/// ```
/// use ocptv::output::HardwareInfo;
/// use ocptv::output::Measurement;
/// use ocptv::output::Subcomponent;
/// use ocptv::output::Validator;
/// use ocptv::output::ValidatorType;
/// use ocptv::output::Value;
///
/// let measurement = Measurement::builder("name", Value::from(50))
///     .hardware_info(&HardwareInfo::builder("id", "name").build())
///     .add_validator(&Validator::builder(ValidatorType::Equal, Value::from(30)).build())
///     .add_metadata("key", Value::from("value"))
///     .subcomponent(&Subcomponent::builder("name").build())
///     .build();
/// ```
pub struct Measurement {
    name: String,
    value: Value,
    unit: Option<String>,
    validators: Option<Vec<Validator>>,
    hardware_info: Option<HardwareInfo>,
    subcomponent: Option<Subcomponent>,
    metadata: Option<Map<String, Value>>,
}

impl Measurement {
    /// Builds a new Measurement object.
    ///
    /// # Examples
    ///
    /// ```
    /// use ocptv::output::Measurement;
    /// use ocptv::output::Value;
    ///
    /// let measurement = Measurement::new("name", Value::from(50));
    /// ```
    pub fn new(name: &str, value: Value) -> Self {
        Measurement {
            name: name.to_string(),
            value: value.clone(),
            unit: None,
            validators: None,
            hardware_info: None,
            subcomponent: None,
            metadata: None,
        }
    }

    /// Builds a new Measurement object using [`MeasurementBuilder`].
    ///
    /// # Examples
    ///
    /// ```
    /// use ocptv::output::HardwareInfo;
    /// use ocptv::output::Measurement;
    /// use ocptv::output::Subcomponent;
    /// use ocptv::output::Validator;
    /// use ocptv::output::ValidatorType;
    /// use ocptv::output::Value;
    ///
    /// let measurement = Measurement::builder("name", Value::from(50))
    ///     .hardware_info(&HardwareInfo::builder("id", "name").build())
    ///     .add_validator(&Validator::builder(ValidatorType::Equal, Value::from(30)).build())
    ///     .add_metadata("key", Value::from("value"))
    ///     .subcomponent(&Subcomponent::builder("name").build())
    ///     .build();
    /// ```
    pub fn builder(name: &str, value: Value) -> MeasurementBuilder {
        MeasurementBuilder::new(name, value)
    }

    /// Creates an artifact from a Measurement object.
    ///
    /// # Examples
    ///
    /// ```
    /// use ocptv::output::Measurement;
    /// use ocptv::output::Value;
    ///
    /// let measurement = Measurement::new("name", Value::from(50));
    /// let _ = measurement.to_artifact();
    /// ```
    pub fn to_artifact(&self) -> models::OutputArtifactDescendant {
        models::OutputArtifactDescendant::TestStepArtifact(models::TestStepArtifactSpec {
            descendant: models::TestStepArtifactDescendant::Measurement(models::MeasurementSpec {
                name: self.name.clone(),
                unit: self.unit.clone(),
                value: self.value.clone(),
                validators: self
                    .validators
                    .clone()
                    .map(|vals| vals.iter().map(|val| val.to_spec()).collect()),
                hardware_info_id: self
                    .hardware_info
                    .as_ref()
                    .map(|hardware_info| hardware_info.id.clone()),
                subcomponent: self
                    .subcomponent
                    .as_ref()
                    .map(|subcomponent| subcomponent.to_spec()),
                metadata: self.metadata.clone(),
            }),
        })
    }
}

/// This structure builds a [`Measurement`] object.
///
/// # Examples
///
/// ```
/// use ocptv::output::HardwareInfo;
/// use ocptv::output::Measurement;
/// use ocptv::output::MeasurementBuilder;
/// use ocptv::output::Subcomponent;
/// use ocptv::output::Validator;
/// use ocptv::output::ValidatorType;
/// use ocptv::output::Value;
///
/// let builder = MeasurementBuilder::new("name", Value::from(50))
///     .hardware_info(&HardwareInfo::builder("id", "name").build())
///     .add_validator(&Validator::builder(ValidatorType::Equal, Value::from(30)).build())
///     .add_metadata("key", Value::from("value"))
///     .subcomponent(&Subcomponent::builder("name").build());
/// let measurement = builder.build();
/// ```
pub struct MeasurementBuilder {
    name: String,
    value: Value,
    unit: Option<String>,
    validators: Option<Vec<Validator>>,
    hardware_info: Option<HardwareInfo>,
    subcomponent: Option<Subcomponent>,
    metadata: Option<Map<String, Value>>,
}

impl MeasurementBuilder {
    /// Creates a new MeasurementBuilder.
    ///
    /// # Examples
    ///
    /// ```
    /// use ocptv::output::MeasurementBuilder;
    /// use ocptv::output::Value;
    ///
    /// let builder = MeasurementBuilder::new("name", Value::from(50));
    /// ```
    pub fn new(name: &str, value: Value) -> Self {
        MeasurementBuilder {
            name: name.to_string(),
            value: value.clone(),
            unit: None,
            validators: None,
            hardware_info: None,
            subcomponent: None,
            metadata: None,
        }
    }

    /// Add a [`Validator`] to a [`MeasurementBuilder`].
    ///
    /// # Examples
    ///
    /// ```
    /// use ocptv::output::HardwareInfo;
    /// use ocptv::output::MeasurementBuilder;
    /// use ocptv::output::Subcomponent;
    /// use ocptv::output::Validator;
    /// use ocptv::output::ValidatorType;
    /// use ocptv::output::Value;
    ///
    /// let builder = MeasurementBuilder::new("name", Value::from(50))
    ///     .add_validator(&Validator::builder(ValidatorType::Equal, Value::from(30)).build());
    /// ```
    pub fn add_validator(mut self, validator: &Validator) -> MeasurementBuilder {
        self.validators = match self.validators {
            Some(mut validators) => {
                validators.push(validator.clone());
                Some(validators)
            }
            None => Some(vec![validator.clone()]),
        };
        self
    }

    /// Add a [`HardwareInfo`] to a [`MeasurementBuilder`].
    ///
    /// # Examples
    ///
    /// ```
    /// use ocptv::output::HardwareInfo;
    /// use ocptv::output::MeasurementBuilder;
    /// use ocptv::output::Value;
    ///
    /// let builder = MeasurementBuilder::new("name", Value::from(50))
    ///     .hardware_info(&HardwareInfo::builder("id", "name").build());
    /// ```
    pub fn hardware_info(mut self, hardware_info: &HardwareInfo) -> MeasurementBuilder {
        self.hardware_info = Some(hardware_info.clone());
        self
    }

    /// Add a [`Subcomponent`] to a [`MeasurementBuilder`].
    ///
    /// # Examples
    ///
    /// ```
    /// use ocptv::output::MeasurementBuilder;
    /// use ocptv::output::Subcomponent;
    /// use ocptv::output::Value;
    ///
    /// let builder = MeasurementBuilder::new("name", Value::from(50))
    ///     .subcomponent(&Subcomponent::builder("name").build());
    /// ```
    pub fn subcomponent(mut self, subcomponent: &Subcomponent) -> MeasurementBuilder {
        self.subcomponent = Some(subcomponent.clone());
        self
    }

    /// Add custom metadata to a [`MeasurementBuilder`].
    ///
    /// # Examples
    ///
    /// ```
    /// use ocptv::output::MeasurementBuilder;
    /// use ocptv::output::Value;
    ///
    /// let builder =
    ///     MeasurementBuilder::new("name", Value::from(50)).add_metadata("key", Value::from("value"));
    /// ```
    pub fn add_metadata(mut self, key: &str, value: Value) -> MeasurementBuilder {
        match self.metadata {
            Some(ref mut metadata) => {
                metadata.insert(key.to_string(), value.clone());
            }
            None => {
                self.metadata = Some(Map::new());
                self.metadata
                    .as_mut()
                    .unwrap()
                    .insert(key.to_string(), value.clone());
            }
        };
        self
    }

    /// Add measurement unit to a [`MeasurementBuilder`].
    ///
    /// # Examples
    ///
    /// ```
    /// use ocptv::output::MeasurementBuilder;
    /// use ocptv::output::Value;
    ///
    /// let builder = MeasurementBuilder::new("name", Value::from(50000)).unit("RPM");
    /// ```
    pub fn unit(mut self, unit: &str) -> MeasurementBuilder {
        self.unit = Some(unit.to_string());
        self
    }

    /// Builds a [`Measurement`] object from a [`MeasurementBuilder`].
    ///
    /// # Examples
    ///
    /// ```
    /// use ocptv::output::MeasurementBuilder;
    /// use ocptv::output::Value;
    ///
    /// let builder = MeasurementBuilder::new("name", Value::from(50));
    /// let measurement = builder.build();
    /// ```
    pub fn build(self) -> Measurement {
        Measurement {
            name: self.name,
            value: self.value,
            unit: self.unit,
            validators: self.validators,
            hardware_info: self.hardware_info,
            subcomponent: self.subcomponent,
            metadata: self.metadata,
        }
    }
}

pub struct MeasurementSeriesStart {
    name: String,
    unit: Option<String>,
    series_id: String,
    validators: Option<Vec<Validator>>,
    hardware_info: Option<HardwareInfo>,
    subcomponent: Option<Subcomponent>,
    metadata: Option<Map<String, Value>>,
}

impl MeasurementSeriesStart {
    pub fn new(name: &str, series_id: &str) -> MeasurementSeriesStart {
        MeasurementSeriesStart {
            name: name.to_string(),
            unit: None,
            series_id: series_id.to_string(),
            validators: None,
            hardware_info: None,
            subcomponent: None,
            metadata: None,
        }
    }

    pub fn builder(name: &str, series_id: &str) -> MeasurementSeriesStartBuilder {
        MeasurementSeriesStartBuilder::new(name, series_id)
    }

    pub fn to_artifact(&self) -> models::OutputArtifactDescendant {
        models::OutputArtifactDescendant::TestStepArtifact(models::TestStepArtifactSpec {
            descendant: models::TestStepArtifactDescendant::MeasurementSeriesStart(
                models::MeasurementSeriesStartSpec {
                    name: self.name.clone(),
                    unit: self.unit.clone(),
                    series_id: self.series_id.clone(),
                    validators: self
                        .validators
                        .clone()
                        .map(|vals| vals.iter().map(|val| val.to_spec()).collect()),
                    hardware_info: self
                        .hardware_info
                        .as_ref()
                        .map(|hardware_info| hardware_info.to_spec()),
                    subcomponent: self
                        .subcomponent
                        .as_ref()
                        .map(|subcomponent| subcomponent.to_spec()),
                    metadata: self.metadata.clone(),
                },
            ),
        })
    }

    pub fn get_series_id(&self) -> &str {
        &self.series_id
    }
}

pub struct MeasurementSeriesStartBuilder {
    name: String,
    unit: Option<String>,
    series_id: String,
    validators: Option<Vec<Validator>>,
    hardware_info: Option<HardwareInfo>,
    subcomponent: Option<Subcomponent>,
    metadata: Option<Map<String, Value>>,
}

impl MeasurementSeriesStartBuilder {
    pub fn new(name: &str, series_id: &str) -> Self {
        MeasurementSeriesStartBuilder {
            name: name.to_string(),
            unit: None,
            series_id: series_id.to_string(),
            validators: None,
            hardware_info: None,
            subcomponent: None,
            metadata: None,
        }
    }
    pub fn add_validator(mut self, validator: &Validator) -> MeasurementSeriesStartBuilder {
        self.validators = match self.validators {
            Some(mut validators) => {
                validators.push(validator.clone());
                Some(validators)
            }
            None => Some(vec![validator.clone()]),
        };
        self
    }

    pub fn hardware_info(mut self, hardware_info: &HardwareInfo) -> MeasurementSeriesStartBuilder {
        self.hardware_info = Some(hardware_info.clone());
        self
    }

    pub fn subcomponent(mut self, subcomponent: &Subcomponent) -> MeasurementSeriesStartBuilder {
        self.subcomponent = Some(subcomponent.clone());
        self
    }

    pub fn add_metadata(mut self, key: &str, value: Value) -> MeasurementSeriesStartBuilder {
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

    pub fn unit(mut self, unit: &str) -> MeasurementSeriesStartBuilder {
        self.unit = Some(unit.to_string());
        self
    }

    pub fn build(self) -> MeasurementSeriesStart {
        MeasurementSeriesStart {
            name: self.name,
            unit: self.unit,
            series_id: self.series_id,
            validators: self.validators,
            hardware_info: self.hardware_info,
            subcomponent: self.subcomponent,
            metadata: self.metadata,
        }
    }
}

pub struct MeasurementSeriesEnd {
    series_id: String,
    total_count: u64,
}

impl MeasurementSeriesEnd {
    pub(crate) fn new(series_id: &str, total_count: u64) -> MeasurementSeriesEnd {
        MeasurementSeriesEnd {
            series_id: series_id.to_string(),
            total_count,
        }
    }

    pub fn to_artifact(&self) -> models::OutputArtifactDescendant {
        models::OutputArtifactDescendant::TestStepArtifact(models::TestStepArtifactSpec {
            descendant: models::TestStepArtifactDescendant::MeasurementSeriesEnd(
                models::MeasurementSeriesEndSpec {
                    series_id: self.series_id.clone(),
                    total_count: self.total_count,
                },
            ),
        })
    }
}

pub struct MeasurementSeriesElement {
    index: u64,
    value: Value,
    timestamp: DateTime<chrono_tz::Tz>,
    series_id: String,
    metadata: Option<Map<String, Value>>,
}

impl MeasurementSeriesElement {
    pub(crate) fn new(
        index: u64,
        value: Value,
        series: &MeasurementSeriesStart,
        metadata: Option<Map<String, Value>>,
    ) -> MeasurementSeriesElement {
        MeasurementSeriesElement {
            index,
            value: value.clone(),
            timestamp: chrono::Local::now().with_timezone(&chrono_tz::Tz::UTC),
            series_id: series.series_id.to_string(),
            metadata,
        }
    }

    pub fn to_artifact(&self) -> models::OutputArtifactDescendant {
        models::OutputArtifactDescendant::TestStepArtifact(models::TestStepArtifactSpec {
            descendant: models::TestStepArtifactDescendant::MeasurementSeriesElement(
                models::MeasurementSeriesElementSpec {
                    index: self.index,
                    value: self.value.clone(),
                    timestamp: self.timestamp,
                    series_id: self.series_id.clone(),
                    metadata: self.metadata.clone(),
                },
            ),
        })
    }
}

#[cfg(test)]
mod tests {
    use assert_json_diff::assert_json_include;
    use serde_json::Map;
    use serde_json::Value;

    use super::*;
    use crate::output::models;
    use crate::output::models::ValidatorType;

    #[test]
    fn test_schema_creation_from_builder() {
        let version = super::SchemaVersion::new();
        assert_eq!(version.major, models::SPEC_VERSION.0);
        assert_eq!(version.minor, models::SPEC_VERSION.1);
    }

    #[test]
    fn test_dut_creation_from_builder_with_defaults() {
        let dut = super::DutInfo::builder("1234").build();
        assert_eq!(dut.id, "1234");
    }

    #[test]
    fn test_log_output_as_test_run_descendant_to_artifact() {
        let log = super::Log::builder("test")
            .severity(super::models::LogSeverity::Info)
            .build();
        let artifact = log.to_artifact(super::ArtifactContext::TestRun);
        assert_eq!(
            artifact,
            super::models::OutputArtifactDescendant::TestRunArtifact(
                super::models::TestRunArtifactSpec {
                    descendant: super::models::TestRunArtifactDescendant::Log(
                        super::models::LogSpec {
                            severity: log.severity.clone(),
                            message: log.message.clone(),
                            source_location: log.source_location.clone(),
                        }
                    ),
                }
            )
        );
    }

    #[test]
    fn test_log_output_as_test_step_descendant_to_artifact() {
        let log = super::Log::builder("test")
            .severity(super::models::LogSeverity::Info)
            .build();
        let artifact = log.to_artifact(super::ArtifactContext::TestStep);
        assert_eq!(
            artifact,
            super::models::OutputArtifactDescendant::TestStepArtifact(
                super::models::TestStepArtifactSpec {
                    descendant: super::models::TestStepArtifactDescendant::Log(
                        super::models::LogSpec {
                            severity: log.severity.clone(),
                            message: log.message.clone(),
                            source_location: log.source_location.clone(),
                        }
                    ),
                }
            )
        );
    }

    #[test]
    fn test_error_output_as_test_run_descendant_to_artifact() {
        let error = super::Error::builder("symptom")
            .message("")
            .add_software_info(&super::SoftwareInfo::builder("id", "name").build())
            .source("", 1)
            .build();
        let artifact = error.to_artifact(super::ArtifactContext::TestRun);
        assert_eq!(
            artifact,
            super::models::OutputArtifactDescendant::TestRunArtifact(
                super::models::TestRunArtifactSpec {
                    descendant: super::models::TestRunArtifactDescendant::Error(
                        super::models::ErrorSpec {
                            symptom: error.symptom.clone(),
                            message: error.message.clone(),
                            software_infos: error.software_infos.clone(),
                            source_location: error.source_location.clone(),
                        }
                    ),
                }
            )
        );
    }

    #[test]
    fn test_error_output_as_test_step_descendant_to_artifact() {
        let error = super::Error::builder("symptom")
            .message("")
            .add_software_info(&super::SoftwareInfo::builder("id", "name").build())
            .source("", 1)
            .build();
        let artifact = error.to_artifact(super::ArtifactContext::TestStep);
        assert_eq!(
            artifact,
            super::models::OutputArtifactDescendant::TestStepArtifact(
                super::models::TestStepArtifactSpec {
                    descendant: super::models::TestStepArtifactDescendant::Error(
                        super::models::ErrorSpec {
                            symptom: error.symptom.clone(),
                            message: error.message.clone(),
                            software_infos: error.software_infos.clone(),
                            source_location: error.source_location.clone(),
                        }
                    ),
                }
            )
        );
    }

    #[test]
    fn test_measurement_as_test_step_descendant_to_artifact() {
        let name = String::from("name");
        let value = Value::from(50);
        let measurement = super::Measurement::new(&name, value.clone());
        let artifact = measurement.to_artifact();
        assert_eq!(
            artifact,
            models::OutputArtifactDescendant::TestStepArtifact(models::TestStepArtifactSpec {
                descendant: models::TestStepArtifactDescendant::Measurement(
                    models::MeasurementSpec {
                        name: name.to_string(),
                        unit: None,
                        value,
                        validators: None,
                        hardware_info_id: None,
                        subcomponent: None,
                        metadata: None,
                    }
                ),
            })
        );
    }

    #[test]
    fn test_measurement_builder_as_test_step_descendant_to_artifact() {
        let name = String::from("name");
        let value = Value::from(50000);
        let hardware_info = HardwareInfo::builder("id", "name").build();
        let validator = Validator::builder(models::ValidatorType::Equal, Value::from(30)).build();
        let meta_key = "key";
        let meta_value = Value::from("value");
        let mut metadata = Map::new();
        metadata.insert(meta_key.to_string(), meta_value.clone());
        metadata.insert(meta_key.to_string(), meta_value.clone());
        let subcomponent = Subcomponent::builder("name").build();
        let unit = "RPM";
        let measurement = Measurement::builder(&name, value.clone())
            .hardware_info(&hardware_info)
            .add_validator(&validator)
            .add_validator(&validator)
            .add_metadata(meta_key, meta_value.clone())
            .add_metadata(meta_key, meta_value.clone())
            .subcomponent(&subcomponent)
            .unit(unit)
            .build();
        let artifact = measurement.to_artifact();
        assert_eq!(
            artifact,
            models::OutputArtifactDescendant::TestStepArtifact(models::TestStepArtifactSpec {
                descendant: models::TestStepArtifactDescendant::Measurement(
                    models::MeasurementSpec {
                        name: name.to_string(),
                        unit: Some(unit.to_string()),
                        value,
                        validators: Some(vec![validator.to_spec(), validator.to_spec()]),
                        hardware_info_id: Some(hardware_info.to_spec().id.clone()),
                        subcomponent: Some(subcomponent.to_spec()),
                        metadata: Some(metadata),
                    }
                ),
            })
        );
    }

    #[test]
    fn test_measurement_series_start_to_artifact() {
        let name = String::from("name");
        let series_id = String::from("series_id");
        let series = super::MeasurementSeriesStart::new(&name, &series_id);
        let artifact = series.to_artifact();
        assert_eq!(
            artifact,
            models::OutputArtifactDescendant::TestStepArtifact(models::TestStepArtifactSpec {
                descendant: models::TestStepArtifactDescendant::MeasurementSeriesStart(
                    models::MeasurementSeriesStartSpec {
                        name: name.to_string(),
                        unit: None,
                        series_id: series_id.to_string(),
                        validators: None,
                        hardware_info: None,
                        subcomponent: None,
                        metadata: None,
                    }
                ),
            })
        );
    }

    #[test]
    fn test_measurement_series_start_builder_to_artifact() {
        let name = String::from("name");
        let series_id = String::from("series_id");
        let validator = Validator::builder(models::ValidatorType::Equal, Value::from(30)).build();
        let validator2 =
            Validator::builder(models::ValidatorType::GreaterThen, Value::from(10)).build();
        let hw_info = HardwareInfo::builder("id", "name").build();
        let subcomponent = Subcomponent::builder("name").build();
        let series = super::MeasurementSeriesStart::builder(&name, &series_id)
            .unit("unit")
            .add_metadata("key", Value::from("value"))
            .add_metadata("key2", Value::from("value2"))
            .add_validator(&validator)
            .add_validator(&validator2)
            .hardware_info(&hw_info)
            .subcomponent(&subcomponent)
            .build();

        let artifact = series.to_artifact();
        assert_eq!(
            artifact,
            models::OutputArtifactDescendant::TestStepArtifact(models::TestStepArtifactSpec {
                descendant: models::TestStepArtifactDescendant::MeasurementSeriesStart(
                    models::MeasurementSeriesStartSpec {
                        name: name.to_string(),
                        unit: Some("unit".to_string()),
                        series_id: series_id.to_string(),
                        validators: Some(vec![validator.to_spec(), validator2.to_spec()]),
                        hardware_info: Some(hw_info.to_spec()),
                        subcomponent: Some(subcomponent.to_spec()),
                        metadata: Some(Map::from_iter([
                            ("key".to_string(), Value::from("value")),
                            ("key2".to_string(), Value::from("value2"))
                        ])),
                    }
                ),
            })
        );
    }

    #[test]
    fn test_measurement_series_end_to_artifact() {
        let series_id = String::from("series_id");
        let series = super::MeasurementSeriesEnd::new(&series_id, 1);
        let artifact = series.to_artifact();
        assert_eq!(
            artifact,
            models::OutputArtifactDescendant::TestStepArtifact(models::TestStepArtifactSpec {
                descendant: models::TestStepArtifactDescendant::MeasurementSeriesEnd(
                    models::MeasurementSeriesEndSpec {
                        series_id: series_id.to_string(),
                        total_count: 1,
                    }
                ),
            })
        );
    }

    #[test]
    fn test_dut_builder() {
        let platform = super::PlatformInfo::builder("platform_info").build();
        let software = super::SoftwareInfo::builder("software_id", "name").build();
        let hardware = super::HardwareInfo::builder("hardware_id", "name").build();
        let dut = super::DutInfo::builder("1234")
            .name("DUT")
            .add_metadata("key", Value::from("value"))
            .add_metadata("key2", Value::from("value2"))
            .add_hardware_info(&hardware)
            .add_hardware_info(&hardware)
            .add_platform_info(&platform)
            .add_platform_info(&platform)
            .add_software_info(&software)
            .add_software_info(&software)
            .build();
        assert_eq!(dut.to_spec().id, "1234");
        assert_eq!(dut.to_spec().name.unwrap(), "DUT");
        assert_eq!(dut.to_spec().metadata.unwrap()["key"], "value");
        assert_eq!(dut.to_spec().metadata.unwrap()["key2"], "value2");
        assert_eq!(
            dut.to_spec().hardware_infos.unwrap().first().unwrap().id,
            "hardware_id"
        );
        assert_eq!(
            dut.to_spec().software_infos.unwrap().first().unwrap().id,
            "software_id"
        );
        assert_eq!(
            dut.to_spec().platform_infos.unwrap().first().unwrap().info,
            "platform_info"
        );
    }

    #[test]
    fn test_error() {
        let expected_run = serde_json::json!({"testRunArtifact": {"error": {"message": "message", "softwareInfoIds": [{"computerSystem": null, "name": "name", "revision": null, "softwareInfoId": "software_id", "softwareType": null, "version": null},  {"computerSystem": null, "name": "name", "revision": null, "softwareInfoId": "software_id", "softwareType": null, "version": null}], "sourceLocation": {"file": "file.rs", "line": 1}, "symptom": "symptom"}}});
        let expected_step = serde_json::json!({"testStepArtifact":{"error":{"message":"message","softwareInfoIds":[{"computerSystem":null,"name":"name","revision":null,"softwareInfoId":"software_id","softwareType":null,"version":null},{"computerSystem":null,"name":"name","revision":null,"softwareInfoId":"software_id","softwareType":null,"version":null}],"sourceLocation":{"file":"file.rs","line":1},"symptom":"symptom"}}});

        let software = super::SoftwareInfo::builder("software_id", "name").build();
        let error = super::ErrorBuilder::new("symptom")
            .message("message")
            .source("file.rs", 1)
            .add_software_info(&software)
            .add_software_info(&software)
            .build();
        let spec = error.to_artifact(ArtifactContext::TestRun);
        let actual = serde_json::json!(spec);
        assert_json_include!(actual: actual, expected: &expected_run);

        let spec = error.to_artifact(ArtifactContext::TestStep);
        let actual = serde_json::json!(spec);
        assert_json_include!(actual: actual, expected: &expected_step);
    }

    #[test]
    fn test_validator() {
        let validator = super::Validator::builder(ValidatorType::Equal, Value::from(30))
            .name("validator")
            .add_metadata("key", Value::from("value"))
            .add_metadata("key2", Value::from("value2"))
            .build();

        assert_eq!(validator.to_spec().name.unwrap(), "validator");
        assert_eq!(validator.to_spec().value, 30);
        assert_eq!(validator.to_spec().validator_type, ValidatorType::Equal);
        assert_eq!(validator.to_spec().metadata.unwrap()["key"], "value");
        assert_eq!(validator.to_spec().metadata.unwrap()["key2"], "value2");
    }

    #[test]
    fn test_hardware_info() {
        let info = super::HardwareInfo::builder("hardware_id", "hardware_name")
            .version("version")
            .revision("revision")
            .location("location")
            .serial_no("serial_no")
            .part_no("part_no")
            .manufacturer("manufacturer")
            .manufacturer_part_no("manufacturer_part_no")
            .odata_id("odata_id")
            .computer_system("computer_system")
            .manager("manager")
            .build();

        assert_eq!(info.to_spec().id, "hardware_id");
        assert_eq!(info.to_spec().name, "hardware_name");
        assert_eq!(info.to_spec().version.unwrap(), "version");
        assert_eq!(info.to_spec().revision.unwrap(), "revision");
        assert_eq!(info.to_spec().location.unwrap(), "location");
        assert_eq!(info.to_spec().serial_no.unwrap(), "serial_no");
        assert_eq!(info.to_spec().part_no.unwrap(), "part_no");
        assert_eq!(info.to_spec().manufacturer.unwrap(), "manufacturer");
        assert_eq!(
            info.to_spec().manufacturer_part_no.unwrap(),
            "manufacturer_part_no"
        );
        assert_eq!(info.to_spec().odata_id.unwrap(), "odata_id");
        assert_eq!(info.to_spec().computer_system.unwrap(), "computer_system");
        assert_eq!(info.to_spec().manager.unwrap(), "manager");
    }

    #[test]
    fn test_subcomponent() {
        let sub = super::Subcomponent::builder("sub_name")
            .subcomponent_type(models::SubcomponentType::Asic)
            .version("version")
            .location("location")
            .revision("revision")
            .build();

        assert_eq!(sub.to_spec().name, "sub_name");
        assert_eq!(sub.to_spec().version.unwrap(), "version");
        assert_eq!(sub.to_spec().revision.unwrap(), "revision");
        assert_eq!(sub.to_spec().location.unwrap(), "location");
        assert_eq!(
            sub.to_spec().subcomponent_type.unwrap(),
            models::SubcomponentType::Asic
        );
    }

    #[test]
    fn test_platform_info() {
        let info = super::PlatformInfo::builder("info").build();

        assert_eq!(info.to_spec().info, "info");
    }

    #[test]
    fn test_software_info() {
        let info = super::SoftwareInfo::builder("software_id", "name")
            .version("version")
            .revision("revision")
            .software_type(models::SoftwareType::Application)
            .computer_system("system")
            .build();

        assert_eq!(info.to_spec().id, "software_id");
        assert_eq!(info.to_spec().name, "name");
        assert_eq!(info.to_spec().version.unwrap(), "version");
        assert_eq!(info.to_spec().revision.unwrap(), "revision");
        assert_eq!(
            info.to_spec().software_type.unwrap(),
            models::SoftwareType::Application
        );
        assert_eq!(info.to_spec().computer_system.unwrap(), "system");
    }
}
