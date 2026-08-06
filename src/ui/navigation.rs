use super::i18n;
use super::routes::Route;

pub struct NavigationItem {
    pub id: &'static str,
    pub key: &'static str,
    pub icon_path: &'static str,
    pub route: Route,
}

impl NavigationItem {
    pub fn label(&self) -> String {
        i18n::t(self.key)
    }
}

pub const MAIN_NAV_ITEMS: &[NavigationItem] = &[
    NavigationItem {
        id: "nav-home",
        key: "nav.home",
        icon_path: "assets/icons/audio-waveform.svg",
        route: Route::Home,
    },
    NavigationItem {
        id: "nav-plugins",
        key: "nav.plugins",
        icon_path: "assets/icons/box.svg",
        route: Route::Plugins,
    },
];

pub const FOOTER_NAV_ITEM: NavigationItem = NavigationItem {
    id: "nav-settings",
    key: "nav.settings",
    icon_path: "assets/icons/settings.svg",
    route: Route::Settings,
};
