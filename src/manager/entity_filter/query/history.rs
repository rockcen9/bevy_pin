use crate::prelude::*;
use crate::ui_layout::theme::palette::{
    COLOR_BUTTON_BG, COLOR_BUTTON_HOVER, COLOR_DESTRUCTIVE_HOVER, COLOR_INPUT_TEXT,
    COLOR_LABEL_TERTIARY, COLOR_MENU_NORMAL,
};
use crate::ui_layout::theme::widgets::{close_button, titled_panel, ScrollableContainer};

#[derive(Resource, Default)]
pub struct QueryHistory(pub Vec<String>);

impl QueryHistory {
    pub fn add(&mut self, value: String) {
        debug!("QueryHistory::add: {}", value);
        self.0.retain(|q| q != &value);
        self.0.push(value);
    }
}

/// Set to `Some(query)` when a history item is clicked; cleared by the insert system.
#[derive(Resource, Default)]
pub struct SelectedHistoryQuery(pub Option<String>);

#[derive(Component, Clone)]
pub struct HistoryItem(pub String);

#[derive(Component, Clone)]
pub struct RemoveHistoryItem(pub String);

pub fn query_history_panel() -> impl Scene {
    titled_panel("Query History", "query-history", 300.0)
}

pub fn plugin(app: &mut App) {
    app.init_resource::<QueryHistory>();
    app.init_resource::<SelectedHistoryQuery>();
    app.add_systems(
        Update,
        (
            rebuild_history_panel,
            handle_history_item_click,
            handle_remove_history_item_click,
            update_history_item_hover,
            update_remove_button_hover,
        ),
    );
}

fn history_item(query_str: String) -> impl Scene {
    let qs_item = query_str.clone();
    let qs_text = query_str.clone();
    let qs_remove = query_str;
    bsn! {
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            width: Val::Percent(100.0),
            column_gap: Val::Px(4.0),
        }
        Children [
            (
                template(move |_| Ok(HistoryItem(qs_item.clone())))
                Button
                Node {
                    padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    flex_grow: 1.0,
                }
                BackgroundColor(COLOR_BUTTON_BG)
                Children [(
                    template(move |_| Ok(Text::new(qs_text.clone())))
                    template(|_| Ok(TextFont::from_font_size(13.0)))
                    TextColor(COLOR_INPUT_TEXT)
                )]
            ),
            close_button(RemoveHistoryItem(qs_remove.clone())),
        ]
    }
}

fn empty_history_hint() -> impl Scene {
    bsn! {
        Text::new("No history yet")
        template(|_| Ok(TextFont::from_font_size(12.0)))
        TextColor(COLOR_LABEL_TERTIARY)
    }
}

fn rebuild_history_panel(
    history: Res<QueryHistory>,
    containers: Query<(Entity, &ScrollableContainer)>,
    added_containers: Query<&ScrollableContainer, Added<ScrollableContainer>>,
    mut commands: Commands,
) {
    let is_newly_added = added_containers.iter().any(|c| c.0 == "query-history");
    if !history.is_changed() && !is_newly_added {
        return;
    }
    let Some((container, _)) = containers.iter().find(|(_, c)| c.0 == "query-history") else {
        return;
    };
    debug!("rebuild_history_panel: {} entries", history.0.len());
    commands.entity(container).despawn_children();

    if history.0.is_empty() {
        let child = commands.spawn_scene(empty_history_hint()).id();
        commands.entity(container).add_child(child);
        return;
    }

    for query_str in history.0.iter().rev() {
        let child = commands.spawn_scene(history_item(query_str.clone())).id();
        commands.entity(container).add_child(child);
    }
}

fn handle_history_item_click(
    items: Query<(&Interaction, &HistoryItem), Changed<Interaction>>,
    mut selected: ResMut<SelectedHistoryQuery>,
) {
    for (interaction, item) in &items {
        if *interaction == Interaction::Pressed {
            debug!("history item clicked: {}", item.0);
            selected.0 = Some(item.0.clone());
        }
    }
}

fn update_history_item_hover(
    mut items: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<HistoryItem>),
    >,
) {
    for (interaction, mut color) in &mut items {
        let new_color = match interaction {
            Interaction::Hovered => COLOR_BUTTON_HOVER,
            _ => COLOR_BUTTON_BG,
        };
        color.set_if_neq(BackgroundColor(new_color));
    }
}

fn handle_remove_history_item_click(
    items: Query<(&Interaction, &RemoveHistoryItem), Changed<Interaction>>,
    mut history: ResMut<QueryHistory>,
) {
    for (interaction, item) in &items {
        if *interaction == Interaction::Pressed {
            debug!("remove history item: {}", item.0);
            history.0.retain(|q| q != &item.0);
        }
    }
}

fn update_remove_button_hover(
    mut items: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<RemoveHistoryItem>),
    >,
) {
    for (interaction, mut color) in &mut items {
        let new_color = match interaction {
            Interaction::Hovered => COLOR_DESTRUCTIVE_HOVER.with_alpha(0.15),
            _ => COLOR_MENU_NORMAL,
        };
        color.set_if_neq(BackgroundColor(new_color));
    }
}
