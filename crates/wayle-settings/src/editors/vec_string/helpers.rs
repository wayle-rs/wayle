use super::VecStringControl;

pub(crate) trait TextLike: Clone + Send + Sync + PartialEq + 'static {
    fn to_entry_text(&self) -> String;
    fn from_entry_text(text: &str) -> Self;
}

impl TextLike for Vec<String> {
    fn to_entry_text(&self) -> String {
        VecStringControl::to_entry_text(self)
    }

    fn from_entry_text(text: &str) -> Self {
        VecStringControl::from_entry_text(text)
    }
}
