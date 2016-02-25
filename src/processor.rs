use yaml::Yaml;
use yaml::yaml::{Array, Hash};
use regex::{Captures, Regex};

use parameter::ParamMap;

pub type ProcessorResult = Result<Option<Yaml>, String>;

pub fn process_yaml(yaml: &mut Yaml, parameters: &ParamMap) -> ProcessorResult {
    match yaml {
        &mut Yaml::Array(ref mut array) => process_array(array, parameters),
        &mut Yaml::Hash(ref mut hash) => process_hash(hash, parameters),
        &mut Yaml::String(ref mut string) => process_string(string, parameters),
        _ => Ok(None),
    }
}

fn process_array(array: &mut Array, parameters: &ParamMap) -> ProcessorResult {
    for value in array {
        match process_yaml(value, parameters) {
            Ok(Some(new_value)) => *value = new_value,
            Err(error) => return Err(error),
            _ => {},
        }
    }

    Ok(None)
}

fn process_hash(hash: &mut Hash, parameters: &ParamMap) -> ProcessorResult {
    for (_, value) in hash {
        match process_yaml(value, parameters) {
            Ok(Some(new_value)) => *value = new_value,
            Err(error) => return Err(error),
            _ => {},
        }
    }

    Ok(None)
}

fn process_string(string: &mut String, parameters: &ParamMap) -> ProcessorResult {
    lazy_static! {
        static ref LITERAL_INTERPOLATION: Regex = Regex::new(
            r"\$\({2}(.*)\){2}"
        ).expect("Failed to compile regex.");
    }

    lazy_static! {
        static ref STRING_INTERPOLATION: Regex = Regex::new(
            r"\$\((.*)\)"
        ).expect("Failed to compile regex.");
    }

    let interpolate = |captures: &Captures| -> String {
        let key = captures.at(1).expect("Failed to extract regex capture group.");

        match parameters.get(key) {
            Some(parameter) => parameter.value.clone().unwrap_or("~".to_owned()),
            None => captures.at(0).expect("Failed to extract regex match.").to_owned(),
        }
    };

    let replacement = LITERAL_INTERPOLATION.replace_all(string, &interpolate);

    let contains_literal_replacement = &replacement != string;

    let final_replacement = STRING_INTERPOLATION.replace_all(&replacement, &interpolate);

    let contains_string_replacement = &final_replacement != &replacement;

    if !contains_literal_replacement && !contains_string_replacement {
        Ok(None)
    } else if contains_literal_replacement && !contains_string_replacement {
        Ok(Some(Yaml::from_str(&final_replacement)))
    } else {
        Ok(Some(Yaml::String(final_replacement)))
    }
}
