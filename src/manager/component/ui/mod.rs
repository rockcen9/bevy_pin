use crate::{manager::component::query::ui::query_panel, prelude::*};

pub mod monitor_panel;

#[derive(Component, Clone, Default)]
#[require(DespawnOnExit::<AppState>(AppState::Component))]
struct ComponentPanelsRoot;

pub fn plugin(app: &mut App) {
    monitor_panel::plugin(app);
}
pub fn component_panels_root() -> impl Scene {
    bsn! {
        #ComponentPanelsRoot
        ComponentPanelsRoot
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            padding: UiRect::all(Val::Px(20.0)),
            column_gap: Val::Px(20.0),
        }
        Children [
            query_panel(),
            monitor_panel::monitor_panel(),
        ]
    }
}
