use anyhow::{anyhow, Result};
use std::time::{SystemTime, UNIX_EPOCH};

/// Decodes a hex string into a UTF-8 string.
/// Returns an error if the hex string is invalid or the decoded bytes aren't valid ASCII printable characters.
pub fn decode_datakey(hex_string: &str) -> Result<String> {
    // Remove 0x prefix if present
    let clean_hex = if hex_string.starts_with("0x") {
        &hex_string[2..]
    } else {
        hex_string
    };

    // Validate hex string length
    if clean_hex.len() % 2 != 0 {
        return Err(anyhow!("datakey decoding failed: odd number of hex digits"));
    }

    // Decode hex to bytes
    let bytes = (0..clean_hex.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&clean_hex[i..i + 2], 16)
                .map_err(|_| anyhow!("datakey decoding failed: invalid hex digit"))
        })
        .collect::<Result<Vec<u8>, _>>()?;

    // Decode bytes to UTF-8 string
    let decoded = String::from_utf8(bytes)
        .map_err(|_| anyhow!("datakey decoding failed: invalid UTF-8 sequence"))?;

    // Check if all characters are printable ASCII (range 0x20 to 0x7E)
    if decoded.chars().all(|c| c >= ' ' && c <= '~') {
        Ok(decoded)
    } else {
        Err(anyhow!(
            "datakey decoding failed: contains non-printable characters"
        ))
    }
}

pub fn make_json_timestamp() -> serde_json::Number {
    let systemtime = SystemTime::now();

    let duration_since_epoch = systemtime
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let secs = duration_since_epoch.as_secs();
    let now: serde_json::Number = secs.into();
    return now;
}
pub fn make_timestamp_secs() -> u64 {
    let systemtime = SystemTime::now();

    let duration_since_epoch = systemtime
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let secs = duration_since_epoch.as_secs();
    return secs;
}
pub fn make_timestamp_ms() -> u128 {
    let systemtime = SystemTime::now();

    let duration_since_epoch = systemtime
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let ms = duration_since_epoch.as_millis();
    return ms;
}
