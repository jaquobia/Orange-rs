use rustc_hash::FxHashMap as HashMap;

use crate::minecraft::{identifier::Identifier, registry::Registerable};

/**
The type of the value that properties store.
This design was decided upon as all integers, booleans, and enums stored
    within minecraft can be logically assigned as an unsigned integer.
*/
pub type PropertyValueType = u32;

/**
Describes the function header used to map values to a name.
*/

#[derive(Debug, Copy, Clone)]
pub enum PropertyError {
    NotValidNamedValue,
    NotValidIndexValue,
}
pub type PropertyResult<T> = Result<T, PropertyError>;

/**
Describes an enum as a part of a property.
*/
pub trait PropertyEnum {
    fn get_values() -> Vec<PropertyValueType>;
    fn name_value(value: PropertyValueType) -> String;
}

pub struct PropertyDefinition {
    m_value_to_name: Box<[String]>,
    m_name_to_value: HashMap<String, PropertyValueType>,
    identifier: Identifier,
}

impl PropertyDefinition {
    pub fn new<S: AsRef<str> + Clone>(identifier: Identifier, strings: &[S]) -> Self {
        let num_to_values = strings.into_iter().map(|s| s.as_ref().to_string()).collect::<Vec<String>>().into_boxed_slice();
        let mut values_to_num = HashMap::default();
        for (num, value) in num_to_values.iter().enumerate() {
            values_to_num.insert(value.clone(), num as PropertyValueType);
        }
        Self {
            m_value_to_name: num_to_values,
            m_name_to_value: values_to_num,
            identifier,
        }
    }

    pub fn name_to_value<S: AsRef<str>>(&self, name: S) -> PropertyResult<PropertyValueType> {
        self.m_name_to_value.get(name.as_ref()).map(|&v|v).ok_or(PropertyError::NotValidNamedValue)
    }
    
    pub fn value_to_name(&self, value: PropertyValueType) -> PropertyResult<&String> {
        self.m_value_to_name.get(value as usize).ok_or(PropertyError::NotValidIndexValue)
    }

    pub fn get_names(&self) -> &[String] {
        &self.m_value_to_name
    }
}

impl Registerable for PropertyDefinition {
    fn get_identifier(&self) -> &Identifier {
        &self.identifier
    }
}
