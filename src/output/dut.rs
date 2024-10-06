// (c) Meta Platforms, Inc. and affiliates.
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

use maplit::{btreemap, convert_args};
use std::collections::BTreeMap;

use crate::output as tv;
use crate::spec;

#[derive(Default, Debug, Clone, PartialEq)]
pub struct DutInfo {
    id: String,
    name: Option<String>,
    platform_infos: Option<Vec<PlatformInfo>>,
    software_infos: Option<Vec<SoftwareInfo>>,
    hardware_infos: Option<Vec<HardwareInfo>>,
    metadata: Option<BTreeMap<String, tv::Value>>,
}

impl DutInfo {
    pub fn builder(id: &str) -> DutInfoBuilder {
        DutInfoBuilder::new(id)
    }

    pub fn new(id: &str) -> DutInfo {
        DutInfoBuilder::new(id).build()
    }

    pub(crate) fn to_spec(&self) -> spec::DutInfo {
        spec::DutInfo {
            id: self.id.clone(),
            name: self.name.clone(),
            platform_infos: self
                .platform_infos
                .clone()
                .map(|infos| infos.iter().map(|info| info.to_spec()).collect()),
            software_infos: self
                .software_infos
                .clone()
                .map(|infos| infos.iter().map(|info| info.to_spec()).collect()),
            hardware_infos: self
                .hardware_infos
                .clone()
                .map(|infos| infos.iter().map(|info| info.to_spec()).collect()),
            metadata: self.metadata.clone(),
        }
    }
}

pub struct DutInfoBuilder {
    id: String,
    name: Option<String>,
    platform_infos: Option<Vec<PlatformInfo>>,
    software_infos: Option<Vec<SoftwareInfo>>,
    hardware_infos: Option<Vec<HardwareInfo>>,
    metadata: Option<BTreeMap<String, tv::Value>>,
}

impl DutInfoBuilder {
    pub fn new(id: &str) -> DutInfoBuilder {
        DutInfoBuilder {
            id: id.to_string(),
            name: None,
            platform_infos: None,
            software_infos: None,
            hardware_infos: None,
            metadata: None,
        }
    }
    pub fn name(mut self, value: &str) -> DutInfoBuilder {
        self.name = Some(value.to_string());
        self
    }

    pub fn add_platform_info(mut self, platform_info: &PlatformInfo) -> DutInfoBuilder {
        self.platform_infos = match self.platform_infos {
            Some(mut platform_infos) => {
                platform_infos.push(platform_info.clone());
                Some(platform_infos)
            }
            None => Some(vec![platform_info.clone()]),
        };
        self
    }

    pub fn add_software_info(mut self, software_info: &SoftwareInfo) -> DutInfoBuilder {
        self.software_infos = match self.software_infos {
            Some(mut software_infos) => {
                software_infos.push(software_info.clone());
                Some(software_infos)
            }
            None => Some(vec![software_info.clone()]),
        };
        self
    }

    pub fn add_hardware_info(mut self, hardware_info: &HardwareInfo) -> DutInfoBuilder {
        self.hardware_infos = match self.hardware_infos {
            Some(mut hardware_infos) => {
                hardware_infos.push(hardware_info.clone());
                Some(hardware_infos)
            }
            None => Some(vec![hardware_info.clone()]),
        };
        self
    }

    pub fn add_metadata(mut self, key: &str, value: tv::Value) -> DutInfoBuilder {
        self.metadata = match self.metadata {
            Some(mut metadata) => {
                metadata.insert(key.to_string(), value.clone());
                Some(metadata)
            }
            None => Some(convert_args!(btreemap!(
                key => value
            ))),
        };
        self
    }

    pub fn build(self) -> DutInfo {
        DutInfo {
            id: self.id,
            name: self.name,
            platform_infos: self.platform_infos,
            software_infos: self.software_infos,
            hardware_infos: self.hardware_infos,
            metadata: self.metadata,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HardwareInfo {
    id: String,
    name: String,
    version: Option<String>,
    revision: Option<String>,
    location: Option<String>,
    serial_no: Option<String>,
    part_no: Option<String>,
    manufacturer: Option<String>,
    manufacturer_part_no: Option<String>,
    odata_id: Option<String>,
    computer_system: Option<String>,
    manager: Option<String>,
}

impl HardwareInfo {
    pub fn builder(id: &str, name: &str) -> HardwareInfoBuilder {
        HardwareInfoBuilder::new(id, name)
    }

    pub fn to_spec(&self) -> spec::HardwareInfo {
        spec::HardwareInfo {
            id: self.id.clone(),
            name: self.name.clone(),
            version: self.version.clone(),
            revision: self.revision.clone(),
            location: self.location.clone(),
            serial_no: self.serial_no.clone(),
            part_no: self.part_no.clone(),
            manufacturer: self.manufacturer.clone(),
            manufacturer_part_no: self.manufacturer_part_no.clone(),
            odata_id: self.odata_id.clone(),
            computer_system: self.computer_system.clone(),
            manager: self.manager.clone(),
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}

#[derive(Debug)]
pub struct HardwareInfoBuilder {
    id: String,
    name: String,
    version: Option<String>,
    revision: Option<String>,
    location: Option<String>,
    serial_no: Option<String>,
    part_no: Option<String>,
    manufacturer: Option<String>,
    manufacturer_part_no: Option<String>,
    odata_id: Option<String>,
    computer_system: Option<String>,
    manager: Option<String>,
}

impl HardwareInfoBuilder {
    fn new(id: &str, name: &str) -> Self {
        HardwareInfoBuilder {
            id: id.to_string(),
            name: name.to_string(),
            version: None,
            revision: None,
            location: None,
            serial_no: None,
            part_no: None,
            manufacturer: None,
            manufacturer_part_no: None,
            odata_id: None,
            computer_system: None,
            manager: None,
        }
    }
    pub fn version(mut self, value: &str) -> HardwareInfoBuilder {
        self.version = Some(value.to_string());
        self
    }
    pub fn revision(mut self, value: &str) -> HardwareInfoBuilder {
        self.revision = Some(value.to_string());
        self
    }
    pub fn location(mut self, value: &str) -> HardwareInfoBuilder {
        self.location = Some(value.to_string());
        self
    }
    pub fn serial_no(mut self, value: &str) -> HardwareInfoBuilder {
        self.serial_no = Some(value.to_string());
        self
    }
    pub fn part_no(mut self, value: &str) -> HardwareInfoBuilder {
        self.part_no = Some(value.to_string());
        self
    }
    pub fn manufacturer(mut self, value: &str) -> HardwareInfoBuilder {
        self.manufacturer = Some(value.to_string());
        self
    }
    pub fn manufacturer_part_no(mut self, value: &str) -> HardwareInfoBuilder {
        self.manufacturer_part_no = Some(value.to_string());
        self
    }
    pub fn odata_id(mut self, value: &str) -> HardwareInfoBuilder {
        self.odata_id = Some(value.to_string());
        self
    }
    pub fn computer_system(mut self, value: &str) -> HardwareInfoBuilder {
        self.computer_system = Some(value.to_string());
        self
    }
    pub fn manager(mut self, value: &str) -> HardwareInfoBuilder {
        self.manager = Some(value.to_string());
        self
    }

    pub fn build(self) -> HardwareInfo {
        HardwareInfo {
            id: self.id,
            name: self.name,
            version: self.version,
            revision: self.revision,
            location: self.location,
            serial_no: self.serial_no,
            part_no: self.part_no,
            manufacturer: self.manufacturer,
            manufacturer_part_no: self.manufacturer_part_no,
            odata_id: self.odata_id,
            computer_system: self.computer_system,
            manager: self.manager,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Subcomponent {
    subcomponent_type: Option<spec::SubcomponentType>,
    name: String,
    location: Option<String>,
    version: Option<String>,
    revision: Option<String>,
}

impl Subcomponent {
    pub fn builder(name: &str) -> SubcomponentBuilder {
        SubcomponentBuilder::new(name)
    }
    pub fn to_spec(&self) -> spec::Subcomponent {
        spec::Subcomponent {
            subcomponent_type: self.subcomponent_type.clone(),
            name: self.name.clone(),
            location: self.location.clone(),
            version: self.version.clone(),
            revision: self.revision.clone(),
        }
    }
}

#[derive(Debug)]
pub struct SubcomponentBuilder {
    subcomponent_type: Option<spec::SubcomponentType>,
    name: String,
    location: Option<String>,
    version: Option<String>,
    revision: Option<String>,
}

impl SubcomponentBuilder {
    fn new(name: &str) -> Self {
        SubcomponentBuilder {
            subcomponent_type: None,
            name: name.to_string(),
            location: None,
            version: None,
            revision: None,
        }
    }
    pub fn subcomponent_type(mut self, value: spec::SubcomponentType) -> SubcomponentBuilder {
        self.subcomponent_type = Some(value);
        self
    }
    pub fn version(mut self, value: &str) -> SubcomponentBuilder {
        self.version = Some(value.to_string());
        self
    }
    pub fn location(mut self, value: &str) -> SubcomponentBuilder {
        self.location = Some(value.to_string());
        self
    }
    pub fn revision(mut self, value: &str) -> SubcomponentBuilder {
        self.revision = Some(value.to_string());
        self
    }

    pub fn build(self) -> Subcomponent {
        Subcomponent {
            subcomponent_type: self.subcomponent_type,
            name: self.name,
            location: self.location,
            version: self.version,
            revision: self.revision,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlatformInfo {
    info: String,
}

impl PlatformInfo {
    pub fn builder(info: &str) -> PlatformInfoBuilder {
        PlatformInfoBuilder::new(info)
    }

    pub fn to_spec(&self) -> spec::PlatformInfo {
        spec::PlatformInfo {
            info: self.info.clone(),
        }
    }
}

#[derive(Debug)]
pub struct PlatformInfoBuilder {
    info: String,
}

impl PlatformInfoBuilder {
    fn new(info: &str) -> Self {
        PlatformInfoBuilder {
            info: info.to_string(),
        }
    }

    pub fn build(self) -> PlatformInfo {
        PlatformInfo { info: self.info }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SoftwareInfo {
    id: String,
    name: String,
    version: Option<String>,
    revision: Option<String>,
    software_type: Option<spec::SoftwareType>,
    computer_system: Option<String>,
}

impl SoftwareInfo {
    pub fn builder(id: &str, name: &str) -> SoftwareInfoBuilder {
        SoftwareInfoBuilder::new(id, name)
    }

    pub fn to_spec(&self) -> spec::SoftwareInfo {
        spec::SoftwareInfo {
            id: self.id.clone(),
            name: self.name.clone(),
            version: self.version.clone(),
            revision: self.revision.clone(),
            software_type: self.software_type.clone(),
            computer_system: self.computer_system.clone(),
        }
    }
}

#[derive(Debug)]
pub struct SoftwareInfoBuilder {
    id: String,
    name: String,
    version: Option<String>,
    revision: Option<String>,
    software_type: Option<spec::SoftwareType>,
    computer_system: Option<String>,
}

impl SoftwareInfoBuilder {
    fn new(id: &str, name: &str) -> Self {
        SoftwareInfoBuilder {
            id: id.to_string(),
            name: name.to_string(),
            version: None,
            revision: None,
            software_type: None,
            computer_system: None,
        }
    }
    pub fn version(mut self, value: &str) -> SoftwareInfoBuilder {
        self.version = Some(value.to_string());
        self
    }
    pub fn revision(mut self, value: &str) -> SoftwareInfoBuilder {
        self.revision = Some(value.to_string());
        self
    }
    pub fn software_type(mut self, value: spec::SoftwareType) -> SoftwareInfoBuilder {
        self.software_type = Some(value);
        self
    }
    pub fn computer_system(mut self, value: &str) -> SoftwareInfoBuilder {
        self.computer_system = Some(value.to_string());
        self
    }

    pub fn build(self) -> SoftwareInfo {
        SoftwareInfo {
            id: self.id,
            name: self.name,
            version: self.version,
            revision: self.revision,
            software_type: self.software_type,
            computer_system: self.computer_system,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec;
    use anyhow::{bail, Result};

    #[test]
    fn test_dut_creation_from_builder_with_defaults() -> Result<()> {
        let dut = DutInfo::builder("1234").build();
        assert_eq!(dut.id, "1234");
        Ok(())
    }

    #[test]
    fn test_dut_builder() -> Result<()> {
        let platform = PlatformInfo::builder("platform_info").build();
        let software = SoftwareInfo::builder("software_id", "name").build();
        let hardware = HardwareInfo::builder("hardware_id", "name").build();
        let dut = DutInfo::builder("1234")
            .name("DUT")
            .add_metadata("key", "value".into())
            .add_metadata("key2", "value2".into())
            .add_hardware_info(&hardware)
            .add_hardware_info(&hardware)
            .add_platform_info(&platform)
            .add_platform_info(&platform)
            .add_software_info(&software)
            .add_software_info(&software)
            .build();

        let spec_dut = dut.to_spec();

        assert_eq!(spec_dut.id, "1234");
        assert_eq!(spec_dut.name, Some("DUT".to_owned()));

        match spec_dut.metadata {
            Some(m) => {
                assert_eq!(m["key"], "value");
                assert_eq!(m["key2"], "value2");
            }
            _ => bail!("metadata is empty"),
        }

        match spec_dut.hardware_infos {
            Some(infos) => match infos.first() {
                Some(info) => {
                    assert_eq!(info.id, "hardware_id");
                }
                _ => bail!("hardware_infos is empty"),
            },
            _ => bail!("hardware_infos is missing"),
        }

        match spec_dut.software_infos {
            Some(infos) => match infos.first() {
                Some(info) => {
                    assert_eq!(info.id, "software_id");
                }
                _ => bail!("software_infos is empty"),
            },
            _ => bail!("software_infos is missing"),
        }

        match spec_dut.platform_infos {
            Some(infos) => match infos.first() {
                Some(info) => {
                    assert_eq!(info.info, "platform_info");
                }
                _ => bail!("platform_infos is empty"),
            },
            _ => bail!("platform_infos is missing"),
        }

        Ok(())
    }

    #[test]
    fn test_hardware_info() -> Result<()> {
        let info = HardwareInfo::builder("hardware_id", "hardware_name")
            .version("version")
            .revision("revision")
            .location("location")
            .serial_no("serial_no")
            .part_no("part_no")
            .manufacturer("manufacturer")
            .manufacturer_part_no("manufacturer_part_no")
            .odata_id("odata_id")
            .computer_system("computer_system")
            .manager("manager")
            .build();

        let spec_hwinfo = info.to_spec();

        assert_eq!(spec_hwinfo.id, "hardware_id");
        assert_eq!(spec_hwinfo.name, "hardware_name");
        assert_eq!(spec_hwinfo.version, Some("version".to_owned()));
        assert_eq!(spec_hwinfo.revision, Some("revision".to_owned()));
        assert_eq!(spec_hwinfo.location, Some("location".to_owned()));
        assert_eq!(spec_hwinfo.serial_no, Some("serial_no".to_owned()));
        assert_eq!(spec_hwinfo.part_no, Some("part_no".to_owned()));
        assert_eq!(spec_hwinfo.manufacturer, Some("manufacturer".to_owned()));
        assert_eq!(
            spec_hwinfo.manufacturer_part_no,
            Some("manufacturer_part_no".to_owned())
        );
        assert_eq!(spec_hwinfo.odata_id, Some("odata_id".to_owned()));
        assert_eq!(
            spec_hwinfo.computer_system,
            Some("computer_system".to_owned())
        );
        assert_eq!(spec_hwinfo.manager, Some("manager".to_owned()));

        Ok(())
    }

    #[test]
    fn test_software_info() -> Result<()> {
        let info = SoftwareInfo::builder("software_id", "name")
            .version("version")
            .revision("revision")
            .software_type(spec::SoftwareType::Application)
            .computer_system("system")
            .build();

        let spec_swinfo = info.to_spec();

        assert_eq!(spec_swinfo.id, "software_id");
        assert_eq!(spec_swinfo.name, "name");
        assert_eq!(spec_swinfo.version, Some("version".to_owned()));
        assert_eq!(spec_swinfo.revision, Some("revision".to_owned()));
        assert_eq!(
            spec_swinfo.software_type,
            Some(spec::SoftwareType::Application)
        );
        assert_eq!(spec_swinfo.computer_system, Some("system".to_owned()));

        Ok(())
    }

    #[test]
    fn test_platform_info() -> Result<()> {
        let info = PlatformInfo::builder("info").build();

        assert_eq!(info.to_spec().info, "info");
        Ok(())
    }

    #[test]
    fn test_subcomponent() -> Result<()> {
        let sub = Subcomponent::builder("sub_name")
            .subcomponent_type(spec::SubcomponentType::Asic)
            .version("version")
            .location("location")
            .revision("revision")
            .build();

        let spec_subcomponent = sub.to_spec();

        assert_eq!(spec_subcomponent.name, "sub_name");
        assert_eq!(spec_subcomponent.version, Some("version".to_owned()));
        assert_eq!(spec_subcomponent.revision, Some("revision".to_owned()));
        assert_eq!(spec_subcomponent.location, Some("location".to_owned()));
        assert_eq!(
            spec_subcomponent.subcomponent_type,
            Some(spec::SubcomponentType::Asic)
        );

        Ok(())
    }
}
