use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File, path::Path};

use semver::Version;

#[derive(Debug, Deserialize, Serialize)]
pub struct AtPacks2SVDsVersionMap {
    mapping: HashMap<String, Version>,
}

impl AtPacks2SVDsVersionMap {
    pub fn new() -> Self {
        AtPacks2SVDsVersionMap { mapping: HashMap::new() }
    }

    pub fn load(path: &Path) -> Result<Self> {
        let f = File::open(path)?;
        let m: AtPacks2SVDsVersionMap = serde_json::from_reader(f)?;
        Ok(m)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let f = File::create(path)?;
        serde_json::to_writer_pretty(f, self)?;
        Ok(())
    }

    pub fn add_or_update(&mut self, svd: &str, pack_version: &Version) {
        self.mapping
            .entry(svd.to_string())
            .and_modify(|e| {
                *e = pack_version.clone();
            })
            .or_insert(pack_version.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::AtPacks2SVDsVersionMap;
    use semver::Version;
    use std::collections::HashMap;

    #[test]
    fn version_bookkeeping() {
        let mut m = AtPacks2SVDsVersionMap::new();

        m.add_or_update("ATSAMS70Q21B.SVD", &Version::parse("4.40.4").unwrap());
        m.add_or_update("ATSAMV71Q20B.SVD", &Version::parse("4.41.3").unwrap());
        m.add_or_update("ATSAMS70Q21B.SVD", &Version::parse("4.42.5").unwrap());

        assert_eq!(
            m.mapping.get("ATSAMS70Q21B.SVD"),
            Some(&Version::parse("4.42.5").unwrap())
        );
        assert_eq!(
            m.mapping.get("ATSAMV71Q20B.SVD"),
            Some(&Version::parse("4.41.3").unwrap())
        );
    }
}
