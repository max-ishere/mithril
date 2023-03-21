use serde::{
    de::{self, Visitor},
    Deserializer,
};
use toml::Value;

/// Sets the donation mining settings. This means that a percentage of the time your miner is working it will mine on an
/// address that supports the project.
///
/// Sample configuration. *NOTE:* The donation key should either be at the top of the file or using the TOML Table format.
/// ```
/// # use mithril::config::DonationConfig;
/// # use serde::Deserialize;
/// # let conf = r#"
///   donation = 2.5 # Default
/// # "#;
/// # let toml: toml::Value = toml::from_str(conf).unwrap();
/// # let parsed: DonationConfig = Deserialize::deserialize(toml["donation"].clone()).unwrap();
/// # assert_eq!(
/// #   parsed,
/// #   DonationConfig(2.5)
/// # );
/// ```
///
/// ```
/// # use mithril::config::DonationConfig;
/// # use serde::Deserialize;
/// # let conf = r#"
///   donation = 0 # Disables donation mining
/// # "#;
/// # let toml: toml::Value = toml::from_str(conf).unwrap();
/// # let parsed: DonationConfig = Deserialize::deserialize(toml["donation"].clone()).unwrap();
/// # assert_eq!(
/// #   parsed,
/// #   DonationConfig(0.0)
/// # );
/// ```
///
/// ```
/// # use mithril::config::DonationConfig;
/// # use serde::Deserialize;
/// # let conf = r#"
///   donation = false # Another way to disable donation mining
/// # "#;
/// # let toml: toml::Value = toml::from_str(conf).unwrap();
/// # let parsed: DonationConfig = Deserialize::deserialize(toml["donation"].clone()).unwrap();
/// # assert_eq!(
/// #   parsed,
/// #   DonationConfig(0.0)
/// # );
/// ```
///
/// ```
/// # use mithril::config::DonationConfig;
/// # use serde::Deserialize;
/// # let conf = r#"
///   [donation]
///   percentage = 2.5 # Use this format if the donation key is not at the top of TOML file
/// # "#;
/// # let toml: toml::Value = toml::from_str(conf).unwrap();
/// # let parsed: DonationConfig = Deserialize::deserialize(toml["donation"].clone()).unwrap();
/// # assert_eq!(
/// #   parsed,
/// #   DonationConfig(2.5)
/// # );
/// ```
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct DonationConfig(
    /// `0` disables donation mining, `1..100` will make your miner mine on
    /// the project's address for a portion of the time.
    #[serde(deserialize_with = "parse_donation")]
    pub f64,
);

impl Default for DonationConfig {
    fn default() -> Self {
        DonationConfig(2.5)
    }
}

/// `false` -> 0, `true` -> `Default::default()`, `percentage = ANY` treated as just the value of `ANY`
fn parse_donation<'de, D: Deserializer<'de>>(deserializer: D) -> Result<f64, D::Error> {
    // https://serde.rs/string-or-struct.html - How to convert several serialized types into one target deserialized type
    impl<'de> Visitor<'de> for DonationConfig {
        type Value = Self;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("parses donation percentage")
        }

        fn visit_bool<E: serde::de::Error>(self, enabled: bool) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            if enabled {
                Ok(DonationConfig::default())
            } else {
                Ok(DonationConfig(0.0))
            }
        }

        fn visit_f64<E>(self, percentage: f64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            if 0.0 <= percentage && percentage <= 100.0 {
                Ok(DonationConfig(percentage))
            } else {
                Err(de::Error::invalid_value(
                    de::Unexpected::Float(percentage),
                    &"should be within 0 to 100",
                ))
            }
        }

        fn visit_i64<E>(self, percentage: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            if (0..=100).contains(&percentage) {
                Ok(DonationConfig(percentage as f64))
            } else {
                Err(de::Error::invalid_value(
                    de::Unexpected::Signed(percentage),
                    &"should be within 0 to 100",
                ))
            }
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::MapAccess<'de>,
        {
            while let Some((key, value)) = map.next_entry::<String, toml::Value>()? {
                if key == "percentage" {
                    return match value {
                        Value::Boolean(enabled) => self.visit_bool(enabled),
                        Value::Float(percentage) => self.visit_f64(percentage),
                        Value::Integer(percentage) => self.visit_i64(percentage as i64),
                        _ => Err(serde::de::Error::invalid_type(
                            de::Unexpected::Other("unsupported type"),
                            &"should be either boolean or number",
                        )),
                    };
                }
            }
            Err(serde::de::Error::missing_field("percentage"))
        }
    }

    Ok(deserializer.deserialize_any(DonationConfig::default())?.0)
}
