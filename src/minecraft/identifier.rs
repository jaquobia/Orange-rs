// Im not sure how efficient string concatenation is via format!(), so I would recommend not
// creating new Indentifiers all the time to save on performance, and rather reuse them

use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};

/// Identitifer construct, a commonly seen concept in modern modded minecraft, also known as ResourceLocation in some earlier versions
/// Used to easily reference specific objects and arbitrarily locate resources based on context
#[derive(Clone, Eq)]
pub struct Identifier {
    namespace: String,
    name: String,
    total_id: String,
}

impl Display for Identifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.total_id)
    }
}

impl PartialEq for Identifier {
    fn eq(&self, other: &Self) -> bool {
        self.total_id == other.total_id
    }
}

impl Hash for Identifier {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.total_id.hash(state)
    }
}

impl Identifier {
    pub fn new(namespace: String, name: String) -> Self {
        let total_id = format!("{}:{}", namespace, name);
        Self {
            namespace,
            name,
            total_id,
        }
    }

    pub fn from_str(identifier: &str) -> Self {
        let nn: Vec<&str> = identifier.split(":").collect();
        let (namespace, name) = if nn.len() == 2 {
            (String::from(nn[0]), String::from(nn[1]))
        } else {
            (String::from("minecraft"), String::from(nn[0]))
        };

        let total_id = format!("{namespace}:{name}");

        Self {
            namespace,
            name,
            total_id,
        }
    }

    pub fn get_identifier(&self) -> &String {
        &self.total_id
    }

    pub fn get_namespace(&self) -> &String {
        &self.namespace
    }

    pub fn get_name(&self) -> &String {
        &self.name
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
