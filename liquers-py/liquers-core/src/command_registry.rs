use crate::{error::Error, query::ActionParameter};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EnumArgumentAlternative {
    pub name: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EnumArgument {
    pub name: String,
    pub values: Vec<EnumArgumentAlternative>,
    pub others_allowed: bool,
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
                    if e.others_allowed{
                        return Ok(());
                    }
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
pub enum ArgumentGUIInfo {
    TextField(usize),
    TextArea(usize, usize),
    IntegerField,
    FloatField,
    Checkbox,
    EnumSelector,
    None,
}

impl Default for ArgumentGUIInfo {
    fn default() -> Self {
        ArgumentGUIInfo::TextField(20)
    }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ArgumentInfo {
    name: String,
    label: String,
    default_value: Option<String>,
    optional: bool,
    argument_type: ArgumentType,
    multiple: bool,
    injected: bool,
    gui_info: ArgumentGUIInfo,
}

impl ArgumentInfo {
    pub fn any_argument(name: &str) -> Self {
        ArgumentInfo {
            name: name.to_string(),
            label: name.replace("_", " ").to_string(),
            optional: false,
            default_value: None,
            argument_type: ArgumentType::Any,
            multiple: false,
            injected: false,
            gui_info: ArgumentGUIInfo::TextField(40),
        }
    }
    pub fn string_argument(name: &str) -> Self {
        ArgumentInfo {
            name: name.to_string(),
            label: name.replace("_", " ").to_string(),
            optional: false,
            default_value: None,
            argument_type: ArgumentType::String,
            multiple: false,
            injected: false,
            gui_info: ArgumentGUIInfo::TextField(40),
        }
    }
    pub fn integer_argument(name: &str, option:bool) -> Self {
        ArgumentInfo {
            name: name.to_string(),
            label: name.replace("_", " ").to_string(),
            optional: option,
            default_value: if option {Some("".to_string())} else {None},
            argument_type: if option {ArgumentType::IntegerOption} else {ArgumentType::Integer},
            multiple: false,
            injected: false,
            gui_info: ArgumentGUIInfo::IntegerField,
        }
    }
    pub fn float_argument(name: &str, option:bool) -> Self {
        ArgumentInfo {
            name: name.to_string(),
            label: name.replace("_", " ").to_string(),
            optional: option,
            default_value: if option {Some("".to_string())} else {None},
            argument_type: if option {ArgumentType::FloatOption} else {ArgumentType::Float},
            multiple: false,
            injected: false,
            gui_info: ArgumentGUIInfo::FloatField,
        }
    }
    pub fn boolean_argument(name: &str) -> Self {
        ArgumentInfo {
            name: name.to_string(),
            label: name.replace("_", " ").to_string(),
            optional: false,
            default_value: None,
            argument_type: ArgumentType::Boolean,
            multiple: false,
            injected: false,
            gui_info: ArgumentGUIInfo::Checkbox,
        }
    }
    pub fn with_default_value(&mut self, value: &str) -> &mut Self {
        self.default_value = Some(value.to_string());
        self.optional = true;
        self
    }
    pub fn true_by_default(&mut self) -> &mut Self {
        self.default_value = Some("t".to_string());
        self.optional = true;
        self
    }
    pub fn false_by_default(&mut self) -> &mut Self {
        self.default_value = Some("f".to_string());
        self.optional = true;
        self
    }

    pub fn with_label(&mut self, label: &str) -> &mut Self {
        self.label = label.to_string();
        self
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommandMetadata {
    pub realm: String,
    pub name: String,
    pub namespace: String,
    pub module: String,
    pub doc: String,
    pub state_argument: ArgumentInfo,
    pub arguments: Vec<ArgumentInfo>,
}

impl CommandMetadata{
    pub fn new(name: &str) -> Self {
        CommandMetadata {
            realm: "".to_string(),
            name: name.to_string(),
            namespace: "root".to_string(),
            module: "".to_string(),
            doc: "".to_string(),
            state_argument: ArgumentInfo::any_argument("state"),
            arguments: Vec::new(),
        }
    }
    pub fn with_state_argument(&mut self, state_argument: ArgumentInfo) -> &mut Self {
        self.state_argument = state_argument;
        self
    }
    pub fn with_argument(&mut self, argument: ArgumentInfo) -> &mut Self {
        self.arguments.push(argument);
        self
    }

    pub fn with_doc(&mut self, doc: &str) -> &mut Self {
        self.doc = doc.to_string();
        self
    }
    pub fn with_realm(&mut self, realm: &str) -> &mut Self {
        self.realm = realm.to_string();
        self
    }
    pub fn with_namespace(&mut self, namespace: &str) -> &mut Self {
        self.namespace = namespace.to_string();
        self
    }
    pub fn with_module(&mut self, module: &str) -> &mut Self {
        self.module = module.to_string();
        self
    }

}