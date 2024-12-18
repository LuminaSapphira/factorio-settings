use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct FactorioVersion {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
    pub build: u16,
}
impl Ord for FactorioVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.major.cmp(&other.major) {
            Ordering::Equal => match self.minor.cmp(&other.minor) {
                Ordering::Equal => match self.patch.cmp(&other.patch) {
                    Ordering::Equal => self.build.cmp(&other.build),
                    other => other,
                },
                other => other,
            },
            other => other,
        }
    }
}
impl PartialOrd for FactorioVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
