// (c) Meta Platforms, Inc. and affiliates.
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

use anyhow::Result;
use serde::Serialize;

use ocptv::output as tv;
use tv::{TestResult, TestStatus};

#[derive(Serialize)]
enum ExtensionType {
    Example,
}

#[derive(Serialize)]
struct ComplexExtension {
    #[serde(rename = "@type")]
    ext_type: ExtensionType,

    field: String,
    subtypes: Vec<u32>,
}

async fn step0(s: tv::ScopedTestStep) -> Result<TestStatus, tv::OcptvError> {
    s.add_extension("simple", "extension_identifier").await?;

    s.add_extension(
        "complex",
        ComplexExtension {
            ext_type: ExtensionType::Example,
            field: "demo".to_owned(),
            subtypes: vec![1, 42],
        },
    )
    .await?;

    Ok(TestStatus::Complete)
}

/// Showcase how to output a vendor specific test step extension.
#[tokio::main]
async fn main() -> Result<()> {
    let dut = tv::DutInfo::builder("dut0").build();

    tv::TestRun::builder("extensions", "1.0")
        .build()
        .scope(dut, |r| async move {
            r.add_step("step0").scope(step0).await?;

            Ok(tv::TestRunOutcome {
                status: TestStatus::Complete,
                result: TestResult::Pass,
            })
        })
        .await?;

    Ok(())
}
