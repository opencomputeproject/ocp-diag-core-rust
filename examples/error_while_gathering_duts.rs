// (c) Meta Platforms, Inc. and affiliates.
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

use anyhow::Result;

use ocptv::ocptv_error;
use ocptv::output as tv;
use tv::TestRun;

/// In case of failure to discover DUT hardware before needing to present it at test run
/// start, we can error out right at the beginning since no Diagnosis can be produced.
/// This is a framework failure.
#[tokio::main]
async fn main() -> Result<()> {
    let run = TestRun::builder("error while gathering duts", "1.0").build();
    ocptv_error!(run, "no-dut", "could not find any valid DUTs").await?;

    Ok(())
}
