// (c) Meta Platforms, Inc. and affiliates.
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

use std::sync::atomic::{self, Ordering};
use std::sync::Arc;

use serde_json::Map;
use serde_json::Value;

use crate::output as tv;
use crate::spec;
use tv::{dut, emitter, step};

/// The measurement series.
/// A Measurement Series is a time-series list of measurements.
///
/// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#measurementseriesstart
pub struct MeasurementSeries {
    start: MeasurementSeriesStart,

    emitter: Arc<step::StepEmitter>,
}

impl MeasurementSeries {
    pub(crate) fn new(series_id: &str, name: &str, emitter: Arc<step::StepEmitter>) -> Self {
        Self {
            start: MeasurementSeriesStart::new(name, series_id),
            emitter,
        }
    }

    // TODO: we should allow the user to start a series with details, but still have the series id on
    // an auto-generator, since it's more of a spec detail
    pub(crate) fn new_with_details(
        start: MeasurementSeriesStart,
        emitter: Arc<step::StepEmitter>,
    ) -> Self {
        Self { start, emitter }
    }

    /// Starts the measurement series.
    ///
    /// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#measurementseriesstart
    ///
    /// # Examples
    ///
    /// ```rust
    /// # tokio_test::block_on(async {
    /// # use ocptv::output::*;
    ///
    /// let run = TestRun::new("diagnostic_name", "my_dut", "1.0").start().await?;
    /// let step = run.step("step_name").start().await?;
    ///
    /// let series = step.measurement_series("name");
    /// series.start().await?;
    ///
    /// # Ok::<(), WriterError>(())
    /// # });
    /// ```
    pub async fn start(self) -> Result<StartedMeasurementSeries, emitter::WriterError> {
        self.emitter
            .emit(&spec::TestStepArtifactImpl::MeasurementSeriesStart(
                self.start.to_artifact(),
            ))
            .await?;

        Ok(StartedMeasurementSeries {
            parent: self,
            seqno: Arc::new(atomic::AtomicU64::new(0)),
        })
    }

    // /// Builds a scope in the [`MeasurementSeries`] object, taking care of starting and
    // /// ending it. View [`MeasurementSeries::start`] and [`MeasurementSeries::end`] methods.
    // /// After the scope is constructed, additional objects may be added to it.
    // /// This is the preferred usage for the [`MeasurementSeries`], since it guarantees
    // /// all the messages are emitted between the start and end messages, the order
    // /// is respected and no messages is lost.
    // ///
    // /// # Examples
    // ///
    // /// ```rust
    // /// # tokio_test::block_on(async {
    // /// # use ocptv::output::*;
    // ///
    // /// let run = TestRun::new("diagnostic_name", "my_dut", "1.0").start().await?;
    // /// let step = run.step("step_name").start().await?;
    // ///
    // /// let series = step.measurement_series("name");
    // /// series.start().await?;
    // /// series.scope(|s| async {
    // ///     s.add_measurement(60.into()).await?;
    // ///     s.add_measurement(70.into()).await?;
    // ///     s.add_measurement(80.into()).await?;
    // ///     Ok(())
    // /// }).await?;
    // ///
    // /// # Ok::<(), WriterError>(())
    // /// # });
    // /// ```
    // pub async fn scope<'s, F, R>(&'s self, func: F) -> Result<(), emitter::WriterError>
    // where
    //     R: Future<Output = Result<(), emitter::WriterError>>,
    //     F: std::ops::FnOnce(&'s MeasurementSeries) -> R,
    // {
    //     self.start().await?;
    //     func(self).await?;
    //     self.end().await?;
    //     Ok(())
    // }
}

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
    /// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#measurementseriesend
    ///
    /// # Examples
    ///
    /// ```rust
    /// # tokio_test::block_on(async {
    /// # use ocptv::output::*;
    ///
    /// let run = TestRun::new("diagnostic_name", "my_dut", "1.0").start().await?;
    /// let step = run.step("step_name").start().await?;
    ///
    /// let series = step.measurement_series("name").start().await?;
    /// series.end().await?;
    ///
    /// # Ok::<(), WriterError>(())
    /// # });
    /// ```
    pub async fn end(&self) -> Result<(), emitter::WriterError> {
        let end = spec::MeasurementSeriesEnd {
            series_id: self.parent.start.series_id.clone(),
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
    /// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#measurementserieselement
    ///
    /// # Examples
    ///
    /// ```rust
    /// # tokio_test::block_on(async {
    /// # use ocptv::output::*;
    ///
    /// let run = TestRun::new("diagnostic_name", "my_dut", "1.0").start().await?;
    /// let step = run.step("step_name").start().await?;
    ///
    /// let series = step.measurement_series("name").start().await?;
    /// series.add_measurement(60.into()).await?;
    ///
    /// # Ok::<(), WriterError>(())
    /// # });
    /// ```
    pub async fn add_measurement(&self, value: Value) -> Result<(), emitter::WriterError> {
        let element = spec::MeasurementSeriesElement {
            index: self.incr_seqno(),
            value: value.clone(),
            timestamp: self.parent.emitter.timestamp_provider().now(),
            series_id: self.parent.start.series_id.clone(),
            metadata: None,
        };

        self.parent
            .emitter
            .emit(&spec::TestStepArtifactImpl::MeasurementSeriesElement(
                element,
            ))
            .await?;

        Ok(())
    }

    /// Adds a measurement element to the measurement series.
    /// This method accepts additional metadata to add to the element.
    ///
    /// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#measurementserieselement
    ///
    /// # Examples
    ///
    /// ```rust
    /// # tokio_test::block_on(async {
    /// # use ocptv::output::*;
    ///
    /// let run = TestRun::new("diagnostic_name", "my_dut", "1.0").start().await?;
    /// let step = run.step("step_name").start().await?;
    ///
    /// let series = step.measurement_series("name").start().await?;
    /// series.add_measurement_with_metadata(60.into(), vec![("key", "value".into())]).await?;
    ///
    /// # Ok::<(), WriterError>(())
    /// # });
    /// ```
    pub async fn add_measurement_with_metadata(
        &self,
        value: Value,
        metadata: Vec<(&str, Value)>,
    ) -> Result<(), emitter::WriterError> {
        let element = spec::MeasurementSeriesElement {
            index: self.incr_seqno(),
            value: value.clone(),
            timestamp: self.parent.emitter.timestamp_provider().now(),
            series_id: self.parent.start.series_id.clone(),
            metadata: Some(Map::from_iter(
                metadata.iter().map(|(k, v)| (k.to_string(), v.clone())),
            )),
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

#[derive(Clone)]
pub struct Validator {
    name: Option<String>,
    validator_type: spec::ValidatorType,
    value: Value,
    metadata: Option<Map<String, Value>>,
}

impl Validator {
    pub fn builder(validator_type: spec::ValidatorType, value: Value) -> ValidatorBuilder {
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

#[derive(Debug)]
pub struct ValidatorBuilder {
    name: Option<String>,
    validator_type: spec::ValidatorType,
    value: Value,
    metadata: Option<Map<String, Value>>,
}

impl ValidatorBuilder {
    fn new(validator_type: spec::ValidatorType, value: Value) -> Self {
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
/// let measurement = Measurement::new("name", 50.into());
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
/// let measurement = Measurement::builder("name", 50.into())
///     .hardware_info(&HardwareInfo::builder("id", "name").build())
///     .add_validator(&Validator::builder(ValidatorType::Equal, 30.into()).build())
///     .add_metadata("key", "value".into())
///     .subcomponent(&Subcomponent::builder("name").build())
///     .build();
/// ```
pub struct Measurement {
    name: String,
    value: Value,
    unit: Option<String>,
    validators: Option<Vec<Validator>>,
    hardware_info: Option<dut::HardwareInfo>,
    subcomponent: Option<dut::Subcomponent>,
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
    /// let measurement = Measurement::new("name", 50.into());
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
    /// let measurement = Measurement::builder("name", 50.into())
    ///     .hardware_info(&HardwareInfo::builder("id", "name").build())
    ///     .add_validator(&Validator::builder(ValidatorType::Equal, 30.into()).build())
    ///     .add_metadata("key", "value".into())
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
            hardware_info_id: self
                .hardware_info
                .as_ref()
                .map(|hardware_info| hardware_info.id().to_owned()),
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
/// use ocptv::output::HardwareInfo;
/// use ocptv::output::Measurement;
/// use ocptv::output::MeasurementBuilder;
/// use ocptv::output::Subcomponent;
/// use ocptv::output::Validator;
/// use ocptv::output::ValidatorType;
/// use ocptv::output::Value;
///
/// let builder = MeasurementBuilder::new("name", 50.into())
///     .hardware_info(&HardwareInfo::builder("id", "name").build())
///     .add_validator(&Validator::builder(ValidatorType::Equal, 30.into()).build())
///     .add_metadata("key", "value".into())
///     .subcomponent(&Subcomponent::builder("name").build());
/// let measurement = builder.build();
/// ```
pub struct MeasurementBuilder {
    name: String,
    value: Value,
    unit: Option<String>,
    validators: Option<Vec<Validator>>,
    hardware_info: Option<dut::HardwareInfo>,
    subcomponent: Option<dut::Subcomponent>,
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
    /// let builder = MeasurementBuilder::new("name", 50.into());
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

    /// Add a [`HardwareInfo`] to a [`MeasurementBuilder`].
    ///
    /// # Examples
    ///
    /// ```
    /// use ocptv::output::HardwareInfo;
    /// use ocptv::output::MeasurementBuilder;
    /// use ocptv::output::Value;
    ///
    /// let builder = MeasurementBuilder::new("name", 50.into())
    ///     .hardware_info(&HardwareInfo::builder("id", "name").build());
    /// ```
    pub fn hardware_info(mut self, hardware_info: &dut::HardwareInfo) -> MeasurementBuilder {
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
    /// use ocptv::output::MeasurementBuilder;
    /// use ocptv::output::Value;
    ///
    /// let builder =
    ///     MeasurementBuilder::new("name", 50.into()).add_metadata("key", "value".into());
    /// ```
    pub fn add_metadata(mut self, key: &str, value: Value) -> MeasurementBuilder {
        match self.metadata {
            Some(ref mut metadata) => {
                metadata.insert(key.to_string(), value.clone());
            }
            None => {
                let mut entries = serde_json::Map::new();
                entries.insert(key.to_owned(), value);
                self.metadata = Some(entries);
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

pub struct MeasurementSeriesStart {
    name: String,
    unit: Option<String>,
    series_id: String,
    validators: Option<Vec<Validator>>,
    hardware_info: Option<dut::HardwareInfo>,
    subcomponent: Option<dut::Subcomponent>,
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

    pub fn to_artifact(&self) -> spec::MeasurementSeriesStart {
        spec::MeasurementSeriesStart {
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
        }
    }
}

pub struct MeasurementSeriesStartBuilder {
    name: String,
    unit: Option<String>,
    series_id: String,
    validators: Option<Vec<Validator>>,
    hardware_info: Option<dut::HardwareInfo>,
    subcomponent: Option<dut::Subcomponent>,
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

    pub fn hardware_info(
        mut self,
        hardware_info: &dut::HardwareInfo,
    ) -> MeasurementSeriesStartBuilder {
        self.hardware_info = Some(hardware_info.clone());
        self
    }

    pub fn subcomponent(
        mut self,
        subcomponent: &dut::Subcomponent,
    ) -> MeasurementSeriesStartBuilder {
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
        let value = Value::from(50);
        let measurement = Measurement::new(&name, value.clone());

        let artifact = measurement.to_artifact();
        assert_eq!(
            artifact,
            spec::Measurement {
                name: name.to_string(),
                unit: None,
                value,
                validators: None,
                hardware_info_id: None,
                subcomponent: None,
                metadata: None,
            }
        );

        Ok(())
    }

    #[test]
    fn test_measurement_builder_as_test_step_descendant_to_artifact() -> Result<()> {
        let name = "name".to_owned();
        let value = Value::from(50000);
        let hardware_info = HardwareInfo::builder("id", "name").build();
        let validator = Validator::builder(spec::ValidatorType::Equal, 30.into()).build();

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
            spec::Measurement {
                name,
                unit: Some(unit.to_string()),
                value,
                validators: Some(vec![validator.to_spec(), validator.to_spec()]),
                hardware_info_id: Some(hardware_info.to_spec().id.clone()),
                subcomponent: Some(subcomponent.to_spec()),
                metadata: Some(metadata),
            }
        );

        Ok(())
    }

    #[test]
    fn test_measurement_series_start_to_artifact() -> Result<()> {
        let name = "name".to_owned();
        let series_id = "series_id".to_owned();
        let series = MeasurementSeriesStart::new(&name, &series_id);

        let artifact = series.to_artifact();
        assert_eq!(
            artifact,
            spec::MeasurementSeriesStart {
                name: name.to_string(),
                unit: None,
                series_id: series_id.to_string(),
                validators: None,
                hardware_info: None,
                subcomponent: None,
                metadata: None,
            }
        );

        Ok(())
    }

    #[test]
    fn test_measurement_series_start_builder_to_artifact() -> Result<()> {
        let name = "name".to_owned();
        let series_id = "series_id".to_owned();
        let validator = Validator::builder(spec::ValidatorType::Equal, 30.into()).build();
        let validator2 = Validator::builder(spec::ValidatorType::GreaterThen, 10.into()).build();
        let hw_info = HardwareInfo::builder("id", "name").build();
        let subcomponent = Subcomponent::builder("name").build();
        let series = MeasurementSeriesStart::builder(&name, &series_id)
            .unit("unit")
            .add_metadata("key", "value".into())
            .add_metadata("key2", "value2".into())
            .add_validator(&validator)
            .add_validator(&validator2)
            .hardware_info(&hw_info)
            .subcomponent(&subcomponent)
            .build();

        let artifact = series.to_artifact();
        assert_eq!(
            artifact,
            spec::MeasurementSeriesStart {
                name,
                unit: Some("unit".to_string()),
                series_id: series_id.to_string(),
                validators: Some(vec![validator.to_spec(), validator2.to_spec()]),
                hardware_info: Some(hw_info.to_spec()),
                subcomponent: Some(subcomponent.to_spec()),
                metadata: Some(serde_json::Map::from_iter([
                    ("key".to_string(), "value".into()),
                    ("key2".to_string(), "value2".into())
                ])),
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
