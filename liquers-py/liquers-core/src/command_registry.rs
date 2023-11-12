use crate::{error::Error, query::ActionParameter};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EnumArgumentAlternative {
    name: String,
    value: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EnumArgument {
    name: String,
    values: Vec<EnumArgumentAlternative>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ArgumentType {
    String,
    Integer,
    IntegerOption,
    Float,
    FloatOption,
    Boolean,
    Enum(EnumArgument),
    Any,
    None,
}

impl ArgumentType {
    pub fn is_option(&self) -> bool {
        match self {
            ArgumentType::IntegerOption => true,
            ArgumentType::FloatOption => true,
            _ => false,
        }
    }
    pub fn verify(&self, parameter: &ActionParameter) -> Result<(), Error> {
        if let Some(value) = parameter.string_value() {
            match self {
                ArgumentType::String => Ok(()),
                ArgumentType::Integer => {
                    if value.parse::<i64>().is_ok() {
                        Ok(())
                    } else {
                        Err(Error::ParameterError {
                            message: "Integer expected".to_string(),
                            position: parameter.position().clone(),
                        })
                    }
                }
                ArgumentType::IntegerOption => {
                    if value == "" {
                        Ok(())
                    } else if value.parse::<i64>().is_ok() {
                        Ok(())
                    } else {
                        Err(Error::ParameterError {
                            message: "Integer (optional) expected".to_string(),
                            position: parameter.position().clone(),
                        })
                    }
                }
                ArgumentType::Float => {
                    if value.parse::<f64>().is_ok() {
                        Ok(())
                    } else {
                        Err(Error::ParameterError {
                            message: "Float expected".to_string(),
                            position: parameter.position().clone(),
                        })
                    }
                }
                ArgumentType::FloatOption => {
                    if value == "" {
                        Ok(())
                    } else if value.parse::<f64>().is_ok() {
                        Ok(())
                    } else {
                        Err(Error::ParameterError {
                            message: "Float (optional) expected".to_string(),
                            position: parameter.position().clone(),
                        })
                    }
                }
                ArgumentType::Boolean => {
                    let value = value.to_lowercase();
                    if value == "true"
                        || value == "false"
                        || value == "t"
                        || value == "f"
                        || value == "1"
                        || value == "0"
                        || value == "yes"
                        || value == "no"
                        || value == "y"
                        || value == "n"
                        || value == ""
                    {
                        Ok(())
                    } else {
                        Err(Error::ParameterError {
                            message: "Boolean expected".to_string(),
                            position: parameter.position().clone(),
                        })
                    }
                }
                ArgumentType::Enum(e) => {
                    let mut found = false;
                    for alternative in &e.values {
                        if alternative.value == value {
                            found = true;
                            break;
                        }
                    }
                    if found {
                        Ok(())
                    } else {
                        Err(Error::ParameterError {
                            message: format!(
                                "Enum {} value expected; one of {}",
                                e.name,
                                e.values
                                    .iter()
                                    .map(|x| x.value.clone())
                                    .collect::<Vec<String>>()
                                    .join(", ")
                            ),
                            position: parameter.position().clone(),
                        })
                    }
                }
                ArgumentType::Any => Ok(()),
                ArgumentType::None => {
                    if value == "" {
                        Ok(())
                    } else {
                        Err(Error::ParameterError {
                            message: "None (epmty parameter) expected".to_string(),
                            position: parameter.position().clone(),
                        })
                    }
                }
            }
        } else {
            Ok(())
        }
    }
}

impl Default for ArgumentType {
    fn default() -> Self {
        ArgumentType::Any
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ArgumentInfo {
    name: String,
    label: String,
    optional: bool,
    argument_type: ArgumentType,
    multiple: bool,
    injected: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommandMetadata {
    realm: String,
    name: String,
    namespace: String,
    module: String,
    doc: String,
    state_argument: ArgumentInfo,
    arguments: Vec<ArgumentInfo>,
}
