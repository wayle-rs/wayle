mod messages;
mod methods;
mod watchers;

use std::{
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};

use gdk4::Display;
use gtk::prelude::*;
use gtk4::{CssProvider, glib, style_context_add_provider_for_display};
use relm4::{gtk, prelude::*};
use wayle_media::{core::player::Player, types::*};
use wayle_widgets::{WatcherToken, prelude::*};

pub(crate) use self::messages::*;
use crate::{
    i18n::t,
    shell::{bar::dropdowns::media::helpers, helpers::COMPONENT_CSS_PRIORITY},
};

static NEXT_ART_CSS_ID: AtomicU64 = AtomicU64::new(1);

fn next_art_css_class() -> String {
    let id = NEXT_ART_CSS_ID.fetch_add(1, Ordering::Relaxed);
    format!("media-artwork-instance-{id}")
}

pub(crate) struct PlayerView {
    player: Option<Arc<Player>>,
    player_watcher: WatcherToken,
    is_active: bool,
    art_css_provider: CssProvider,
    art_css_class: String,
    seek_slider: DebouncedSlider,

    has_player: bool,
    title: String,
    artist: String,
    album: String,
    cover_art: Option<String>,

    playback_state: PlaybackState,
    position: Duration,
    length: Option<Duration>,

    loop_mode: LoopMode,
    shuffle_mode: ShuffleMode,

    can_go_previous: bool,
    can_go_next: bool,
    can_seek: bool,
    can_loop: bool,
    can_shuffle: bool,

    player_identity: String,
    source_icon: String,
}

#[relm4::component(pub(crate))]
impl Component for PlayerView {
    type Init = PlayerViewInit;
    type Input = PlayerViewInput;
    type Output = PlayerViewOutput;
    type CommandOutput = PlayerViewCmd;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,

            if model.has_player {
                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_vexpand: true,

                    #[name = "header"]
                    gtk::Box {
                        add_css_class: "media-header",

                        #[name = "header_title"]
                        gtk::Label {
                            add_css_class: "media-header-title",
                            set_label: &t!("dropdown-media-title"),
                            set_hexpand: true,
                            set_halign: gtk::Align::Start,
                            set_ellipsize: gtk::pango::EllipsizeMode::End,
                        },

                        #[name = "source_button"]
                        gtk::Button {
                            add_css_class: "media-source-button",
                            set_cursor_from_name: Some("pointer"),
                            connect_clicked => PlayerViewInput::ShowSourcePickerClicked,

                            gtk::Box {
                                gtk::Image {
                                    add_css_class: "media-source-icon",
                                    #[watch]
                                    set_icon_name: Some(&model.source_icon),
                                },

                                #[name = "source_name"]
                                gtk::Label {
                                    add_css_class: "media-source-name",
                                    #[watch]
                                    set_label: &model.player_identity,
                                    set_ellipsize: gtk::pango::EllipsizeMode::End,
                                },

                                gtk::Image {
                                    add_css_class: "media-source-chevron",
                                    set_icon_name: Some("ld-chevron-right-symbolic"),
                                },
                            },
                        },
                    },

                    #[name = "artwork"]
                    gtk::Box {
                        add_css_class: "media-artwork",
                        add_css_class: &model.art_css_class,
                        set_vexpand: true,

                        #[name = "artwork_placeholder"]
                        gtk::Box {
                            add_css_class: "media-artwork-placeholder",
                            set_hexpand: true,
                            set_halign: gtk::Align::Center,
                            set_valign: gtk::Align::Center,
                            #[watch]
                            set_visible: model.cover_art.is_none(),

                            gtk::Image {
                                add_css_class: "media-artwork-placeholder-icon",
                                set_icon_name: Some("ld-disc-3-symbolic"),
                            },
                        },
                    },

                    #[name = "info"]
                    gtk::Box {
                        add_css_class: "media-info",
                        set_orientation: gtk::Orientation::Vertical,
                        set_halign: gtk::Align::Fill,

                        #[name = "title_label"]
                        gtk::Label {
                            add_css_class: "media-title",
                            #[watch]
                            set_label: &model.display_title(),
                            #[watch]
                            set_class_active: ("placeholder", model.title.is_empty()),
                            set_ellipsize: gtk::pango::EllipsizeMode::End,
                            set_max_width_chars: 1,
                            set_xalign: 0.5,
                        },

                        #[name = "artist_label"]
                        gtk::Label {
                            add_css_class: "media-artist",
                            #[watch]
                            set_label: &model.display_artist(),
                            #[watch]
                            set_class_active: ("placeholder", model.artist.is_empty()),
                            set_ellipsize: gtk::pango::EllipsizeMode::End,
                            set_max_width_chars: 1,
                            set_xalign: 0.5,
                        },

                        #[name = "album_label"]
                        gtk::Label {
                            add_css_class: "media-album",
                            #[watch]
                            set_label: &model.display_album(),
                            #[watch]
                            set_class_active: ("placeholder", model.album.is_empty()),
                            set_ellipsize: gtk::pango::EllipsizeMode::End,
                            set_max_width_chars: 1,
                            set_xalign: 0.5,
                        },
                    },

                    #[name = "progress"]
                    gtk::Box {
                        add_css_class: "media-progress",
                        set_orientation: gtk::Orientation::Vertical,

                        #[local_ref]
                        seek_slider -> DebouncedSlider {
                            #[watch]
                            set_sensitive: model.can_seek,
                        },

                        #[name = "progress_times"]
                        gtk::Box {
                            add_css_class: "media-progress-times",

                            #[name = "position_label"]
                            gtk::Label {
                                add_css_class: "media-time",
                                set_hexpand: true,
                                set_halign: gtk::Align::Start,
                                #[watch]
                                set_label: &helpers::format_duration(model.position),
                            },

                            #[name = "length_label"]
                            gtk::Label {
                                add_css_class: "media-time",
                                set_halign: gtk::Align::End,
                                #[watch]
                                set_label: &model.length.map_or_else(
                                    || String::from("0:00"),
                                    helpers::format_duration,
                                ),
                            },
                        },
                    },

                    #[name = "controls"]
                    gtk::Box {
                        add_css_class: "media-controls",
                        set_halign: gtk::Align::Center,

                        #[name = "shuffle_button"]
                        gtk::Button {
                            set_css_classes: &["media-control", "secondary"],
                            set_valign: gtk::Align::Center,
                            set_cursor_from_name: Some("pointer"),
                            #[watch]
                            set_class_active: ("active", model.shuffle_mode == ShuffleMode::On),
                            #[watch]
                            set_sensitive: model.can_shuffle,
                            connect_clicked => PlayerViewInput::ShuffleClicked,

                            gtk::Image {
                                set_icon_name: Some("ld-shuffle-symbolic"),
                            },
                        },

                        #[name = "previous_button"]
                        gtk::Button {
                            add_css_class: "media-control",
                            set_valign: gtk::Align::Center,
                            set_cursor_from_name: Some("pointer"),
                            #[watch]
                            set_sensitive: model.can_go_previous,
                            connect_clicked => PlayerViewInput::PreviousClicked,

                            gtk::Image {
                                set_icon_name: Some("ld-skip-back-symbolic"),
                            },
                        },

                        #[name = "play_pause_button"]
                        gtk::Button {
                            set_css_classes: &["media-control", "main"],
                            set_valign: gtk::Align::Center,
                            set_cursor_from_name: Some("pointer"),
                            connect_clicked => PlayerViewInput::PlayPauseClicked,

                            gtk::Image {
                                #[watch]
                                set_icon_name: Some(model.play_pause_icon()),
                            },
                        },

                        #[name = "next_button"]
                        gtk::Button {
                            add_css_class: "media-control",
                            set_valign: gtk::Align::Center,
                            set_cursor_from_name: Some("pointer"),
                            #[watch]
                            set_sensitive: model.can_go_next,
                            connect_clicked => PlayerViewInput::NextClicked,

                            gtk::Image {
                                set_icon_name: Some("ld-skip-forward-symbolic"),
                            },
                        },

                        #[name = "loop_button"]
                        gtk::Button {
                            set_css_classes: &["media-control", "secondary"],
                            set_valign: gtk::Align::Center,
                            set_cursor_from_name: Some("pointer"),
                            #[watch]
                            set_class_active: ("active", model.loop_mode != LoopMode::None && model.loop_mode != LoopMode::Unsupported),
                            #[watch]
                            set_sensitive: model.can_loop,
                            connect_clicked => PlayerViewInput::LoopClicked,

                            gtk::Image {
                                #[watch]
                                set_icon_name: Some(model.loop_icon()),
                            },
                        },
                    },
                }
            } else {
                #[template]
                EmptyState {
                    #[template_child]
                    icon {
                        set_icon_name: Some("ld-play-symbolic"),
                    },
                    #[template_child]
                    title {
                        set_label: &t!("dropdown-media-no-player-title"),
                    },
                    #[template_child]
                    description {
                        set_label: &t!("dropdown-media-no-player-description"),
                    },
                }
            },
        }
    }

    fn init(
        init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        watchers::spawn_static(&sender, &init.media);

        let art_css_provider = CssProvider::new();
        #[allow(clippy::expect_used)]
        let display = Display::default().expect("display required for media dropdown");
        style_context_add_provider_for_display(&display, &art_css_provider, COMPONENT_CSS_PRIORITY);

        let seek_slider = DebouncedSlider::new(0.0);
        if let Some(scale) = seek_slider.scale() {
            scale.add_css_class("media-seek-slider");
        }

        let commit_sender = sender.input_sender().clone();
        seek_slider.connect_closure(
            "committed",
            false,
            glib::closure_local!(move |_slider: DebouncedSlider, percentage: f64| {
                commit_sender.emit(PlayerViewInput::SeekCommitted(percentage));
            }),
        );

        let model = Self {
            player: None,
            player_watcher: WatcherToken::new(),
            is_active: false,
            art_css_provider,
            art_css_class: next_art_css_class(),
            seek_slider: seek_slider.clone(),

            has_player: false,
            title: String::new(),
            artist: String::new(),
            album: String::new(),
            cover_art: None,

            playback_state: PlaybackState::Stopped,
            position: Duration::ZERO,
            length: None,

            loop_mode: LoopMode::None,
            shuffle_mode: ShuffleMode::Off,

            can_go_previous: false,
            can_go_next: false,
            can_seek: false,
            can_loop: false,
            can_shuffle: false,

            player_identity: String::new(),
            source_icon: String::from("ld-music-symbolic"),
        };

        let seek_slider = &model.seek_slider;
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            PlayerViewInput::SetActive(active) => {
                self.set_active(active, &sender);
            }

            PlayerViewInput::ShowSourcePickerClicked => {
                let _ = sender.output(PlayerViewOutput::ShowSourcePicker);
            }

            PlayerViewInput::PlayPauseClicked => {
                self.fire_player_command(
                    &sender,
                    |player| async move { player.play_pause().await },
                    "play/pause failed",
                );
            }

            PlayerViewInput::NextClicked => {
                self.fire_player_command(
                    &sender,
                    |player| async move { player.next().await },
                    "next track failed",
                );
            }

            PlayerViewInput::PreviousClicked => {
                self.fire_player_command(
                    &sender,
                    |player| async move { player.previous().await },
                    "previous track failed",
                );
            }

            PlayerViewInput::ShuffleClicked => {
                self.fire_player_command(
                    &sender,
                    |player| async move { player.toggle_shuffle().await },
                    "toggle shuffle failed",
                );
            }

            PlayerViewInput::LoopClicked => {
                self.fire_player_command(
                    &sender,
                    |player| async move { player.toggle_loop().await },
                    "toggle loop failed",
                );
            }

            PlayerViewInput::SeekCommitted(percentage) => {
                let Some(length) = self.length else {
                    return;
                };
                let target = Duration::from_secs_f64(length.as_secs_f64() * percentage / 100.0);
                self.fire_player_command(
                    &sender,
                    move |player| async move { player.set_position(target).await },
                    "seek failed",
                );
            }
        }
    }

    fn update_cmd(
        &mut self,
        msg: PlayerViewCmd,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            PlayerViewCmd::PlayerChanged(player) => {
                self.update_player(player, &sender);
            }

            PlayerViewCmd::MetadataChanged => {
                self.refresh_metadata();
            }

            PlayerViewCmd::CapabilitiesChanged => {
                self.refresh_capabilities();
            }

            PlayerViewCmd::PlaybackStateChanged(state) => {
                self.playback_state = state;
            }

            PlayerViewCmd::PositionTick(position) => {
                self.position = position;
                self.seek_slider.set_value(self.progress_fraction() * 100.0);
            }

            PlayerViewCmd::LoopModeChanged(mode) => {
                self.loop_mode = mode;
            }

            PlayerViewCmd::ShuffleModeChanged(mode) => {
                self.shuffle_mode = mode;
            }

            PlayerViewCmd::Noop => {}
        }
    }
}
