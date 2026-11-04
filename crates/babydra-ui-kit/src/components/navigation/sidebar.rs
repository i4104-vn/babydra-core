use gtk4::prelude::*;

/// Standard sidebar scaffold shared by Settings and Explore.
///
/// Applications provide their own navigation data and event handling through
/// the header, navigation and footer slots.
pub struct SidebarLayout {
    root: gtk4::Box,
    header: gtk4::Box,
    scroll: gtk4::ScrolledWindow,
    navigation: gtk4::Box,
    footer: gtk4::Box,
}

impl SidebarLayout {
    pub fn new(css_class: &str) -> Self {
        let root = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
        root.add_css_class("sidebar");
        root.add_css_class("bd-sidebar-layout");
        if !css_class.is_empty() {
            root.add_css_class(css_class);
        }
        root.set_width_request(68);
        root.set_hexpand(false);
        root.set_vexpand(true);
        root.set_margin_top(12);
        root.set_margin_bottom(12);
        root.set_margin_start(10);
        root.set_margin_end(2);
        root.set_halign(gtk4::Align::Center);

        let header = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        header.add_css_class("bd-sidebar-header");
        header.set_halign(gtk4::Align::Center);

        let scroll = gtk4::ScrolledWindow::new();
        scroll.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);
        scroll.set_vexpand(true);
        scroll.set_halign(gtk4::Align::Center);

        let navigation = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        navigation.add_css_class("bd-sidebar-navigation");
        navigation.set_halign(gtk4::Align::Center);
        scroll.set_child(Some(&navigation));

        let footer = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        footer.add_css_class("bd-sidebar-footer");
        footer.set_halign(gtk4::Align::Center);
        footer.set_margin_bottom(6);

        root.append(&header);
        root.append(&scroll);
        root.append(&footer);

        Self { root, header, scroll, navigation, footer }
    }

    pub fn widget(&self) -> &gtk4::Box { &self.root }
    pub fn header(&self) -> &gtk4::Box { &self.header }
    pub fn scroll(&self) -> &gtk4::ScrolledWindow { &self.scroll }
    pub fn navigation(&self) -> &gtk4::Box { &self.navigation }
    pub fn footer(&self) -> &gtk4::Box { &self.footer }
}
