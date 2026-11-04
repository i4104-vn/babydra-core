use gtk4::prelude::*;

/// A shared card-like group for app-provided action rows.
pub struct ListGroup {
    root: gtk4::Box,
    rows: gtk4::Box,
}

impl ListGroup {
    pub fn new(title: Option<&str>) -> Self {
        let root = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        root.add_css_class("bd-list-group");
        root.add_css_class("settings-card");
        if let Some(title) = title.filter(|value| !value.is_empty()) {
            let label = gtk4::Label::new(Some(title));
            label.add_css_class("bd-list-group-title");
            label.add_css_class("settings-card-title");
            label.set_halign(gtk4::Align::Start);
            root.append(&label);
        }
        let rows = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        rows.add_css_class("bd-list-group-rows");
        root.append(&rows);
        Self { root, rows }
    }

    pub fn append(&self, row: &impl IsA<gtk4::Widget>) { self.rows.append(row); }
    pub fn widget(&self) -> &gtk4::Box { &self.root }
    pub fn rows(&self) -> &gtk4::Box { &self.rows }
}
