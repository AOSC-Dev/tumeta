//! # topic_manifest
//!
//! Collection of types for serializing, deserializing, and processing topic manifests for AOSC OS.

pub mod conventional;
pub mod cumulative;
pub mod packages;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

use serde::{Deserialize, Serialize};

use std::collections::BTreeMap;

pub use localized::{Locale, Localized};

pub use conventional::Conventional;
pub use cumulative::Cumulative;
pub use packages::Packages;

/// Internal type for deserializing untagged manifest data
#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
enum ManifestUntagged {
    Conventional(Conventional),
    Cumulative(Cumulative),
}

/// Topic update manifest
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(from = "ManifestUntagged")]
#[serde(rename_all = "lowercase")]
pub enum Manifest {
    /// A conventional topic
    Conventional(Conventional),
    /// A cumulative topic
    Cumulative(Cumulative),
}

/// Collection of multiple topic manifests
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ManifestCollection {
    #[serde(flatten)]
    topics: BTreeMap<String, Manifest>,
}

impl From<ManifestUntagged> for Manifest {
    fn from(value: ManifestUntagged) -> Self {
        match value {
            ManifestUntagged::Conventional(inner) => Self::Conventional(inner),
            ManifestUntagged::Cumulative(inner) => Self::Cumulative(inner),
        }
    }
}

impl From<BTreeMap<String, Manifest>> for ManifestCollection {
    fn from(value: BTreeMap<String, Manifest>) -> Self {
        Self { topics: value }
    }
}

impl Manifest {
    /// Is this a conventional topic
    pub fn is_conventional(&self) -> bool {
        match self {
            Self::Conventional(_) => true,
            _ => false,
        }
    }

    /// Is this a cumulative topic
    pub fn is_cumulative(&self) -> bool {
        match self {
            Self::Cumulative(_) => true,
            _ => false,
        }
    }
}

impl ManifestCollection {
    /// Get length of the collection
    pub fn len(&self) -> usize {
        self.topics.len()
    }

    /// Is the manifest collection empty
    pub fn is_empty(&self) -> bool {
        self.topics.is_empty()
    }

    /// Get a list of missing topics in the manifest collection
    pub fn find_missing_topics(&self) -> Vec<(String, Vec<String>)> {
        #[cfg(not(feature = "parallel"))]
        let iter = self.topics.iter();
        #[cfg(feature = "parallel")]
        let iter = self.topics.par_iter();

        iter.filter_map(|(k, v)| {
            match v {
                Manifest::Conventional(_) => None,
                Manifest::Cumulative(c) => Some((k, c.get_topics())),
            }
        }).filter_map(|(k, v)| {
            let missing: Vec<String> = v.iter().filter_map(|topic: &String| {
                if ! self.topics.contains_key(topic) {
                    Some(topic.to_string())
                } else {
                    None
                }
            }).collect();
            if missing.is_empty() {
                None
            } else {
                Some((k.to_string(), missing))
            }
        }).collect()
    }

    /// Is this topic manifest collection consistent
    pub fn is_consistent(&self) -> bool {
        self.find_missing_topics().is_empty()
    }
}

#[cfg(test)]
mod test {
    use eyre::Result;

    use std::collections::BTreeMap;

    use super::{Manifest, ManifestCollection};

    #[test]
    fn test_manifest_serde() -> Result<()> {
        let example1 = r#"
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

        let example2 = r#"
        name.default = "Winter 2023 Cumulative Update for amd64 AOSC OS systems"
        name.zh_MS = "适用于 amd64 AOSC OS 版本的 23 冬季累计更新"

        # Must not exist alongside [packages].
        topics = [
            "kde-survey-20231201",
            "core-12.1.0"
        ]
        "#;

        let converted1 = toml::from_str::<Manifest>(example1)?;
        let converted2 = toml::from_str::<Manifest>(example2)?;
        assert!(matches!(converted1, Manifest::Conventional(_)));
        assert!(matches!(converted2, Manifest::Cumulative(_)));

        let manifests = ManifestCollection {
            topics: BTreeMap::from([
                ("kde-survey-20231201".to_string(), converted1),
                ("cumulative-2023H3".to_string(), converted2),
            ]),
        };
        assert_eq!(manifests.find_missing_topics(), vec![("cumulative-2023H3".to_string(), vec!["core-12.1.0".to_string()])]);
        assert!(! manifests.is_consistent());

        let manifests_text = "{\"cumulative-2023H3\":{\"type\":\"cumulative\",\"name\":{\"default\":\"Winter 2023 Cumulative Update for amd64 AOSC OS systems\",\"zh_MS\":\"适用于 amd64 AOSC OS 版本的 23 冬季累计更新\"},\"topics\":[\"kde-survey-20231201\",\"core-12.1.0\"]},\"kde-survey-20231201\":{\"type\":\"conventional\",\"name\":{\"default\":\"KDE Updates (Winter 2023)\",\"zh_CN\":\"KDE 更新（2023 年冬季）\"},\"security\":true,\"caution\":{\"default\":\"This topic may use significantly more memory after reboot. Our testing finds that the new KDE version may use up to 16GiB of RAM.\",\"zh_CN\":\"本次更新重启后可能会需要更多内存。据我社维护者测试，新版 KDE 可能需要接近 16GiB 内存。\"},\"packages\":{\"dolphin\":\"23.04.1\",\"konsole\":\"23.04.1-1\",\"pykde\":null}}}";
        let _: ManifestCollection =
            serde_json::from_str(r#"
            {
                "cumulative-2023H3": {
                    "type": "cumulative",
                    "name": {
                        "default": "Winter 2023 Cumulative Update for amd64 AOSC OS systems",
                        "zh_MS": "适用于 amd64 AOSC OS 版本的 23 冬季累计更新"
                    },
                    "topics": [
                        "kde-survey-20231201",
                        "core-12.1.0"
                    ]
                },
                "kde-survey-20231201": {
                    "type": "conventional",
                    "name": {
                        "default": "KDE Updates (Winter 2023)",
                        "zh_CN": "KDE 更新（2023 年冬季）"
                    },
                    "security": true,
                    "caution": {
                        "default": "This topic may use significantly more memory after reboot. Our testing finds that the new KDE version may use up to 16GiB of RAM.",
                        "zh_CN": "本次更新重启后可能会需要更多内存。据我社维护者测试，新版 KDE 可能需要接近 16GiB 内存。"
                    },
                    "packages": {
                        "dolphin": "23.04.1",
                        "konsole": "23.04.1-1",
                        "pykde": null
                    }
                }
            }"#).expect("Failed to parse manifest collection");
        // println!("{}", serde_json::to_string(&manifests).unwrap());
        assert_eq!(manifests_text, serde_json::to_string(&manifests).unwrap());
        Ok(())
    }
}
