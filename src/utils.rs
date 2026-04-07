pub fn entity_display_label(raw_id: u64) -> String {
    let entity_index = raw_id as u32;
    let display_index = if entity_index > 4_000_000_000 {
        u32::MAX - entity_index
    } else {
        entity_index
    };
    format!("v{}", display_index)
}
