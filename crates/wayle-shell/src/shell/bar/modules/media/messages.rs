use std::{rc::Rc, sync::Arc};

use wayle_config::ConfigService;
use wayle_media::{MediaService, core::player::Player};
use wayle_widgets::prelude::BarSettings;

use crate::shell::bar::dropdowns::DropdownRegistry;

pub(crate) struct MediaInit {
    pub settings: BarSettings,
    pub media: Arc<MediaService>,
    pub config: Arc<ConfigService>,
    pub dropdowns: Rc<DropdownRegistry>,
}

#[derive(Debug)]
pub(crate) enum MediaMsg {
    LeftClick,
    RightClick,
    MiddleClick,
    ScrollUp,
    ScrollDown,
}

#[derive(Debug)]
pub(crate) enum MediaCmd {
    PlayerChanged(Option<Arc<Player>>),
    MetadataChanged,
    PlaybackStateChanged,
    VisibilityChanged,
    UpdateIcon(String),
    IconTypeChanged,
}
