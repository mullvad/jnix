use proc_macro2::Span;
use std::collections::{HashMap, HashSet};
use syn::{Attribute, Ident, Lit, LitStr, MetaNameValue};

pub struct JnixAttributes {
    flags: HashSet<String>,
    key_value_pairs: HashMap<String, LitStr>,
}

impl JnixAttributes {
    pub fn empty() -> Self {
        JnixAttributes {
            flags: HashSet::new(),
            key_value_pairs: HashMap::new(),
        }
    }

    pub fn new(attributes: &Vec<Attribute>) -> Self {
        let jnix_ident = Ident::new("jnix", Span::call_site());
        let mut flags = HashSet::new();
        let mut key_value_pairs = HashMap::new();

        for attribute in attributes {
            if attribute.path.is_ident(&jnix_ident) {
                if let Ok(flag) = attribute.parse_args::<Ident>() {
                    flags.insert(flag.to_string());
                } else if let Ok(key_value_pair) = attribute.parse_args::<MetaNameValue>() {
                    let key = key_value_pair
                        .path
                        .get_ident()
                        .expect("Invalid jnix attribute key")
                        .to_string();

                    let value = match key_value_pair.lit {
                        Lit::Str(value) => value,
                        _ => panic!("Invalid jnix attribute value"),
                    };

                    key_value_pairs.insert(key, value);
                } else {
                    panic!("Invalid jnix attribute");
                }
            }
        }

        JnixAttributes {
            flags,
            key_value_pairs,
        }
    }

    pub fn has_flag(&self, flag: &str) -> bool {
        self.flags.contains(flag)
    }

    pub fn get_value(&self, key: &str) -> Option<LitStr> {
        self.key_value_pairs.get(key).cloned()
    }
}
