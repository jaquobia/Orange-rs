/**
The type of the value that properties store.
This design was decided upon as all integers, booleans, and enums stored
    within minecraft can be logically assigned as an unsigned integer.
*/
pub type PropertyValueType = u32;

/**
Describes the function header used to map values to a name.
*/
pub type ValueNameMapType = fn(PropertyValueType) -> String;

/**
Describes an enum as a part of a property.
*/
pub trait PropertyEnum {
    fn get_values() -> Vec<PropertyValueType>;
    fn name_value(value: PropertyValueType) -> String;
}

/**
Map 0 to "false", and 1 to "true"
*/
pub fn bool_name_map(value: PropertyValueType) -> String {
    match value {
        0 => "False",
        1 => "True",
        _ => "",
    }.to_string()
}

/**
Return the integer value as its string representative
 */
pub fn int_name_map(value: PropertyValueType) -> String {
    value.to_string()
}

/**
A representation of a set of values to a name,
and a mapping of values to value-names
 */
pub struct Property  {
    values: Vec<PropertyValueType>,
    name: String,
    value_name_map: ValueNameMapType,
}

impl PartialEq for Property {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Property {
    /**
    Create a property from a set of values, a name, and a map
     */
    pub fn new<T: IntoIterator<Item=PropertyValueType>, V: AsRef<str>>(values: T, name: V, value_name_map: ValueNameMapType) -> Self {
        Self::new_impl(values.into_iter().collect(), name.as_ref(), value_name_map)
    }

    /**
    Create a boolean based property with a name
     */
    pub fn new_bool<V: AsRef<str>>(name: V) -> Self {
        Self::new_impl(vec![0, 1], name.as_ref(), bool_name_map)
    }

    /**
    Create an integer based property with a name and a range of values
     */
    pub fn new_int<V: AsRef<str>>(name: V, min: PropertyValueType, max: PropertyValueType) -> Self {
        Self::new_impl((min..max).collect(), name.as_ref(), int_name_map)
    }

    /**
    Create an enum based property with a name
     */
    pub fn new_enum<T: PropertyEnum, V: AsRef<str>>(name: V) -> Self {
        Self::new_impl(T::get_values(), name.as_ref(), T::name_value)
    }

    fn new_impl(values: Vec<PropertyValueType>, name: &str, value_name_map: ValueNameMapType) -> Self {
        Self {
            values,
            name: name.to_string(),
            value_name_map,
        }
    }
    /**
    Returns whether the value is a legal value of the property
     */
    pub fn contains(&self, value: PropertyValueType) -> bool {
        self.values.contains(&value)
    }

    /**
    Return the name of the property
    */
    pub fn property_name(&self) -> String {
        self.name.to_string()
    }

    /**
    Return the name of the value within the property
     */
    pub fn value_name(&self, value: PropertyValueType) -> Option<String> {
        if self.contains(value) {
            Some((self.value_name_map)(value))
        } else {
            None
        }
    }


}

