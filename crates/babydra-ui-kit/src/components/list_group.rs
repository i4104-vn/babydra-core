use gtk4::prelude::*;

/// Creates a standard row with an icon badge, title, subtitle, and an optional right-aligned widget.
pub fn create_list_row(
    icon_name: &str,
    title: &str,
    subtitle: &str,
    right_widget: Option<&impl IsA<gtk4::Widget>>,
) -> gtk4::Box {
    crate::components::lists::create_action_row(
        Some(icon_name),
        title,
        Some(subtitle),
        right_widget,
    )
}

/// Creates a standard row with an icon badge, title, subtitle, and an interactive CustomSwitch.
pub fn create_switch_row(
    icon_name: &str,
    title: &str,
    subtitle: &str,
    active: bool,
) -> (gtk4::Box, crate::components::switch::CustomSwitch) {
    let switch = crate::components::switch::CustomSwitch::new(active);
    switch.container.set_valign(gtk4::Align::Center);
    let row = create_list_row(icon_name, title, subtitle, Some(&switch.container));
    (row, switch)
}

/// Removes all children from a ListBox.
pub fn clear_list_box(list_box: &gtk4::ListBox) {
    while let Some(child) = list_box.first_child() {
        list_box.remove(&child);
    }
}

/// Removes all children from a Box container.
pub fn clear_box(box_container: &gtk4::Box) {
    while let Some(child) = box_container.first_child() {
        box_container.remove(&child);
    }
}

/// Removes all children from a FlowBox container.
pub fn clear_flow_box(flow_box: &gtk4::FlowBox) {
    while let Some(child) = flow_box.first_child() {
        flow_box.remove(&child);
    }
}
