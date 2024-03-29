use serde::de::{Error, MapAccess, Unexpected, Visitor};
use serde::Deserialize;

use std::collections::BTreeMap;
use std::fmt;
use std::marker::PhantomData;

pub use super::Packages;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum PackageVersion {
    Bool(bool),
    Ver(String),
    OptionVer(Option<String>),
}

impl<'de> Deserialize<'de> for Packages {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct PackagesVisitor {
            marker: PhantomData<fn() -> Packages>,
        }

        impl<'de> Visitor<'de> for PackagesVisitor {
            type Value = Packages;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("Nullable package versions")
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                // False positive, the hash function won't read the mutable fields
                #[allow(clippy::mutable_key_type)]
                let mut inner = BTreeMap::new();
                while let Some((k, v)) = map.next_entry::<String, PackageVersion>()? {
                    inner.insert(
                        k,
                        match v {
                            PackageVersion::Bool(false) => None,
                            PackageVersion::Bool(true) => {
                                return Err(Error::invalid_value(
                                    Unexpected::Bool(false),
                                    &"false or a string",
                                ))
                            }
                            PackageVersion::Ver(ver) => Some(ver),
                            PackageVersion::OptionVer(ver) => ver,
                        },
                    );
                }
                Ok(Self::Value { inner })
            }
        }

        deserializer.deserialize_map(PackagesVisitor {
            marker: PhantomData,
        })
    }
}

#[cfg(test)]
mod test {
    use eyre::Result;

    use super::Packages;

    #[test]
    fn test_de() -> Result<()> {
        let example_packages = r#"
        konsole = "23.04.1-1"
        dolphin = "23.04.1"
        # Package removed as part of the topic
        pykde = false
        "#;

        let converted = toml::from_str::<Packages>(example_packages)?;
        println!("{:?}", converted);
        assert_eq!(converted.as_ref().len(), 3);
        assert_eq!(converted.as_ref()["konsole"], Some("23.04.1-1".to_string()));
        assert_eq!(converted.as_ref()["dolphin"], Some("23.04.1".to_string()));
        assert_eq!(converted.as_ref()["pykde"], None);
        Ok(())
    }
}
