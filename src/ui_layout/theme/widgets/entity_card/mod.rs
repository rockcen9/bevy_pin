mod components;
mod interaction;
mod layout;
mod resize;
mod rpc;

pub use components::{
    DragHandle, EntityCard, PinCardDataCache, PinCardEntry, PinCardExpandState,
    PinCardHighlight, PinCardTitle, pincard_key,
};
pub use layout::*;

use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.init_resource::<components::PinCardExpandState>()
        .init_resource::<components::PinCardDataCache>()
        .init_resource::<components::PinCardKnownMarkerComponents>()
        .add_observer(interaction::on_drag_handle_added)
        .add_observer(resize::on_resize_handle_added)
        .add_observer(resize::on_resize_handle_bottom_added)
        .add_observer(resize::on_resize_handle_left_added)
        .add_observer(resize::on_resize_handle_top_added)
        .add_observer(resize::on_resize_corner_br_added)
        .add_observer(resize::on_resize_corner_bl_added)
        .add_observer(resize::on_resize_corner_tr_added)
        .add_observer(resize::on_resize_corner_tl_added)
        .add_systems(
            Update,
            (
                rpc::init_entity_cards,
                interaction::drive_pincard_highlight,
                rpc::trigger_initial_fetch,
                rpc::tick_pincard_polls,
                interaction::update_header_hover,
                interaction::handle_expand_toggle,
                interaction::render_from_cache_on_expand_change
                    .after(interaction::handle_expand_toggle),
                rpc::submit_pincard_field,
                rpc::handle_pincard_insert_submit,
                rpc::handle_pincard_remove_component_button,
                interaction::auto_select_on_focus,
                interaction::restore_scroll_height,
            ),
        );
}
