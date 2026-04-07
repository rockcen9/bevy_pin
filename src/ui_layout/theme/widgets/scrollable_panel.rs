use crate::prelude::*;
use crate::ui_layout::theme::palette::{
    COLOR_HEADER_BG, COLOR_PANEL_BG, COLOR_SCROLLBAR_THUMB, COLOR_SCROLLBAR_TRACK, COLOR_TITLE,
};
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::picking::hover::HoverMap;

/// Marker for the scrollable content container.
/// Use the same `key` string to add children or query scroll state from outside the widget.
#[derive(Component, Clone, Default)]
pub struct ScrollableContainer(pub String);

#[derive(Component, Clone, Default)]
struct ScrollbarTrack(String);

#[derive(Component, Clone, Default)]
struct ScrollbarThumb(String);

pub fn plugin(app: &mut App) {
    app.add_observer(on_scrollbar_track_added);
    app.add_observer(on_scrollbar_thumb_added);
    app.add_systems(Update, (update_scrollbar, scroll_on_mouse_wheel));
}

/// A panel with a title header and a scrollable list body. No close button.
pub fn titled_panel(title: impl Into<String>, key: impl Into<String>, max_height: f32) -> impl Scene {
    let title = title.into();
    let key = key.into();
    bsn! {
        Node {
            flex_direction: FlexDirection::Column,
            min_width: Val::Px(280.0),
            max_width: Val::Px(280.0),
            border_radius: BorderRadius::all(Val::Px(10.0)),
        }
        BackgroundColor(COLOR_PANEL_BG)
        Children [
            (
                Node {
                    padding: UiRect::axes(Val::Px(14.0), Val::Px(10.0)),
                    border_radius: BorderRadius::top(Val::Px(10.0)),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                }
                BackgroundColor(COLOR_HEADER_BG)
                Children [(
                    Text::new( title.clone() )
                    template(|_| Ok(TextFont::from_font_size(18.0)))
                    TextColor(COLOR_TITLE)
                )]
            ),
            scrollable_list(key, max_height),
        ]
    }
}

/// A scrollable list area with an auto-hiding scrollbar.
/// `key` uniquely identifies this instance. `max_height` caps the visible height in pixels.
pub fn scrollable_list(key: impl Into<String>, max_height: f32) -> impl Scene {
    let key = key.into();
    let track_key = key.clone();
    let thumb_key = key.clone();
    let content_key = key.clone();
    bsn! {
        Node {
            flex_direction: FlexDirection::Row,
            max_height: Val::Px({ max_height }),
        }
        Children [
            (
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.0),
                    padding: UiRect::all(Val::Px(10.0)),
                    flex_grow: 1.0,
                    overflow: Overflow::scroll_y(),
                }
                ScrollPosition::default()
                ScrollableContainer({ content_key.clone() })
            ),
            (
                Node {
                    width: Val::Px(6.0),
                    align_self: AlignSelf::Stretch,
                }
                BackgroundColor(COLOR_SCROLLBAR_TRACK)
                ScrollbarTrack({ track_key.clone() })
                Children [(
                    Node {
                        position_type: PositionType::Absolute,
                        width: Val::Px(6.0),
                        top: Val::Px(0.0),
                        border_radius: BorderRadius::all(Val::Px(3.0)),
                    }
                    Pickable::default()
                    BackgroundColor(COLOR_SCROLLBAR_THUMB)
                    ScrollbarThumb({ thumb_key.clone() })
                )]
            ),
        ]
    }
}

fn on_scrollbar_track_added(trigger: On<Add, ScrollbarTrack>, mut commands: Commands) {
    commands.entity(trigger.entity).insert(Visibility::Hidden);
}

fn on_scrollbar_thumb_added(trigger: On<Add, ScrollbarThumb>, mut commands: Commands) {
    commands.entity(trigger.entity).observe(on_thumb_drag);
}

fn on_thumb_drag(
    drag: On<Pointer<Drag>>,
    thumbs: Query<&ScrollbarThumb>,
    tracks: Query<(&ScrollbarTrack, &ComputedNode)>,
    mut containers: Query<(&ScrollableContainer, &mut ScrollPosition, &ComputedNode)>,
) {
    let Ok(thumb) = thumbs.get(drag.entity) else {
        return;
    };
    let Some((_, track_computed)) = tracks.iter().find(|(t, _)| t.0 == thumb.0) else {
        return;
    };
    let Some((_, mut scroll_pos, computed)) =
        containers.iter_mut().find(|(c, _, _)| c.0 == thumb.0)
    else {
        return;
    };

    let scale = computed.inverse_scale_factor();
    let content_h = computed.content_size().y * scale;
    let visible_h = computed.size().y * scale;
    let max_scroll = (content_h - visible_h).max(0.0);
    if max_scroll <= 0.0 {
        return;
    }

    let track_h = track_computed.size().y * track_computed.inverse_scale_factor();
    let thumb_h = (visible_h / content_h * track_h).max(20.0);
    let scroll_range = (track_h - thumb_h).max(1.0);
    scroll_pos.0.y = (scroll_pos.0.y + drag.delta.y * max_scroll / scroll_range)
        .clamp(0.0, max_scroll);
}

fn update_scrollbar(
    containers: Query<(&ScrollableContainer, &ScrollPosition, &ComputedNode)>,
    mut thumbs: Query<(&ScrollbarThumb, &mut Node)>,
    mut tracks: Query<(&ScrollbarTrack, &mut Visibility, &ComputedNode)>,
) {
    for (container, scroll_pos, computed) in &containers {
        let scale = computed.inverse_scale_factor();
        let content_h = computed.content_size().y * scale;
        let visible_h = computed.size().y * scale;

        let Some((_, mut track_vis, track_computed)) =
            tracks.iter_mut().find(|(t, _, _)| t.0 == container.0)
        else {
            continue;
        };

        let overflows = content_h > visible_h && content_h > 0.0;
        track_vis.set_if_neq(if overflows {
            Visibility::Visible
        } else {
            Visibility::Hidden
        });

        let Some((_, mut thumb_node)) = thumbs.iter_mut().find(|(t, _)| t.0 == container.0) else {
            continue;
        };

        if !overflows {
            thumb_node.height = Val::Percent(100.0);
            thumb_node.top = Val::Px(0.0);
            continue;
        }

        let track_h = track_computed.size().y * track_computed.inverse_scale_factor();
        let thumb_h = (visible_h / content_h * track_h).max(20.0);
        let max_scroll = content_h - visible_h;
        let thumb_top = (scroll_pos.0.y / max_scroll) * (track_h - thumb_h);

        thumb_node.height = Val::Px(thumb_h);
        thumb_node.top = Val::Px(thumb_top.clamp(0.0, track_h - thumb_h));
    }
}

fn scroll_on_mouse_wheel(
    mut mouse_wheel: MessageReader<MouseWheel>,
    hover_map: Res<HoverMap>,
    parents: Query<&ChildOf>,
    mut containers: Query<(&mut ScrollPosition, &ComputedNode), With<ScrollableContainer>>,
) {
    let total_delta: f32 = mouse_wheel
        .read()
        .map(|e| match e.unit {
            MouseScrollUnit::Line => -e.y * 20.0,
            MouseScrollUnit::Pixel => -e.y,
        })
        .sum();

    if total_delta == 0.0 {
        return;
    }

    let mut scrolled: HashSet<Entity> = Default::default();

    for hit_map in hover_map.values() {
        for &hovered in hit_map.keys() {
            let mut entity = hovered;
            loop {
                if containers.contains(entity) && scrolled.insert(entity) {
                    if let Ok((mut scroll, computed)) = containers.get_mut(entity) {
                        let scale = computed.inverse_scale_factor();
                        let max_scroll =
                            ((computed.content_size().y - computed.size().y) * scale).max(0.0);
                        scroll.0.y = (scroll.0.y + total_delta).clamp(0.0, max_scroll);
                    }
                    break;
                }
                match parents.get(entity) {
                    Ok(child_of) => entity = child_of.parent(),
                    Err(_) => break,
                }
            }
        }
    }
}
