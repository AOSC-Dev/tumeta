//! Cumulative topic

use localized::Localized;
use serde::{Deserialize, Serialize};

/// Cumulative topic
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Cumulative {
    name: Localized<String>,
    topics: Vec<String>,
}

impl Cumulative {
    /// Name of the topic
    pub fn get_name(&self) -> &Localized<String> {
        &self.name
    }

    /// Conventional topics used in this topic
    pub fn get_topics(&self) -> &[String] {
        &self.topics
    }
}

#[cfg(test)]
mod test {
    use eyre::Result;
    use localized::{Locale, Localized};

    use std::collections::BTreeMap;

    use super::Cumulative;

    #[test]
    fn test_de() -> Result<()> {
        let example = r#"
        name.default = "Winter 2023 Cumulative Update for amd64 AOSC OS systems"
        name.zh_MS = "适用于 amd64 AOSC OS 版本的 23 冬季累计更新"

        # Must not exist alongside [packages].
        topics = [
            "kde-survey-20231201",
            "core-12.1.0"
        ]
        "#;

        let converted = toml::from_str::<Cumulative>(example)?;
        assert_eq!(
            converted.name,
            Localized::<String> {
                default: Some("Winter 2023 Cumulative Update for amd64 AOSC OS systems".into()),
                content: BTreeMap::from([(
                    Locale::new("zh_MS"),
                    "适用于 amd64 AOSC OS 版本的 23 冬季累计更新".into()
                ),]),
            }
        );
        assert_eq!(
            converted.topics,
            ["kde-survey-20231201".to_string(), "core-12.1.0".to_string(),]
        );
        Ok(())
    }
}
