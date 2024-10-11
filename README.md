# ocp-diag-core-rust

[![codecov](https://codecov.io/github/opencomputeproject/ocp-diag-core-rust/graph/badge.svg?token=IJOG0T8XZ3)](https://codecov.io/github/opencomputeproject/ocp-diag-core-rust)

The **OCP Test & Validation Initiative** is a collaboration between datacenter hyperscalers having the goal of standardizing aspects of the hardware validation/diagnosis space, along with providing necessary tooling to enable both diagnostic developers and executors to leverage these interfaces.

Specifically, the [ocp-diag-core-rust](https://github.com/opencomputeproject/ocp-diag-core-rust) project makes it easy for developers to use the **OCP Test & Validation specification** artifacts by presenting a pure-rust api that can be used to output spec compliant JSON messages.

To start, please see below for [installation instructions](https://github.com/opencomputeproject/ocp-diag-core-rust#installation) and [usage](https://github.com/opencomputeproject/ocp-diag-core-rust#usage).

This project is part of [ocp-diag-core](https://github.com/opencomputeproject/ocp-diag-core) and exists under the same [MIT License Agreement](https://github.com/opencomputeproject/ocp-diag-core-rust/LICENSE).

### Installation

Stable releases of the **ocp-diag-core-rust** codebase are published to **crates.io** under the package name [ocptv](https://crates.io/crates/ocptv), and can be easily installed with rust package manager.

For general usage, the following steps should be sufficient to get the latest stable version using the [Package Installer for Rust](https://github.com/rust-lang/cargo):

- **\[option 1]** using `cargo add`

    ```bash
    $ cargo add ocptv
    ```

- **\[option 2]** specifying it in Cargo.toml file

    
    ```toml
    [dependencies]
    ocptv = "1.0.0"
    ```
    and then run
    ```bash
    $ cargo update
    ```

To use the bleeding edge instead of the stable version, the git repository should be cloned.
This assumes that the clone is manually kept up to date by git pulling whenever there are new commits upstream. All of the installation methods below will automatically use the latest library code.

First clone the upstream latest code:
```bash
$ git clone https://github.com/opencomputeproject/ocp-diag-core-rust.git
$ cd ocp-diag-core-rust
$ git checkout dev # dev branch has the latest code
```
Then edit Cargo.toml in your project and add

```
[dependencies]
ocptv = { version = "1.0.0", path = "/path/to/ocp-diag-core-rust" }
```

The instructions above assume a Linux-type system. However, the steps should be identical on Windows and MacOS platforms.

See [The Cargo Book](https://doc.rust-lang.org/cargo/index.html) for more details on how to use cargo.

### Usage

The specification does not impose any particular level of usage. To be compliant, a diagnostic package just needs output the correct artifact messages in the correct format. However, any particular such diagnostic is free to choose what aspects it needs to use/output; eg. a simple validation test may not output any measurements, opting to just have a final Diagnosis outcome.

**Full API reference is available [here](https://docs.rs/ocptv).**

A very simple starter example, which just outputs a diagnosis:
```rust
use anyhow::Result;
use futures::FutureExt;

use ocptv::output as tv;
use ocptv::{ocptv_diagnosis_fail, ocptv_diagnosis_pass};
use rand::Rng;
use tv::{TestResult, TestStatus};

fn get_fan_speed() -> i32 {
    let mut rng = rand::thread_rng();
    rng.gen_range(1500..1700)
}

async fn run_diagnosis_step(step: &tv::StartedTestStep) -> Result<TestStatus, tv::OcptvError> {
    let fan_speed = get_fan_speed();

    if fan_speed >= 1600 {
        step.diagnosis("fan_ok", tv::DiagnosisType::Pass).await?;
    } else {
        step.diagnosis("fan_low", tv::DiagnosisType::Fail).await?;
    }

    Ok(TestStatus::Complete)
}

async fn run_diagnosis_macros_step(
    step: &tv::StartedTestStep,
) -> Result<TestStatus, tv::OcptvError> {
    let fan_speed = get_fan_speed();

    if fan_speed >= 1600 {
        ocptv_diagnosis_pass!(step, "fan_ok").await?;
    } else {
        ocptv_diagnosis_fail!(step, "fan_low").await?;
    }

    Ok(TestStatus::Complete)
}

#[tokio::main]
async fn main() -> Result<()> {
    let dut = tv::DutInfo::builder("dut0").build();

    tv::TestRun::builder("simple measurement", "1.0")
        .build()
        .scope(dut, |r| {
            async move {
                /// add diagnosis without source location
                r.add_step("step0")
                    .scope(|s| run_diagnosis_step(s).boxed())
                    .await?;
                /// using the macro, the source location is filled automatically
                r.add_step("step1")
                    .scope(|s| run_diagnosis_macros_step(s).boxed())
                    .await?;

                Ok(tv::TestRunOutcome {
                    status: TestStatus::Complete,
                    result: TestResult::Pass,
                })
            }
            .boxed()
        })
        .await?;

    Ok(())
}
```

Expected output (slightly reformatted for readability):

```json
{"sequenceNumber":0, "schemaVersion":{"major":2,"minor":0},"timestamp":"2024-10-11T11:39:07.026Z"}

{"sequenceNumber":1,"testRunArtifact":{
  "testRunStart":{
    "name":"simple diagnosis", 
    "commandLine":"","parameters":{},"version":"1.0", 
    "dutInfo":{"dutInfoId":"dut0"}
  }},"timestamp":"2024-10-11T11:39:07.026Z"}

{"sequenceNumber":2,"testStepArtifact":{
  "testStepId":"step0","testStepStart":{"name":"step0"}
  },"timestamp":"2024-10-11T11:39:07.026Z"}

{"sequenceNumber":3,"testStepArtifact":{
  "diagnosis":{"type":"PASS","verdict":"fan_ok"},
  "testStepId":"step0"},"timestamp":"2024-10-11T11:39:07.027Z"}

{"sequenceNumber":4,"testStepArtifact":{
  "testStepEnd":{"status":"COMPLETE"},"testStepId":"step0"
  },"timestamp":"2024-10-11T11:39:07.027Z"}

{"sequenceNumber":5,"testStepArtifact":{
  "testStepId":"step1","testStepStart":{"name":"step1"}
  },"timestamp":"2024-10-11T11:39:07.027Z"}

{"sequenceNumber":6,"testStepArtifact":{
  "diagnosis":{
    "sourceLocation":{"file":"examples/diagnosis.rs","line":40},
    "type":"FAIL","verdict":"fan_low"
    },"testStepId":"step1"
  },"timestamp":"2024-10-11T11:39:07.027Z"}

{"sequenceNumber":7,"testStepArtifact":{
  "testStepEnd":{"status":"COMPLETE"},"testStepId":"step1"
  },"timestamp":"2024-10-11T11:39:07.027Z"}

{"sequenceNumber":8,"testRunArtifact":{
  "testRunEnd":{"result":"PASS","status":"COMPLETE"}
  },"timestamp":"2024-10-11T11:39:07.027Z"}

```

### Examples

The examples in [examples folder](https://github.com/opencomputeproject/ocp-diag-core-rust/tree/dev/examples) could be run using cargo.

This is one of the example configured in Cargo.toml:

```toml
[[example]]
name = "diagnosis"
required-features = ["boxed-scopes"]
```

```bash
# run diagnosis example
$ cargo run --example diagnosis --features="boxed-scopes"
```

### Developer notes

If you would like to contribute, please head over to [developer notes](https://github.com/opencomputeproject/ocp-diag-core-rust/tree/dev/DEVELOPERS.md) for instructions.

### Contact

Feel free to start a new [discussion](https://github.com/opencomputeproject/ocp-diag-core-rust/discussions), or otherwise post an [issue/request](https://github.com/opencomputeproject/ocp-diag-core-rust/issues).

An email contact is also available at: ocp-test-validation@OCP-All.groups.io

<!--
due to https://github.com/pypa/readme_renderer/issues/163 we must use absolute links everywhere
-->