extern crate proc_macro;
extern crate quote;
extern crate regex;
extern crate syn;

use proc_macro::TokenStream;
use quote::quote;
use regex::Regex;
use std::env;
use std::path::PathBuf;
use syn::{parse_macro_input, LitStr};

mod migration;
use migration::Migration;

#[proc_macro]
pub fn nomad_migrations(input: TokenStream) -> TokenStream {
    let file_regex = Regex::new(r"^(\d+).*\.sql$").expect("Literal regex is known good");

    let input = parse_macro_input!(input as LitStr);
    let crate_root = env::var("CARGO_MANIFEST_DIR").unwrap();
    let path = PathBuf::from(crate_root).join(input.value());

    if !path.exists() {
        panic!("\"{}\" doesn't exist", path.display());
    }

    let mut sorted_migrations = Vec::new();
    for entry in path.read_dir().expect("Failed to get children") {
        let entry = entry.expect("Could not load child");
        let entry_path = entry.path();

        let filename = entry_path
            .file_name()
            .unwrap()
            .to_str()
            .expect("Migration filenames must be valid UTF-8");

        let captures = file_regex.captures(filename).expect(&format!(
            "\"{}\" is not a valid migration file name",
            filename
        ));

        sorted_migrations.push(Migration {
            version: captures
                .get(1)
                .expect("Version number missing")
                .as_str()
                .parse::<u64>()
                .expect("Unable to parse version number"),
            sql: std::fs::read_to_string(entry_path).expect("Failed to read migration SQL"),
        });
    }

    sorted_migrations.sort();

    // We've sorted the migrations ahead of time, so the invariant is preserved
    TokenStream::from(
        quote! { unsafe { ::nomad::ordered::OrderedArray::new_unsafe([#(#sorted_migrations), *]) } }
    )
}
