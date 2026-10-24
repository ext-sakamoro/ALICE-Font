//! Fuzz target: FontLicense 32-byte wire format decode が任意 byte 列で panic しないことを検証
//!
//!
//! 起こり得る危険:
//! - malformed byte 列で LicenseType::from_u8 が None を返す → decode 全体 None であるべき
//! - reserved 領域の値が signalling 情報として誤解釈される
//! - roundtrip (decode → encode) で 32-byte 一致性が壊れる (Some case のみ)

#![no_main]

use alice_font::FontLicense;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.len() < FontLicense::SIZE {
        return;
    }

    // 32-byte fixed-size decode
    let mut buf = [0u8; 32];
    buf.copy_from_slice(&data[..FontLicense::SIZE]);

    match FontLicense::decode(&buf) {
        Some(lic) => {
            // decode 成功 → encode roundtrip で bit-exact 一致を検証
            let re_encoded = lic.encode();
            assert_eq!(
                buf, re_encoded,
                "FontLicense encode/decode roundtrip drift"
            );
        }
        None => {
            // decode 失敗 (LicenseType::from_u8 が unknown discriminant で None)
            // panic せず None が返ることが検証済み
        }
    }
});
