// ── Component field parsing (shared by inspector and pincard) ─────────────────

/// Parses a JSON-style array string like `"[0.0,1.0,2.0]"` into individual value strings.
/// Returns `None` if the string is not a multi-element array.
pub fn parse_array_field(s: &str) -> Option<Vec<String>> {
    let s = s.trim();
    if !s.starts_with('[') || !s.ends_with(']') {
        return None;
    }
    let inner = &s[1..s.len() - 1];
    let parts: Vec<String> = inner.split(',').map(|v| v.trim().to_string()).collect();
    if parts.len() >= 2 { Some(parts) } else { None }
}

const TRANSFORM_FIELD_ORDER: &[&str] = &["translation", "rotation", "scale"];

/// Converts a reflected component value into a list of `(field_name, display_string)` pairs.
pub fn parse_fields(val: &serde_json::Value) -> Vec<(String, String)> {
    match val {
        serde_json::Value::Object(map) => {
            let mut fields: Vec<(String, String)> = map
                .iter()
                .map(|(k, v)| (k.clone(), value_to_display_string(v)))
                .collect();
            let keys: Vec<&str> = fields.iter().map(|(k, _)| k.as_str()).collect();
            if TRANSFORM_FIELD_ORDER.iter().all(|f| keys.contains(f)) {
                fields.sort_by_key(|(k, _)| {
                    TRANSFORM_FIELD_ORDER
                        .iter()
                        .position(|f| *f == k.as_str())
                        .unwrap_or(usize::MAX)
                });
            }
            fields
        }
        serde_json::Value::Array(arr) => decompose_affine(arr)
            .unwrap_or_else(|| vec![("value".to_string(), value_to_display_string(val))]),
        other if !other.is_null() => vec![("value".to_string(), value_to_display_string(other))],
        _ => vec![],
    }
}

/// Unwraps newtype wrappers: `[inner]` → `inner`, `{"0": inner}` → `inner`.
pub fn unwrap_newtype(val: &serde_json::Value) -> &serde_json::Value {
    if let Some(arr) = val.as_array() {
        if arr.len() == 1 {
            return &arr[0];
        }
    }
    if let Some(map) = val.as_object() {
        if map.len() == 1 {
            if let Some(inner) = map.get("0") {
                return inner;
            }
        }
    }
    val
}

/// Formats a JSON value for display (strips string quotes, passes others through).
pub fn value_to_display_string(val: &serde_json::Value) -> String {
    match val {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Null => "null".to_string(),
        other => other.to_string(),
    }
}

/// Decomposes a flat 12-float Affine3A array (column-major) into translation/rotation/scale.
fn decompose_affine(arr: &[serde_json::Value]) -> Option<Vec<(String, String)>> {
    if arr.len() != 12 {
        return None;
    }
    let f: Vec<f64> = arr.iter().filter_map(|v| v.as_f64()).collect();
    if f.len() != 12 {
        return None;
    }

    let (m00, m10, m20) = (f[0], f[1], f[2]);
    let (m01, m11, m21) = (f[3], f[4], f[5]);
    let (m02, m12, m22) = (f[6], f[7], f[8]);
    let (tx, ty, tz) = (f[9], f[10], f[11]);

    let sx = (m00 * m00 + m10 * m10 + m20 * m20).sqrt();
    let sy = (m01 * m01 + m11 * m11 + m21 * m21).sqrt();
    let sz = (m02 * m02 + m12 * m12 + m22 * m22).sqrt();

    let eps = 1e-10_f64;
    let (r00, r10, r20) = if sx > eps {
        (m00 / sx, m10 / sx, m20 / sx)
    } else {
        (1.0, 0.0, 0.0)
    };
    let (r01, r11, r21) = if sy > eps {
        (m01 / sy, m11 / sy, m21 / sy)
    } else {
        (0.0, 1.0, 0.0)
    };
    let (r02, r12, r22) = if sz > eps {
        (m02 / sz, m12 / sz, m22 / sz)
    } else {
        (0.0, 0.0, 1.0)
    };

    let trace = r00 + r11 + r22;
    let (qx, qy, qz, qw) = if trace > 0.0 {
        let s = (trace + 1.0).sqrt() * 2.0;
        ((r21 - r12) / s, (r02 - r20) / s, (r10 - r01) / s, 0.25 * s)
    } else if r00 > r11 && r00 > r22 {
        let s = (1.0 + r00 - r11 - r22).sqrt() * 2.0;
        (0.25 * s, (r01 + r10) / s, (r02 + r20) / s, (r21 - r12) / s)
    } else if r11 > r22 {
        let s = (1.0 - r00 + r11 - r22).sqrt() * 2.0;
        ((r01 + r10) / s, 0.25 * s, (r12 + r21) / s, (r02 - r20) / s)
    } else {
        let s = (1.0 - r00 - r11 + r22).sqrt() * 2.0;
        ((r02 + r20) / s, (r12 + r21) / s, 0.25 * s, (r10 - r01) / s)
    };

    Some(vec![
        (
            "translation".to_string(),
            format!("[{tx:.3},{ty:.3},{tz:.3}]"),
        ),
        (
            "rotation".to_string(),
            format!("[{qx:.3},{qy:.3},{qz:.3},{qw:.3}]"),
        ),
        ("scale".to_string(), format!("[{sx:.3},{sy:.3},{sz:.3}]")),
    ])
}

pub fn parse_json_value(s: &str) -> serde_json::Value {
    if let Ok(n) = s.parse::<i64>() {
        return serde_json::json!(n);
    }
    if let Ok(n) = s.parse::<f64>() {
        return serde_json::json!(n);
    }
    if let Ok(b) = s.parse::<bool>() {
        return serde_json::json!(b);
    }
    let normalized = normalize_bare_decimals(s);
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&normalized) {
        return v;
    }
    serde_json::json!(s)
}

fn normalize_bare_decimals(s: &str) -> String {
    let mut result = String::with_capacity(s.len() + 4);
    let chars: Vec<char> = s.chars().collect();
    for i in 0..chars.len() {
        result.push(chars[i]);
        if chars[i] == '.' {
            match chars.get(i + 1).copied() {
                Some(c) if c.is_ascii_digit() => {}
                _ => result.push('0'),
            }
        }
    }
    result
}

pub fn entity_display_label(raw_id: u64) -> String {
    // Casting to u32 automatically truncates the top 32 bits, leaving just the index
    let entity_index = raw_id as u32;

    let display_index = if entity_index > 4_000_000_000 {
        u32::MAX - entity_index
    } else {
        entity_index
    };

    // Shift the bits right by 32 to move the generation data to the bottom half
    let generation = (raw_id >> 32) as u32;

    format!("{}v{}", display_index, generation)
}
