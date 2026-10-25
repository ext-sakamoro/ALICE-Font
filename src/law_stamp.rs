//! Stamp a Law identifier on the serialized glyph geometry (ALICE-Law G7 / G8)
//!
//! The std140 UBO bytes a payload produces ([`crate::shader`], [`crate::outline`],
//! [`crate::skeleton_shader`]) are the raw numbers a GPU reads; those bytes are left exactly as they
//! are, because the consumer is this crate's own shader in the *same build* (it reads the glyph data
//! at the layout the `to_std140*` methods document, with no header). What crosses a process or file
//! boundary instead gets an envelope: [`StampedForm::stamped`] prepends a fixed header
//! ([`STAMP_MAGIC`], a major and minor version, and the identifier) and [`read`] checks it, so a
//! reader can tell whether the bytes were produced by a build that computes the same geometry.
//!
//! The identifier is alice-zip's v2 Law identifier ([`alice_zip::law_id::LawIdHasher`]) of the form's
//! `kind` tag and [`crate::semantics::FONT_SEMANTICS_ID`] over the body bytes. Equal identifier means
//! the two builds compute the same bytes for that form (the guarantee of `FONT_SEMANTICS_ID`, G6).
//!
//! # One arithmetic
//!
//! `FONT_SEMANTICS_ID` was folded against one `alice-det-math` generation; evaluating the identifier
//! against another would silently mix arithmetics. [`DET_MATH_GENERATION`] is that generation and the
//! `const` block below fails the compile if the resolved `alice_det_math::SEMANTICS_ID` is not it.
//! alice-zip pins the same `alice-det-math` revision, so cargo resolves one copy across the graph.

use alloc::vec::Vec;

use alice_zip::law_id::LawIdHasher;

use crate::semantics::FONT_SEMANTICS_ID;

/// The `alice-det-math` generation [`FONT_SEMANTICS_ID`] was folded against (its `SEMANTICS_ID`).
pub const DET_MATH_GENERATION: [u8; 32] = [
    0x85, 0xe0, 0x88, 0x46, 0x23, 0xb1, 0x54, 0xa3, 0xb2, 0xb2, 0x64, 0x37, 0x49, 0x07, 0xb6, 0xcb,
    0xac, 0x09, 0x30, 0x7c, 0xa3, 0xf9, 0xa0, 0x83, 0x39, 0xe4, 0x8c, 0x56, 0x7d, 0x3c, 0xb3, 0x15,
];

const fn same32(a: &[u8; 32], b: &[u8; 32]) -> bool {
    let mut i = 0;
    while i < 32 {
        if a[i] != b[i] {
            return false;
        }
        i += 1;
    }
    true
}

// A different arithmetic would make FONT_SEMANTICS_ID (and every identifier here) name geometry a
// consumer cannot reproduce. Re-record FONT_SEMANTICS_ID and this generation together when it moves.
const _: () = assert!(
    same32(&alice_det_math::SEMANTICS_ID, &DET_MATH_GENERATION),
    "alice-det-math generation changed: re-record FONT_SEMANTICS_ID (G6) and DET_MATH_GENERATION",
);

/// The envelope magic: a non-ASCII first byte (so no ASCII magic of another format collides, and a
/// transfer that strips the eighth bit changes it), then `ALF`, then the usual CR LF ^Z LF.
pub const STAMP_MAGIC: [u8; 8] = [0x89, b'A', b'L', b'F', 0x0D, 0x0A, 0x1A, 0x0A];

/// The envelope's major version. A reader rejects a major it does not know.
pub const STAMP_MAJOR: u16 = 1;

/// The envelope's minor version. A reader accepts a minor at or below the one it knows.
pub const STAMP_MINOR: u16 = 0;

/// magic (8) + major (2) + minor (2) + identifier (32)
pub const HEADER_LEN: usize = 8 + 2 + 2 + 32;

/// Why [`read`] rejected the bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StampError {
    /// The leading bytes are not [`STAMP_MAGIC`] (not a stamped form, or a different format).
    BadMagic,
    /// The major version is one this build does not know, or the minor is newer than it knows.
    UnsupportedVersion {
        /// The major version read from the header.
        major: u16,
        /// The minor version read from the header.
        minor: u16,
    },
    /// The bytes are shorter than a header (the header carries no length; a short body surfaces as a
    /// `SemanticsMismatch` instead, since the identifier is recomputed over whatever body is present).
    Truncated,
    /// The `kind` tag does not match the one the reader expected (wrong form).
    WrongKind,
    /// The stamped identifier is not the one this build computes for the body: the bytes were
    /// produced by a build that computes different geometry, or the body was altered or truncated in
    /// transit. Both identifiers are named.
    SemanticsMismatch {
        /// The identifier the bytes carry.
        stamped: [u8; 32],
        /// The identifier this build computes for the body.
        recomputed: [u8; 32],
    },
}

/// The v2 Law identifier of `body` for a form's `kind` under this build's [`FONT_SEMANTICS_ID`].
#[must_use]
pub fn law_id(kind: &[u8], body: &[u8]) -> [u8; 32] {
    LawIdHasher::new(kind, &FONT_SEMANTICS_ID)
        .bytes(body)
        .finish()
}

/// A canonical, unambiguous byte string for a multi-chunk form: the chunk count as a big-endian
/// `u32`, then each chunk's length as a big-endian `u64` followed by its bytes. The length framing
/// keeps two different chunkings from producing the same body.
#[must_use]
pub(crate) fn frame_chunks(chunks: &[Vec<u8>]) -> Vec<u8> {
    let total: usize = 4 + chunks.iter().map(|c| 8 + c.len()).sum::<usize>();
    let mut out = Vec::with_capacity(total);
    out.extend_from_slice(&(chunks.len() as u32).to_be_bytes());
    for c in chunks {
        out.extend_from_slice(&(c.len() as u64).to_be_bytes());
        out.extend_from_slice(c);
    }
    out
}

/// A serialized glyph form that can be stamped and read back (ALICE-Law G7 / G8).
pub trait StampedForm {
    /// The form's tag, mixed into the identifier so one form's bytes cannot read as another's.
    const KIND: &'static [u8];

    /// The canonical body bytes: the raw std140 bytes for a single buffer, or `frame_chunks` of
    /// the UBO chunks for a multi-chunk form.
    fn stamp_body(&self) -> Vec<u8>;

    /// The identifier of this form's body under this build.
    #[must_use]
    fn law_id(&self) -> [u8; 32] {
        law_id(Self::KIND, &self.stamp_body())
    }

    /// The body with the envelope header prepended, for sending across a process or file boundary.
    #[must_use]
    fn stamped(&self) -> Vec<u8> {
        let body = self.stamp_body();
        let id = law_id(Self::KIND, &body);
        let mut out = Vec::with_capacity(HEADER_LEN + body.len());
        out.extend_from_slice(&STAMP_MAGIC);
        out.extend_from_slice(&STAMP_MAJOR.to_be_bytes());
        out.extend_from_slice(&STAMP_MINOR.to_be_bytes());
        out.extend_from_slice(&id);
        out.extend_from_slice(&body);
        out
    }
}

/// Reads a stamped form, checking the magic, version and identifier, and returns the body bytes.
///
/// `expected_kind` is the [`StampedForm::KIND`] of the form the caller is reading; the identifier is
/// recomputed with it, so bytes stamped as another form fail with [`StampError::SemanticsMismatch`].
///
/// # Errors
///
/// [`StampError`] when the magic, the version, the length or the identifier does not check out.
pub fn read<'b>(expected_kind: &[u8], bytes: &'b [u8]) -> Result<&'b [u8], StampError> {
    if bytes.len() < HEADER_LEN {
        return Err(StampError::Truncated);
    }
    if bytes[..8] != STAMP_MAGIC {
        return Err(StampError::BadMagic);
    }
    let major = u16::from_be_bytes([bytes[8], bytes[9]]);
    let minor = u16::from_be_bytes([bytes[10], bytes[11]]);
    if major != STAMP_MAJOR || minor > STAMP_MINOR {
        return Err(StampError::UnsupportedVersion { major, minor });
    }
    let mut stamped = [0u8; 32];
    stamped.copy_from_slice(&bytes[12..HEADER_LEN]);
    let body = &bytes[HEADER_LEN..];
    let recomputed = law_id(expected_kind, body);
    if recomputed != stamped {
        return Err(StampError::SemanticsMismatch {
            stamped,
            recomputed,
        });
    }
    Ok(body)
}

impl StampedForm for crate::shader::AnalyticalGlyphPayload {
    const KIND: &'static [u8] = b"alice-font/std140/analytical/v1";
    fn stamp_body(&self) -> Vec<u8> {
        self.to_std140_bytes()
    }
}

impl StampedForm for crate::shader::MultiGlyphPayload {
    const KIND: &'static [u8] = b"alice-font/std140/multi-analytical/v1";
    fn stamp_body(&self) -> Vec<u8> {
        frame_chunks(&self.to_std140_ubo_chunks())
    }
}

impl StampedForm for crate::outline::OutlineGlyphPayload {
    const KIND: &'static [u8] = b"alice-font/std140/outline/v1";
    fn stamp_body(&self) -> Vec<u8> {
        self.to_std140_bytes()
    }
}

impl StampedForm for crate::outline::MultiOutlineGlyphPayload {
    const KIND: &'static [u8] = b"alice-font/std140/multi-outline/v1";
    fn stamp_body(&self) -> Vec<u8> {
        frame_chunks(&self.to_std140_ubo_chunks())
    }
}

impl StampedForm for crate::skeleton_shader::MultiSkeletonGlyphPayload {
    const KIND: &'static [u8] = b"alice-font/std140/skeleton/v1";
    fn stamp_body(&self) -> Vec<u8> {
        frame_chunks(&self.to_std140_ubo_chunks())
    }
}

/// The JSON-interchange form of a stamp (ALICE-Law G7 / G8, feature `serde`).
///
/// The shader payload types do not implement `Deserialize` (there is no silent path from bytes to a
/// typed payload); the only thing that deserializes is this envelope, and [`StampedEnvelope::open`]
/// checks the identifier before handing back the body. It carries the canonical std140 body, the same
/// bytes and identifier as [`StampedForm::stamped`], so the two forms agree.
#[cfg(feature = "serde")]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StampedEnvelope {
    /// [`STAMP_MAGIC`].
    pub magic: [u8; 8],
    /// [`STAMP_MAJOR`].
    pub major: u16,
    /// [`STAMP_MINOR`].
    pub minor: u16,
    /// The identifier of `body` for `kind` under the sealing build.
    pub law_id: [u8; 32],
    /// The form tag ([`StampedForm::KIND`]).
    pub kind: alloc::string::String,
    /// The canonical std140 body bytes.
    pub body: Vec<u8>,
}

#[cfg(feature = "serde")]
impl StampedEnvelope {
    /// Seals a form's body under this build's identifier.
    #[must_use]
    pub fn seal(kind: &[u8], body: Vec<u8>) -> Self {
        let law_id = law_id(kind, &body);
        Self {
            magic: STAMP_MAGIC,
            major: STAMP_MAJOR,
            minor: STAMP_MINOR,
            law_id,
            kind: alloc::string::String::from_utf8_lossy(kind).into_owned(),
            body,
        }
    }

    /// Checks the magic, version, kind and identifier, and returns the body.
    ///
    /// # Errors
    ///
    /// [`StampError`] when any of them does not check out; [`StampError::SemanticsMismatch`] names
    /// both the identifier the envelope carries and the one this build computes for the body.
    pub fn open(&self, expected_kind: &[u8]) -> Result<&[u8], StampError> {
        if self.magic != STAMP_MAGIC {
            return Err(StampError::BadMagic);
        }
        if self.major != STAMP_MAJOR || self.minor > STAMP_MINOR {
            return Err(StampError::UnsupportedVersion {
                major: self.major,
                minor: self.minor,
            });
        }
        if self.kind.as_bytes() != expected_kind {
            return Err(StampError::WrongKind);
        }
        let recomputed = law_id(expected_kind, &self.body);
        if recomputed != self.law_id {
            return Err(StampError::SemanticsMismatch {
                stamped: self.law_id,
                recomputed,
            });
        }
        Ok(&self.body)
    }
}

/// The JSON-interchange envelope of a form's body (feature `serde`).
#[cfg(feature = "serde")]
pub fn seal<F: StampedForm>(form: &F) -> StampedEnvelope {
    StampedEnvelope::seal(F::KIND, form.stamp_body())
}
