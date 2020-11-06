use quote::{ quote, ToTokens };

// Wraps a migration as parsed from the disk.
pub(crate) struct Migration {
    pub version: u64,
    pub sql: String,
}

impl PartialEq for Migration {
    fn eq(&self, other: &Self) -> bool {
        self.version.eq(&other.version)
    }
}
impl Eq for Migration {}

impl PartialOrd for Migration {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.version.partial_cmp(&other.version)
    }
}

impl Ord for Migration {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.version.cmp(&other.version)
    }
}

impl ToTokens for Migration {
    fn to_tokens(&self, tokens: &mut quote::__private::TokenStream) {
        let version = &self.version;
        let sql = &self.sql;

        let tok = quote! { ::nomad::Migration { version: #version, sql: #sql } };

        tok.to_tokens(tokens)
    }
}
