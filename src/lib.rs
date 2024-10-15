// (c) Meta Platforms, Inc. and affiliates.
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

//! The **OCP Test & Validation Initiative** is a collaboration between datacenter hyperscalers having the goal of standardizing aspects of the hardware validation/diagnosis space, along with providing necessary tooling to enable both diagnostic developers and executors to leverage these interfaces.
//!
//! Specifically, the [ocp-diag-core-rust](https://github.com/opencomputeproject/ocp-diag-core-rust) project makes it easy for developers to use the **OCP Test & Validation specification** artifacts by presenting a pure-rust api that can be used to output spec compliant JSON messages.
//!
//! To start, please see below for [installation instructions](https://github.com/opencomputeproject/ocp-diag-core-rust#installation) and [usage](https://github.com/opencomputeproject/ocp-diag-core-rust#usage).
//!
//! This project is part of [ocp-diag-core](https://github.com/opencomputeproject/ocp-diag-core) and exists under the same [MIT License Agreement](https://github.com/opencomputeproject/ocp-diag-core-rust/LICENSE).
//!
//! ### Usage
//!
//! The [specification](https://github.com/opencomputeproject/ocp-diag-core/tree/main/json_spec) does not impose any particular level of usage. To be compliant, a diagnostic package just needs output the correct artifact messages in the correct format. However, any particular such diagnostic is free to choose what aspects it needs to use/output; eg. a simple validation test may not output any measurements, opting to just have a final Diagnosis outcome.
//!
//!   A very simple starter example, which just outputs a diagnosis:
//!   ```rust
//!   use anyhow::Result;
//!   
//!   use ocptv::output as tv;
//!   use ocptv::{ocptv_diagnosis_fail, ocptv_diagnosis_pass};
//!   use rand::Rng;
//!   use tv::{TestResult, TestStatus};
//!   
//!   fn get_fan_speed() -> i32 {
//!       let mut rng = rand::thread_rng();
//!       rng.gen_range(1500..1700)
//!   }
//!   
//!   async fn run_diagnosis_step(step: tv::ScopedTestStep) -> Result<TestStatus, tv::OcptvError> {
//!       let fan_speed = get_fan_speed();
//!   
//!       if fan_speed >= 1600 {
//!           step.add_diagnosis("fan_ok", tv::DiagnosisType::Pass).await?;
//!       } else {
//!           step.add_diagnosis("fan_low", tv::DiagnosisType::Fail).await?;
//!       }
//!   
//!       Ok(TestStatus::Complete)
//!   }
//!   
//!   async fn run_diagnosis_macros_step(step: tv::ScopedTestStep) -> Result<TestStatus, tv::OcptvError> {
//!       let fan_speed = get_fan_speed();
//!   
//!       /// using the macro, the source location is filled automatically
//!       if fan_speed >= 1600 {
//!           ocptv_diagnosis_pass!(step, "fan_ok").await?;
//!       } else {
//!           ocptv_diagnosis_fail!(step, "fan_low").await?;
//!       }
//!   
//!       Ok(TestStatus::Complete)
//!   }
//!   
//!   #[tokio::main]
//!   async fn main() -> Result<()> {
//!       let dut = tv::DutInfo::builder("dut0").build();
//!   
//!       tv::TestRun::builder("simple measurement", "1.0")
//!           .build()
//!           .scope(dut, |r| async move {
//!               r.add_step("step0")
//!                   .scope(run_diagnosis_step)
//!                   .await?;
//!   
//!               r.add_step("step1")
//!                   .scope(run_diagnosis_macros_step)
//!                   .await?;
//!   
//!               Ok(tv::TestRunOutcome {
//!                   status: TestStatus::Complete,
//!                   result: TestResult::Pass,
//!               })
//!           })
//!           .await?;
//!   
//!       Ok(())
//!   }
//!   ```

pub mod output;
mod spec;
