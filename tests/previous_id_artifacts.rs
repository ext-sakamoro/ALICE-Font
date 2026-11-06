//! Stamped forms written by the build before the function identifiers still read.
//!
//! `tests/data/previous-id/` holds what alice-font at 4b50232 wrote (the build whose identifier folded
//! alice-det-math's whole identifier, `210bcb70…`). The files are that build's output, not made here: each
//! carries the identifier computed under the old value, and its body is byte for byte what this build computes.

use alice_font::font_dsl::dsl_lookup;
use alice_font::law_stamp::{read, StampError, StampedForm, HEADER_LEN};
use alice_font::outline::{
    pack_outline_for_shader, pack_outline_string, MultiOutlineGlyphPayload, OutlineGlyph,
    OutlineGlyphPayload,
};
use alice_font::semantics::{FONT_SEMANTICS_ID, PREVIOUS_SEMANTICS_IDS};
use alice_font::shader::{
    pack_glyph_for_shader, pack_string_for_shader, AnalyticalGlyphPayload, MultiGlyphPayload,
};
use alice_font::skeleton_shader::{pack_skeleton_string, MultiSkeletonGlyphPayload};
use alice_font::MetaFontParams;
use alice_zip::law_id::LawIdHasher;

const OLD: &str = "210bcb70027cfb9415d15c7b752e2c0ee4350965ca8651285dc713af117366fd";

fn old_id() -> [u8; 32] {
    let mut out = [0u8; 32];
    for (i, o) in out.iter_mut().enumerate() {
        *o = u8::from_str_radix(&OLD[2 * i..2 * i + 2], 16).expect("hex digit");
    }
    out
}

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

/// (file, the form's tag, the body this build computes)
fn forms() -> Vec<(&'static [u8], &'static [u8], Vec<u8>)> {
    let sans = MetaFontParams::sans_regular();
    vec![
        (
            include_bytes!("data/previous-id/analytical.bin"),
            <AnalyticalGlyphPayload as StampedForm>::KIND,
            analytical().stamp_body(),
        ),
        (
            include_bytes!("data/previous-id/multi-analytical.bin"),
            <MultiGlyphPayload as StampedForm>::KIND,
            pack_string_for_shader("森林", &sans)
                .expect("packs")
                .stamp_body(),
        ),
        (
            include_bytes!("data/previous-id/outline.bin"),
            <OutlineGlyphPayload as StampedForm>::KIND,
            pack_outline_for_shader(&star_over_box())
                .expect("packs")
                .stamp_body(),
        ),
        (
            include_bytes!("data/previous-id/multi-outline.bin"),
            <MultiOutlineGlyphPayload as StampedForm>::KIND,
            pack_outline_string("ab", |_ch| Some(star_over_box()))
                .expect("packs")
                .stamp_body(),
        ),
        (
            include_bytes!("data/previous-id/skeleton.bin"),
            <MultiSkeletonGlyphPayload as StampedForm>::KIND,
            pack_skeleton_string("三", dsl_lookup)
                .expect("packs")
                .stamp_body(),
        ),
    ]
}

#[test]
fn the_old_identifier_is_a_previous_one_and_not_this_one() {
    assert!(PREVIOUS_SEMANTICS_IDS.contains(&old_id()));
    assert_ne!(FONT_SEMANTICS_ID, old_id());
}

#[test]
fn old_stamped_forms_read_and_their_bodies_are_what_this_build_computes() {
    let all = forms();
    assert_eq!(all.len(), 5);
    for (file, kind, body) in all {
        let stored = LawIdHasher::new(kind, &old_id()).bytes(&body).finish();
        assert_eq!(
            file[12..HEADER_LEN],
            stored,
            "the file carries the identifier under the old value"
        );
        let this = LawIdHasher::new(kind, &FONT_SEMANTICS_ID)
            .bytes(&body)
            .finish();
        assert_ne!(stored, this, "the file is not this build's stamp");
        assert_eq!(read(kind, file).expect("reads"), &body[..]);
    }
}

#[test]
fn an_old_form_with_a_changed_body_is_refused() {
    for (file, kind, _) in forms() {
        let mut changed = file.to_vec();
        changed[HEADER_LEN] ^= 1;
        assert!(
            matches!(
                read(kind, &changed),
                Err(StampError::SemanticsMismatch { .. })
            ),
            "a changed body read"
        );
    }
}

#[test]
fn an_old_form_read_as_another_form_is_refused() {
    let all = forms();
    let (file, _, _) = all[0];
    let (_, other_kind, _) = all[2];
    assert!(matches!(
        read(other_kind, file),
        Err(StampError::SemanticsMismatch { .. })
    ));
}

#[cfg(feature = "serde")]
#[test]
fn an_old_json_envelope_opens() {
    use alice_font::law_stamp::StampedEnvelope;
    let json = include_str!("data/previous-id/envelope.json");
    let env: StampedEnvelope = serde_json::from_str(json).expect("from json");
    let kind = <AnalyticalGlyphPayload as StampedForm>::KIND;
    assert_eq!(
        env.open(kind).expect("opens"),
        &analytical().stamp_body()[..]
    );
}
