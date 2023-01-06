// Im not sure how efficient string concatenation is via format!(), so I would recommend not
// creating new Indentifiers all the time to save on performance, and rather reuse them

use std::fmt::Display;

/// Identitifer construct, a commonly seen concept in modern modded minecraft, also known as ResourceLocation in some earlier versions
/// Used easily reference specific objects and arbitrarily locate resources based on context
#[derive(Clone)]
pub struct Identifier {
    namespace: String,
    name: String,
    total_id: String,
}

impl Identifier {
    pub fn new(namespace: String, name: String) -> Self {
        let total_id =  format!("{}:{}", namespace, name);
        Self {
            namespace,
            name,
            total_id,
        }
    }

    fn from_str(identifier: &str) -> Self {
        let nn: Vec<&str> = identifier.split(":").collect();
        let (namespace, name) = if nn.len() == 2 {
            (String::from(nn[0]), String::from(nn[1]))
        } else {
            (String::from("minecraft"), String::from(nn[0]))
        };

        Self {
            namespace,
            name,
            total_id: String::from(identifier),
        }
    }

    pub fn get_total_identifier(&self) -> &String {
       &self.total_id
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
       f.write_str(self.total_id.as_str())
    }
}

impl From<&str> for Identifier {
    fn from(t: &str) -> Self {
       Self::from_str(t) 
    }
}

impl From<String> for Identifier {
    fn from(t: String) -> Self {
       Self::from_str(t.as_str()) 
    }
}
