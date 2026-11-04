use gtk4::prelude::*;

/// Shared horizontal application header with leading, title and action slots.
pub struct HeaderBar {
    root: gtk4::Box,
    leading: gtk4::Box,
    content: gtk4::Box,
    title: gtk4::Label,
    actions: gtk4::Box,
}

impl HeaderBar {
    pub fn new(title: &str) -> Self {
        let root = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        root.add_css_class("bd-header-bar");
        root.set_hexpand(true);
        root.set_valign(gtk4::Align::Center);

        let leading = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
        leading.add_css_class("bd-header-bar-leading");
        leading.set_valign(gtk4::Align::Center);

        let content = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        content.add_css_class("bd-header-bar-content");
        content.set_hexpand(true);
        content.set_valign(gtk4::Align::Center);

        let title = gtk4::Label::new(Some(title));
        title.add_css_class("bd-header-bar-title");
        title.set_halign(gtk4::Align::Start);
        title.set_hexpand(true);
        title.set_ellipsize(gtk4::pango::EllipsizeMode::End);

        let actions = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
        actions.add_css_class("bd-header-bar-actions");
        actions.set_valign(gtk4::Align::Center);

        root.append(&leading);
        root.append(&content);
        root.append(&title);
        root.append(&actions);
        if title.text().is_empty() {
            title.set_visible(false);
        }
        Self { root, leading, content, title, actions }
    }

    pub fn widget(&self) -> &gtk4::Box { &self.root }
    pub fn leading(&self) -> &gtk4::Box { &self.leading }
    pub fn content(&self) -> &gtk4::Box { &self.content }
    pub fn title(&self) -> &gtk4::Label { &self.title }
    pub fn actions(&self) -> &gtk4::Box { &self.actions }
}
