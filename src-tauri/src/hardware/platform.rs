use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformInfo {
    pub os: String,
    pub arch: String,
    pub edition: String,
    pub support_level: String,
}

#[must_use]
pub fn collect_platform_info() -> PlatformInfo {
    let edition = if cfg!(feature = "v3") {
        "v3"
    } else if cfg!(feature = "v2") {
        "v2"
    } else {
        "v1"
    };

    let stable = cfg!(all(
        target_os = "windows",
        target_arch = "x86_64",
        feature = "v1",
        not(feature = "v2")
    ));

    PlatformInfo {
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        edition: edition.to_string(),
        support_level: if stable { "Stable" } else { "Experimental" }.to_string(),
    }
}
