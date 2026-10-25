//! The Law identifier stamp on the serialized geometry (ALICE-Law G7 / G8).
//!
//! The raw std140 bytes are left as they are (the same-build shader reads them); the stamped envelope
//! and [`read`] are for bytes crossing a process or file boundary. These tests pin that a round trip
//! returns the body, that a tampered identifier, the wrong form, a bad magic, an unknown version and a
//! truncated buffer are all rejected, and that an identifier computed from a different semantics id is
//! rejected (the stamp binds to this build's `FONT_SEMANTICS_ID`).

use alice_font::font_dsl::dsl_lookup;
use alice_font::law_stamp::{
    read, StampError, StampedForm, HEADER_LEN, STAMP_MAGIC, STAMP_MAJOR, STAMP_MINOR,
};
use alice_font::outline::{
    pack_outline_for_shader, pack_outline_string, MultiOutlineGlyphPayload, OutlineGlyph,
    OutlineGlyphPayload,
};
use alice_font::shader::{
    pack_glyph_for_shader, pack_string_for_shader, AnalyticalGlyphPayload, MultiGlyphPayload,
};
use alice_font::skeleton_shader::{pack_skeleton_string, MultiSkeletonGlyphPayload};
use alice_font::MetaFontParams;

use alice_zip::law_id::LawIdHasher;

fn star_over_box() -> OutlineGlyph {
    const STAR: [[f32; 2]; 10] = [
        [0.5, 0.78],
        [0.58, 0.47],
        [0.9, 0.46],
        [0.63, 0.29],
        [0.74, -0.03],
        [0.5, 0.17],
        [0.26, -0.03],
        [0.37, 0.29],
        [0.1, 0.46],
        [0.42, 0.47],
    ];
    const BOX: [f32; 4] = [0.25, 0.0625, 0.75, 0.6875];
    OutlineGlyph {
        codepoint: 'x',
        contours: vec![
            STAR.to_vec(),
            vec![
                [BOX[0], BOX[1]],
                [BOX[2], BOX[1]],
                [BOX[2], BOX[3]],
                [BOX[0], BOX[3]],
            ],
        ],
        advance: 0.75,
    }
}

fn analytical() -> AnalyticalGlyphPayload {
    pack_glyph_for_shader('森', &MetaFontParams::sans_regular()).expect("packs")
}
fn multi_analytical() -> MultiGlyphPayload {
    pack_string_for_shader("森林", &MetaFontParams::sans_regular()).expect("packs")
}
fn outline() -> OutlineGlyphPayload {
    pack_outline_for_shader(&star_over_box()).expect("packs")
}
fn multi_outline() -> MultiOutlineGlyphPayload {
    pack_outline_string("ab", |_ch| Some(star_over_box())).expect("packs")
}
fn skeleton() -> MultiSkeletonGlyphPayload {
    pack_skeleton_string("三", dsl_lookup).expect("packs")
}

/// A stamped form reads back to its body, and the embedded id is this build's id for the body.
fn round_trip<F: StampedForm>(f: &F) {
    let body = f.stamp_body();
    assert!(!body.is_empty(), "the form produced no body");
    let stamped = f.stamped();
    assert_eq!(&stamped[..8], &STAMP_MAGIC, "magic");
    assert_eq!(stamped.len(), HEADER_LEN + body.len(), "envelope length");
    let got = read(F::KIND, &stamped).expect("reads");
    assert_eq!(got, &body[..], "body round-trips");
    assert_eq!(f.law_id(), f.law_id(), "id is deterministic");
}

#[test]
fn every_form_round_trips() {
    round_trip(&analytical());
    round_trip(&multi_analytical());
    round_trip(&outline());
    round_trip(&multi_outline());
    round_trip(&skeleton());
}

#[test]
fn a_tampered_identifier_is_rejected() {
    let stamped = analytical().stamped();
    let mut bad = stamped.clone();
    bad[12] ^= 0x01; // flip a bit of the stamped identifier (bytes 12..44)
    match read(<AnalyticalGlyphPayload as StampedForm>::KIND, &bad) {
        Err(StampError::SemanticsMismatch {
            stamped: s,
            recomputed: r,
        }) => {
            assert_ne!(s, r, "the two identifiers must differ");
        }
        other => panic!("expected SemanticsMismatch, got {other:?}"),
    }
}

#[test]
fn the_wrong_form_is_rejected() {
    // bytes stamped as the analytical form, read as the outline form
    let stamped = analytical().stamped();
    match read(<OutlineGlyphPayload as StampedForm>::KIND, &stamped) {
        Err(StampError::SemanticsMismatch { .. }) => {}
        other => panic!("expected SemanticsMismatch for the wrong kind, got {other:?}"),
    }
}

#[test]
fn a_bad_magic_is_rejected() {
    let mut bytes = analytical().stamped();
    bytes[0] ^= 0xFF;
    assert_eq!(
        read(<AnalyticalGlyphPayload as StampedForm>::KIND, &bytes),
        Err(StampError::BadMagic)
    );
}

#[test]
fn an_unknown_version_is_rejected() {
    let mut bytes = analytical().stamped();
    // major is bytes[8..10], big-endian: bump it past what this build knows
    let major = STAMP_MAJOR + 1;
    bytes[8..10].copy_from_slice(&major.to_be_bytes());
    match read(<AnalyticalGlyphPayload as StampedForm>::KIND, &bytes) {
        Err(StampError::UnsupportedVersion { major: m, minor }) => {
            assert_eq!(m, STAMP_MAJOR + 1);
            assert_eq!(minor, STAMP_MINOR);
        }
        other => panic!("expected UnsupportedVersion, got {other:?}"),
    }
}

#[test]
fn a_truncated_buffer_is_rejected() {
    let bytes = analytical().stamped();
    assert_eq!(
        read(
            <AnalyticalGlyphPayload as StampedForm>::KIND,
            &bytes[..HEADER_LEN - 1]
        ),
        Err(StampError::Truncated)
    );
}

/// An identifier computed from a different semantics id, with a correct magic and version, is
/// rejected: the stamp binds to this build's `FONT_SEMANTICS_ID`, not merely to a well-formed header.
#[test]
fn an_identifier_from_another_semantics_is_rejected() {
    let payload = analytical();
    let body = payload.stamp_body();
    // forge an id the way the crate does, but under a semantics id that is not this build's
    let other_semantics = [0x11u8; 32];
    let forged = LawIdHasher::new(
        <AnalyticalGlyphPayload as StampedForm>::KIND,
        &other_semantics,
    )
    .bytes(&body)
    .finish();
    let mut forged_envelope = Vec::new();
    forged_envelope.extend_from_slice(&STAMP_MAGIC);
    forged_envelope.extend_from_slice(&STAMP_MAJOR.to_be_bytes());
    forged_envelope.extend_from_slice(&STAMP_MINOR.to_be_bytes());
    forged_envelope.extend_from_slice(&forged);
    forged_envelope.extend_from_slice(&body);
    match read(
        <AnalyticalGlyphPayload as StampedForm>::KIND,
        &forged_envelope,
    ) {
        Err(StampError::SemanticsMismatch {
            stamped,
            recomputed,
        }) => {
            assert_eq!(stamped, forged, "the forged id is what was read");
            assert_ne!(stamped, recomputed, "this build recomputes a different id");
        }
        other => panic!("expected SemanticsMismatch, got {other:?}"),
    }
}

/// ALICE-Law G8 hard refusal: none of the shader payload types implement `Deserialize`, so there is
/// no silent path from bytes to a typed payload — the only thing that deserializes is the stamped
/// envelope, which validates. A static scan of the payload modules; it fails if it checks no type
/// (the grep found nothing) or if any payload type carries a `Deserialize` derive.
#[test]
fn no_payload_type_implements_deserialize() {
    use std::path::Path;
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    // (file, payload types it defines)
    let modules: &[(&str, &[&str])] = &[
        (
            "src/shader.rs",
            &["AnalyticalGlyphPayload", "MultiGlyphPayload"],
        ),
        (
            "src/outline.rs",
            &["OutlineGlyphPayload", "MultiOutlineGlyphPayload"],
        ),
        ("src/skeleton_shader.rs", &["MultiSkeletonGlyphPayload"]),
    ];
    let mut checked = 0usize;
    for (file, types) in modules {
        let src = std::fs::read_to_string(root.join(file)).expect("payload module");
        assert!(
            !src.contains("Deserialize"),
            "{file} mentions Deserialize: a shader payload must not implement it (G8 hard refusal)"
        );
        for ty in *types {
            assert!(
                src.contains(&format!("pub struct {ty}")),
                "{file} no longer defines {ty}: the scan would miss a payload that regained Deserialize"
            );
            checked += 1;
        }
    }
    assert!(
        checked > 0,
        "no payload type was checked (the scan found nothing)"
    );
    assert_eq!(
        checked, 5,
        "expected to check all five shader payload types"
    );
}

#[cfg(feature = "serde")]
#[test]
fn serde_envelope_round_trips_and_rejects_tampering() {
    use alice_font::law_stamp::{seal, StampedEnvelope};

    let payload = analytical();
    let kind = <AnalyticalGlyphPayload as StampedForm>::KIND;
    let env = seal(&payload);
    // a correct envelope opens to the body
    assert_eq!(env.open(kind).expect("opens"), &payload.stamp_body()[..]);

    // it round-trips through JSON
    let json = serde_json::to_string(&env).expect("to json");
    let back: StampedEnvelope = serde_json::from_str(&json).expect("from json");
    assert_eq!(back.open(kind).expect("opens"), &payload.stamp_body()[..]);

    // a tampered identifier is rejected, naming both ids
    let mut tampered = env.clone();
    tampered.law_id[0] ^= 0x01;
    match tampered.open(kind) {
        Err(StampError::SemanticsMismatch {
            stamped,
            recomputed,
        }) => {
            assert_eq!(stamped, tampered.law_id);
            assert_ne!(stamped, recomputed);
        }
        other => panic!("expected SemanticsMismatch, got {other:?}"),
    }

    // the wrong form is rejected
    let outline_kind = <OutlineGlyphPayload as StampedForm>::KIND;
    assert_eq!(env.open(outline_kind), Err(StampError::WrongKind));
}
