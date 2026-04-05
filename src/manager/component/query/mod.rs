use crate::prelude::*;

pub mod ui;

#[derive(Debug, Clone)]
pub struct QueryEntry {
    pub raw: String,
}

fn normalize_part(part: &str) -> String {
    let trimmed = part.trim();
    let lower = trimmed.to_lowercase();
    if lower.starts_with("without<") && lower.ends_with('>') {
        let inner = &trimmed[8..trimmed.len() - 1];
        format!("Without<{}>", inner.trim())
    } else if lower.starts_with("with<") && lower.ends_with('>') {
        let inner = &trimmed[5..trimmed.len() - 1];
        format!("With<{}>", inner.trim())
    } else {
        format!("With<{}>", trimmed)
    }
}

impl QueryEntry {
    pub fn new(raw: impl Into<String>) -> Self {
        let raw = raw.into();
        let normalized = raw
            .split(',')
            .map(|part| normalize_part(part))
            .collect::<Vec<_>>()
            .join(", ");
        Self { raw: normalized }
    }

    pub fn with_names(&self) -> Vec<String> {
        self.raw
            .split(',')
            .filter_map(|part| {
                part.trim()
                    .strip_prefix("With<")
                    .and_then(|s| s.strip_suffix('>'))
                    .map(|s| s.trim().to_string())
            })
            .collect()
    }

    pub fn without_names(&self) -> Vec<String> {
        self.raw
            .split(',')
            .filter_map(|part| {
                part.trim()
                    .strip_prefix("Without<")
                    .and_then(|s| s.strip_suffix('>'))
                    .map(|s| s.trim().to_string())
            })
            .collect()
    }
}

#[derive(Resource, Default, Debug)]
pub struct ComponentQueries(pub Vec<QueryEntry>);

impl ComponentQueries {
    pub fn insert(&mut self, entry: QueryEntry) {
        if !self.0.iter().any(|q| q.raw == entry.raw) {
            self.0.push(entry);
        }
    }
}

pub fn plugin(app: &mut App) {
    app.init_resource::<ComponentQueries>();
    ui::plugin(app);

    #[cfg(feature = "dev")]
    app.add_systems(Update, print_queries);
}

#[cfg(feature = "dev")]
fn print_queries(queries: Res<ComponentQueries>, mut timer: Local<f32>, time: Res<Time>) {
    *timer += time.delta_secs();
    if *timer < 1.0 {
        return;
    }
    *timer = 0.0;
    for q in queries.0.iter() {
        debug!(
            "ComponentQueries: {:<10}, With {:?}, Without {:?}",
            q.raw,
            q.with_names(),
            q.without_names()
        );
    }
}
