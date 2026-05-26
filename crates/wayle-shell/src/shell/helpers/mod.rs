mod bootstrap;
pub(crate) mod layer_shell;
pub(crate) mod monitors;

pub(crate) use bootstrap::{
    COMPONENT_CSS_PRIORITY, init_css_provider, init_icons, register_app_actions,
};
