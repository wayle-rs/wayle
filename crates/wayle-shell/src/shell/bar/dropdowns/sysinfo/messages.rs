use std::sync::Arc;

use wayle_config::ConfigService;
use wayle_sysinfo::SysinfoService;

use super::helpers::{AmdData, CpuSummary, MemoryInfo, NvidiaData};

pub(crate) struct SysinfoDropdownInit {
    pub sysinfo: Arc<SysinfoService>,
    pub config: Arc<ConfigService>,
}

#[derive(Debug)]
pub(crate) enum SysinfoDropdownInput {}

#[derive(Debug)]
pub(crate) enum SysinfoDropdownCmd {
    ScaleChanged(f32),
    UpdateNvidia(NvidiaData),
    UpdateAmd(AmdData),
    UpdateCpu(CpuSummary),
    UpdateMemory(MemoryInfo),
}
