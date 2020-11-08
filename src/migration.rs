use std::cmp::Ordering;

#[derive(Debug, Copy, Clone)]
pub struct Migration<'a> {
    pub version: u64,
    pub sql: &'a str,
}

impl PartialEq for Migration<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.version.eq(&other.version)
    }
}

impl Eq for Migration<'_> {}

impl PartialOrd for Migration<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.version.partial_cmp(&other.version)
    }
}

impl Ord for Migration<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.version.cmp(&other.version)
    }
}
