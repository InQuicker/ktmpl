use std::collections::HashSet;
use std::error::Error;

use base64::encode;
use yaml::yaml::Hash;
use yaml::{EmitError, Yaml, YamlEmitter, YamlLoader};

use parameter::{ParamMap, Parameter, ParameterValues};
use processor::process_yaml;
use secret::Secret;

/// A Kubernetes manifest template and the values for each of its parameters.
#[derive(Debug)]
pub struct Template {
    objects: Vec<Yaml>,
    param_map: ParamMap,
    secrets: Option<HashSet<Secret>>,
}

impl Template {
    /// Creates a new template.
    ///
    /// # Parameters
    ///
    /// * template_contents: The YAML template file's contents.
    /// * parameter_values: A map of the template's parameters and the user-supplied values for
    ///   each.
    /// * secrets: A list of Kubernetes secrets whose data keys should be Base64 encoded after
    ///   parameter interpolation.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// * There was more than one YAML document present in the template contents.
    /// * The YAML document did not contain an "objects" key or it was not an array value.
    /// * The YAML document did not contain a "parameters" key or it was not an array value.
    /// * One of the parameters doesn't have a "name" key.
    /// * One of the parameters specifies an invalid "parameterType".
    /// * One of the parameters requires a value which wasn't supplied.
    /// * Any of the provided secrets were not found in the template.
    /// * There was an error in the structure of a secret that prevented its data from being Base64
    /// encoded.
    pub fn new(
        template_contents: String,
        parameter_values: ParameterValues,
        secrets: Option<HashSet<Secret>>,
    ) -> Result<Self, String> {
        let docs = YamlLoader::load_from_str(&template_contents)
            .map_err(|err| err.description().to_owned())?;

        if docs.len() != 1 {
            return Err("Only one YAML document can be present in the template.".to_owned());
        }

        let doc = &docs[0];

        let mut template_objects = vec![];
        let objects = match doc["objects"].as_vec() {
            Some(objects) => objects,
            None => return Err("Key \"objects\" must be present and must be an array.".to_owned())
        };

        for object in objects {
            template_objects.push(object.clone());
        }

        let mut param_map = ParamMap::new();
        let parameter_specs = match doc["parameters"].as_vec() {
            Some(parameter_specs) => parameter_specs,
            None => return Err("Key \"parameters\" must be present and must be an array.".to_owned())
        };

        for parameter_spec in parameter_specs {
            let parameter = Parameter::new(parameter_spec, &parameter_values)?;

            param_map.insert(parameter.name.clone(), parameter);
        }

        Ok(Template {
            objects: template_objects,
            param_map: param_map,
            secrets: secrets,
        })
    }

    /// Interpolates the parameters' values into the YAML template, returning the results.
    ///
    /// # Errors
    ///
    /// Returns an error if the processed template was not valid YAML, or if any specified secrets
    /// could not be found and Base64 encoded.
    pub fn process(mut self) -> Result<String, String> {
        let mut secrets_encoded = 0;

        for object in self.objects.iter_mut() {
            process_yaml(object, &self.param_map);

            if let Some(ref secrets) = self.secrets {
                if maybe_base64_encode_secret(secrets, object)? {
                    secrets_encoded += 1;
                }
            }
        }

        if let Some(ref secrets) = self.secrets {
            if secrets_encoded != secrets.len() {
                return Err("Not all secrets specified were found.".to_string());
            }
        }

        dump(self.objects)
    }

}

fn maybe_base64_encode_secret(secrets: &HashSet<Secret>, object: &mut Yaml)
-> Result<bool, String> {
    let mut hash = match object {
        &mut Yaml::Hash(ref mut hash) => hash,
        _ => return Ok(false),
    };

    if let Some(kind) = hash.get(&Yaml::String("kind".to_string())) {
        match kind {
            &Yaml::String(ref kind_string) => {
                if kind_string != "Secret" {
                    return Ok(false);
                }
            }
            _ => return Err(
                "Encountered a resource with a non-string value for the \"kind\" field.".to_string()
            ),
        }
    } else {
        return Err("Encountered a resource without a \"kind\" field.".to_string());
    }

    let metadata = match hash.get(&Yaml::String("metadata".to_string())) {
        Some(&Yaml::Hash(ref metadata)) => metadata.clone(),
        Some(_) => return Err(
            "Encountered a resource with a non-hash \"metadata\" field.".to_string()
        ),
        None => return Err(
            "Encountered a resource without a \"metadata\" field.".to_string()
        ),
    };

    let name = match metadata.get(&ystring("name")) {
        Some(&Yaml::String(ref name)) => name.to_string(),
        Some(_) => return Err(
            "Encountered a resource with a non-string \"metadata.name\" field.".to_string()
        ),
        None => return Err(
            "Encountered a resource without a \"metadata.name\" field.".to_string()
        ),
    };

    let namespace = match metadata.get(&ystring("namespace")) {
        Some(&Yaml::String(ref namespace)) => namespace.to_string(),
        Some(_) => return Err(
            "Encountered a resource with a non-string \"metadata.namespace\" field.".to_string()
        ),
        None => "default".to_string(),
    };

    let secret = Secret {
        name: name,
        namespace: namespace,
    };

    if secrets.contains(&secret) {
        if let Some(data) = hash.get_mut(&ystring("data")) {
            match data {
                &mut Yaml::Hash(ref mut data_hash) => {
                    base64_encode_secret_data(data_hash)?;
                    return Ok(true);
                }
                _ => return Err(
                    "Encountered secret with non-hash \"data\" field.".to_string()
                ),
            }
        }
    }

    return Ok(false);
}
fn base64_encode_secret_data(data: &mut Hash) -> Result<(), String> {
    for (_, value) in data.iter_mut() {
        let encoded = match value {
            &mut Yaml::String(ref value_string) => encode(value_string.as_bytes()),
            _ => return Err("Encountered non-string secret data value.".to_string()),
        };

        *value = ystring(&encoded);
    }

    Ok(())
}

fn dump(objects: Vec<Yaml>) -> Result<String, String> {
    let mut manifests = String::new();
    let last = objects.len() - 1;

    for (i, object) in objects.iter().enumerate() {
        {
            let mut emitter = YamlEmitter::new(&mut manifests);
            emitter.dump(&object).map_err(|error| {
                match error {
                    EmitError::FmtError(error) => format!("{}", error),
                    EmitError::BadHashmapKey => "Bad hashmap key in YAML structure.".to_owned(),
                }
            })?;
        }

        if i != last {
            manifests.push_str("\n");
        }
    }

    Ok(manifests)
}

fn ystring(s: &str) -> Yaml {
    Yaml::String(s.to_string())
}
