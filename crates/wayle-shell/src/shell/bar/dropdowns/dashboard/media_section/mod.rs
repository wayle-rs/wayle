mod helpers;
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

use gtk::{CssProvider, gdk::Display, glib, prelude::*, style_context_add_provider_for_display};
use relm4::{gtk, prelude::*};
use wayle_media::{MediaService, core::player::Player, types::PlaybackState};
use wayle_widgets::{WatcherToken, prelude::*};

use self::messages::MediaSectionCmd;
pub(crate) use self::messages::{MediaSectionInit, MediaSectionInput};
use crate::{i18n::t, shell::helpers::COMPONENT_CSS_PRIORITY};

const PERCENTAGE_SCALE: f64 = 100.0;

static NEXT_DASHBOARD_ART_CSS_ID: AtomicU64 = AtomicU64::new(1);

fn next_dashboard_art_css_class() -> String {
    let id = NEXT_DASHBOARD_ART_CSS_ID.fetch_add(1, Ordering::Relaxed);
    format!("dashboard-media-art-instance-{id}")
}

pub(crate) struct MediaSection {
    media: Option<Arc<MediaService>>,
    player: Option<Arc<Player>>,
    player_watcher: WatcherToken,
    is_active: bool,
    art_css_provider: CssProvider,
    art_css_class: String,
    seek_slider: DebouncedSlider,

    has_player: bool,
    has_multiple_players: bool,
    title: String,
    artist: String,
    cover_art: Option<String>,
    playback_state: PlaybackState,
    position: Duration,
    length: Option<Duration>,
    can_previous: bool,
    can_next: bool,
    can_seek: bool,
}

#[relm4::component(pub(crate))]
impl Component for MediaSection {
    type Init = MediaSectionInit;
    type Input = MediaSectionInput;
    type Output = ();
    type CommandOutput = MediaSectionCmd;

    view! {
        #[root]
        gtk::Box {
            set_css_classes: &["card", "dashboard-card"],
            set_orientation: gtk::Orientation::Vertical,

            #[name = "header"]
            gtk::Box {
                add_css_class: "card-header",

                #[name = "card_title"]
                gtk::Box {
                    add_css_class: "card-title",
                    set_hexpand: true,

                    gtk::Image {
                        set_icon_name: Some("ld-disc-3-symbolic"),
                    },

                    gtk::Label {
                        set_label: &t!("dropdown-dashboard-now-playing"),
                    },
                },

                #[template]
                #[name = "switch_player_btn"]
                GhostIconButton {
                    add_css_class: "media-switch-btn",
                    #[watch]
                    set_visible: model.has_multiple_players,
                    connect_clicked => MediaSectionInput::SwitchPlayerClicked,

                    gtk::Image {
                        set_icon_name: Some("ld-arrow-left-right-symbolic"),
                    },
                },
            },

            if model.has_player {
                #[name = "player_content"]
                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,

                    #[name = "media_compact"]
                    gtk::Box {
                        add_css_class: "media-compact",

                        #[name = "art_container"]
                        gtk::Box {
                            add_css_class: "dashboard-media-art",
                            add_css_class: &model.art_css_class,
                            set_halign: gtk::Align::Start,
                            set_valign: gtk::Align::Center,
                            set_hexpand: false,

                            #[name = "art_placeholder"]
                            gtk::Box {
                                add_css_class: "dashboard-media-art-placeholder",
                                set_hexpand: true,
                                set_vexpand: true,
                                set_halign: gtk::Align::Center,
                                set_valign: gtk::Align::Center,
                                #[watch]
                                set_visible: model.cover_art.is_none(),

                                gtk::Image {
                                    set_icon_name: Some("ld-music-symbolic"),
                                },
                            },
                        },

                        #[name = "media_info"]
                        gtk::Box {
                            add_css_class: "media-info",
                            set_orientation: gtk::Orientation::Vertical,
                            set_hexpand: true,
                            set_valign: gtk::Align::Center,

                            #[name = "track_label"]
                            gtk::Label {
                                add_css_class: "media-track",
                                set_halign: gtk::Align::Fill,
                                set_xalign: 0.0,
                                set_ellipsize: gtk::pango::EllipsizeMode::End,
                                set_max_width_chars: 1,
                                #[watch]
                                set_label: &model.title,
                            },

                            #[name = "artist_label"]
                            gtk::Label {
                                add_css_class: "media-artist",
                                set_halign: gtk::Align::Fill,
                                set_xalign: 0.0,
                                set_ellipsize: gtk::pango::EllipsizeMode::End,
                                set_max_width_chars: 1,
                                #[watch]
                                set_label: &model.artist,
                            },
                        },

                        #[name = "media_controls"]
                        gtk::Box {
                            add_css_class: "media-controls",
                            set_valign: gtk::Align::Center,

                            #[template]
                            #[name = "prev_btn"]
                            GhostIconButton {
                                add_css_class: "media-btn",
                                #[watch]
                                set_sensitive: model.can_previous,
                                connect_clicked => MediaSectionInput::PreviousClicked,

                                gtk::Image {
                                    set_icon_name: Some("ld-skip-back-symbolic"),
                                },
                            },

                            #[name = "play_pause_btn"]
                            gtk::Button {
                                set_css_classes: &["media-btn", "play"],
                                set_cursor_from_name: Some("pointer"),
                                connect_clicked => MediaSectionInput::PlayPauseClicked,

                                gtk::Image {
                                    #[watch]
                                    set_icon_name: Some(
                                        if model.playback_state == PlaybackState::Playing {
                                            "ld-pause-symbolic"
                                        } else {
                                            "ld-play-symbolic"
                                        }
                                    ),
                                },
                            },

                            #[template]
                            #[name = "next_btn"]
                            GhostIconButton {
                                add_css_class: "media-btn",
                                #[watch]
                                set_sensitive: model.can_next,
                                connect_clicked => MediaSectionInput::NextClicked,

                                gtk::Image {
                                    set_icon_name: Some("ld-skip-forward-symbolic"),
                                },
                            },
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

                            #[name = "elapsed_label"]
                            gtk::Label {
                                add_css_class: "media-time",
                                set_hexpand: true,
                                set_halign: gtk::Align::Start,
                                #[watch]
                                set_label: &helpers::format_duration(model.position),
                            },

                            #[name = "duration_label"]
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
                }
            } else {
                #[template]
                EmptyState {
                    #[template_child]
                    icon {
                        add_css_class: "sm",
                        set_icon_name: Some("ld-music-symbolic"),
                    },
                    #[template_child]
                    title {
                        set_label: &t!("dropdown-dashboard-no-media-title"),
                    },
                    #[template_child]
                    description {
                        set_label: &t!("dropdown-dashboard-no-media-description"),
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
        let art_css_provider = CssProvider::new();

        #[allow(clippy::expect_used)]
        let display = Display::default().expect("display required for dashboard media");
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
                commit_sender.emit(MediaSectionInput::SeekCommitted(percentage));
            }),
        );

        watchers::spawn(&sender, &init.media);

        let model = Self {
            media: init.media,
            player: None,
            player_watcher: WatcherToken::new(),
            is_active: false,

            art_css_provider,
            art_css_class: next_dashboard_art_css_class(),
            seek_slider: seek_slider.clone(),

            has_player: false,
            has_multiple_players: false,
            title: String::new(),
            artist: String::new(),
            cover_art: None,
            playback_state: PlaybackState::Stopped,
            position: Duration::ZERO,
            length: None,
            can_previous: false,
            can_next: false,
            can_seek: false,
        };

        let seek_slider = &model.seek_slider;
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            MediaSectionInput::SetActive(active) => {
                if self.is_active == active {
                    return;
                }

                self.is_active = active;
                let _ = self.player_watcher.reset();

                if active {
                    let active_player = self
                        .media
                        .as_ref()
                        .and_then(|media| media.active_player.get());
                    self.player = active_player;
                    sender.oneshot_command(async { MediaSectionCmd::PlayerChanged });
                }
            }

            MediaSectionInput::PreviousClicked => {
                self.fire_player_command(
                    &sender,
                    |player| async move { player.previous().await },
                    "previous failed",
                );
            }

            MediaSectionInput::PlayPauseClicked => {
                self.fire_player_command(
                    &sender,
                    |player| async move { player.play_pause().await },
                    "play/pause failed",
                );
            }

            MediaSectionInput::NextClicked => {
                self.fire_player_command(
                    &sender,
                    |player| async move { player.next().await },
                    "next failed",
                );
            }

            MediaSectionInput::SwitchPlayerClicked => {
                self.cycle_player(&sender);
            }

            MediaSectionInput::SeekCommitted(percentage) => {
                let Some(length) = self.length else {
                    return;
                };

                let target =
                    Duration::from_secs_f64(length.as_secs_f64() * percentage / PERCENTAGE_SCALE);

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
        msg: MediaSectionCmd,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            MediaSectionCmd::PlayerChanged => {
                self.handle_player_changed(&sender);
            }

            MediaSectionCmd::MetadataChanged {
                title,
                artist,
                cover_art,
                length,
            } => {
                self.title = title;
                self.artist = artist;
                self.cover_art = cover_art;
                self.length = length;

                self.update_artwork_css();
            }

            MediaSectionCmd::PlaybackStateChanged(state) => {
                self.playback_state = state;
            }

            MediaSectionCmd::PositionTick(position) => {
                self.position = position;

                self.seek_slider
                    .set_value(self.progress_fraction() * PERCENTAGE_SCALE);
            }

            MediaSectionCmd::CanSeekChanged(can_seek) => {
                self.can_seek = can_seek;
            }

            MediaSectionCmd::PlayerListChanged(count) => {
                self.has_multiple_players = count > 1;
            }

            MediaSectionCmd::Noop => {}
        }
    }
}
