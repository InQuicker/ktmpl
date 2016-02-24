use std::str::FromStr;

use yaml::Yaml;

pub struct Parameter {
    pub description: Option<String>,
    pub display_name: Option<String>,
    pub name: String,
    pub parameter_type: Option<ParameterType>,
    pub required: bool,
    pub value: Option<ParameterValue>,
}

pub enum ParameterType {
    Bool,
    Int,
    String,
}

pub enum ParameterValue {
    Bool(bool),
    Int(i64),
    String(String),
}

impl Parameter {
    pub fn from_yaml(yaml: &Yaml) -> Result<Self, String> {
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
        let value = match yaml["value"] {
            Yaml::Boolean(ref value) => Some(ParameterValue::Bool(value.clone())),
            Yaml::Integer(ref value) => Some(ParameterValue::Int(value.clone())),
            Yaml::String(ref value) => Some(ParameterValue::String(value.clone())),
            _ => return Err("Parameter values must be boolean, i64, or string.".to_owned()),
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
