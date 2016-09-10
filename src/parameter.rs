use std::collections::HashMap;
use std::str::FromStr;

use base64::encode;
use clap::Values;
use yaml::Yaml;

pub struct Parameter {
    pub description: Option<String>,
    pub display_name: Option<String>,
    pub name: String,
    pub parameter_type: Option<ParameterType>,
    pub required: bool,
    pub value: Option<String>,
}

#[derive(PartialEq)]
pub enum ParameterType {
    Base64,
    Bool,
    Int,
    String,
}

pub struct UserValue {
    pub base64_encoded: bool,
    pub value: String,
}

pub type ParamMap = HashMap<String, Parameter>;
pub type UserValues = HashMap<String, UserValue>;

fn should_base64_encode(parameter_type: &Option<ParameterType>, user_value: &UserValue)
-> bool {
    if parameter_type.is_none() || parameter_type.as_ref().unwrap() != &ParameterType::Base64 {
        return false;
    }

    !user_value.base64_encoded
}

impl Parameter {
    pub fn new(yaml: &Yaml, user_values: &UserValues) -> Result<Self, String> {
        let description = match yaml["description"] {
            Yaml::String(ref description) => Some(description.clone()),
            _ => None,
        };
        let display_name = match yaml["displayName"] {
            Yaml::String(ref description) => Some(description.clone()),
            _ => None,
        };
        let name = match yaml["name"] {
            Yaml::String(ref name) => name.clone(),
            _ => return Err("Parameters must have a \"name\" field.".to_owned()),
        };
        let parameter_type = match yaml["parameterType"].as_str() {
            Some(ref parameter_type) => Some(try!(parameter_type.parse())),
            None => None,
        };
        let required = yaml["required"].as_bool().unwrap_or(false);
        let value = match user_values.get(&name) {
            Some(user_value) => {
                if should_base64_encode(&parameter_type, &user_value) {
                    Some(encode(user_value.value.as_bytes()))
                } else {
                    Some(user_value.value.clone())
                }
            }
            None => match yaml["value"] {
                Yaml::Boolean(ref value)  => Some(format!("{}", value)),
                Yaml::Integer(ref value) => Some(format!("{}", value)),
                Yaml::String(ref value) => Some(value.clone()),
                _ => if required {
                    return Err(
                        format!(
                            "Parameter {} required and must be {}",
                            display_name.unwrap_or(name),
                            parameter_type.map(|pt| match pt {
                                ParameterType::Base64 => "base64",
                                ParameterType::Bool => "a bool",
                                ParameterType::Int => "an int",
                                ParameterType::String => "a string"
                            }).unwrap_or("base64, bool, int, or string")
                        )
                    )
                } else {
                    None
                },
            }
        };

        Ok(Parameter {
            description: description,
            display_name: display_name,
            name: name,
            parameter_type: parameter_type,
            required: required,
            value: value,
        })
    }
}

impl FromStr for ParameterType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "base64" => Ok(ParameterType::Base64),
            "bool" => Ok(ParameterType::Bool),
            "int" => Ok(ParameterType::Int),
            "string" => Ok(ParameterType::String),
            _ => Err("parameterType must be base64, bool, int, or string.".to_owned()),
        }
    }
}

pub fn user_values(mut parameters: Values, base64_encoded: bool) -> UserValues {
    let mut user_values = UserValues::new();

    loop {
        if let Some(name) = parameters.next() {
            let value = parameters.next().expect("Parameter was missing its value.");

            let user_value = UserValue {
                base64_encoded: base64_encoded,
                value: value.to_string(),
            };

            user_values.insert(name.to_string(), user_value);
        } else {
            break;
        }
    }

    user_values
}
