// (c) Meta Platforms, Inc. and affiliates.
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
// #![allow(warnings)]

use anyhow::Result;

use futures::FutureExt;
use ocptv::output as tv;
use tv::{
    DutHardwareInfo, DutInfo, HardwareInfo, Measurement, MeasurementSeriesInfo, PlatformInfo,
    SoftwareInfo, StartedTestStep, SubcomponentType, TestResult, TestRun, TestRunOutcome,
    TestStatus,
};

async fn run_measure_step(
    step: &StartedTestStep,
    ram0: DutHardwareInfo,
) -> Result<TestStatus, tv::OcptvError> {
    step.add_measurement_with_details(
        &Measurement::builder("temp0", 100.5.into())
            .unit("F")
            .hardware_info(&ram0)
            .subcomponent(&tv::Subcomponent::builder("chip0").build())
            .build(),
    )
    .await?;

    let chip1_temp = step.add_measurement_series_with_details(
        MeasurementSeriesInfo::builder("temp1")
            .unit("C")
            .hardware_info(&ram0)
            .subcomponent(
                &tv::Subcomponent::builder("chip1")
                    .location("U11")
                    .version("1")
                    .revision("1")
                    .subcomponent_type(SubcomponentType::Unspecified)
                    .build(),
            )
            .build(),
    );

    chip1_temp
        .scope(|s| {
            async move {
                s.add_measurement(79.into()).await?;

                Ok(())
            }
            .boxed()
        })
        .await?;

    Ok(TestStatus::Complete)
}

/// This is a more comprehensive example with a DUT having both hardware and software
/// components specified. The measurements reference the hardware items.
#[tokio::main]
async fn main() -> Result<()> {
    let mut dut = DutInfo::builder("dut0")
        .name("host0.example.com")
        .add_platform_info(&PlatformInfo::new("memory-optimized"))
        .build();

    dut.add_software_info(
        SoftwareInfo::builder("bmc0")
            .software_type(tv::SoftwareType::Firmware)
            .version("10")
            .revision("11")
            .computer_system("primary_node")
            .build(),
    );

    let ram0 = dut.add_hardware_info(
        HardwareInfo::builder("ram0")
            .version("1")
            .revision("2")
            .location("MB/DIMM_A1")
            .serial_no("HMA2022029281901")
            .part_no("P03052-091")
            .manufacturer("hynix")
            .manufacturer_part_no("HMA84GR7AFR4N-VK")
            .odata_id("/redfish/v1/Systems/System.Embedded.1/Memory/DIMMSLOTA1")
            .computer_system("primary_node")
            .manager("bmc0")
            .build(),
    );

    #[cfg(feature = "boxed-scopes")]
    TestRun::builder("simple measurement", "1.0")
        .build()
        .scope(dut, |r| {
            async move {
                r.add_step("step0")
                    .scope(|s| run_measure_step(s, ram0).boxed())
                    .await?;

                Ok(TestRunOutcome {
                    status: TestStatus::Complete,
                    result: TestResult::Pass,
                })
            }
            .boxed()
        })
        .await?;

    Ok(())
}
