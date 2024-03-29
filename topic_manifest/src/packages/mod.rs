//! Collection of package names and versions

mod de;

use serde::Serialize;

use std::collections::BTreeMap;

/// Collection of package names and versions
#[derive(Clone, Debug, Serialize)]
pub struct Packages {
    #[serde(flatten)]
    inner: BTreeMap<String, Option<String>>,
}

impl AsRef<BTreeMap<String, Option<String>>> for Packages {
    fn as_ref(&self) -> &BTreeMap<String, Option<String>> {
        &self.inner
    }
}
