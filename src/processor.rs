use yaml::Yaml;
use yaml::yaml::{Array, Hash};

use parameter::ParamMap;

pub type ProcessorResult = Result<(), String>;

pub fn process_yaml(yaml: &mut Yaml, parameters: &ParamMap) -> ProcessorResult {
    match yaml {
        &mut Yaml::Array(ref mut array) => try!(process_array(array, parameters)),
        &mut Yaml::Hash(ref mut hash) => try!(process_hash(hash, parameters)),
        &mut Yaml::String(ref mut string) => try!(process_string(string, parameters)),
        _ => {},
    }

    Ok(())
}

fn process_array(array: &mut Array, parameters: &ParamMap) -> ProcessorResult {
    for value in array {
        try!(process_yaml(value, parameters));
    }

    Ok(())
}

fn process_hash(hash: &mut Hash, parameters: &ParamMap) -> ProcessorResult {
    for (_, value) in hash {
        try!(process_yaml(value, parameters));
    }

    Ok(())
}

fn process_string(string: &mut String, parameters: &ParamMap) -> ProcessorResult {
    // TODO: Interpolate any parameters.

    Ok(())
}
