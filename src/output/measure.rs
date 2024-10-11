// (c) Meta Platforms, Inc. and affiliates.
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

use std::collections::BTreeMap;
use std::sync::atomic::{self, Ordering};
use std::sync::Arc;

#[cfg(feature = "boxed-scopes")]
use futures::future::BoxFuture;
use maplit::{btreemap, convert_args};

use crate::output as tv;
use crate::spec;
use tv::{dut, step, Ident};

use super::trait_ext::VecExt;

/// The measurement series.
/// A Measurement Series is a time-series list of measurements.
///
/// ref: <https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#measurementseriesstart>
pub struct MeasurementSeries {
    id: String,
    info: MeasurementSeriesInfo,

    emitter: Arc<step::StepEmitter>,
}

impl MeasurementSeries {
    // note: this object is crate public but users should only construct
    // instances through the `StartedTestStep.add_measurement_series_*` apis
    pub(crate) fn new(
        series_id: &str,
        info: MeasurementSeriesInfo,
        emitter: Arc<step::StepEmitter>,
    ) -> Self {
        Self {
            id: series_id.to_owned(),
            info,
            emitter,
        }
    }

    /// Starts the measurement series.
    ///
    /// ref: <https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#measurementseriesstart>
    ///
    /// # Examples
    ///
    /// ```rust
    /// # tokio_test::block_on(async {
    /// # use ocptv::output::*;
    /// let dut = DutInfo::new("my_dut");
    /// let run = TestRun::new("diagnostic_name", "1.0").start(dut).await?;
    /// let step = run.add_step("step_name").start().await?;
    ///
    /// let series = step.add_measurement_series("name");
    /// series.start().await?;
    ///
    /// # Ok::<(), OcptvError>(())
    /// # });
    /// ```
    pub async fn start(self) -> Result<StartedMeasurementSeries, tv::OcptvError> {
        let info = &self.info;

        let start = spec::MeasurementSeriesStart {
            name: info.name.clone(),
            unit: info.unit.clone(),
            series_id: self.id.clone(),
            validators: info.validators.map_option(Validator::to_spec),
            hardware_info: info
                .hardware_info
                .as_ref()
                .map(dut::DutHardwareInfo::to_spec),
            subcomponent: info.subcomponent.as_ref().map(dut::Subcomponent::to_spec),
            metadata: info.metadata.clone(),
        };

        self.emitter
            .emit(&spec::TestStepArtifactImpl::MeasurementSeriesStart(start))
            .await?;

        Ok(StartedMeasurementSeries {
            parent: self,
            seqno: Arc::new(atomic::AtomicU64::new(0)),
        })
    }

    /// Builds a scope in the [`MeasurementSeries`] object, taking care of starting and
    /// ending it. View [`MeasurementSeries::start`] and [`StartedMeasurementSeries::end`] methods.
    /// After the scope is constructed, additional objects may be added to it.
    /// This is the preferred usage for the [`MeasurementSeries`], since it guarantees
    /// all the messages are emitted between the start and end messages, the order
    /// is respected and no messages is lost.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # tokio_test::block_on(async {
    /// # use futures::FutureExt;
    /// # use ocptv::output::*;
    /// let dut = DutInfo::new("my_dut");
    /// let run = TestRun::new("diagnostic_name", "1.0").start(dut).await?;
    /// let step = run.add_step("step_name").start().await?;
    ///
    /// let series = step.add_measurement_series("name");
    /// series.scope(|s| {
    ///     async move {
    ///         s.add_measurement(60.into()).await?;
    ///         s.add_measurement(70.into()).await?;
    ///         s.add_measurement(80.into()).await?;
    ///         Ok(())
    ///     }.boxed()
    /// }).await?;
    ///
    /// # Ok::<(), OcptvError>(())
    /// # });
    /// ```
    #[cfg(feature = "boxed-scopes")]
    pub async fn scope<F>(self, func: F) -> Result<(), tv::OcptvError>
    where
        F: FnOnce(&StartedMeasurementSeries) -> BoxFuture<'_, Result<(), tv::OcptvError>>,
    {
        let series = self.start().await?;
        func(&series).await?;
        series.end().await?;

        Ok(())
    }
}

/// TODO: docs
pub struct StartedMeasurementSeries {
    parent: MeasurementSeries,

    seqno: Arc<atomic::AtomicU64>,
}

impl StartedMeasurementSeries {
    fn incr_seqno(&self) -> u64 {
        self.seqno.fetch_add(1, Ordering::AcqRel)
    }

    /// Ends the measurement series.
    ///
    /// ref: <https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#measurementseriesend>
    ///
    /// # Examples
    ///
    /// ```rust
    /// # tokio_test::block_on(async {
    /// # use ocptv::output::*;
    /// let dut = DutInfo::new("my_dut");
    /// let run = TestRun::new("diagnostic_name", "1.0").start(dut).await?;
    /// let step = run.add_step("step_name").start().await?;
    ///
    /// let series = step.add_measurement_series("name").start().await?;
    /// series.end().await?;
    ///
    /// # Ok::<(), OcptvError>(())
    /// # });
    /// ```
    pub async fn end(self) -> Result<(), tv::OcptvError> {
        let end = spec::MeasurementSeriesEnd {
            series_id: self.parent.id.clone(),
            total_count: self.seqno.load(Ordering::Acquire),
        };

        self.parent
            .emitter
            .emit(&spec::TestStepArtifactImpl::MeasurementSeriesEnd(end))
            .await?;

        Ok(())
    }

    /// Adds a measurement element to the measurement series.
    ///
    /// ref: <https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#measurementserieselement>
    ///
    /// # Examples
    ///
    /// ```rust
    /// # tokio_test::block_on(async {
    /// # use ocptv::output::*;
    /// let dut = DutInfo::new("my_dut");
    /// let run = TestRun::new("diagnostic_name", "1.0").start(dut).await?;
    /// let step = run.add_step("step_name").start().await?;
    ///
    /// let series = step.add_measurement_series("name").start().await?;
    /// series.add_measurement(60.into()).await?;
    ///
    /// # Ok::<(), OcptvError>(())
    /// # });
    /// ```
    pub async fn add_measurement(&self, value: tv::Value) -> Result<(), tv::OcptvError> {
        self.add_measurement_with_details(MeasurementSeriesElemDetails {
            value,
            ..Default::default()
        })
        .await
    }

    /// Adds a measurement element to the measurement series.
    /// This method accepts a full set of details for the measurement element.
    ///
    /// ref: <https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#measurementserieselement>
    ///
    /// # Examples
    ///
    /// ```rust
    /// # tokio_test::block_on(async {
    /// # use ocptv::output::*;
    /// let dut = DutInfo::new("my_dut");
    /// let run = TestRun::new("diagnostic_name", "1.0").start(dut).await?;
    /// let step = run.add_step("step_name").start().await?;
    ///
    /// let series = step.add_measurement_series("name").start().await?;
    /// let elem = MeasurementSeriesElemDetails::builder(60.into()).add_metadata("key", "value".into()).build();
    /// series.add_measurement_with_details(elem).await?;
    ///
    /// # Ok::<(), OcptvError>(())
    /// # });
    /// ```
    pub async fn add_measurement_with_details(
        &self,
        details: MeasurementSeriesElemDetails,
    ) -> Result<(), tv::OcptvError> {
        let element = spec::MeasurementSeriesElement {
            index: self.incr_seqno(),
            value: details.value,
            timestamp: details
                .timestamp
                .unwrap_or(self.parent.emitter.timestamp_provider().now()),
            series_id: self.parent.id.clone(),
            metadata: details.metadata,
        };

        self.parent
            .emitter
            .emit(&spec::TestStepArtifactImpl::MeasurementSeriesElement(
                element,
            ))
            .await?;

        Ok(())
    }
}

/// TODO: docs
#[derive(Default)]
pub struct MeasurementSeriesElemDetails {
    value: tv::Value,
    timestamp: Option<chrono::DateTime<chrono_tz::Tz>>,

    metadata: Option<BTreeMap<String, tv::Value>>,
}

impl MeasurementSeriesElemDetails {
    pub fn builder(value: tv::Value) -> MeasurementSeriesElemDetailsBuilder {
        MeasurementSeriesElemDetailsBuilder::new(value)
    }
}

/// TODO: docs
#[derive(Default)]
pub struct MeasurementSeriesElemDetailsBuilder {
    value: tv::Value,
    timestamp: Option<chrono::DateTime<chrono_tz::Tz>>,

    metadata: Option<BTreeMap<String, tv::Value>>,
}

impl MeasurementSeriesElemDetailsBuilder {
    pub fn new(value: tv::Value) -> Self {
        Self {
            value,
            ..Default::default()
        }
    }

    pub fn timestamp(mut self, value: chrono::DateTime<chrono_tz::Tz>) -> Self {
        self.timestamp = Some(value);
        self
    }

    pub fn add_metadata(mut self, key: &str, value: tv::Value) -> Self {
        self.metadata = match self.metadata {
            Some(mut metadata) => {
                metadata.insert(key.to_string(), value);
                Some(metadata)
            }
            None => Some(convert_args!(btreemap!(
                key => value,
            ))),
        };
        self
    }

    pub fn build(self) -> MeasurementSeriesElemDetails {
        MeasurementSeriesElemDetails {
            value: self.value,
            timestamp: self.timestamp,
            metadata: self.metadata,
        }
    }
}

/// TODO: docs
#[derive(Clone)]
pub struct Validator {
    name: Option<String>,
    validator_type: spec::ValidatorType,
    value: tv::Value,
    metadata: Option<BTreeMap<String, tv::Value>>,
}

impl Validator {
    pub fn builder(validator_type: spec::ValidatorType, value: tv::Value) -> ValidatorBuilder {
        ValidatorBuilder::new(validator_type, value)
    }
    pub fn to_spec(&self) -> spec::Validator {
        spec::Validator {
            name: self.name.clone(),
            validator_type: self.validator_type.clone(),
            value: self.value.clone(),
            metadata: self.metadata.clone(),
        }
    }
}

/// TODO: docs
#[derive(Debug)]
pub struct ValidatorBuilder {
    name: Option<String>,
    validator_type: spec::ValidatorType,
    value: tv::Value,
    metadata: Option<BTreeMap<String, tv::Value>>,
}

impl ValidatorBuilder {
    fn new(validator_type: spec::ValidatorType, value: tv::Value) -> Self {
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
    pub fn add_metadata(mut self, key: &str, value: tv::Value) -> ValidatorBuilder {
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

    pub fn build(self) -> Validator {
        Validator {
            name: self.name,
            validator_type: self.validator_type,
            value: self.value,
            metadata: self.metadata,
        }
    }
}

/// This structure represents a Measurement message.
/// ref: <https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#measurement>
///
/// # Examples
///
/// ## Create a Measurement object with the `new` method
///
/// ```
/// # use ocptv::output::*;
/// let measurement = Measurement::new("name", 50.into());
/// ```
///
/// ## Create a Measurement object with the `builder` method
///
/// ```
/// # use ocptv::output::*;
/// let mut dut = DutInfo::new("dut0");
/// let hw_info = dut.add_hardware_info(HardwareInfo::builder("name").build());
///
/// let measurement = Measurement::builder("name", 50.into())
///     .add_validator(&Validator::builder(ValidatorType::Equal, 30.into()).build())
///     .add_metadata("key", "value".into())
///     .hardware_info(&hw_info)
///     .subcomponent(&Subcomponent::builder("name").build())
///     .build();
/// ```
pub struct Measurement {
    name: String,

    value: tv::Value,
    unit: Option<String>,
    validators: Option<Vec<Validator>>,

    hardware_info: Option<dut::DutHardwareInfo>,
    subcomponent: Option<dut::Subcomponent>,

    metadata: Option<BTreeMap<String, tv::Value>>,
}

impl Measurement {
    /// Builds a new Measurement object.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ocptv::output::*;
    /// let measurement = Measurement::new("name", 50.into());
    /// ```
    pub fn new(name: &str, value: tv::Value) -> Self {
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
    /// # use ocptv::output::*;
    ///
    /// let mut dut = DutInfo::new("dut0");
    /// let hw_info = dut.add_hardware_info(HardwareInfo::builder("name").build());
    ///
    /// let measurement = Measurement::builder("name", 50.into())
    ///     .add_validator(&Validator::builder(ValidatorType::Equal, 30.into()).build())
    ///     .add_metadata("key", "value".into())
    ///     .hardware_info(&hw_info)
    ///     .subcomponent(&Subcomponent::builder("name").build())
    ///     .build();
    /// ```
    pub fn builder(name: &str, value: tv::Value) -> MeasurementBuilder {
        MeasurementBuilder::new(name, value)
    }

    /// Creates an artifact from a Measurement object.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ocptv::output::*;
    /// let measurement = Measurement::new("name", 50.into());
    /// let _ = measurement.to_artifact();
    /// ```
    pub fn to_artifact(&self) -> spec::Measurement {
        spec::Measurement {
            name: self.name.clone(),
            unit: self.unit.clone(),
            value: self.value.clone(),
            validators: self
                .validators
                .clone()
                .map(|vals| vals.iter().map(|val| val.to_spec()).collect()),
            hardware_info: self
                .hardware_info
                .as_ref()
                .map(dut::DutHardwareInfo::to_spec),
            subcomponent: self
                .subcomponent
                .as_ref()
                .map(|subcomponent| subcomponent.to_spec()),
            metadata: self.metadata.clone(),
        }
    }
}

/// This structure builds a [`Measurement`] object.
///
/// # Examples
///
/// ```
/// # use ocptv::output::*;
/// let mut dut = DutInfo::new("dut0");
/// let hw_info = dut.add_hardware_info(HardwareInfo::builder("name").build());
///
/// let builder = MeasurementBuilder::new("name", 50.into())
///     .add_validator(&Validator::builder(ValidatorType::Equal, 30.into()).build())
///     .add_metadata("key", "value".into())
///     .hardware_info(&hw_info)
///     .subcomponent(&Subcomponent::builder("name").build());
/// let measurement = builder.build();
/// ```
pub struct MeasurementBuilder {
    name: String,

    value: tv::Value,
    unit: Option<String>,
    validators: Option<Vec<Validator>>,

    hardware_info: Option<dut::DutHardwareInfo>,
    subcomponent: Option<dut::Subcomponent>,

    metadata: Option<BTreeMap<String, tv::Value>>,
}

impl MeasurementBuilder {
    /// Creates a new MeasurementBuilder.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ocptv::output::*;
    /// let builder = MeasurementBuilder::new("name", 50.into());
    /// ```
    pub fn new(name: &str, value: tv::Value) -> Self {
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
    /// # use ocptv::output::*;
    /// let builder = MeasurementBuilder::new("name", 50.into())
    ///     .add_validator(&Validator::builder(ValidatorType::Equal, 30.into()).build());
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

    /// Add a [`tv::HardwareInfo`] to a [`MeasurementBuilder`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use ocptv::output::*;
    /// let mut dut = DutInfo::new("dut0");
    /// let hw_info = dut.add_hardware_info(HardwareInfo::builder("name").build());
    ///
    /// let builder = MeasurementBuilder::new("name", 50.into())
    ///     .hardware_info(&hw_info);
    /// ```
    pub fn hardware_info(mut self, hardware_info: &dut::DutHardwareInfo) -> MeasurementBuilder {
        self.hardware_info = Some(hardware_info.clone());
        self
    }

    /// Add a [`tv::Subcomponent`] to a [`MeasurementBuilder`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use ocptv::output::*;
    /// let builder = MeasurementBuilder::new("name", 50.into())
    ///     .subcomponent(&Subcomponent::builder("name").build());
    /// ```
    pub fn subcomponent(mut self, subcomponent: &dut::Subcomponent) -> MeasurementBuilder {
        self.subcomponent = Some(subcomponent.clone());
        self
    }

    /// Add custom metadata to a [`MeasurementBuilder`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use ocptv::output::*;
    /// let builder =
    ///     MeasurementBuilder::new("name", 50.into()).add_metadata("key", "value".into());
    /// ```
    pub fn add_metadata(mut self, key: &str, value: tv::Value) -> MeasurementBuilder {
        match self.metadata {
            Some(ref mut metadata) => {
                metadata.insert(key.to_string(), value.clone());
            }
            None => {
                self.metadata = Some(convert_args!(btreemap!(
                    key => value,
                )));
            }
        };
        self
    }

    /// Add measurement unit to a [`MeasurementBuilder`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use ocptv::output::*;
    /// let builder = MeasurementBuilder::new("name", 50000.into()).unit("RPM");
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
    /// let builder = MeasurementBuilder::new("name", 50.into());
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

/// TODO: docs
pub struct MeasurementSeriesInfo {
    // note: this object is crate public and we need access to this field
    // when making a new series in `StartedTestStep.add_measurement_series*`
    pub(crate) id: tv::Ident,
    name: String,

    unit: Option<String>,
    validators: Vec<Validator>,

    hardware_info: Option<dut::DutHardwareInfo>,
    subcomponent: Option<dut::Subcomponent>,

    metadata: Option<BTreeMap<String, tv::Value>>,
}

impl MeasurementSeriesInfo {
    pub fn new(name: &str) -> MeasurementSeriesInfo {
        MeasurementSeriesInfoBuilder::new(name).build()
    }

    pub fn builder(name: &str) -> MeasurementSeriesInfoBuilder {
        MeasurementSeriesInfoBuilder::new(name)
    }
}

/// TODO: docs
#[derive(Default)]
pub struct MeasurementSeriesInfoBuilder {
    id: tv::Ident,
    name: String,

    unit: Option<String>,
    validators: Vec<Validator>,

    hardware_info: Option<dut::DutHardwareInfo>,
    subcomponent: Option<dut::Subcomponent>,

    metadata: Option<BTreeMap<String, tv::Value>>,
}

impl MeasurementSeriesInfoBuilder {
    pub fn new(name: &str) -> Self {
        MeasurementSeriesInfoBuilder {
            id: Ident::Auto,
            name: name.to_string(),
            ..Default::default()
        }
    }

    pub fn id(mut self, id: tv::Ident) -> MeasurementSeriesInfoBuilder {
        self.id = id;
        self
    }

    pub fn unit(mut self, unit: &str) -> MeasurementSeriesInfoBuilder {
        self.unit = Some(unit.to_string());
        self
    }

    pub fn add_validator(mut self, validator: &Validator) -> MeasurementSeriesInfoBuilder {
        self.validators.push(validator.clone());
        self
    }

    pub fn hardware_info(
        mut self,
        hardware_info: &dut::DutHardwareInfo,
    ) -> MeasurementSeriesInfoBuilder {
        self.hardware_info = Some(hardware_info.clone());
        self
    }

    pub fn subcomponent(
        mut self,
        subcomponent: &dut::Subcomponent,
    ) -> MeasurementSeriesInfoBuilder {
        self.subcomponent = Some(subcomponent.clone());
        self
    }

    pub fn add_metadata(mut self, key: &str, value: tv::Value) -> MeasurementSeriesInfoBuilder {
        self.metadata = match self.metadata {
            Some(mut metadata) => {
                metadata.insert(key.to_string(), value.clone());
                Some(metadata)
            }
            None => Some(convert_args!(btreemap!(
                key => value
            ))),
        };
        self
    }

    pub fn build(self) -> MeasurementSeriesInfo {
        MeasurementSeriesInfo {
            id: self.id,
            name: self.name,
            unit: self.unit,
            validators: self.validators,
            hardware_info: self.hardware_info,
            subcomponent: self.subcomponent,
            metadata: self.metadata,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output as tv;
    use crate::spec;
    use tv::dut::*;
    use tv::ValidatorType;

    use anyhow::{bail, Result};

    #[test]
    fn test_measurement_as_test_step_descendant_to_artifact() -> Result<()> {
        let name = "name".to_owned();
        let value = tv::Value::from(50);
        let measurement = Measurement::new(&name, value.clone());

        let artifact = measurement.to_artifact();
        assert_eq!(
            artifact,
            spec::Measurement {
                name: name.to_string(),
                unit: None,
                value,
                validators: None,
                hardware_info: None,
                subcomponent: None,
                metadata: None,
            }
        );

        Ok(())
    }

    #[test]
    fn test_measurement_builder_as_test_step_descendant_to_artifact() -> Result<()> {
        let mut dut = DutInfo::new("dut0");

        let name = "name".to_owned();
        let value = tv::Value::from(50000);
        let hw_info = dut.add_hardware_info(HardwareInfo::builder("name").build());
        let validator = Validator::builder(spec::ValidatorType::Equal, 30.into()).build();

        let meta_key = "key";
        let meta_value = tv::Value::from("value");
        let metadata = convert_args!(btreemap!(
            meta_key => meta_value.clone(),
        ));

        let subcomponent = Subcomponent::builder("name").build();

        let unit = "RPM";
        let measurement = Measurement::builder(&name, value.clone())
            .unit(unit)
            .add_validator(&validator)
            .add_validator(&validator)
            .hardware_info(&hw_info)
            .subcomponent(&subcomponent)
            .add_metadata(meta_key, meta_value.clone())
            .build();

        let artifact = measurement.to_artifact();
        assert_eq!(
            artifact,
            spec::Measurement {
                name,
                value,
                unit: Some(unit.to_string()),
                validators: Some(vec![validator.to_spec(), validator.to_spec()]),
                hardware_info: Some(hw_info.to_spec()),
                subcomponent: Some(subcomponent.to_spec()),
                metadata: Some(metadata),
            }
        );

        Ok(())
    }

    #[test]
    fn test_validator() -> Result<()> {
        let validator = Validator::builder(ValidatorType::Equal, 30.into())
            .name("validator")
            .add_metadata("key", "value".into())
            .add_metadata("key2", "value2".into())
            .build();

        let spec_validator = validator.to_spec();

        assert_eq!(spec_validator.name, Some("validator".to_owned()));
        assert_eq!(spec_validator.value, 30);
        assert_eq!(spec_validator.validator_type, ValidatorType::Equal);

        match spec_validator.metadata {
            Some(m) => {
                assert_eq!(m["key"], "value");
                assert_eq!(m["key2"], "value2");
            }
            _ => bail!("metadata is none"),
        }

        Ok(())
    }
}
