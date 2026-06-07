//! Helpers for rendering stored (encrypted) image bytes in the webview, and for
//! turning a stored [`ImageTransform`] into CSS (`IMG-7`, `IMG-8`).

use base64::Engine;

use lib_soulfire::images::ImageTransform;

use soulfire_core::store::ImageOwnerKind;

use crate::state::AppContext;

/// A `data:` URI for an entity's stored image, decoded from the encrypted store
/// in memory only (`IMG-4`/`SEC-3`). `None` when no image is stored.
pub fn data_uri(app: &AppContext, kind: ImageOwnerKind, owner_id: &str) -> Option<String> {
    let img = app.store.image(kind, owner_id).ok().flatten()?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&img.bytes);
    Some(format!("data:{};base64,{}", img.mime, b64))
}

/// CSS for applying a stored pan/zoom transform to an `<img>` filling a framed
/// container (`object-fit: cover`): translate by pan percent, scale by zoom
/// percent (`IMG-7`).
pub fn transform_style(t: ImageTransform) -> String {
    let scale = (t.zoom_percent as f64 / 100.0).max(0.1);
    format!(
        "width:100%;height:100%;object-fit:cover;transform:translate({}%, {}%) scale({});",
        t.pan_x_percent, t.pan_y_percent, scale
    )
}
