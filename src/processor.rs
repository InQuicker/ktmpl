use yaml::Yaml;
use yaml::yaml::{Array, Hash};
use regex::{Captures, Regex};

use parameter::ParamMap;

pub fn process_yaml(yaml: &mut Yaml, parameters: &ParamMap) -> Option<Yaml> {
    match yaml {
        &mut Yaml::Array(ref mut array) => process_array(array, parameters),
        &mut Yaml::Hash(ref mut hash) => process_hash(hash, parameters),
        &mut Yaml::String(ref mut string) => process_string(string, parameters),
        _ => None,
    }
}

fn process_array(array: &mut Array, parameters: &ParamMap) -> Option<Yaml> {
    for value in array {
        match process_yaml(value, parameters) {
            Some(new_value) => *value = new_value,
            _ => {},
        }
    }

    None
}

fn process_hash(hash: &mut Hash, parameters: &ParamMap) -> Option<Yaml> {
    for (_, value) in hash {
        match process_yaml(value, parameters) {
            Some(new_value) => *value = new_value,
            _ => {},
        }
    }

    None
}

fn process_string(string: &mut String, parameters: &ParamMap) -> Option<Yaml> {
    lazy_static! {
        static ref LITERAL_INTERPOLATION: Regex = Regex::new(
            r"\$\({2}([^\)]*)\){2}"
        ).expect("Failed to compile regex.");
    }

    lazy_static! {
        static ref STRING_INTERPOLATION: Regex = Regex::new(
            r"\$\(([^\)]*)\)"
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
        None
    } else if contains_literal_replacement && !contains_string_replacement {
        Some(Yaml::from_str(&final_replacement))
    } else {
        Some(Yaml::String(final_replacement))
    }
}
