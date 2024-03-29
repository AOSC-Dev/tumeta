//! Conventional topic

use localized::Localized;
use serde::{Deserialize, Serialize};

use std::collections::BTreeMap;

use super::packages::Packages;

/// A conventional topic
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Conventional {
    name: Localized<String>,
    security: bool,
    caution: Localized<String>,
    packages: Packages,
}

impl Conventional {
    /// Get name of the topic
    pub fn get_name(&self) -> &Localized<String> {
        &self.name
    }

    /// Is this topic a security update
    pub fn is_security_update(&self) -> bool {
        self.security
    }

    /// Get localized caution strings of the topic
    pub fn get_caution(&self) -> &Localized<String> {
        &self.caution
    }

    /// Get package updates in this topic
    pub fn get_packages(&self) -> &BTreeMap<String, Option<String>> {
        self.packages.as_ref()
    }
}

#[cfg(test)]
mod test {
    use eyre::Result;
    use localized::{Locale, Localized};

    use std::collections::BTreeMap;

    use super::Conventional;

    #[test]
    fn test_de() -> Result<()> {
        let example = r#"
        name.default = "KDE Updates (Winter 2023)"
        name.zh_CN = "KDE 更新（2023 年冬季）"
        # Security update (true/false)?
        security = true
        # OPTIONAL: PSA message for users.
        caution.default = """This topic may use significantly more memory after reboot. Our testing finds that the new KDE version may use up to 16GiB of RAM."""
        caution.zh_CN = """本次更新重启后可能会需要更多内存。据我社维护者测试，新版 KDE 可能需要接近 16GiB 内存。"""

        [packages]
        konsole = "23.04.1-1"
        dolphin = "23.04.1"
        # Package removed as part of the topic.
        pykde = false
        "#;

        let converted = toml::from_str::<Conventional>(example)?;
        assert_eq!(
            converted.name,
            Localized::<String> {
                default: Some("KDE Updates (Winter 2023)".into()),
                content: BTreeMap::from(
                    [(Locale::new("zh-CN"), "KDE 更新（2023 年冬季）".into()),]
                ),
            }
        );
        assert_eq!(converted.security, true);
        assert_eq!(converted.caution, Localized::<String> {
            default: Some(r#"This topic may use significantly more memory after reboot. Our testing finds that the new KDE version may use up to 16GiB of RAM."#.into()),
            content: BTreeMap::from([
                (Locale::new("zh-CN"), "本次更新重启后可能会需要更多内存。据我社维护者测试，新版 KDE 可能需要接近 16GiB 内存。".into()),
            ]),
        });
        assert_eq!(converted.packages.as_ref().len(), 3);
        assert_eq!(
            converted.packages.as_ref()["konsole"],
            Some("23.04.1-1".to_string())
        );
        assert_eq!(
            converted.packages.as_ref()["dolphin"],
            Some("23.04.1".to_string())
        );
        assert_eq!(converted.packages.as_ref()["pykde"], None);
        Ok(())
    }
}
