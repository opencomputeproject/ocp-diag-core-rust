// (c) Meta Platforms, Inc. and affiliates.
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

use anyhow::Result;
#[cfg(feature = "boxed-scopes")]
use futures::FutureExt;

#[cfg(feature = "boxed-scopes")]
use ocptv::ocptv_log_info;
use ocptv::{
    ocptv_error, ocptv_log_debug,
    output::{self as tv},
};
use tv::{DutInfo, TestResult, TestRun, TestStatus};
#[cfg(feature = "boxed-scopes")]
use tv::{SoftwareInfo, SoftwareType, TestRunOutcome};

macro_rules! run_demo {
    ($name: ident) => {
        println!("{}", format!("{:->width$}", "", width = 80));
        println!("{}", stringify!($name));
        println!("{}", format!("{:->width$}", "", width = 80));

        let _ = $name().await;
        println!();
    };
}

/// Show that a run/step can be manually started and ended.
///
/// The scope version should be preferred, as it makes it safer not to miss the end
/// artifacts in case of unhandled exceptions or code misuse.
async fn demo_no_scopes() -> Result<()> {
    let dut = DutInfo::builder("dut0").build();
    let run = TestRun::builder("with dut", "1.0")
        .build()
        .start(dut)
        .await?;

    let step = run.add_step("step0").start().await?;
    ocptv_log_debug!(step, "Some interesting message.").await?;
    step.end(TestStatus::Complete).await?;

    run.end(TestStatus::Complete, TestResult::Pass).await?;
    Ok(())
}

/// Show a context-scoped run that automatically exits the whole func
/// because of the marker exception that triggers SKIP outcome.
#[cfg(feature = "boxed-scopes")]
async fn demo_scope_run_skip() -> Result<()> {
    let dut = DutInfo::builder("dut0").build();
    TestRun::builder("with dut", "1.0")
        .build()
        .scope(dut, |_r| {
            async move {
                // intentional short return
                return Ok(TestRunOutcome {
                    status: TestStatus::Skip,
                    result: TestResult::NotApplicable,
                });
            }
            .boxed()
        })
        .await?;

    Ok(())
}

/// Show a scoped run with scoped steps, everything starts at "with" time and
/// ends automatically when the block ends (regardless of unhandled exceptions).
#[cfg(feature = "boxed-scopes")]
async fn demo_scope_step_fail() -> Result<()> {
    let dut = DutInfo::builder("dut0").build();
    TestRun::builder("with dut", "1.0")
        .build()
        .scope(dut, |r| {
            async move {
                r.add_step("step0")
                    .scope(|s| {
                        async move {
                            ocptv_log_info!(s, "info log").await?;
                            Ok(TestStatus::Complete)
                        }
                        .boxed()
                    })
                    .await?;

                r.add_step("step1")
                    .scope(|_s| async move { Ok(TestStatus::Error) }.boxed())
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

/// In case of failure to discover DUT hardware before needing to present it at test run
/// start, we can error out right at the beginning since no Diagnosis can be produced.
/// This is a framework failure.
async fn demo_error_while_gathering_duts() -> Result<()> {
    let run = TestRun::builder("failed run", "1.0").build();
    ocptv_error!(run, "no-dut", "could not find any valid DUTs").await?;

    Ok(())
}

/// Show outputting an error message, triggered by a specific software component of the DUT.
#[cfg(feature = "boxed-scopes")]
async fn demo_run_error_with_dut() -> Result<()> {
    let mut dut = DutInfo::builder("dut0").name("dut0.server.net").build();
    let sw_info = dut.add_software_info(
        SoftwareInfo::builder("bmc")
            .software_type(SoftwareType::Firmware)
            .version("2.5")
            .build(),
    );

    TestRun::builder("with dut", "1.0")
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

#[tokio::main]
async fn main() {
    run_demo!(demo_no_scopes);
    run_demo!(demo_error_while_gathering_duts);

    #[cfg(feature = "boxed-scopes")]
    {
        run_demo!(demo_scope_run_skip);
        run_demo!(demo_scope_step_fail);
        run_demo!(demo_run_error_with_dut);
    }
}
