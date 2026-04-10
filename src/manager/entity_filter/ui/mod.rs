use crate::prelude::*;

#[derive(Component, Clone, Default)]
struct LeftQueryRoot;

#[derive(Component, Clone, Default)]
struct RightInfoPanelRoot;
pub fn plugin(_app: &mut App) {}
pub fn left_query_root() -> impl Scene {
    bsn! {
        #LeftQueryRoot
        LeftQueryRoot
        DespawnOnExit::<SidebarState>(SidebarState::EntityFilter)
        Node {
            flex_grow: 1.0,
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::FlexStart,
            padding: UiRect::all(Val::Px(20.0)),
            column_gap: Val::Px(20.0),
            overflow: Overflow::scroll_x(),
        }
    }
}
pub fn right_info_root() -> impl Scene {
    bsn! {
        #RightInfoPanelRoot
        RightInfoPanelRoot
        DespawnOnExit::<SidebarState>(SidebarState::EntityFilter)
        Node {
            width: Val::Px(640.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::FlexStart,
            padding: UiRect::all(Val::Px(20.0)),
            column_gap: Val::Px(20.0),
            overflow: Overflow::scroll_x(),
        }
    }
}
