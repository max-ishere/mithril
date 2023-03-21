// Because doctest uses the whole block, but docstring - only string content
// which isn't valid rust code.
#[allow(rustdoc::invalid_rust_codeblocks)]
/// Pool connection settings inside the TOML file.
///
/// Sample configuration:
/// ```rust
/// # use mithril::config::PoolConfig;
/// # use serde::Deserialize;
/// # let conf = r#"
///   [pool]
///   url = "xmr.example.com:1111"
///   pass = "x"
///   user = "800...dead"
/// # "#;
/// # let toml: toml::Value = toml::from_str(conf).unwrap();
/// # let parsed: PoolConfig = Deserialize::deserialize(toml["pool"].clone()).unwrap();
/// # assert_eq!(
/// #   parsed,
/// #   PoolConfig::new("xmr.example.com:1111", "x", "800...dead")
/// # );
/// ```
///
/// Most pools will expect `user` to be your XMR payout wallet (long string of numbers and letters),
/// but some may use an email instead. The user string may also include things like rig IDs and share difficulty.
/// So be sure to read what's on their website.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct PoolConfig {
    pub url: String,
    /// Password for pool to accept your connection.
    pub pass: String,
    pub user: String,
}

impl PoolConfig {
    pub fn new(url: &str, pass: &str, user: &str) -> Self {
        Self {
            url: url.to_string(),
            pass: pass.to_string(),
            user: user.to_string(),
        }
    }

    pub fn donation_mode() -> Self {
        Self {
            url: "xmrpool.eu:3333".to_string(),
            pass: "x".to_string(),
            user: "48y3RCT5SzSS4jumHm9rRL91eWWzd6xcVGSCF1KUZGWYJ6npqwFxHee4xkLLNUqY4NjiswdJhxFALeRqzncHoToeJMg2bhL".to_string()
        }
    }
}
