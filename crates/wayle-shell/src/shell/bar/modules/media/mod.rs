mod factory;
mod helpers;
mod messages;
mod methods;
mod watchers;

use std::{rc::Rc, sync::Arc};

use gtk::prelude::WidgetExt;
use relm4::prelude::*;
use wayle_config::{
    ConfigProperty, ConfigService,
    schemas::{modules::MediaIconType, styling::CssToken},
};
use wayle_media::{MediaService, types::PlaybackState};
use wayle_widgets::{
    WatcherToken,
    prelude::{
        BarButton, BarButtonBehavior, BarButtonColors, BarButtonInit, BarButtonInput,
        BarButtonOutput,
    },
};

pub(crate) use self::{
    factory::Factory,
    messages::{MediaCmd, MediaInit, MediaMsg},
};
use crate::shell::bar::dropdowns::{self, DropdownRegistry};

pub(crate) struct MediaModule {
    bar_button: Controller<BarButton>,
    config: Arc<ConfigService>,
    active_player_watcher_token: WatcherToken,
    media: Arc<MediaService>,
    dropdowns: Rc<DropdownRegistry>,
}

#[relm4::component(pub(crate))]
impl Component for MediaModule {
    type Init = MediaInit;
    type Input = MediaMsg;
    type Output = ();
    type CommandOutput = MediaCmd;

    view! {
        gtk::Box {
            add_css_class: "media",

            #[local_ref]
            bar_button -> gtk::MenuButton {},
        }
    }

    fn init(
        init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let config = init.config.config();
        let media_config = &config.modules.media;

        let visible = Self::update_visibility(media_config, &init.media);

        let bar_button = BarButton::builder()
            .launch(BarButtonInit {
                icon: media_config.icon_name.get().clone(),
                label: String::from("--"),
                tooltip: None,
                colors: BarButtonColors {
                    icon_color: media_config.icon_color.clone(),
                    label_color: media_config.label_color.clone(),
                    icon_background: media_config.icon_bg_color.clone(),
                    button_background: media_config.button_bg_color.clone(),
                    border_color: media_config.border_color.clone(),
                    auto_icon_color: CssToken::Blue,
                },
                behavior: BarButtonBehavior {
                    label_max_chars: media_config.label_max_length.clone(),
                    show_icon: media_config.icon_show.clone(),
                    show_label: media_config.label_show.clone(),
                    show_border: media_config.border_show.clone(),
                    visible: ConfigProperty::new(visible),
                },
                settings: init.settings,
            })
            .forward(sender.input_sender(), |output| match output {
                BarButtonOutput::LeftClick => MediaMsg::LeftClick,
                BarButtonOutput::RightClick => MediaMsg::RightClick,
                BarButtonOutput::MiddleClick => MediaMsg::MiddleClick,
                BarButtonOutput::ScrollUp => MediaMsg::ScrollUp,
                BarButtonOutput::ScrollDown => MediaMsg::ScrollDown,
            });

        watchers::spawn_watchers(&sender, media_config, &init.media);

        let model = Self {
            bar_button,
            config: init.config,
            active_player_watcher_token: WatcherToken::new(),
            media: init.media,
            dropdowns: init.dropdowns,
        };
        let bar_button = model.bar_button.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        let config = &self.config.config().modules.media;

        let action = match msg {
            MediaMsg::LeftClick => config.left_click.get(),
            MediaMsg::RightClick => config.right_click.get(),
            MediaMsg::MiddleClick => config.middle_click.get(),
            MediaMsg::ScrollUp => config.scroll_up.get(),
            MediaMsg::ScrollDown => config.scroll_down.get(),
        };

        dropdowns::dispatch_click(&action, &self.dropdowns, &self.bar_button);
    }

    fn update_cmd(&mut self, msg: MediaCmd, sender: ComponentSender<Self>, root: &Self::Root) {
        let media_config = &self.config.config().modules.media;

        match msg {
            MediaCmd::PlayerChanged(player) => {
                let use_disc =
                    player.is_some() && media_config.icon_type.get() == MediaIconType::SpinningDisc;
                Self::update_disc_mode(root, use_disc);

                // Update visibility when the active player changes.
                let visible = Self::update_visibility(media_config, &self.media);
                self.bar_button.emit(BarButtonInput::SetVisible(visible));

                if let Some(player) = player {
                    let label = helpers::build_label(media_config, &player);
                    self.bar_button.emit(BarButtonInput::SetLabel(label));

                    let icon = helpers::build_icon(media_config, &player);
                    self.bar_button.emit(BarButtonInput::SetIcon(icon));

                    let state = player.playback_state.get();
                    Self::update_spinning_state(root, state);

                    let token = self.active_player_watcher_token.reset();
                    watchers::spawn_player_watchers(&sender, &player, token);
                } else {
                    self.bar_button
                        .emit(BarButtonInput::SetLabel(String::from("--")));
                    self.bar_button
                        .emit(BarButtonInput::SetIcon(media_config.icon_name.get()));
                    Self::update_spinning_state(root, PlaybackState::Stopped);
                }
            }
            MediaCmd::MetadataChanged => {
                if let Some(player) = self.media.active_player() {
                    let label = helpers::build_label(media_config, &player);
                    self.bar_button.emit(BarButtonInput::SetLabel(label));
                }
            }
            MediaCmd::PlaybackStateChanged => {
                // Update visibility when playback status (playing/paused) changes.
                let visible = Self::update_visibility(media_config, &self.media);
                self.bar_button.emit(BarButtonInput::SetVisible(visible));

                if let Some(player) = self.media.active_player() {
                    let label = helpers::build_label(media_config, &player);
                    self.bar_button.emit(BarButtonInput::SetLabel(label));
                    let state = player.playback_state.get();
                    Self::update_spinning_state(root, state);
                }
            }
            MediaCmd::VisibilityChanged => {
                // Update visibility when the 'show-only-playing' config option is toggled.
                let visible = Self::update_visibility(media_config, &self.media);
                self.bar_button.emit(BarButtonInput::SetVisible(visible));
            }
            MediaCmd::UpdateIcon(icon) => {
                self.bar_button.emit(BarButtonInput::SetIcon(icon));
            }
            MediaCmd::IconTypeChanged => {
                let use_disc = self.media.active_player().is_some()
                    && media_config.icon_type.get() == MediaIconType::SpinningDisc;
                Self::update_disc_mode(root, use_disc);

                if let Some(player) = self.media.active_player() {
                    let icon = helpers::build_icon(media_config, &player);
                    self.bar_button.emit(BarButtonInput::SetIcon(icon));

                    let state = player.playback_state.get();
                    Self::update_spinning_state(root, state);
                }
            }
        }
    }
}
