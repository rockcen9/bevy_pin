use crate::manager::entity_lookup::FoundEntity;
// use crate::manager::entity_filter::component_list::InspectedEntity;
use crate::prelude::*;
use crate::ui_layout::theme::palette::{
    COLOR_BUTTON_BG, COLOR_BUTTON_HOVER, COLOR_INPUT_TEXT, COLOR_SEPARATOR,
};
use crate::ui_layout::theme::widgets::{ScrollableContainer, titled_panel};

pub struct LookupEntry {
    pub entity_id: u64,
    pub name: Option<String>,
}

#[derive(Resource, Default)]
pub struct LookupHistory(pub Vec<LookupEntry>);

impl LookupHistory {
    pub fn push(&mut self, entity_id: u64) {
        self.0.retain(|e| e.entity_id != entity_id);
        self.0.insert(
            0,
            LookupEntry {
                entity_id,
                name: None,
            },
        );
    }

    pub fn update_name(&mut self, entity_id: u64, name: String) {
        if let Some(entry) = self.0.iter_mut().find(|e| e.entity_id == entity_id) {
            entry.name = Some(name);
        }
    }
}

#[derive(Component, Clone)]
struct HistoryItem(u64);

pub fn plugin(app: &mut App) {
    app.init_resource::<LookupHistory>().add_systems(
        Update,
        (
            rebuild_history_panel,
            handle_history_item_click,
            update_history_item_hover,
        ),
    );
}

pub fn history_panel() -> impl Scene {
    bsn! {
        titled_panel("Lookup History", "lookup-history", 300.0)
        DespawnOnExit::<SidebarState>(SidebarState::EntityLookup)
    }
}

fn rebuild_history_panel(
    history: Res<LookupHistory>,
    containers: Query<(Entity, &ScrollableContainer)>,
    added_containers: Query<&ScrollableContainer, Added<ScrollableContainer>>,
    mut commands: Commands,
) {
    let is_newly_added = added_containers.iter().any(|c| c.0 == "lookup-history");
    if !history.is_changed() && !is_newly_added {
        return;
    }

    for (container, _) in containers.iter().filter(|(_, c)| c.0 == "lookup-history") {
        commands.entity(container).despawn_children();

        if history.0.is_empty() {
            let child = commands
                .spawn((
                    Text::new("No lookups yet"),
                    TextFont::from_font_size(12.0),
                    TextColor(COLOR_SEPARATOR),
                ))
                .id();
            commands.entity(container).add_child(child);
            continue;
        }

        for entry in &history.0 {
            let entity_id = entry.entity_id;
            let label = match &entry.name {
                Some(name) => format!("{} {}", crate::utils::entity_display_label(entity_id), name),
                None => crate::utils::entity_display_label(entity_id),
            };
            let child = commands
                .spawn((
                    Button,
                    HistoryItem(entity_id),
                    Node {
                        padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                        border_radius: BorderRadius::all(Val::Px(4.0)),
                        width: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(COLOR_BUTTON_BG),
                ))
                .with_child((
                    Text::new(label),
                    TextFont::from_font_size(13.0),
                    TextColor(COLOR_INPUT_TEXT),
                ))
                .id();
            commands.entity(container).add_child(child);
        }
    }
}

fn handle_history_item_click(
    items: Query<(&Interaction, &HistoryItem), Changed<Interaction>>,
    mut inspected: ResMut<FoundEntity>,
) {
    for (interaction, item) in &items {
        if *interaction == Interaction::Pressed {
            debug!("EntityLookup history: selecting entity #{}", item.0);
            inspected.0 = Some(item.0);
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
        color.set_if_neq(BackgroundColor(match interaction {
            Interaction::Hovered => COLOR_BUTTON_HOVER,
            _ => COLOR_BUTTON_BG,
        }));
    }
}
