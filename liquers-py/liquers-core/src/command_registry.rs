use crate::{error::Error, query::{ActionParameter, Query}, state};

/// A structure holding a description of an identified issue with a command registry
/// Issue can be either a warning or an error (when is_error is true)
/// Command can be identified by realm, name and namespace
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommandRegistryIssue {
    pub realm: String,
    pub namespace: String,
    pub name: String,
    pub is_error: bool,
    pub message: String,
}

impl CommandRegistryIssue {
    pub fn new(realm: &str, namespace: &str, name: &str, is_error: bool, message: String) -> Self {
        CommandRegistryIssue {
            realm: realm.to_string(),
            namespace: namespace.to_string(),
            name: name.to_string(),
            is_error,
            message: message.to_string(),
        }
    }
    pub fn warning(realm: &str, namespace: &str, name: &str, message: String) -> Self {
        CommandRegistryIssue::new(realm, name, namespace, false, message)
    }
    pub fn error(realm: &str, namespace: &str, name: &str, message: String) -> Self {
        CommandRegistryIssue::new(realm, name, namespace, true, message)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EnumArgumentAlternative {
    pub name: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EnumArgumentType {
    #[serde(rename = "string")]
    String,
    #[serde(rename = "int")]
    Integer,
    #[serde(rename = "int_opt")]
    IntegerOption,
    #[serde(rename = "float")]
    Float,
    #[serde(rename = "float_opt")]
    FloatOption,
    #[serde(rename = "bool")]
    Boolean
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EnumArgument {
    pub name: String,
    pub values: Vec<EnumArgumentAlternative>,
    pub others_allowed: bool,
    pub value_type: EnumArgumentType,
}

impl EnumArgument {
    pub fn new(name: &str) -> Self {
        EnumArgument {
            name: name.to_string(),
            values: Vec::new(),
            others_allowed: false,
            value_type: EnumArgumentType::String,
        }
    }
    pub fn with_value(&mut self, name: &str, value: &str) -> &mut Self {
        self.values.push(EnumArgumentAlternative {
            name: name.to_string(),
            value: value.to_string(),
        });
        self
    }
    pub fn with_value_type(&mut self, value_type: EnumArgumentType) -> &mut Self {
        self.value_type = value_type;
        self
    }
    pub fn with_others_allowed(&mut self) -> &mut Self {
        self.others_allowed = true;
        self
    }
    pub fn name_to_value(&self, value:&str)->Option<String>{
        for alternative in &self.values {
            if alternative.name == value {
                return Some(alternative.value.clone());
            }
        }
        if self.others_allowed{
            return Some(value.to_string());
        }
        None
    }
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ArgumentType {
    #[serde(rename = "string")]
    String,
    #[serde(rename = "int")]
    Integer,
    #[serde(rename = "int_opt")]
    IntegerOption,
    #[serde(rename = "float")]
    Float,
    #[serde(rename = "float_opt")]
    FloatOption,
    #[serde(rename = "bool")]
    Boolean,
    Enum(EnumArgument),
    #[serde(rename = "any")]
    Any,
    #[serde(rename = "none")]
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


#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ArgumentInfo {
    pub name: String,
    pub label: String,
    pub default_value: Option<String>,
    pub default_query: Option<Query>,
    pub optional: bool,
    pub argument_type: ArgumentType,
    pub multiple: bool,
    pub gui_info: ArgumentGUIInfo,
}

impl ArgumentInfo {
    pub fn any_argument(name: &str) -> Self {
        ArgumentInfo {
            name: name.to_string(),
            label: name.replace("_", " ").to_string(),
            default_value: None,
            default_query: None,
            optional: false,
            argument_type: ArgumentType::Any,
            multiple: false,
            gui_info: ArgumentGUIInfo::TextField(40),
        }
    }
    fn check(&self, realm:&str, namespace:&str, name:&str) -> Vec<CommandRegistryIssue> {
        let mut issues = Vec::new();
        if let Some(default_value) = &self.default_value {
            if let Some(default_query) = &self.default_query {
                issues.push(CommandRegistryIssue::warning(
                    realm,
                    namespace,
                    name,
                    format!(
                        "Argument {} has both default value and default query",
                        self.name
                    ),
                ));
            }
        }
        issues
    }

    pub fn string_argument(name: &str) -> Self {
        ArgumentInfo {
            name: name.to_string(),
            label: name.replace("_", " ").to_string(),
            default_value: None,
            default_query: None,
            optional: false,
            argument_type: ArgumentType::String,
            multiple: false,
            gui_info: ArgumentGUIInfo::TextField(40),
        }
    }
    pub fn integer_argument(name: &str, option:bool) -> Self {
        ArgumentInfo {
            name: name.to_string(),
            label: name.replace("_", " ").to_string(),
            default_value: if option {Some("".to_string())} else {None},
            default_query: None,
            optional: false,
            argument_type: if option {ArgumentType::IntegerOption} else {ArgumentType::Integer},
            multiple: false,
            gui_info: ArgumentGUIInfo::IntegerField,
        }
    }
    pub fn float_argument(name: &str, option:bool) -> Self {
        ArgumentInfo {
            name: name.to_string(),
            label: name.replace("_", " ").to_string(),
            default_value: if option {Some("".to_string())} else {None},
            default_query: None,
            optional: false,
            argument_type: if option {ArgumentType::FloatOption} else {ArgumentType::Float},
            multiple: false,
            gui_info: ArgumentGUIInfo::FloatField,
        }
    }
    pub fn boolean_argument(name: &str) -> Self {
        ArgumentInfo {
            name: name.to_string(),
            label: name.replace("_", " ").to_string(),
            default_value: None,
            default_query: None,
            optional: false,
            argument_type: ArgumentType::Boolean,
            multiple: false,
            gui_info: ArgumentGUIInfo::Checkbox,
        }
    }
    pub fn with_default_none(&mut self) -> &mut Self {
        self.default_value = None;
        self.default_query = None;
        self.optional = true;
        self
    }
    pub fn with_default_value(&mut self, value: &str) -> &mut Self {
        self.default_value = Some(value.to_string());
        self.default_query = None;
        self.optional = true;
        self
    }
    pub fn true_by_default(&mut self) -> &mut Self {
        self.default_value = Some("t".to_string());
        self.default_query = None;
        self.optional = true;
        self
    }
    pub fn false_by_default(&mut self) -> &mut Self {
        self.default_value = Some("f".to_string());
        self.default_query = None;
        self.optional = true;
        self
    }

    pub fn with_label(&mut self, label: &str) -> &mut Self {
        self.label = label.to_string();
        self
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CommandMetadata {
    pub realm: String,
    pub namespace: String,
    pub name: String,
    pub module: String,
    pub doc: String,
    pub state_argument: ArgumentInfo,
    pub arguments: Vec<ArgumentInfo>,
}

impl CommandMetadata{
    pub fn new(name: &str) -> Self {
        CommandMetadata {
            realm: "".to_string(),
            namespace: "root".to_string(),
            name: name.to_string(),
            module: "".to_string(),
            doc: "".to_string(),
            state_argument: ArgumentInfo::any_argument("state"),
            arguments: Vec::new(),
        }
    }
    pub fn check(&self)->Vec<CommandRegistryIssue>{
        let mut issues = Vec::new();
        for a in self.arguments.iter(){
            issues.append(&mut a.check(&self.realm, &self.namespace, &self.name));
        }
        issues
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


/// Command registry is a structure holding description (metadata) of all commands available in the system
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommandRegistry {
    pub commands: Vec<CommandMetadata>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        CommandRegistry { commands: Vec::new() }
    }
    pub fn add_command(&mut self, command: CommandMetadata) -> &mut Self {
        self.commands.push(command);
        self
    }
    pub fn find_command(&self, realm:&str, namespace:&str, name: &str) -> Option<CommandMetadata> {
        for command in &self.commands {
            if command.realm == realm && command.namespace == namespace && command.name == name {
                return Some(command.clone());
            }
        }
        None
    }
    pub fn find_command_in_namespaces(&self, realm:&str, namespaces:&Vec<String>, name:&str) -> Option<CommandMetadata> {
        for namespace in namespaces {
            if let Some(command) = self.find_command(realm, namespace, name) {
                return Some(command);
            }
        }
        None
    }
}