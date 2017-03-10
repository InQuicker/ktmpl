use std::collections::HashMap;
use std::str::FromStr;

use base64::encode;
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

/// The user-supplied value of a template parameter, either plain text or Base64-encoded.
pub enum ParameterValue {
    /// A plain text parameter value.
    Plain(String),
    /// A Base64-encoded parameter value.
    Encoded(String),
}

pub type ParamMap = HashMap<String, Parameter>;

/// A map of parameter names to user-supplied values of the parameters.
pub type ParameterValues = HashMap<String, ParameterValue>;

fn maybe_base64_encode(parameter_type: &Option<ParameterType>, user_value: &ParameterValue) -> String {
    if parameter_type.is_none() || parameter_type.as_ref().unwrap() != &ParameterType::Base64 {
        return match *user_value {
            ParameterValue::Plain(ref value) => value.clone(),
            ParameterValue::Encoded(ref value) => value.clone(),
        };
    }

    match *user_value {
        ParameterValue::Plain(ref value) => encode(value.as_bytes()),
        ParameterValue::Encoded(ref value) => value.clone(),
    }
}

impl Parameter {
    pub fn new(yaml: &Yaml, user_values: &ParameterValues) -> Result<Self, String> {
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
            Some(ref parameter_type) => Some(parameter_type.parse()?),
            None => None,
        };
        let required = yaml["required"].as_bool().unwrap_or(false);
        let value = match user_values.get(&name) {
            Some(user_value) => Some(maybe_base64_encode(&parameter_type, &user_value)),
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
