use std::collections::HashMap;
use std::str::FromStr;

use yaml::Yaml;

pub struct Parameter {
    pub description: Option<String>,
    pub display_name: Option<String>,
    pub name: String,
    pub parameter_type: Option<ParameterType>,
    pub required: bool,
    pub value: Option<String>,
}

pub enum ParameterType {
    Bool,
    Int,
    String,
}

pub type ParamMap = HashMap<String, Parameter>;
pub type UserValues = HashMap<String, String>;

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
            Some(value) => Some(value.clone()),
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
            "base64" => Ok(ParameterType::String),
            "bool" => Ok(ParameterType::Bool),
            "int" => Ok(ParameterType::Int),
            "string" => Ok(ParameterType::String),
            _ => Err("parameterType must be base64, bool, int, or string.".to_owned()),
        }
    }
}

pub fn user_values(parameters: Vec<String>) -> Result<UserValues, String> {
    let mut user_values = HashMap::new();

    for parameter in parameters {
        let mut parts: Vec<String> = parameter.split('=').map(|s| s.to_string()).collect();

        if parts.len() < 2 {
            return Err("Parameters must be supplied in the form KEY=VALUE.".to_string());
        } else {
            user_values.insert(parts.remove(0), parts.concat());
        }
    }

    Ok(user_values)
}
