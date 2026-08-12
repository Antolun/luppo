use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter};

#[derive(Clone, Eq, PartialEq)]
pub struct PisiVersion(pub String);

impl PisiVersion {
    pub fn new(v: &str) -> Self {
        PisiVersion(v.to_string())
    }
}

impl Display for PisiVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Debug for PisiVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{}\"", self.0)
    }
}

impl Ord for PisiVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        // Simple version comparison logic splitting by . and -
        // E.g. "1.10.0" > "1.2.0"
        let parts_a: Vec<&str> = self.0.split(|c: char| !c.is_alphanumeric()).collect();
        let parts_b: Vec<&str> = other.0.split(|c: char| !c.is_alphanumeric()).collect();

        for i in 0..std::cmp::max(parts_a.len(), parts_b.len()) {
            let a = parts_a.get(i).unwrap_or(&"0");
            let b = parts_b.get(i).unwrap_or(&"0");

            if let (Ok(num_a), Ok(num_b)) = (a.parse::<u64>(), b.parse::<u64>()) {
                match num_a.cmp(&num_b) {
                    Ordering::Equal => continue,
                    other => return other,
                }
            } else {
                match a.cmp(b) {
                    Ordering::Equal => continue,
                    other => return other,
                }
            }
        }
        Ordering::Equal
    }
}

impl PartialOrd for PisiVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
