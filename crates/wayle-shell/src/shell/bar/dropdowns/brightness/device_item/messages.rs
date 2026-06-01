pub(crate) struct BrightnessDeviceInit {
    pub name: String,
    pub subtitle: Option<String>,
    pub icon: &'static str,
    pub percentage: f64,
}

#[derive(Debug)]
pub(crate) enum BrightnessDeviceItemMsg {
    SetBackendBrightness(f64),
    BrightnessCommitted(f64),
}

#[derive(Debug)]
pub(crate) enum BrightnessDeviceItemOutput {
    BrightnessChanged(String, f64),
}
