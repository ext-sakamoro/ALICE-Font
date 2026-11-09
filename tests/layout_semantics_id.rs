//! LAYOUT_SEMANTICS_ID: each pin recomputed from its source, then the fold.
use alice_font::layout::{
    jis, BreakClass, LAYOUT_PREVIOUS_SEMANTICS_IDS, LAYOUT_SEMANTICS_ID, LAYOUT_SEMANTICS_PINS,
};
use sha2::{Digest, Sha256};

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

fn pin(name: &str) -> [u8; 32] {
    LAYOUT_SEMANTICS_PINS
        .iter()
        .find(|(n, _)| *n == name)
        .expect(name)
        .1
}

#[test]
fn the_class_table_pin_is_the_digest_of_every_code_point() {
    let mut h = Sha256::new();
    let mut n = 0;
    for cp in 0..=0x10_FFFFu32 {
        let Some(ch) = char::from_u32(cp) else {
            continue;
        };
        let c = jis::class(ch);
        if c != BreakClass::Other {
            h.update(cp.to_le_bytes());
            h.update([c as u8]);
            n += 1;
        }
    }
    assert!(n > 100, "the table classifies {n} code points");
    assert_eq!(hex(&h.finalize()), hex(&pin("jis class table")));
}

#[test]
fn the_corpus_pin_is_the_digest_the_oracle_records() {
    // read from the oracle's source, not a copy
    let src = include_str!("layout_oracle.rs");
    let at = src
        .find("fn the_corpus_layout_is_the_recorded_bits")
        .unwrap();
    let rest = &src[at..];
    let q = rest.find("assert_eq!(").unwrap();
    let digest: String = rest[q..].split('"').nth(1).unwrap().to_string();
    assert_eq!(digest.len(), 64);
    assert_eq!(digest, hex(&pin("layout corpus")));
}

#[test]
fn the_no_wrap_corpus_pin_is_the_digest_the_oracle_records() {
    let src = include_str!("layout_oracle.rs");
    let at = src
        .find("fn the_no_wrap_corpus_layout_is_the_recorded_bits")
        .unwrap();
    let rest = &src[at..];
    let q = rest.find("assert_eq!(").unwrap();
    let digest: String = rest[q..].split('"').nth(1).unwrap().to_string();
    assert_eq!(digest.len(), 64);
    assert_eq!(digest, hex(&pin("layout corpus, no wrap")));
}

#[test]
fn the_identifier_is_the_fold_of_the_pins() {
    let mut e: Vec<_> = LAYOUT_SEMANTICS_PINS.to_vec();
    e.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
    let mut h = Sha256::new();
    for (name, d) in &e {
        h.update((name.len() as u32).to_be_bytes());
        h.update(name.as_bytes());
        h.update(d);
    }
    assert_eq!(hex(&h.finalize()), hex(&LAYOUT_SEMANTICS_ID));
    assert_ne!(
        LAYOUT_SEMANTICS_ID,
        alice_font::semantics::FONT_SEMANTICS_ID
    );
}

// b832f7ef folded the class table and the wrapped corpus only: its fold over the current values of
// those two pins must be b832f7ef, so the scenarios it pinned give the same bits in this build
#[test]
fn the_previous_identifier_is_the_fold_of_the_pins_it_had() {
    let mut e: Vec<_> = LAYOUT_SEMANTICS_PINS
        .iter()
        .filter(|(n, _)| *n == "jis class table" || *n == "layout corpus")
        .copied()
        .collect();
    assert_eq!(e.len(), 2);
    e.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
    let mut h = Sha256::new();
    for (name, d) in &e {
        h.update((name.len() as u32).to_be_bytes());
        h.update(name.as_bytes());
        h.update(d);
    }
    let got = hex(&h.finalize());
    assert_eq!(
        got,
        "b832f7ef7acc942e36be83d8a9ef872a470f9bc56e66cf4d86fc45d84610e835"
    );
    assert!(LAYOUT_PREVIOUS_SEMANTICS_IDS.iter().any(|p| hex(p) == got));
}
