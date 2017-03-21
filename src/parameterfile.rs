use std::error::Error;
use std::fs::File;
use std::io::Read;

use yaml::{Yaml, YamlLoader};
use parameter::{ParameterValue, ParameterValues};

#[derive(Debug)]
/// ParameterFile struct
pub struct ParameterFile {
  /// filename
  pub filename: String,
  /// document string
  pub doc_str: String,
  /// parameters allocated from this file
  pub parameters: ParameterValues,
}

fn parse_document(doc_str: &String, parameter_values: &mut ParameterValues) {
    let docs = YamlLoader::load_from_str(doc_str).unwrap();
    for doc in &docs {
        let primary_key = "";
        let param_values = parse_yaml(doc, primary_key);
        parameter_values.extend(param_values);
      }
}

fn parse_yaml(doc: &Yaml, primary_key: &str) -> ParameterValues {
    let mut param_values = ParameterValues::new();
    match doc {
        &Yaml::Hash(ref h) => {
            for (key,value) in h {
                let combined_key = primary_key.to_string() + key.as_str().unwrap();
                match value {
                    &Yaml::String(ref s) => {
                        let pv = ParameterValue::Plain(s.to_string());
                        param_values.insert(combined_key,pv);
                    },
                    &Yaml::Integer(ref i) => {
                        let pv = ParameterValue::Plain(i.to_string());
                        param_values.insert(combined_key,pv);
                    },
                    &Yaml::Real(ref r) => {
                        let pv = ParameterValue::Plain(r.to_string());
                        param_values.insert(combined_key,pv);
                    },
                    _ => {
                        // Value type not supported
                        // Array, Alias and None
                    }
                }
            }
        },
        &Yaml::String(ref s) => {
            let pv = ParameterValue::Plain(s.to_string());
            param_values.insert(primary_key.to_string(),pv);
        },
        _ => {
            // Key type not supported
        }
    }
    param_values
}

impl ParameterFile {
    /// Create a new parameterfile object, composed of a filename
    /// and the parsed parameters
    pub fn from_file(filename: &str) -> Result<Self, String> {
        let mut parameter_values = ParameterValues::new();
        let mut fh = File::open(filename).map_err(|err| err.description().to_owned()).unwrap();
        let mut contents = String::new();
        fh.read_to_string(&mut contents).map_err(|err| err.description().to_owned())?;
        parse_document(&contents, &mut parameter_values);

        Ok(ParameterFile {
            filename: String::from(filename),
            doc_str: String::from(contents),
            parameters: parameter_values,
        })
    }

    /// Create a new parameterfile object from a String representing a yaml document
    pub fn from_str(doc_str: String) -> Result<Self, String> {
        let mut parameter_values = ParameterValues::new();
        parse_document(&doc_str, &mut parameter_values);

        Ok(ParameterFile {
            filename: String::from(""),
            doc_str: String::from(doc_str),
            parameters: parameter_values,
        })
    }
}
