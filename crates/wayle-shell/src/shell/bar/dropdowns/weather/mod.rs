mod daily_forecast;
mod factory;
mod hourly_forecast;
mod messages;
mod methods;
mod stats_grid;
mod sun_times;
mod watchers;
mod weather_header;

use std::sync::Arc;

use gtk::{pango, prelude::*};
use relm4::{gtk, prelude::*};
use wayle_config::ConfigService;
use wayle_weather::{WeatherErrorKind, WeatherService, WeatherStatus};
use wayle_widgets::prelude::*;

pub(super) use self::factory::Factory;
use self::{
    daily_forecast::{DailyForecast, DailyForecastInit},
    hourly_forecast::{HourlyForecast, HourlyForecastInit},
    messages::{WeatherDropdownCmd, WeatherDropdownInit, WeatherDropdownInput, WeatherPage},
    stats_grid::{StatsGrid, StatsGridInit},
    sun_times::{SunTimes, SunTimesInit},
    weather_header::{WeatherHeader, WeatherHeaderInit},
};
use crate::{i18n::t, shell::bar::dropdowns::scaled_dimension};

const BASE_WIDTH: f32 = 395.0;
const BASE_HEIGHT: f32 = 695.0;

pub(crate) struct WeatherDropdown {
    weather: Arc<WeatherService>,
    config: Arc<ConfigService>,

    scaled_width: i32,
    scaled_height: i32,
    page: WeatherPage,
    error_kind: Option<WeatherErrorKind>,

    weather_header: Controller<WeatherHeader>,
    stats_grid: Controller<StatsGrid>,
    hourly_forecast: Controller<HourlyForecast>,
    daily_forecast: Controller<DailyForecast>,
    sun_times: Controller<SunTimes>,
}

#[relm4::component(pub(crate))]
impl Component for WeatherDropdown {
    type Init = WeatherDropdownInit;
    type Input = WeatherDropdownInput;
    type Output = ();
    type CommandOutput = WeatherDropdownCmd;

    view! {
        #[root]
        gtk::Popover {
            set_css_classes: &["dropdown", "weather-dropdown"],
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
                        set_icon_name: Some("ld-sun-symbolic"),
                    },
                    #[template_child]
                    label {
                        set_label: &t!("dropdown-weather-title"),
                    },
                    #[template_child]
                    actions {
                        #[template]
                        GhostIconButton {
                            set_icon_name: "tb-refresh-symbolic",
                            set_tooltip_text: Some(&t!("dropdown-weather-refresh")),
                            connect_clicked => WeatherDropdownInput::Retry,
                        },
                    },
                },

                #[template]
                DropdownContent {
                    set_vexpand: true,

                    #[name = "page_stack"]
                    gtk::Stack {
                        set_transition_type: gtk::StackTransitionType::Crossfade,
                        set_vhomogeneous: true,
                        set_hhomogeneous: true,
                        #[name = "loaded_page"]
                        add_named[Some("loaded")] = &gtk::ScrolledWindow {
                            set_hscrollbar_policy: gtk::PolicyType::Never,
                            set_vexpand: true,
                            set_propagate_natural_height: true,
                            add_css_class: "weather-scroll",

                            #[wrap(Some)]
                            set_child = &gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,
                                set_vexpand: true,

                                #[local_ref]
                                weather_header_widget -> gtk::Box {},

                                #[local_ref]
                                stats_grid_widget -> gtk::Box {},

                                #[local_ref]
                                hourly_forecast_widget -> gtk::Box {},

                                #[local_ref]
                                daily_forecast_widget -> gtk::Box {},

                                #[local_ref]
                                sun_times_widget -> gtk::Box {},
                            },
                        },

                        #[name = "loading_page"]
                        add_named[Some("loading")] = &gtk::Box {
                            add_css_class: "loading-weather",
                            set_orientation: gtk::Orientation::Vertical,
                            set_valign: gtk::Align::Center,
                            set_halign: gtk::Align::Fill,
                            set_vexpand: true,

                            gtk::Image {
                                add_css_class: "loading-icon",
                                set_icon_name: Some("ld-sun-symbolic"),
                            },

                            gtk::Label {
                                add_css_class: "loading-text",
                                set_label: &t!("dropdown-weather-loading"),
                                set_ellipsize: pango::EllipsizeMode::End,
                            },
                        },

                        #[name = "error_page"]
                        add_named[Some("error")] = &gtk::Box {
                            add_css_class: "error-weather",
                            set_orientation: gtk::Orientation::Vertical,
                            set_valign: gtk::Align::Center,
                            set_halign: gtk::Align::Fill,
                            set_vexpand: true,

                            gtk::Image {
                                add_css_class: "error-icon",
                                set_icon_name: Some("ld-info-symbolic"),
                            },

                            gtk::Label {
                                add_css_class: "error-title",
                                set_label: &t!("dropdown-weather-error-title"),
                                set_ellipsize: pango::EllipsizeMode::End,
                            },

                            gtk::Label {
                                add_css_class: "error-text",
                                #[watch]
                                set_label: &model.error_description(),
                                set_ellipsize: pango::EllipsizeMode::End,
                            },

                            gtk::Button {
                                add_css_class: "weather-retry-btn",
                                set_halign: gtk::Align::Center,
                                set_cursor_from_name: Some("pointer"),
                                set_label: &t!("dropdown-weather-retry"),
                                connect_clicked => WeatherDropdownInput::Retry,
                            },
                        },

                        #[watch]
                        set_visible_child_name: model.page.name(),
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
        let weather_header = WeatherHeader::builder()
            .launch(WeatherHeaderInit {
                weather: init.weather.clone(),
                config: init.config.clone(),
            })
            .detach();

        let stats_grid = StatsGrid::builder()
            .launch(StatsGridInit {
                weather: init.weather.clone(),
                config: init.config.clone(),
            })
            .detach();

        let hourly_forecast = HourlyForecast::builder()
            .launch(HourlyForecastInit {
                weather: init.weather.clone(),
                config: init.config.clone(),
            })
            .detach();

        let daily_forecast = DailyForecast::builder()
            .launch(DailyForecastInit {
                weather: init.weather.clone(),
                config: init.config.clone(),
            })
            .detach();

        let sun_times = SunTimes::builder()
            .launch(SunTimesInit {
                weather: init.weather.clone(),
                config: init.config.clone(),
            })
            .detach();

        let scale = init.config.config().styling.scale.get().value();
        let (page, error_kind) = match init.weather.status.get() {
            WeatherStatus::Loading => (WeatherPage::Loading, None),
            WeatherStatus::Loaded => (WeatherPage::Loaded, None),
            WeatherStatus::Error(kind) => (WeatherPage::Error, Some(kind)),
        };

        watchers::spawn(&sender, &init.weather, &init.config);

        let model = Self {
            weather: init.weather,
            config: init.config,
            scaled_width: scaled_dimension(BASE_WIDTH, scale),
            scaled_height: scaled_dimension(BASE_HEIGHT, scale),
            page,
            error_kind,
            weather_header,
            stats_grid,
            hourly_forecast,
            daily_forecast,
            sun_times,
        };

        let weather_header_widget = model.weather_header.widget();
        let stats_grid_widget = model.stats_grid.widget();
        let hourly_forecast_widget = model.hourly_forecast.widget();
        let daily_forecast_widget = model.daily_forecast.widget();
        let sun_times_widget = model.sun_times.widget();

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            WeatherDropdownInput::Retry => {
                self.trigger_refresh();
            }
        }
    }

    fn update_cmd(
        &mut self,
        msg: Self::CommandOutput,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            WeatherDropdownCmd::ScaleChanged(scale) => {
                self.scaled_width = scaled_dimension(BASE_WIDTH, scale);
                self.scaled_height = scaled_dimension(BASE_HEIGHT, scale);
            }

            WeatherDropdownCmd::PageChanged { page, error } => {
                self.page = page;
                self.error_kind = error;
            }
        }
    }
}
