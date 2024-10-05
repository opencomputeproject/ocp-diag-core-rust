// (c) Meta Platforms, Inc. and affiliates.
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

use chrono::DateTime;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Map;
use serde_json::Value;

pub const SPEC_VERSION: (i8, i8) = (2, 0);

mod rfc3339_format {
    use chrono::DateTime;
    use chrono::SecondsFormat;
    use serde::Deserialize;
    use serde::Deserializer;
    use serde::Serializer;
    use serde::{self};

    pub fn serialize<S>(date: &DateTime<chrono_tz::Tz>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = date.to_rfc3339_opts(SecondsFormat::Millis, true);
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<chrono_tz::Tz>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let dt = DateTime::parse_from_rfc3339(&s).map_err(serde::de::Error::custom)?;
        Ok(dt.with_timezone(&chrono_tz::Tz::UTC))
    }
}

#[derive(Debug, Serialize, PartialEq, Clone)]
pub enum TestRunArtifactDescendant {
    #[serde(rename = "testRunStart")]
    TestRunStart(TestRunStartSpec),

    #[serde(rename = "testRunEnd")]
    TestRunEnd(TestRunEndSpec),

    #[serde(rename = "log")]
    Log(LogSpec),

    #[serde(rename = "error")]
    Error(ErrorSpec),
}

#[derive(Debug, Serialize, PartialEq, Clone)]
pub enum RootArtifactSpec {
    #[serde(rename = "schemaVersion")]
    SchemaVersion(SchemaVersionSpec),

    #[serde(rename = "testRunArtifact")]
    TestRunArtifact(TestRunArtifactSpec),

    #[serde(rename = "testStepArtifact")]
    TestStepArtifact(TestStepArtifactSpec),
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Serialize, PartialEq, Clone)]
pub enum TestStepArtifactDescendant {
    #[serde(rename = "testStepStart")]
    TestStepStart(TestStepStartSpec),

    #[serde(rename = "testStepEnd")]
    TestStepEnd(TestStepEndSpec),

    #[serde(rename = "measurement")]
    Measurement(MeasurementSpec),

    #[serde(rename = "measurementSeriesStart")]
    MeasurementSeriesStart(MeasurementSeriesStartSpec),

    #[serde(rename = "measurementSeriesEnd")]
    MeasurementSeriesEnd(MeasurementSeriesEndSpec),

    #[serde(rename = "measurementSeriesElement")]
    MeasurementSeriesElement(MeasurementSeriesElementSpec),

    #[serde(rename = "diagnosis")]
    Diagnosis(DiagnosisSpec),

    #[serde(rename = "log")]
    Log(LogSpec),

    #[serde(rename = "error")]
    Error(ErrorSpec),

    #[serde(rename = "file")]
    File(FileSpec),

    #[serde(rename = "extension")]
    Extension(ExtensionSpec),
}

#[derive(Debug, Serialize, Clone, PartialEq)]
pub enum ValidatorType {
    #[serde(rename = "EQUAL")]
    Equal,
    #[serde(rename = "NOT_EQUAL")]
    NotEqual,
    #[serde(rename = "LESS_THAN")]
    LessThan,
    #[serde(rename = "LESS_THAN_OR_EQUAL")]
    LessThenOrEqual,
    #[serde(rename = "GREATER_THAN")]
    GreaterThen,
    #[serde(rename = "GREATER_THAN_OR_EQUAL")]
    GreaterThenOrEqual,
    #[serde(rename = "REGEX_MATCH")]
    RegexMatch,
    #[serde(rename = "REGEX_NO_MATCH")]
    RegexNoMatch,
    #[serde(rename = "IN_SET")]
    InSet,
    #[serde(rename = "NOT_IN_SET")]
    NotInSet,
}

#[derive(Debug, Serialize, Clone, PartialEq)]
pub enum SubcomponentType {
    #[serde(rename = "UNSPECIFIED")]
    Unspecified,
    #[serde(rename = "ASIC")]
    Asic,
    #[serde(rename = "ASIC-SUBSYSTEM")]
    AsicSubsystem,
    #[serde(rename = "BUS")]
    Bus,
    #[serde(rename = "FUNCTION")]
    Function,
    #[serde(rename = "CONNECTOR")]
    Connector,
}

// TODO: this should be better typed
#[derive(Debug, Serialize, PartialEq, Clone)]
pub enum ExtensionContentType {
    #[serde(rename = "float")]
    Float(f64),
    #[serde(rename = "int")]
    Int(i64),
    #[serde(rename = "bool")]
    Bool(bool),
    #[serde(rename = "str")]
    Str(String),
}

/// Outcome of a diagnosis operation.
/// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#diagnosistype
/// schema url: https://github.com/opencomputeproject/ocp-diag-core/blob/main/json_spec/output/diagnosis.json
/// schema ref: https://github.com/opencomputeproject/ocp-diag-core/diagnosis/$defs/type
#[derive(Debug, Serialize, PartialEq, Clone)]
pub enum DiagnosisType {
    #[serde(rename = "PASS")]
    Pass,
    #[serde(rename = "FAIL")]
    Fail,
    #[serde(rename = "UNKNOWN")]
    Unknown,
}

/// Represents the final execution status of a test.
/// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#teststatus
/// schema url: https://github.com/opencomputeproject/ocp-diag-core/blob/main/json_spec/output/test_status.json
/// schema ref: https://github.com/opencomputeproject/ocp-diag-core/testStatus
#[derive(Debug, Serialize, Clone, PartialEq)]
#[serde(rename = "testStatus")]
pub enum TestStatus {
    #[serde(rename = "COMPLETE")]
    Complete,
    #[serde(rename = "ERROR")]
    Error,
    #[serde(rename = "SKIP")]
    Skip,
}

/// Represents the final outcome of a test execution.
/// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#testresult
/// schema url: https://github.com/opencomputeproject/ocp-diag-core/blob/main/json_spec/output/test_run_end.json
/// schema ref: https://github.com/opencomputeproject/ocp-diag-core/testRunEnd/$defs/testResult
#[derive(Debug, Serialize, Clone, PartialEq)]
#[serde(rename = "testResult")]
pub enum TestResult {
    #[serde(rename = "PASS")]
    Pass,
    #[serde(rename = "FAIL")]
    Fail,
    #[serde(rename = "NOT_APPLICABLE")]
    NotApplicable,
}
/// Known log severity variants.
/// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#severity
/// schema url: https://github.com/opencomputeproject/ocp-diag-core/blob/main/json_spec/output/log.json
/// schema ref: https://github.com/opencomputeproject/ocp-diag-core/log/$defs/severity
#[derive(Debug, Serialize, Clone, PartialEq)]
pub enum LogSeverity {
    #[serde(rename = "DEBUG")]
    Debug,
    #[serde(rename = "INFO")]
    Info,
    #[serde(rename = "WARNING")]
    Warning,
    #[serde(rename = "ERROR")]
    Error,
    #[serde(rename = "FATAL")]
    Fatal,
}

/// Type specification for a software component of the DUT.
/// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#softwaretype
/// schema url: https://github.com/opencomputeproject/ocp-diag-core/blob/main/json_spec/output/dut_info.json
/// schema ref: https://github.com/opencomputeproject/ocp-diag-core/dutInfo/$defs/softwareInfo/properties/softwareType
#[derive(Debug, Serialize, Clone, PartialEq)]
#[serde(rename = "softwareType")]
pub enum SoftwareType {
    #[serde(rename = "UNSPECIFIED")]
    Unspecified,
    #[serde(rename = "FIRMWARE")]
    Firmware,
    #[serde(rename = "SYSTEM")]
    System,
    #[serde(rename = "APPLICATION")]
    Application,
}

#[derive(Debug, Serialize, Clone)]
pub struct RootSpec {
    #[serde(flatten)]
    pub artifact: RootArtifactSpec,

    // TODO : manage different timezones
    #[serde(rename = "timestamp")]
    #[serde(with = "rfc3339_format")]
    pub timestamp: DateTime<chrono_tz::Tz>,

    #[serde(rename = "sequenceNumber")]
    pub seqno: u64,
}

/// Low-level model for the `schemaVersion` spec object.
/// Specifies the version that should be used to interpret following json outputs.
/// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#schemaversion
/// schema url: https://github.com/opencomputeproject/ocp-diag-core/blob/main/json_spec/output/root.json
/// schema ref: https://github.com/opencomputeproject/ocp-diag-core/output/$defs/schemaVersion
#[derive(Debug, Serialize, Clone, PartialEq)]
#[serde(rename = "schemaVersion")]
pub struct SchemaVersionSpec {
    #[serde(rename = "major")]
    pub major: i8,

    #[serde(rename = "minor")]
    pub minor: i8,
}

/// Low-level model for the `testRunArtifact` spec object.
/// Container for the run level artifacts.
/// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#test-run-artifacts
/// schema url: https://github.com/opencomputeproject/ocp-diag-core/blob/main/json_spec/output/test_run_artifact.json
/// schema ref: https://github.com/opencomputeproject/ocp-diag-core/testRunArtifact
#[derive(Debug, Serialize, PartialEq, Clone)]
pub struct TestRunArtifactSpec {
    #[serde(flatten)]
    pub artifact: TestRunArtifactDescendant,
}

/// Low-level model for the `testRunStart` spec object.
/// Start marker for the beginning of a diagnostic test.
/// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#testrunstart
/// schema url: https://github.com/opencomputeproject/ocp-diag-core/blob/main/json_spec/output/test_run_start.json
/// schema ref: https://github.com/opencomputeproject/ocp-diag-core/testRunStart
#[derive(Debug, Serialize, Clone, PartialEq)]
#[serde(rename = "testRunStart")]
pub struct TestRunStartSpec {
    #[serde(rename = "name")]
    pub name: String,

    #[serde(rename = "version")]
    pub version: String,

    #[serde(rename = "commandLine")]
    pub command_line: String,

    #[serde(rename = "parameters")]
    pub parameters: Map<String, Value>,

    #[serde(rename = "dutInfo")]
    pub dut_info: DutInfoSpec,

    #[serde(rename = "metadata")]
    pub metadata: Option<Map<String, Value>>,
}

/// Low-level model for the `dutInfo` spec object.
/// Contains all relevant information describing the DUT.
/// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#dutinfo
/// schema url: https://github.com/opencomputeproject/ocp-diag-core/blob/main/json_spec/output/dut_info.json
/// schema ref: https://github.com/opencomputeproject/ocp-diag-core/dutInfo
#[derive(Debug, Serialize, Default, Clone, PartialEq)]
#[serde(rename = "dutInfo")]
pub struct DutInfoSpec {
    #[serde(rename = "dutInfoId")]
    pub id: String,

    #[serde(rename = "name")]
    pub name: Option<String>,

    #[serde(rename = "platformInfos")]
    pub platform_infos: Option<Vec<PlatformInfoSpec>>,

    #[serde(rename = "softwareInfos")]
    pub software_infos: Option<Vec<SoftwareInfoSpec>>,

    #[serde(rename = "hardwareInfos")]
    pub hardware_infos: Option<Vec<HardwareInfoSpec>>,

    #[serde(rename = "metadata")]
    pub metadata: Option<Map<String, Value>>,
}

/// Low-level model for the `platformInfo` spec object.
/// Describe platform specific attributes of the DUT.
/// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#platforminfo
/// schema url: https://github.com/opencomputeproject/ocp-diag-core/blob/main/json_spec/output/dut_info.json
/// schema ref: https://github.com/opencomputeproject/ocp-diag-core/dutInfo/$defs/platformInfo
#[derive(Debug, Serialize, Default, Clone, PartialEq)]
#[serde(rename = "platformInfo")]
pub struct PlatformInfoSpec {
    #[serde(rename = "info")]
    pub info: String,
}

/// Low-level model for the `softwareInfo` spec object.
/// Represents information of a discovered or exercised software component of the DUT.
/// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#softwareinfo
/// schema url: https://github.com/opencomputeproject/ocp-diag-core/blob/main/json_spec/output/dut_info.json
/// schema ref: https://github.com/opencomputeproject/ocp-diag-core/dutInfo/$defs/softwareInfo
#[derive(Debug, Serialize, Clone, PartialEq)]
#[serde(rename = "softwareInfo")]
pub struct SoftwareInfoSpec {
    #[serde(rename = "softwareInfoId")]
    pub id: String,

    #[serde(rename = "name")]
    pub name: String,

    #[serde(rename = "version")]
    pub version: Option<String>,

    #[serde(rename = "revision")]
    pub revision: Option<String>,

    #[serde(rename = "softwareType")]
    pub software_type: Option<SoftwareType>,

    #[serde(rename = "computerSystem")]
    pub computer_system: Option<String>,
}

/// Low-level model for the `hardwareInfo` spec object.
/// Represents information of an enumerated or exercised hardware component of the DUT.
/// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#hardwareinfo
/// schema url: https://github.com/opencomputeproject/ocp-diag-core/blob/main/json_spec/output/dut_info.json
/// schema ref: https://github.com/opencomputeproject/ocp-diag-core/dutInfo/$defs/hardwareInfo
#[derive(Debug, Serialize, Default, Clone, PartialEq)]
#[serde(rename = "hardwareInfo")]
pub struct HardwareInfoSpec {
    #[serde(rename = "hardwareInfoId")]
    pub id: String,

    #[serde(rename = "name")]
    pub name: String,

    #[serde(rename = "version")]
    pub version: Option<String>,

    #[serde(rename = "revision")]
    pub revision: Option<String>,

    #[serde(rename = "location")]
    pub location: Option<String>,

    #[serde(rename = "serialNumber")]
    pub serial_no: Option<String>,

    #[serde(rename = "partNumber")]
    pub part_no: Option<String>,

    #[serde(rename = "manufacturer")]
    pub manufacturer: Option<String>,

    #[serde(rename = "manufacturerPartNumber")]
    pub manufacturer_part_no: Option<String>,

    #[serde(rename = "odataId")]
    pub odata_id: Option<String>,

    #[serde(rename = "computerSystem")]
    pub computer_system: Option<String>,

    #[serde(rename = "manager")]
    pub manager: Option<String>,
}

/// Low-level model for the `testRunEnd` spec object.
/// End marker signaling the finality of a diagnostic test.
/// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#testrunend
/// schema url: https://github.com/opencomputeproject/ocp-diag-core/blob/main/json_spec/output/test_run_end.json
/// schema ref: https://github.com/opencomputeproject/ocp-diag-core/testRunEnd
#[derive(Debug, Serialize, Clone, PartialEq)]
#[serde(rename = "testRunEnd")]
pub struct TestRunEndSpec {
    #[serde(rename = "status")]
    pub status: TestStatus,

    #[serde(rename = "result")]
    pub result: TestResult,
}

/// Low-level model for the `error` spec object.
/// Represents an error encountered by the diagnostic software. It may refer to a DUT
/// component or the diagnostic itself.
/// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#error
/// schema url: https://github.com/opencomputeproject/ocp-diag-core/blob/main/json_spec/output/error.json
/// schema ref: https://github.com/opencomputeproject/ocp-diag-core/error
#[derive(Debug, Serialize, Default, Clone, PartialEq)]
#[serde(rename = "error")]
pub struct ErrorSpec {
    #[serde(rename = "symptom")]
    pub symptom: String,

    #[serde(rename = "message")]
    pub message: Option<String>,

    // TODO: support this field during serialization to print only the id of SoftwareInfo struct
    #[serde(rename = "softwareInfoIds")]
    pub software_infos: Option<Vec<SoftwareInfoSpec>>,

    #[serde(rename = "sourceLocation")]
    pub source_location: Option<SourceLocationSpec>,
}

/// Low-level model for `log` spec object.
/// Is currently relevant for test run and test step artifact types.
/// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#log
/// schema url: https://github.com/opencomputeproject/ocp-diag-core/blob/main/json_spec/output/log.json
/// schema ref: https://github.com/opencomputeproject/ocp-diag-core/log
#[derive(Debug, Serialize, Clone, PartialEq)]
#[serde(rename = "log")]
pub struct LogSpec {
    #[serde(rename = "severity")]
    pub severity: LogSeverity,

    #[serde(rename = "message")]
    pub message: String,

    #[serde(rename = "sourceLocation")]
    pub source_location: Option<SourceLocationSpec>,
}

/// Provides information about which file/line of the source code in
/// the diagnostic package generated the output.
/// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#sourcelocation
/// schema url: https://github.com/opencomputeproject/ocp-diag-core/blob/main/json_spec/output/source_location.json
/// schema ref: https://github.com/opencomputeproject/ocp-diag-core/sourceLocation
#[derive(Debug, Serialize, Clone, Default, PartialEq)]
#[serde(rename = "sourceLocation")]
pub struct SourceLocationSpec {
    #[serde(rename = "file")]
    pub file: String,

    #[serde(rename = "line")]
    pub line: i32,
}

/// Low-level model for the `testStepArtifact` spec object.
/// Container for the step level artifacts.
/// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#test-step-artifacts
/// schema url: https://github.com/opencomputeproject/ocp-diag-core/blob/main/json_spec/output/test_step_artifact.json
/// schema ref: https://github.com/opencomputeproject/ocp-diag-core/testStepArtifact
#[derive(Debug, Serialize, PartialEq, Clone)]
pub struct TestStepArtifactSpec {
    #[serde(flatten)]
    pub descendant: TestStepArtifactDescendant,
}

/// Low-level model for the `testStepStart` spec object.
/// Start marker for a test step inside a diagnosis run.
/// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#teststepstart
/// schema url: https://github.com/opencomputeproject/ocp-diag-core/blob/main/json_spec/output/test_step_start.json
/// schema ref: https://github.com/opencomputeproject/ocp-diag-core/testStepStart
#[derive(Debug, Serialize, PartialEq, Clone)]
#[serde(rename = "testStepStart")]
pub struct TestStepStartSpec {
    #[serde(rename = "name")]
    pub name: String,
}

/// Low-level model for the `testStepEnd` spec object.
/// End marker for a test step inside a diagnosis run.
/// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#teststepend
/// schema url: https://github.com/opencomputeproject/ocp-diag-core/blob/main/json_spec/output/test_step_end.json
/// schema ref: https://github.com/opencomputeproject/ocp-diag-core/testStepEnd
#[derive(Debug, Serialize, PartialEq, Clone)]
#[serde(rename = "testStepEnd")]
pub struct TestStepEndSpec {
    #[serde(rename = "status")]
    pub status: TestStatus,
}

/// Low-level model for the `measurement` spec object.
/// Represents an individual measurement taken by the diagnostic regarding the DUT.
/// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#measurement
/// schema url: https://github.com/opencomputeproject/ocp-diag-core/blob/main/json_spec/output/measurement.json
/// schema ref: https://github.com/opencomputeproject/ocp-diag-core/measurement
#[derive(Debug, Serialize, PartialEq, Clone)]
#[serde(rename = "measurement")]
pub struct MeasurementSpec {
    #[serde(rename = "name")]
    pub name: String,

    #[serde(rename = "value")]
    pub value: Value,

    #[serde(rename = "unit")]
    pub unit: Option<String>,

    #[serde(rename = "validators")]
    pub validators: Option<Vec<ValidatorSpec>>,

    #[serde(rename = "hardwareInfoId")]
    pub hardware_info_id: Option<String>,

    #[serde(rename = "subcomponent")]
    pub subcomponent: Option<SubcomponentSpec>,

    #[serde(rename = "metadata")]
    pub metadata: Option<Map<String, Value>>,
}

/// Low-level model for the `validator` spec object.
/// Contains the validation logic that the diagnostic applied for a specific measurement.
/// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#validator
/// schema url: https://github.com/opencomputeproject/ocp-diag-core/blob/main/json_spec/output/validator.json
/// schema ref: https://github.com/opencomputeproject/ocp-diag-core/validator
#[derive(Debug, Serialize, Clone, PartialEq)]
#[serde(rename = "validator")]
pub struct ValidatorSpec {
    #[serde(rename = "name")]
    pub name: Option<String>,

    #[serde(rename = "type")]
    pub validator_type: ValidatorType,

    #[serde(rename = "value")]
    pub value: Value,

    #[serde(rename = "metadata")]
    pub metadata: Option<Map<String, Value>>,
}

/// Low-level model for the `subcomponent` spec object.
/// Represents a physical subcomponent of a DUT hardware element.
/// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#subcomponent
/// schema url: https://github.com/opencomputeproject/ocp-diag-core/blob/main/json_spec/output/subcomponent.json
/// schema ref: https://github.com/opencomputeproject/ocp-diag-core/subcomponent
#[derive(Debug, Serialize, Clone, PartialEq)]
#[serde(rename = "subcomponent")]
pub struct SubcomponentSpec {
    #[serde(rename = "type")]
    pub subcomponent_type: Option<SubcomponentType>,

    #[serde(rename = "name")]
    pub name: String,

    #[serde(rename = "location")]
    pub location: Option<String>,

    #[serde(rename = "version")]
    pub version: Option<String>,

    #[serde(rename = "revision")]
    pub revision: Option<String>,
}

/// Low-level model for the `measurementSeriesStart` spec object.
/// Start marker for a time based series of measurements.
/// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#measurementseriesstart
/// schema url: https://github.com/opencomputeproject/ocp-diag-core/blob/main/json_spec/output/measurement_series_start.json
/// schema ref: https://github.com/opencomputeproject/ocp-diag-core/measurementSeriesStart
#[derive(Debug, Serialize, PartialEq, Clone)]
#[serde(rename = "measurementSeriesStart")]
pub struct MeasurementSeriesStartSpec {
    #[serde(rename = "name")]
    pub name: String,

    #[serde(rename = "unit")]
    pub unit: Option<String>,

    #[serde(rename = "measurementSeriesId")]
    pub series_id: String,

    #[serde(rename = "validators")]
    pub validators: Option<Vec<ValidatorSpec>>,

    #[serde(rename = "hardwareInfoId")]
    pub hardware_info: Option<HardwareInfoSpec>,

    #[serde(rename = "subComponent")]
    pub subcomponent: Option<SubcomponentSpec>,

    #[serde(rename = "metadata")]
    pub metadata: Option<Map<String, Value>>,
}

/// Low-level model for the `measurementSeriesEnd` spec object.
/// End marker for a time based series of measurements.
/// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#measurementseriesend
/// schema url: https://github.com/opencomputeproject/ocp-diag-core/blob/main/json_spec/output/measurement_series_end.json
/// schema ref: https://github.com/opencomputeproject/ocp-diag-core/measurementSeriesEnd
#[derive(Debug, Serialize, PartialEq, Clone)]
#[serde(rename = "measurementSeriesEnd")]
pub struct MeasurementSeriesEndSpec {
    #[serde(rename = "measurementSeriesId")]
    pub series_id: String,

    #[serde(rename = "totalCount")]
    pub total_count: u64,
}

/// Low-level model for the `measurementSeriesElement` spec object.
/// Equivalent to the `Measurement` model but inside a time based series.
/// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#measurementserieselement
/// schema url: https://github.com/opencomputeproject/ocp-diag-core/blob/main/json_spec/output/measurement_series_element.json
/// schema ref: https://github.com/opencomputeproject/ocp-diag-core/measurementSeriesElement
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename = "measurementSeriesElement")]
pub struct MeasurementSeriesElementSpec {
    #[serde(rename = "index")]
    pub index: u64,

    #[serde(rename = "value")]
    pub value: Value,

    #[serde(with = "rfc3339_format")]
    pub timestamp: DateTime<chrono_tz::Tz>,

    #[serde(rename = "measurementSeriesId")]
    pub series_id: String,

    #[serde(rename = "metadata")]
    pub metadata: Option<Map<String, Value>>,
}

/// Low-level model for the `diagnosis` spec object.
/// Contains the verdict given by the diagnostic regarding the DUT that was inspected.
/// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#diagnosis
/// schema url: https://github.com/opencomputeproject/ocp-diag-core/blob/main/json_spec/output/diagnosis.json
/// schema ref: https://github.com/opencomputeproject/ocp-diag-core/diagnosis
#[derive(Debug, Serialize, PartialEq, Clone)]
#[serde(rename = "diagnosis")]
pub struct DiagnosisSpec {
    #[serde(rename = "verdict")]
    pub verdict: String,

    #[serde(rename = "type")]
    pub diagnosis_type: DiagnosisType,

    #[serde(rename = "message")]
    pub message: Option<String>,

    #[serde(rename = "validators")]
    pub hardware_info: Option<HardwareInfoSpec>,

    #[serde(rename = "subComponent")]
    pub subcomponent: Option<SubcomponentSpec>,

    #[serde(rename = "sourceLocation")]
    pub source_location: Option<SourceLocationSpec>,
}

/// Low-level model for the `file` spec object.
/// Represents a file artifact that was generated by running the diagnostic.
/// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#file
/// schema url: https://github.com/opencomputeproject/ocp-diag-core/blob/main/json_spec/output/file.json
/// schema ref: https://github.com/opencomputeproject/ocp-diag-core/file
#[derive(Debug, Serialize, PartialEq, Clone)]
#[serde(rename = "file")]
pub struct FileSpec {
    #[serde(rename = "name")]
    pub name: String,

    #[serde(rename = "uri")]
    pub uri: String,

    #[serde(rename = "isSnapshot")]
    pub is_snapshot: bool,

    #[serde(rename = "description")]
    pub description: Option<String>,

    #[serde(rename = "contentType")]
    pub content_type: Option<String>,

    #[serde(rename = "metadata")]
    pub metadata: Option<Map<String, Value>>,
}

/// Low-level model for the `extension` spec object.
/// Left as an implementation detail, the `Extension` just has a name and arbitrary data.
/// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#extension
/// schema url: https://github.com/opencomputeproject/ocp-diag-core/blob/main/json_spec/output/test_step_artifact.json
/// schema ref: https://github.com/opencomputeproject/ocp-diag-core/testStepArtifact/$defs/extension
#[derive(Debug, Serialize, PartialEq, Clone)]
#[serde(rename = "extension")]
pub struct ExtensionSpec {
    #[serde(rename = "name")]
    pub name: String,

    #[serde(rename = "content")]
    pub content: ExtensionContentType,
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use assert_json_diff::assert_json_include;
    use chrono::SecondsFormat;
    use serde_json::json;

    use super::*;

    #[test]
    fn test_rfc3339_format_serialize() -> Result<()> {
        let test_date = "2022-01-01T00:00:00.000Z";
        let msr = MeasurementSeriesElementSpec {
            index: 0,
            value: 1.0.into(),
            timestamp: DateTime::parse_from_rfc3339(test_date)?.with_timezone(&chrono_tz::UTC),
            series_id: "test".to_string(),
            metadata: None,
        };
        let json = serde_json::to_value(msr)?;
        assert_json_include!(actual: json, expected: json!({
            "timestamp": test_date,
        }));

        Ok(())
    }

    #[test]
    fn test_rfc3339_format_deserialize() -> Result<()> {
        let test_date = "2022-01-01T00:00:00.000Z";
        let json = json!({"index":0,"measurementSeriesId":"test","metadata":null,"timestamp":"2022-01-01T00:00:00.000Z","value":1.0});

        let msr = serde_json::from_value::<MeasurementSeriesElementSpec>(json)?;
        assert_eq!(
            msr.timestamp.to_rfc3339_opts(SecondsFormat::Millis, true),
            test_date
        );

        Ok(())
    }
}
