//! #1080 and the loop blend: a fillet too large for the geometry beside its edge is refused,
//! naming the largest size that fits; a chamfer of the same size is clamped to the largest that
//! fits instead, and one that fits still builds.

use std::path::Path;

use serde_json::json;

use super::{run, volume};
use crate::Session;

fn open_fixture() -> Session {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/tests/fixtures/cubeish-chamfer-too-large.solidcraft");
    let mut s = Session::default();
    run(&mut s, "doc.open", json!({"path": path.to_string_lossy()}));
    s
}

fn chamfer_error(s: &Session) -> Option<String> {
    s.model.results.iter().find(|r| r.name == "Chamfer1").and_then(|r| r.error.clone())
}

fn chamfer_warning(s: &Session) -> String {
    s.model.results.iter().find(|r| r.name == "Chamfer1").and_then(|r| r.warning.clone()).unwrap_or_default()
}

/// The report's design: a 20 mm cube with a 10 mm square opening into each of three faces,
/// then a 9 mm chamfer on the top edge over one side opening. The openings are 5 mm from the
/// edge, so the requested chamfer would cut through their rims. It is clamped to 5 mm, the
/// largest that fits, and the feature says so.
#[test]
fn chamfer_through_the_openings_is_reduced_to_the_largest_that_fits() {
    let mut s = open_fixture();
    assert_eq!(chamfer_error(&s), None, "the 9 mm chamfer is clamped, not refused");
    assert!(chamfer_warning(&s).contains("reduced to 5 mm"), "{}", chamfer_warning(&s));
    let want = 4000.0 - 0.5 * 5.0 * 5.0 * 20.0;
    assert!((volume(&mut s) - want).abs() < 1e-6, "the chamfer is cut at 5 mm: {}", volume(&mut s));
}

/// Sizes that stop short of the openings build as asked, and take away exactly the chamfer's
/// triangle along the 20 mm edge.
#[test]
fn chamfer_that_fits_still_builds() {
    let mut s = open_fixture();
    for d in [4.0, 5.0] {
        run(&mut s, "timeline.edit", json!({"feature": "Chamfer1", "set": {"distance": format!("{d} mm")}}));
        assert_eq!(chamfer_error(&s), None, "{d} mm fits");
        let want = 4000.0 - 0.5 * d * d * 20.0;
        assert!((volume(&mut s) - want).abs() < 1e-6, "{d} mm: {} (want {want})", volume(&mut s));
    }
}
