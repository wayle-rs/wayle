mod factory;
mod helpers;
mod messages;
mod watchers;

use std::sync::Arc;

use gtk::prelude::*;
use relm4::{gtk, prelude::*};
use wayle_config::ConfigService;
use wayle_sysinfo::SysinfoService;
use wayle_widgets::prelude::*;

pub(super) use self::factory::Factory;
use self::messages::{SysinfoDropdownCmd, SysinfoDropdownInit, SysinfoDropdownInput};
use crate::shell::bar::dropdowns::scaled_dimension;

const BASE_WIDTH: f32 = 440.0;
const BASE_HEIGHT: f32 = 630.0;

pub(crate) struct SysinfoDropdown {
    #[allow(dead_code)]
    sysinfo: Arc<SysinfoService>,
    #[allow(dead_code)]
    config: Arc<ConfigService>,

    scaled_width: i32,
    scaled_height: i32,

    // NVIDIA
    nvidia_available: bool,
    nvidia_name: String,
    nvidia_usage: String,
    nvidia_temp: String,
    nvidia_vram: String,
    nvidia_gpu_clock: String,
    nvidia_mem_clock: String,
    nvidia_fan: String,
    nvidia_power: String,

    // AMD iGPU
    amd_available: bool,
    amd_name: String,
    amd_usage: String,
    amd_temp: String,
    amd_vram: String,

    // CPU
    cpu_name: String,
    cpu_usage: String,
    cpu_temp: String,
    cpu_avg_freq: String,
    cpu_max_freq: String,
    cpu_cores: String,

    // Memory
    ram_info: String,
    swap_info: String,
}

#[relm4::component(pub(crate))]
impl Component for SysinfoDropdown {
    type Init = SysinfoDropdownInit;
    type Input = SysinfoDropdownInput;
    type Output = ();
    type CommandOutput = SysinfoDropdownCmd;

    view! {
        #[root]
        gtk::Popover {
            set_css_classes: &["dropdown", "sysinfo-dropdown"],
            set_has_arrow: false,
            #[watch]
            set_width_request: model.scaled_width,
            #[watch]
            set_height_request: model.scaled_height,

            #[template]
            Dropdown {

                #[template]
                DropdownHeader {
                    #[template_child]
                    icon {
                        set_icon_name: Some("md-developer_board-symbolic"),
                        set_visible: true,
                    },
                    #[template_child]
                    label {
                        set_label: "System Info",
                    },
                },

                #[template]
                DropdownContent {
                    set_vexpand: true,

                    gtk::ScrolledWindow {
                        set_policy: (gtk::PolicyType::Never, gtk::PolicyType::Automatic),
                        set_vexpand: true,

                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_spacing: 8,
                            set_margin_top: 4,
                            set_margin_bottom: 8,

                            // ── NVIDIA Card ─────────────────────────
                            gtk::Box {
                                add_css_class: "card",
                                add_css_class: "sysinfo-card",
                                set_orientation: gtk::Orientation::Vertical,
                                #[watch]
                                set_visible: model.nvidia_available,

                                gtk::Box {
                                    add_css_class: "sysinfo-card-header",
                                    gtk::Image {
                                        set_icon_name: Some("md-monitor-symbolic"),
                                        add_css_class: "sysinfo-header-icon",
                                        set_pixel_size: 18,
                                    },
                                    gtk::Label {
                                        #[watch]
                                        set_label: &model.nvidia_name,
                                        add_css_class: "sysinfo-header-label",
                                        set_wrap: true,
                                        set_wrap_mode: gtk::pango::WrapMode::WordChar,
                                    },
                                },

                                gtk::Box {
                                    set_orientation: gtk::Orientation::Vertical,
                                    set_spacing: 1,
                                    add_css_class: "sysinfo-stat-list",

                                    gtk::Box { add_css_class: "sysinfo-stat",
                                        gtk::Label { set_label: "Usage", add_css_class: "sysinfo-stat-key", set_hexpand: true, set_xalign: 0.0, },
                                        gtk::Label { #[watch] set_label: &model.nvidia_usage, add_css_class: "sysinfo-stat-val", set_xalign: 1.0, },
                                    },
                                    gtk::Box { add_css_class: "sysinfo-stat",
                                        gtk::Label { set_label: "Temp", add_css_class: "sysinfo-stat-key", set_hexpand: true, set_xalign: 0.0, },
                                        gtk::Label { #[watch] set_label: &model.nvidia_temp, add_css_class: "sysinfo-stat-val", set_xalign: 1.0, },
                                    },
                                    gtk::Box { add_css_class: "sysinfo-stat",
                                        gtk::Label { set_label: "VRAM", add_css_class: "sysinfo-stat-key", set_hexpand: true, set_xalign: 0.0, },
                                        gtk::Label { #[watch] set_label: &model.nvidia_vram, add_css_class: "sysinfo-stat-val", set_xalign: 1.0, },
                                    },
                                    gtk::Box { add_css_class: "sysinfo-stat",
                                        gtk::Label { set_label: "GPU Clock", add_css_class: "sysinfo-stat-key", set_hexpand: true, set_xalign: 0.0, },
                                        gtk::Label { #[watch] set_label: &model.nvidia_gpu_clock, add_css_class: "sysinfo-stat-val", set_xalign: 1.0, },
                                    },
                                    gtk::Box { add_css_class: "sysinfo-stat",
                                        gtk::Label { set_label: "Mem Clock", add_css_class: "sysinfo-stat-key", set_hexpand: true, set_xalign: 0.0, },
                                        gtk::Label { #[watch] set_label: &model.nvidia_mem_clock, add_css_class: "sysinfo-stat-val", set_xalign: 1.0, },
                                    },
                                    gtk::Box { add_css_class: "sysinfo-stat",
                                        gtk::Label { set_label: "Fan", add_css_class: "sysinfo-stat-key", set_hexpand: true, set_xalign: 0.0, },
                                        gtk::Label { #[watch] set_label: &model.nvidia_fan, add_css_class: "sysinfo-stat-val", set_xalign: 1.0, },
                                    },
                                    gtk::Box { add_css_class: "sysinfo-stat",
                                        gtk::Label { set_label: "Power", add_css_class: "sysinfo-stat-key", set_hexpand: true, set_xalign: 0.0, },
                                        gtk::Label { #[watch] set_label: &model.nvidia_power, add_css_class: "sysinfo-stat-val", set_xalign: 1.0, },
                                    },
                                },
                            },

                            // ── AMD iGPU Card ───────────────────────
                            gtk::Box {
                                add_css_class: "card",
                                add_css_class: "sysinfo-card",
                                set_orientation: gtk::Orientation::Vertical,
                                #[watch]
                                set_visible: model.amd_available,

                                gtk::Box {
                                    add_css_class: "sysinfo-card-header",
                                    gtk::Image {
                                        set_icon_name: Some("md-monitor-symbolic"),
                                        add_css_class: "sysinfo-header-icon",
                                        set_pixel_size: 18,
                                    },
                                    gtk::Label {
                                        #[watch]
                                        set_label: &model.amd_name,
                                        add_css_class: "sysinfo-header-label",
                                        set_wrap: true,
                                        set_wrap_mode: gtk::pango::WrapMode::WordChar,
                                    },
                                },

                                gtk::Box {
                                    set_orientation: gtk::Orientation::Vertical,
                                    set_spacing: 1,
                                    add_css_class: "sysinfo-stat-list",

                                    gtk::Box { add_css_class: "sysinfo-stat",
                                        gtk::Label { set_label: "Usage", add_css_class: "sysinfo-stat-key", set_hexpand: true, set_xalign: 0.0, },
                                        gtk::Label { #[watch] set_label: &model.amd_usage, add_css_class: "sysinfo-stat-val", set_xalign: 1.0, },
                                    },
                                    gtk::Box { add_css_class: "sysinfo-stat",
                                        gtk::Label { set_label: "Temp", add_css_class: "sysinfo-stat-key", set_hexpand: true, set_xalign: 0.0, },
                                        gtk::Label { #[watch] set_label: &model.amd_temp, add_css_class: "sysinfo-stat-val", set_xalign: 1.0, },
                                    },
                                    gtk::Box { add_css_class: "sysinfo-stat",
                                        gtk::Label { set_label: "VRAM", add_css_class: "sysinfo-stat-key", set_hexpand: true, set_xalign: 0.0, },
                                        gtk::Label { #[watch] set_label: &model.amd_vram, add_css_class: "sysinfo-stat-val", set_xalign: 1.0, },
                                    },
                                },
                            },

                            // ── CPU Card ────────────────────────────
                            gtk::Box {
                                add_css_class: "card",
                                add_css_class: "sysinfo-card",
                                set_orientation: gtk::Orientation::Vertical,

                                gtk::Box {
                                    add_css_class: "sysinfo-card-header",
                                    gtk::Image {
                                        set_icon_name: Some("md-speed-symbolic"),
                                        add_css_class: "sysinfo-header-icon",
                                        set_pixel_size: 18,
                                    },
                                    gtk::Label {
                                        #[watch]
                                        set_label: &model.cpu_name,
                                        add_css_class: "sysinfo-header-label",
                                        set_wrap: true,
                                        set_wrap_mode: gtk::pango::WrapMode::WordChar,
                                    },
                                },

                                gtk::Box {
                                    set_orientation: gtk::Orientation::Vertical,
                                    set_spacing: 1,
                                    add_css_class: "sysinfo-stat-list",

                                    gtk::Box { add_css_class: "sysinfo-stat",
                                        gtk::Label { set_label: "Usage", add_css_class: "sysinfo-stat-key", set_hexpand: true, set_xalign: 0.0, },
                                        gtk::Label { #[watch] set_label: &model.cpu_usage, add_css_class: "sysinfo-stat-val", set_xalign: 1.0, },
                                    },
                                    gtk::Box { add_css_class: "sysinfo-stat",
                                        gtk::Label { set_label: "Temp", add_css_class: "sysinfo-stat-key", set_hexpand: true, set_xalign: 0.0, },
                                        gtk::Label { #[watch] set_label: &model.cpu_temp, add_css_class: "sysinfo-stat-val", set_xalign: 1.0, },
                                    },
                                    gtk::Box { add_css_class: "sysinfo-stat",
                                        gtk::Label { set_label: "Avg Freq", add_css_class: "sysinfo-stat-key", set_hexpand: true, set_xalign: 0.0, },
                                        gtk::Label { #[watch] set_label: &model.cpu_avg_freq, add_css_class: "sysinfo-stat-val", set_xalign: 1.0, },
                                    },
                                    gtk::Box { add_css_class: "sysinfo-stat",
                                        gtk::Label { set_label: "Max Freq", add_css_class: "sysinfo-stat-key", set_hexpand: true, set_xalign: 0.0, },
                                        gtk::Label { #[watch] set_label: &model.cpu_max_freq, add_css_class: "sysinfo-stat-val", set_xalign: 1.0, },
                                    },
                                    gtk::Box { add_css_class: "sysinfo-stat",
                                        gtk::Label { set_label: "Cores", add_css_class: "sysinfo-stat-key", set_hexpand: true, set_xalign: 0.0, },
                                        gtk::Label { #[watch] set_label: &model.cpu_cores, add_css_class: "sysinfo-stat-val", set_xalign: 1.0, },
                                    },
                                },
                            },

                            // ── Memory Card ─────────────────────────
                            gtk::Box {
                                add_css_class: "card",
                                add_css_class: "sysinfo-card",
                                set_orientation: gtk::Orientation::Vertical,

                                gtk::Box {
                                    add_css_class: "sysinfo-card-header",
                                    gtk::Image {
                                        set_icon_name: Some("md-memory-symbolic"),
                                        add_css_class: "sysinfo-header-icon",
                                        set_pixel_size: 18,
                                    },
                                    gtk::Label {
                                        set_label: "MEMORY",
                                        add_css_class: "sysinfo-header-label",
                                    },
                                },

                                gtk::Box {
                                    set_orientation: gtk::Orientation::Vertical,
                                    set_spacing: 1,
                                    add_css_class: "sysinfo-stat-list",

                                    gtk::Box { add_css_class: "sysinfo-stat",
                                        gtk::Label { set_label: "RAM", add_css_class: "sysinfo-stat-key", set_hexpand: true, set_xalign: 0.0, },
                                        gtk::Label { #[watch] set_label: &model.ram_info, add_css_class: "sysinfo-stat-val", set_xalign: 1.0, },
                                    },
                                    gtk::Box { add_css_class: "sysinfo-stat",
                                        gtk::Label { set_label: "Swap", add_css_class: "sysinfo-stat-key", set_hexpand: true, set_xalign: 0.0, },
                                        gtk::Label { #[watch] set_label: &model.swap_info, add_css_class: "sysinfo-stat-val", set_xalign: 1.0, },
                                    },
                                },
                            },
                        },
                    },
                },
            },
        }
    }

    fn init(
        init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let scale = init.config.config().styling.scale.get().value();

        // Detect available GPUs
        let nvml = helpers::init_nvml();
        let nvidia_available = nvml.is_some();
        let amd_card = helpers::detect_amd_card();
        let amd_available = amd_card.is_some();

        // Read initial data
        let nvidia = nvml
            .as_ref()
            .map(|n| helpers::read_nvidia(n, 0))
            .unwrap_or_default();
        let amd = amd_card.map(helpers::read_amd).unwrap_or_default();
        let cpu_name = helpers::read_cpu_name();
        let cpu_data = init.sysinfo.cpu.get();
        let mem_data = init.sysinfo.memory.get();

        let to_gb = |bytes: u64| bytes as f64 / (1024.0 * 1024.0 * 1024.0);

        watchers::spawn(
            &sender,
            &init.sysinfo,
            &init.config,
            nvml.map(Arc::new),
            amd_card,
            cpu_name.clone(),
        );

        let model = Self {
            sysinfo: init.sysinfo,
            config: init.config,

            scaled_width: scaled_dimension(BASE_WIDTH, scale),
            scaled_height: scaled_dimension(BASE_HEIGHT, scale),

            nvidia_available,
            nvidia_name: nvidia.name.clone(),
            nvidia_usage: format!("{:.0}%", nvidia.usage_percent),
            nvidia_temp: helpers::fmt_temp(nvidia.temperature_celsius),
            nvidia_vram: helpers::fmt_vram(nvidia.vram_used_mb, nvidia.vram_total_mb),
            nvidia_gpu_clock: helpers::fmt_clock(nvidia.gpu_clock_mhz),
            nvidia_mem_clock: helpers::fmt_clock(nvidia.mem_clock_mhz),
            nvidia_fan: helpers::fmt_fan(nvidia.fan_speed_percent),
            nvidia_power: helpers::fmt_power(nvidia.power_watts, nvidia.power_limit_watts),

            amd_available,
            amd_name: amd.name.clone(),
            amd_usage: format!("{:.0}%", amd.usage_percent),
            amd_temp: helpers::fmt_temp(amd.temperature_celsius),
            amd_vram: helpers::fmt_vram(amd.vram_used_mb, amd.vram_total_mb),

            cpu_name,
            cpu_usage: format!("{:.0}%", cpu_data.usage_percent),
            cpu_temp: helpers::fmt_temp(cpu_data.temperature_celsius),
            cpu_avg_freq: format!("{:.1} GHz", cpu_data.avg_frequency_mhz as f64 / 1000.0),
            cpu_max_freq: format!("{:.1} GHz", cpu_data.max_frequency_mhz as f64 / 1000.0),
            cpu_cores: cpu_data.cores.len().to_string(),

            ram_info: helpers::fmt_gb(
                to_gb(mem_data.used_bytes),
                to_gb(mem_data.total_bytes),
                mem_data.usage_percent,
            ),
            swap_info: helpers::fmt_gb(
                to_gb(mem_data.swap_used_bytes),
                to_gb(mem_data.swap_total_bytes),
                mem_data.swap_percent,
            ),
        };

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, _msg: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {}

    fn update_cmd(
        &mut self,
        msg: Self::CommandOutput,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            SysinfoDropdownCmd::ScaleChanged(scale) => {
                self.scaled_width = scaled_dimension(BASE_WIDTH, scale);
                self.scaled_height = scaled_dimension(BASE_HEIGHT, scale);
            }

            SysinfoDropdownCmd::UpdateNvidia(data) => {
                self.nvidia_name = data.name;
                self.nvidia_usage = format!("{:.0}%", data.usage_percent);
                self.nvidia_temp = helpers::fmt_temp(data.temperature_celsius);
                self.nvidia_vram = helpers::fmt_vram(data.vram_used_mb, data.vram_total_mb);
                self.nvidia_gpu_clock = helpers::fmt_clock(data.gpu_clock_mhz);
                self.nvidia_mem_clock = helpers::fmt_clock(data.mem_clock_mhz);
                self.nvidia_fan = helpers::fmt_fan(data.fan_speed_percent);
                self.nvidia_power = helpers::fmt_power(data.power_watts, data.power_limit_watts);
            }

            SysinfoDropdownCmd::UpdateAmd(data) => {
                self.amd_name = data.name;
                self.amd_usage = format!("{:.0}%", data.usage_percent);
                self.amd_temp = helpers::fmt_temp(data.temperature_celsius);
                self.amd_vram = helpers::fmt_vram(data.vram_used_mb, data.vram_total_mb);
            }

            SysinfoDropdownCmd::UpdateCpu(data) => {
                self.cpu_name = data.name;
                self.cpu_usage = format!("{:.0}%", data.usage_percent);
                self.cpu_temp = helpers::fmt_temp(data.temperature_celsius);
                self.cpu_avg_freq = format!("{:.1} GHz", data.avg_freq_ghz);
                self.cpu_max_freq = format!("{:.1} GHz", data.max_freq_ghz);
                self.cpu_cores = data.core_count.to_string();
            }

            SysinfoDropdownCmd::UpdateMemory(data) => {
                self.ram_info = helpers::fmt_gb(data.ram_used_gb, data.ram_total_gb, data.ram_percent);
                self.swap_info = helpers::fmt_gb(
                    data.swap_used_gb,
                    data.swap_total_gb,
                    data.swap_percent,
                );
            }
        }
    }
}
