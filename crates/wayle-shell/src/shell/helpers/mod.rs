mod bootstrap;
pub(crate) mod layer_shell;
pub(crate) mod monitors;
pub(crate) mod surface_commit;

pub(crate) use bootstrap::{init_css_provider, init_icons, register_app_actions};
pub(crate) use wayle_styling::COMPONENT_CSS_PRIORITY;
