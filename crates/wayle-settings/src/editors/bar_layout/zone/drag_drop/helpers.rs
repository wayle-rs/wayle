//! Pure hit-testing and highlight helpers for FlowBox drop targets.

use gtk::prelude::*;
use relm4::gtk;

pub(super) fn compute_drop_position(container: &gtk::FlowBox, drop_x: f64, drop_y: f64) -> usize {
    let children = flow_children(container);

    if let Some(hit) = container.child_at_pos(drop_x as i32, drop_y as i32) {
        let hit_index = hit.index() as usize;
        let Some(bounds) = hit.compute_bounds(container) else {
            return hit_index;
        };

        let center_x = f64::from(bounds.x() + bounds.width() / 2.0);
        if drop_x < center_x {
            return hit_index;
        }
        return hit_index + 1;
    }

    let mut row_last: Option<usize> = None;

    for (index, child) in children.iter().enumerate() {
        let Some(bounds) = child.compute_bounds(container) else {
            continue;
        };

        let row_top = f64::from(bounds.y());
        let row_bottom = f64::from(bounds.y() + bounds.height());

        if drop_y < row_top || drop_y >= row_bottom {
            continue;
        }

        row_last = Some(index);
    }

    row_last.map_or(children.len(), |index| index + 1)
}

pub(super) fn highlight_drop_position(container: &gtk::FlowBox, position: usize) {
    let children = flow_children(container);

    if let Some(target) = children.get(position) {
        target.add_css_class("drop-before");
        return;
    }

    if let Some(last) = children.last() {
        last.add_css_class("drop-after");
    }
}

pub(super) fn clear_drop_highlight(container: &gtk::FlowBox) {
    for child in flow_children(container) {
        child.remove_css_class("drop-before");
        child.remove_css_class("drop-after");
    }
}

fn flow_children(container: &gtk::FlowBox) -> Vec<gtk::FlowBoxChild> {
    let mut children = Vec::new();
    let mut index = 0;

    while let Some(child) = container.child_at_index(index) {
        children.push(child);
        index += 1;
    }

    children
}
