use std::cmp::Ordering;

use serde::{Serialize, Deserialize};

#[derive(Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct FactorioVersion {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
    pub build: u16,
}
impl Ord for FactorioVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.major == other.major {
            if self.minor == other.minor {
                if self.patch == other.patch {
                    if self.build == other.build {
                        Ordering::Equal
                    } else if self.build > other.build {
                        Ordering::Greater
                    } else if self.build < other.build {
                        Ordering::Less
                    } else { unreachable!() }
                } else if self.patch > other.patch {
                    Ordering::Greater
                } else if self.patch < other.patch {
                    Ordering::Less
                } else { unreachable!() }
            } else if self.minor > other.minor {
                Ordering::Greater
            } else if self.minor < other.minor {
                Ordering::Less
            } else { unreachable!() }
        } else if self.major > other.major {
            Ordering::Greater
        } else if self.major < other.major {
            Ordering::Less
        } else { unreachable!() }
    }
}
impl PartialOrd for FactorioVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}