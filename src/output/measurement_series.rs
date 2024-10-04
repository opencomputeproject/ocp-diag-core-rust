// (c) Meta Platforms, Inc. and affiliates.
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

use std::future::Future;
use std::sync::atomic;
use std::sync::Arc;

use serde_json::Map;
use serde_json::Value;
use tokio::sync::Mutex;

use crate::output as tv;
use tv::{emitters, objects, state};

/// The measurement series.
/// A Measurement Series is a time-series list of measurements.
///
/// ref: https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec#measurementseriesstart
pub struct MeasurementSeries {
    state: Arc<Mutex<state::TestState>>,
    seq_no: Arc<Mutex<atomic::AtomicU64>>,
    start: objects::MeasurementSeriesStart,
}

impl MeasurementSeries {
    pub fn new(series_id: &str, name: &str, state: Arc<Mutex<state::TestState>>) -> Self {
        Self {
            state,
            seq_no: Arc::new(Mutex::new(atomic::AtomicU64::new(0))),
            start: objects::MeasurementSeriesStart::new(name, series_id),
        }
    }

    pub fn new_with_details(
        start: objects::MeasurementSeriesStart,
        state: Arc<Mutex<state::TestState>>,
    ) -> Self {
        Self {
            state,
            seq_no: Arc::new(Mutex::new(atomic::AtomicU64::new(0))),
            start,
        }
    }

    async fn current_sequence_no(&self) -> u64 {
        self.seq_no.lock().await.load(atomic::Ordering::SeqCst)
    }

    async fn increment_sequence_no(&self) {
        self.seq_no
            .lock()
            .await
            .fetch_add(1, atomic::Ordering::SeqCst);
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
    pub async fn start(&self) -> Result<(), emitters::WriterError> {
        self.state
            .lock()
            .await
            .emitter
            .emit(&self.start.to_artifact())
            .await?;
        Ok(())
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
    /// let series = step.measurement_series("name");
    /// series.start().await?;
    /// series.end().await?;
    ///
    /// # Ok::<(), WriterError>(())
    /// # });
    /// ```
    pub async fn end(&self) -> Result<(), emitters::WriterError> {
        let end = objects::MeasurementSeriesEnd::new(
            self.start.get_series_id(),
            self.current_sequence_no().await,
        );
        self.state
            .lock()
            .await
            .emitter
            .emit(&end.to_artifact())
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
    /// let series = step.measurement_series("name");
    /// series.start().await?;
    /// series.add_measurement(60.into()).await?;
    ///
    /// # Ok::<(), WriterError>(())
    /// # });
    /// ```
    pub async fn add_measurement(&self, value: Value) -> Result<(), emitters::WriterError> {
        let element = objects::MeasurementSeriesElement::new(
            self.current_sequence_no().await,
            value,
            &self.start,
            None,
        );
        self.increment_sequence_no().await;
        self.state
            .lock()
            .await
            .emitter
            .emit(&element.to_artifact())
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
    /// let series = step.measurement_series("name");
    /// series.start().await?;
    /// series.add_measurement_with_metadata(60.into(), vec![("key", "value".into())]).await?;
    ///
    /// # Ok::<(), WriterError>(())
    /// # });
    /// ```
    pub async fn add_measurement_with_metadata(
        &self,
        value: Value,
        metadata: Vec<(&str, Value)>,
    ) -> Result<(), emitters::WriterError> {
        let element = objects::MeasurementSeriesElement::new(
            self.current_sequence_no().await,
            value,
            &self.start,
            Some(Map::from_iter(
                metadata.iter().map(|(k, v)| (k.to_string(), v.clone())),
            )),
        );
        self.increment_sequence_no().await;
        self.state
            .lock()
            .await
            .emitter
            .emit(&element.to_artifact())
            .await?;
        Ok(())
    }

    /// Builds a scope in the [`MeasurementSeries`] object, taking care of starting and
    /// ending it. View [`MeasurementSeries::start`] and [`MeasurementSeries::end`] methods.
    /// After the scope is constructed, additional objects may be added to it.
    /// This is the preferred usage for the [`MeasurementSeries`], since it guarantees
    /// all the messages are emitted between the start and end messages, the order
    /// is respected and no messages is lost.
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
    /// series.scope(|s| async {
    ///     s.add_measurement(60.into()).await?;
    ///     s.add_measurement(70.into()).await?;
    ///     s.add_measurement(80.into()).await?;
    ///     Ok(())
    /// }).await?;
    ///
    /// # Ok::<(), WriterError>(())
    /// # });
    /// ```
    pub async fn scope<'a, F, R>(&'a self, func: F) -> Result<(), emitters::WriterError>
    where
        R: Future<Output = Result<(), emitters::WriterError>>,
        F: std::ops::FnOnce(&'a MeasurementSeries) -> R,
    {
        self.start().await?;
        func(self).await?;
        self.end().await?;
        Ok(())
    }
}
