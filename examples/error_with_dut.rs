// (c) Meta Platforms, Inc. and affiliates.
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
#![allow(warnings)]

use anyhow::Result;
use futures::FutureExt;

use ocptv::output as tv;
use tv::{DutInfo, TestResult, TestRun, TestStatus};
use tv::{SoftwareInfo, SoftwareType, TestRunOutcome};

/// Show outputting an error message, triggered by a specific software component of the DUT.
#[tokio::main]
async fn main() -> Result<()> {
    let mut dut = DutInfo::builder("dut0").name("dut0.server.net").build();
    let sw_info = dut.add_software_info(
        SoftwareInfo::builder("bmc")
            .software_type(SoftwareType::Firmware)
            .version("2.5")
            .build(),
    );

    #[cfg(feature = "boxed-scopes")]
    TestRun::builder("run error with dut", "1.0")
        .build()
        .scope(dut, |r| {
            async move {
                r.add_error_with_details(
                    &tv::Error::builder("power-fail")
                        .add_software_info(&sw_info)
                        .build(),
                )
                .await?;

                Ok(TestRunOutcome {
                    status: TestStatus::Complete,
                    result: TestResult::Fail,
                })
            }
            .boxed()
        })
        .await?;

    Ok(())
}
