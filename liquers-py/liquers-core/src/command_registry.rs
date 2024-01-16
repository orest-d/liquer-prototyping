use crate::error::Error;
use crate::query::{ActionParameter, Query};
use serde::{Deserialize, Serialize};
use serde_json::Value;

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
    pub value: Value,
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
    pub fn with_value(&mut self, name: &str, value: Value) -> &mut Self {
        self.values.push(EnumArgumentAlternative {
            name: name.to_string(),
            value
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
    pub fn name_to_value(&self, name:String)->Option<Value>{
        for alternative in &self.values {
            if alternative.name == name {
                return Some(alternative.value.clone());
            }
        }
        if self.others_allowed{
            return Some(Value::String(name));
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
pub enum DefaultValue{
    Value(Value),
    Query(Query),
    NoDefault
}

impl DefaultValue{
    fn new()->Self{
        DefaultValue::NoDefault
    }
    fn null()->Self{
        DefaultValue::Value(Value::Null)
    }
    fn is_null(&self)->bool{
        match self{
            DefaultValue::Value(value)=>value.is_null(),
            _=>false
        }
    }
    fn from_value(value:Value)->Self{
        DefaultValue::Value(value)
    }
    fn from_query(query:Query)->Self{
        DefaultValue::Query(query)
    }
    fn from_string(value:&str)->Self{
        DefaultValue::Value(Value::String(value.to_string()))
    }
    fn from_integer(value:i64)->Self{
        DefaultValue::Value(Value::Number(serde_json::Number::from(value)))
    }
    fn from_float(value:f64)->Self{
        DefaultValue::Value(Value::Number(serde_json::Number::from_f64(value).unwrap()))
    }
}

impl Default for DefaultValue{
    fn default() -> Self {
        DefaultValue::NoDefault
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ArgumentInfo {
    pub name: String,
    pub label: String,
    pub default: DefaultValue,
    pub argument_type: ArgumentType,
    pub multiple: bool,
    pub gui_info: ArgumentGUIInfo,
}

impl ArgumentInfo {
    pub fn any_argument(name: &str) -> Self {
        ArgumentInfo {
            name: name.to_string(),
            label: name.replace("_", " ").to_string(),
            default: DefaultValue::NoDefault,
            argument_type: ArgumentType::Any,
            multiple: false,
            gui_info: ArgumentGUIInfo::TextField(40),
        }
    }
    fn check(&self, realm:&str, namespace:&str, name:&str) -> Vec<CommandRegistryIssue> {
        let mut issues = Vec::new();
        issues
    }

    pub fn argument(name: &str) -> Self {
        ArgumentInfo {
            name: name.to_string(),
            label: name.replace("_", " ").to_string(),
            default: DefaultValue::NoDefault,
            argument_type: ArgumentType::Any,
            multiple: false,
            gui_info: ArgumentGUIInfo::TextField(40),
        }
    }
    pub fn string_argument(name: &str) -> Self {
        ArgumentInfo {
            name: name.to_string(),
            label: name.replace("_", " ").to_string(),
            default: DefaultValue::NoDefault,
            argument_type: ArgumentType::String,
            multiple: false,
            gui_info: ArgumentGUIInfo::TextField(40),
        }
    }
    pub fn integer_argument(name: &str, option:bool) -> Self {
        ArgumentInfo {
            name: name.to_string(),
            label: name.replace("_", " ").to_string(),
            default: if option {DefaultValue::null()} else {DefaultValue::NoDefault},
            argument_type: if option {ArgumentType::IntegerOption} else {ArgumentType::Integer},
            multiple: false,
            gui_info: ArgumentGUIInfo::IntegerField,
        }
    }
    pub fn float_argument(name: &str, option:bool) -> Self {
        ArgumentInfo {
            name: name.to_string(),
            label: name.replace("_", " ").to_string(),
            default: if option {DefaultValue::null()} else {DefaultValue::NoDefault},
            argument_type: if option {ArgumentType::FloatOption} else {ArgumentType::Float},
            multiple: false,
            gui_info: ArgumentGUIInfo::FloatField,
        }
    }
    pub fn boolean_argument(name: &str) -> Self {
        ArgumentInfo {
            name: name.to_string(),
            label: name.replace("_", " ").to_string(),
            default: DefaultValue::NoDefault,
            argument_type: ArgumentType::Boolean,
            multiple: false,
            gui_info: ArgumentGUIInfo::Checkbox,
        }
    }
    pub fn with_default_none(&mut self) -> &mut Self {
        self.default = DefaultValue::null();
        self
    }
    pub fn with_default(&mut self, value: &str) -> &mut Self {
        self.default = DefaultValue::from_string(value);
        self
    }
    pub fn true_by_default(&mut self) -> &mut Self {
        self.default = DefaultValue::from_value(Value::Bool(true));
        self
    }
    pub fn false_by_default(&mut self) -> &mut Self {
        self.default = DefaultValue::from_value(Value::Bool(false));
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
    pub fn add_command(&mut self, command: &CommandMetadata) -> &mut Self {
        self.commands.push(command.to_owned());
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