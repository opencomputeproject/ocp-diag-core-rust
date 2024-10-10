// (c) Meta Platforms, Inc. and affiliates.
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

use anyhow::Result;
use futures::FutureExt;

use ocptv::output as tv;
use tv::{TestResult, TestStatus};

/// Show a context-scoped run that automatically exits the whole func
/// because of the marker exception that triggers SKIP outcome.
#[tokio::main]
async fn main() -> Result<()> {
    let dut = tv::DutInfo::builder("dut0").build();

    tv::TestRun::builder("run skip", "1.0")
        .build()
        .scope(dut, |_r| {
            async move {
                // intentional short return
                return Ok(tv::TestRunOutcome {
                    status: TestStatus::Skip,
                    result: TestResult::NotApplicable,
                });
            }
            .boxed()
        })
        .await?;

    Ok(())
}
