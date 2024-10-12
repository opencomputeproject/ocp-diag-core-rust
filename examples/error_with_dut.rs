// (c) Meta Platforms, Inc. and affiliates.
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

use anyhow::Result;

use ocptv::output as tv;
use tv::{SoftwareType, TestResult, TestStatus};

/// Show outputting an error message, triggered by a specific software component of the DUT.
#[tokio::main]
async fn main() -> Result<()> {
    let mut dut = tv::DutInfo::builder("dut0").name("dut0.server.net").build();
    let sw_info = dut.add_software_info(
        tv::SoftwareInfo::builder("bmc")
            .software_type(SoftwareType::Firmware)
            .version("2.5")
            .build(),
    );

    #[cfg(feature = "boxed-scopes")]
    tv::TestRun::builder("run error with dut", "1.0")
        .build()
        .scope(dut, |r| async move {
            r.add_error_detail(
                tv::Error::builder("power-fail")
                    .add_software_info(&sw_info)
                    .build(),
            )
            .await?;

            Ok(tv::TestRunOutcome {
                status: TestStatus::Complete,
                result: TestResult::Fail,
            })
        })
        .await?;

    Ok(())
}
