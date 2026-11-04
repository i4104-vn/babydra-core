use gtk4::prelude::*;

/// Props-driven setting/action row. The application owns the trailing control
/// and receives its signals; this component only lays out the row.
pub struct ActionRow {
    root: gtk4::Box,
    text: gtk4::Box,
}

impl ActionRow {
    pub fn new(icon: Option<&str>, title: &str, subtitle: Option<&str>) -> Self {
        let root = gtk4::Box::new(gtk4::Orientation::Horizontal, 14);
        root.add_css_class("settings-card-row");
        root.add_css_class("bd-action-row");
        root.set_margin_top(4);
        root.set_margin_bottom(4);
        root.set_margin_start(8);
        root.set_margin_end(8);

        if let Some(icon) = icon.filter(|name| !name.is_empty()) {
            let badge = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
            badge.add_css_class("blue-icon-badge-sm");
            badge.set_valign(gtk4::Align::Center);
            let image = crate::ui::icon::get_icon(icon, 18);
            image.set_pixel_size(18);
            image.set_valign(gtk4::Align::Center);
            image.set_halign(gtk4::Align::Center);
            badge.append(&image);
            root.append(&badge);
        }

        let text = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
        text.set_valign(gtk4::Align::Center);
        text.set_hexpand(true);
        let title_label = gtk4::Label::new(Some(title));
        title_label.add_css_class("settings-row-title");
        title_label.add_css_class("settings-label");
        title_label.set_halign(gtk4::Align::Start);
        text.append(&title_label);
        if let Some(subtitle) = subtitle.filter(|value| !value.is_empty()) {
            let label = gtk4::Label::new(Some(subtitle));
            label.add_css_class("settings-row-desc");
            label.add_css_class("settings-desc");
            label.set_halign(gtk4::Align::Start);
            text.append(&label);
        }
        root.append(&text);
        Self { root, text }
    }

    pub fn append_action(&self, widget: &impl IsA<gtk4::Widget>) {
        widget.set_valign(gtk4::Align::Center);
        widget.set_halign(gtk4::Align::End);
        self.root.append(widget);
    }

    pub fn widget(&self) -> &gtk4::Box { &self.root }
    pub fn text(&self) -> &gtk4::Box { &self.text }
}

pub fn create_action_row(
    icon: Option<&str>,
    title: &str,
    subtitle: Option<&str>,
    action: Option<&impl IsA<gtk4::Widget>>,
) -> gtk4::Box {
    let row = ActionRow::new(icon, title, subtitle);
    if let Some(action) = action {
        row.append_action(action);
    }
    row.root
}
