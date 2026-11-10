use relm4::ComponentController;

use super::{
    MullvadDropdown, country_item::CountryItemInit, current_connection::CurrentConnectionInput,
};

impl MullvadDropdown {
    /// Rebuilds the country -> city -> relay tree from the current relay list.
    pub(super) fn rebuild_countries(&mut self) {
        let Some(service) = &self.mullvad else {
            return;
        };

        let mut countries = service.mullvad.networks.get();
        countries.sort_by(|a, b| a.name.cmp(&b.name));

        let mut guard = self.countries.guard();
        guard.clear();
        for country in countries {
            guard.push_back(CountryItemInit {
                name: country.name,
                code: country.code,
                cities: country.cities,
            });
        }
    }

    /// Pushes the current connection status + selected relay into the pinned
    /// card (the crate resolves both to display-ready names).
    pub(super) fn push_status(&self) {
        let Some(service) = &self.mullvad else {
            return;
        };

        self.current.emit(CurrentConnectionInput::SetState(
            service.mullvad.status.get(),
            service.mullvad.selected.get(),
        ));
    }
}
