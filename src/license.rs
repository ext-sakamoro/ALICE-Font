//! Font license — usage rights tracking for parametric fonts
//!
//! Provides a compact 32-byte wire format for tracking font usage
//! rights in game development contexts: per-title licensing,
//! platform restrictions, and seat limits.
//!
//! Parametric fonts (ALICE-Font) are license-free by design since
//! they are generated from 40-byte parameters rather than loaded
//! from copyrighted font files. This module makes that distinction
//! explicit with `LicenseType::Parametric`.
//!
//! License: MIT
//! Author: Moroya Sakamoto

/// Usage rights bit field (u16)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UsageRights(pub u16);

impl UsageRights {
    pub const COMMERCIAL: u16 = 0x01;
    pub const MODIFICATION: u16 = 0x02;
    pub const REDISTRIBUTION: u16 = 0x04;
    pub const EMBEDDING: u16 = 0x08;
    pub const DERIVATIVE: u16 = 0x10;
    pub const SERVER_USE: u16 = 0x20;
    pub const BROADCAST: u16 = 0x40;
    pub const PRINT: u16 = 0x80;
    pub const GAME_BUNDLE: u16 = 0x100;
    pub const CJK_EXTENDED: u16 = 0x200;

    /// Standard game distribution rights
    pub const GAME_STANDARD: u16 =
        Self::COMMERCIAL | Self::EMBEDDING | Self::GAME_BUNDLE | Self::MODIFICATION;

    /// All rights granted
    pub const ALL: u16 = 0x03FF;

    #[inline]
    pub const fn empty() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn all() -> Self {
        Self(Self::ALL)
    }

    #[inline]
    pub const fn game_standard() -> Self {
        Self(Self::GAME_STANDARD)
    }

    #[inline]
    pub const fn has(self, flag: u16) -> bool {
        (self.0 & flag) == flag
    }

    #[inline]
    pub const fn with(self, flag: u16) -> Self {
        Self(self.0 | flag)
    }

    #[inline]
    pub const fn without(self, flag: u16) -> Self {
        Self(self.0 & !flag)
    }
}

/// License type identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum LicenseType {
    OpenSource = 0,
    Commercial = 1,
    Trial = 2,
    Internal = 3,
    /// Parametric fonts generated from MetaFontParams — inherently free
    Parametric = 4,
}

impl LicenseType {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::OpenSource),
            1 => Some(Self::Commercial),
            2 => Some(Self::Trial),
            3 => Some(Self::Internal),
            4 => Some(Self::Parametric),
            _ => None,
        }
    }
}

/// Platform restriction bit field (u8)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlatformRestriction(pub u8);

impl PlatformRestriction {
    pub const PC: u8 = 0x01;
    pub const CONSOLE: u8 = 0x02;
    pub const MOBILE: u8 = 0x04;
    pub const WEB: u8 = 0x08;
    pub const VR: u8 = 0x10;
    pub const ARCADE: u8 = 0x20;

    /// All platforms
    pub const ALL: u8 = 0x3F;

    #[inline]
    pub const fn all() -> Self {
        Self(Self::ALL)
    }

    #[inline]
    pub const fn empty() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn has(self, flag: u8) -> bool {
        (self.0 & flag) == flag
    }

    #[inline]
    pub const fn with(self, flag: u8) -> Self {
        Self(self.0 | flag)
    }
}

/// FNV-1a hash (file-local)
#[inline(always)]
fn fnv1a(data: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for &b in data {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

/// Font license metadata — 32-byte wire format
///
/// Encodes all licensing information needed for runtime validation
/// of font usage in games. Parametric fonts (ALICE-Font) always
/// pass validation since they carry no third-party license.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct FontLicense {
    /// Content hash (FNV-1a of deterministic fields)
    pub content_hash: u64, // 8B  offset 0
    /// MetaFontParams hash for binding license to specific parameters
    pub params_hash: u64, // 8B  offset 8
    /// Game title ID (0 = unrestricted)
    pub title_id: u32, // 4B  offset 16
    /// Expiration epoch seconds (0 = permanent)
    pub expires_epoch: u32, // 4B  offset 20
    /// Usage rights
    pub rights: UsageRights, // 2B  offset 24
    /// Maximum seat count (0 = unlimited)
    pub max_seats: u16, // 2B  offset 26
    /// License type
    pub license_type: LicenseType, // 1B  offset 28
    /// Platform restrictions
    pub platforms: PlatformRestriction, // 1B offset 29
    /// Reserved for future use
    pub _reserved: [u8; 2], // 2B  offset 30
                            // Total: 32B
}

impl FontLicense {
    /// Wire format size
    pub const SIZE: usize = 32;

    /// Create a free parametric font license (all rights, all platforms)
    pub fn parametric_free(params_encoded: &[u8; 40]) -> Self {
        let params_hash = fnv1a(params_encoded);
        let mut lic = Self {
            content_hash: 0,
            params_hash,
            title_id: 0,
            expires_epoch: 0,
            rights: UsageRights::all(),
            max_seats: 0,
            license_type: LicenseType::Parametric,
            platforms: PlatformRestriction::all(),
            _reserved: [0; 2],
        };
        lic.content_hash = lic.compute_hash();
        lic
    }

    /// Create a game-title-specific license
    pub fn for_game_title(
        params_encoded: &[u8; 40],
        title_id: u32,
        platforms: PlatformRestriction,
    ) -> Self {
        let params_hash = fnv1a(params_encoded);
        let mut lic = Self {
            content_hash: 0,
            params_hash,
            title_id,
            expires_epoch: 0,
            rights: UsageRights::game_standard(),
            max_seats: 0,
            license_type: LicenseType::Commercial,
            platforms,
            _reserved: [0; 2],
        };
        lic.content_hash = lic.compute_hash();
        lic
    }

    /// Encode to 32-byte wire format (little-endian, matches repr(C) layout)
    pub fn encode(&self) -> [u8; 32] {
        let mut buf = [0u8; 32];
        buf[0..8].copy_from_slice(&self.content_hash.to_le_bytes());
        buf[8..16].copy_from_slice(&self.params_hash.to_le_bytes());
        buf[16..20].copy_from_slice(&self.title_id.to_le_bytes());
        buf[20..24].copy_from_slice(&self.expires_epoch.to_le_bytes());
        buf[24..26].copy_from_slice(&self.rights.0.to_le_bytes());
        buf[26..28].copy_from_slice(&self.max_seats.to_le_bytes());
        buf[28] = self.license_type as u8;
        buf[29] = self.platforms.0;
        buf[30..32].copy_from_slice(&self._reserved);
        buf
    }

    /// Decode from 32-byte wire format (little-endian, matches repr(C) layout)
    pub fn decode(data: &[u8; 32]) -> Option<Self> {
        let content_hash = u64::from_le_bytes([
            data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
        ]);
        let params_hash = u64::from_le_bytes([
            data[8], data[9], data[10], data[11], data[12], data[13], data[14], data[15],
        ]);
        let title_id = u32::from_le_bytes([data[16], data[17], data[18], data[19]]);
        let expires_epoch = u32::from_le_bytes([data[20], data[21], data[22], data[23]]);
        let rights = UsageRights(u16::from_le_bytes([data[24], data[25]]));
        let max_seats = u16::from_le_bytes([data[26], data[27]]);
        let license_type = LicenseType::from_u8(data[28])?;
        let platforms = PlatformRestriction(data[29]);
        let _reserved = [data[30], data[31]];

        Some(Self {
            content_hash,
            params_hash,
            title_id,
            expires_epoch,
            rights,
            max_seats,
            license_type,
            platforms,
            _reserved,
        })
    }

    /// Compute deterministic content hash
    fn compute_hash(&self) -> u64 {
        let mut key = [0u8; 20];
        key[0] = self.license_type as u8;
        key[1] = self.platforms.0;
        key[2..4].copy_from_slice(&self.rights.0.to_le_bytes());
        key[4..8].copy_from_slice(&self.title_id.to_le_bytes());
        key[8..10].copy_from_slice(&self.max_seats.to_le_bytes());
        key[10..14].copy_from_slice(&self.expires_epoch.to_le_bytes());
        key[14..20].copy_from_slice(&self.params_hash.to_le_bytes()[..6]);
        fnv1a(&key)
    }
}

/// Validation result for license checks
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationResult {
    Valid,
    Expired,
    PlatformDenied,
    RightDenied,
    TitleMismatch,
    SeatLimitExceeded,
    ParamsMismatch,
}

/// Stateless license validator
pub struct LicenseValidator;

impl LicenseValidator {
    /// Validate a font license against runtime context
    pub fn validate(
        license: &FontLicense,
        current_epoch: u32,
        platform: u8,
        required_right: u16,
        title_id: u32,
        current_seats: u16,
        params_encoded: &[u8; 40],
    ) -> ValidationResult {
        // Parametric fonts are always valid
        if license.license_type == LicenseType::Parametric {
            let params_hash = fnv1a(params_encoded);
            if params_hash != license.params_hash {
                return ValidationResult::ParamsMismatch;
            }
            return ValidationResult::Valid;
        }

        // Check expiration
        if license.expires_epoch > 0 && current_epoch > license.expires_epoch {
            return ValidationResult::Expired;
        }

        // Check platform
        if !license.platforms.has(platform) {
            return ValidationResult::PlatformDenied;
        }

        // Check rights
        if !license.rights.has(required_right) {
            return ValidationResult::RightDenied;
        }

        // Check title
        if license.title_id != 0 && license.title_id != title_id {
            return ValidationResult::TitleMismatch;
        }

        // Check seat limit
        if license.max_seats > 0 && current_seats > license.max_seats {
            return ValidationResult::SeatLimitExceeded;
        }

        // Check params hash
        let params_hash = fnv1a(params_encoded);
        if params_hash != license.params_hash {
            return ValidationResult::ParamsMismatch;
        }

        ValidationResult::Valid
    }

    /// Convenience: validate for commercial game distribution
    pub fn validate_commercial_game(
        license: &FontLicense,
        current_epoch: u32,
        platform: u8,
        title_id: u32,
        params_encoded: &[u8; 40],
    ) -> ValidationResult {
        Self::validate(
            license,
            current_epoch,
            platform,
            UsageRights::COMMERCIAL | UsageRights::GAME_BUNDLE,
            title_id,
            0,
            params_encoded,
        )
    }

    /// Check if a license represents a free parametric font
    pub fn is_parametric_free(license: &FontLicense) -> bool {
        license.license_type == LicenseType::Parametric
            && license.rights.0 == UsageRights::ALL
            && license.platforms.0 == PlatformRestriction::ALL
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::param::MetaFontParams;

    fn test_params_encoded() -> [u8; 40] {
        MetaFontParams::sans_regular().encode()
    }

    #[test]
    fn test_font_license_size() {
        assert_eq!(core::mem::size_of::<FontLicense>(), FontLicense::SIZE);
    }

    #[test]
    fn test_encode_decode_roundtrip() {
        let params = test_params_encoded();
        let lic = FontLicense::parametric_free(&params);
        let encoded = lic.encode();
        assert_eq!(encoded.len(), 32);
        let decoded = FontLicense::decode(&encoded).unwrap();
        assert_eq!(lic, decoded);
    }

    #[test]
    fn test_usage_rights_bits() {
        let rights = UsageRights::game_standard();
        assert!(rights.has(UsageRights::COMMERCIAL));
        assert!(rights.has(UsageRights::EMBEDDING));
        assert!(rights.has(UsageRights::GAME_BUNDLE));
        assert!(rights.has(UsageRights::MODIFICATION));
        assert!(!rights.has(UsageRights::REDISTRIBUTION));
    }

    #[test]
    fn test_usage_rights_with_without() {
        let r = UsageRights::empty().with(UsageRights::COMMERCIAL);
        assert!(r.has(UsageRights::COMMERCIAL));
        let r = r.without(UsageRights::COMMERCIAL);
        assert!(!r.has(UsageRights::COMMERCIAL));
    }

    #[test]
    fn test_platform_restriction_bits() {
        let plat = PlatformRestriction::all();
        assert!(plat.has(PlatformRestriction::PC));
        assert!(plat.has(PlatformRestriction::CONSOLE));
        assert!(plat.has(PlatformRestriction::MOBILE));
        assert!(plat.has(PlatformRestriction::VR));
    }

    #[test]
    fn test_parametric_free_all_rights() {
        let params = test_params_encoded();
        let lic = FontLicense::parametric_free(&params);
        assert_eq!(lic.license_type, LicenseType::Parametric);
        assert_eq!(lic.rights.0, UsageRights::ALL);
        assert_eq!(lic.platforms.0, PlatformRestriction::ALL);
        assert_eq!(lic.title_id, 0);
        assert_eq!(lic.max_seats, 0);
        assert_eq!(lic.expires_epoch, 0);
    }

    #[test]
    fn test_for_game_title() {
        let params = test_params_encoded();
        let plat = PlatformRestriction(PlatformRestriction::PC | PlatformRestriction::CONSOLE);
        let lic = FontLicense::for_game_title(&params, 42, plat);
        assert_eq!(lic.license_type, LicenseType::Commercial);
        assert_eq!(lic.title_id, 42);
        assert!(lic.platforms.has(PlatformRestriction::PC));
        assert!(!lic.platforms.has(PlatformRestriction::MOBILE));
    }

    #[test]
    fn test_validate_parametric_valid() {
        let params = test_params_encoded();
        let lic = FontLicense::parametric_free(&params);
        let result = LicenseValidator::validate(
            &lic,
            0,
            PlatformRestriction::PC,
            UsageRights::COMMERCIAL,
            0,
            0,
            &params,
        );
        assert_eq!(result, ValidationResult::Valid);
    }

    #[test]
    fn test_validate_expired() {
        let params = test_params_encoded();
        let plat = PlatformRestriction::all();
        let mut lic = FontLicense::for_game_title(&params, 1, plat);
        lic.expires_epoch = 1000;
        let result = LicenseValidator::validate(
            &lic,
            2000,
            PlatformRestriction::PC,
            UsageRights::COMMERCIAL,
            1,
            0,
            &params,
        );
        assert_eq!(result, ValidationResult::Expired);
    }

    #[test]
    fn test_validate_platform_denied() {
        let params = test_params_encoded();
        let plat = PlatformRestriction(PlatformRestriction::PC);
        let lic = FontLicense::for_game_title(&params, 1, plat);
        let result = LicenseValidator::validate(
            &lic,
            0,
            PlatformRestriction::MOBILE,
            UsageRights::COMMERCIAL,
            1,
            0,
            &params,
        );
        assert_eq!(result, ValidationResult::PlatformDenied);
    }

    #[test]
    fn test_validate_title_mismatch() {
        let params = test_params_encoded();
        let plat = PlatformRestriction::all();
        let lic = FontLicense::for_game_title(&params, 42, plat);
        let result = LicenseValidator::validate(
            &lic,
            0,
            PlatformRestriction::PC,
            UsageRights::COMMERCIAL,
            99,
            0,
            &params,
        );
        assert_eq!(result, ValidationResult::TitleMismatch);
    }

    #[test]
    fn test_hash_determinism() {
        let params = test_params_encoded();
        let lic1 = FontLicense::parametric_free(&params);
        let lic2 = FontLicense::parametric_free(&params);
        assert_eq!(lic1.content_hash, lic2.content_hash);
        assert_ne!(lic1.content_hash, 0);
    }

    #[test]
    fn test_is_parametric_free() {
        let params = test_params_encoded();
        let lic = FontLicense::parametric_free(&params);
        assert!(LicenseValidator::is_parametric_free(&lic));

        let plat = PlatformRestriction::all();
        let commercial = FontLicense::for_game_title(&params, 1, plat);
        assert!(!LicenseValidator::is_parametric_free(&commercial));
    }

    #[test]
    fn test_license_type_from_u8() {
        assert_eq!(LicenseType::from_u8(0), Some(LicenseType::OpenSource));
        assert_eq!(LicenseType::from_u8(4), Some(LicenseType::Parametric));
        assert_eq!(LicenseType::from_u8(255), None);
    }

    #[test]
    fn test_decode_invalid_license_type() {
        let params = test_params_encoded();
        let lic = FontLicense::parametric_free(&params);
        let mut encoded = lic.encode();
        encoded[28] = 255; // invalid license type (offset 28 in wire format)
        assert!(FontLicense::decode(&encoded).is_none());
    }
}
