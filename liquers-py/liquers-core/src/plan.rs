use std::fmt::Display;

use itertools::Itertools;
use nom::Err;

use crate::command_registry::{
    self, ArgumentInfo, ArgumentType, CommandMetadata, CommandRegistry, EnumArgumentType,
};
use crate::error::Error;
use crate::query::{ActionParameter, ActionRequest, Key, Query, QuerySegment, ResourceName, ResourceQuerySegment};
use crate::value::ValueInterface;

#[derive(Serialize, Deserialize, Debug, Clone)]

pub enum Step {
    GetResource(Key),
    GetResourceMetadata(Key),
    Evaluate(Query),
    ApplyAction {
        ns: Option<Vec<ActionParameter>>,
        action: ActionRequest,
    },
    Filename(ResourceName),
    Info(String),
    Warning(String),
    Error(String),
    Plan(Plan),
}

impl Step {
    pub fn is_error(&self) -> bool {
        match self {
            Step::Error(_) => true,
            _ => false,
        }
    }
    pub fn is_warning(&self) -> bool {
        match self {
            Step::Warning(_) => true,
            _ => false,
        }
    }
}

impl Display for Step {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Step::GetResource(s) => write!(f, "GET RES         {}", s.encode()),
            Step::GetResourceMetadata(s) => {
                write!(f, "GET RES META    {}", s.encode())
            }
            Step::Evaluate(s) => write!(f, "EVALUATE        {}", s.encode()),
            Step::ApplyAction { ns, action } => write!(
                f,
                "APPLY ACTION    ({}): {}",
                ns.as_ref()
                    .map_or("root".into(), |ap| ap.iter().map(|x| x.encode()).join(",")),
                action.encode()
            ),
            Step::Filename(s) => write!(f, "FILENAME        {}", s.encode()),
            Step::Info(s) => write!(f, "INFO            {s}"),
            Step::Warning(s) => write!(f, "WARNING         {s}"),
            Step::Error(s) => write!(f, "ERROR           {s}"),
            Step::Plan(p) => write!(f, "PLAN            {p}"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResolvedParameters<V: ValueInterface> {
    pub parameters: Vec<V>,
    pub links: Vec<(usize, Query)>,
}

impl<V: ValueInterface> ResolvedParameters<V> {
    pub fn new() -> Self {
        ResolvedParameters {
            parameters: Vec::new(),
            links: Vec::new(),
        }
    }
}

struct PlanBuilder<'c, V: ValueInterface> {
    query: Query,
    command_registry: &'c CommandRegistry,
    command_metadata: CommandMetadata,
    action_request: ActionRequest,
    resolved_parameters: ResolvedParameters<V>,
    parameter_number: usize,
    arginfo_number: usize,
    plan: Plan,
}

impl<'c, V: ValueInterface> PlanBuilder<'c, V> {
    fn new(query: Query, command_registry: &'c CommandRegistry) -> Self {
        PlanBuilder {
            query,
            command_registry,
            command_metadata: CommandMetadata::default(),
            action_request: ActionRequest::default(),
            resolved_parameters: ResolvedParameters::new(),
            parameter_number: 0,
            arginfo_number: 0,
            plan: Plan::new(),
        }
    }

    fn build(&mut self) -> Result<Plan, Error> {
        let query = self.query.clone();
        self.process_query(&query)?;
        Ok(self.plan.clone())
    }

    fn get_namespaces(&self, query: &Query) -> Result<Vec<String>, Error> {
        let mut namespaces = Vec::new();
        if let Some(ns) = query.last_ns() {
            for x in ns.iter(){
                match x {
                    ActionParameter::String(s, _) => {
                        namespaces.push(s.to_string())
                    },
                    _ => {
                        return Err(Error::NotSupported {
                          message: "Only string parameters are supported in ns".into(),
                        })
                    }
                }
            }
        }
        // TODO: get default namespaces from command registry
        namespaces.push("".to_string());
        namespaces.push("root".to_string());
        // TODO: check if the namespaces are registered in command registry
        Ok(namespaces)
    }

    fn get_command_metadata(&mut self, query: &Query, action_request:&ActionRequest) -> Result<(), Error> {
        let namespaces = self.get_namespaces(query)?;
        let realm = query.last_transform_query_name().unwrap_or("".to_string());
        if let Some(command_metadata) = self.command_registry.find_command_in_namespaces(
            &realm,
            &namespaces,
            &self.action_request.name,
        ) {
            self.command_metadata = command_metadata.clone();
        } else {
            return Err(Error::ActionNotRegistered {
                message: format!(
                    "Action '{}' not registered in namespaces {}",
                    self.action_request.name,
                    namespaces.iter().map(|ns| format!("'{}'",ns)).join(", ")
                ),
            });
        }
        Ok(())
    }

    fn process_resource_query(&mut self, rqs:&ResourceQuerySegment)->Result<(),Error>{
        self.plan.steps.push(Step::GetResource(rqs.key.clone()));
        Ok(())
    }

    fn process_command(&mut self, command_metadata:&CommandMetadata, action_request:&ActionRequest){

    }
    fn process_query(&mut self, query:&Query) -> Result<(), Error> {
        if query.is_empty() {
            return Ok(());
        }
        if let Some(rq) = query.resource_query(){
            self.process_resource_query(&rq)?;
            return Ok(());
        }

        let (p, q) = query.predecessor();

        if let Some(p) = p.as_ref() {
            if !p.is_empty() {
                self.process_query(p)?;
            }
        }

        Ok(())
    }

    fn pop_action_parameter(&mut self, arginfo: &ArgumentInfo) -> Result<Option<String>, Error> {
        match self.action_request.parameters.get(self.parameter_number) {
            Some(ActionParameter::String(v, _)) => {
                self.parameter_number += 1;
                Ok(Some(v.to_owned()))
            }
            Some(ActionParameter::Link(q, _)) => {
                self.resolved_parameters
                    .links
                    .push((self.resolved_parameters.parameters.len(), q.clone()));
                self.parameter_number += 1;
                Ok(None)
            }
            None => match (&arginfo.default_value, &arginfo.default_query) {
                (Some(v), None) => Ok(Some(v.to_owned())),
                (None, Some(q)) => {
                    self.resolved_parameters
                        .links
                        .push((self.resolved_parameters.parameters.len(), q.clone()));
                    Ok(None)
                }
                (None, None) => {
                    if arginfo.optional {
                        Ok(None)
                    } else {
                        Err(Error::missing_argument(
                            self.arginfo_number,
                            &arginfo.name,
                            &self.action_request.position,
                        ))
                    }
                }
                (Some(_), Some(_)) => Err(Error::NotSupported {
                    message: "Default value and default query are not supported".into(),
                }),
            },
        }
    }

    fn pop_value(&mut self, arginfo: &ArgumentInfo) -> Result<V, Error> {
        match (&arginfo.argument_type, self.pop_action_parameter(arginfo)?) {
            (_, None) => Ok(V::none()),
            (ArgumentType::String, Some(x)) => Ok(V::from_string(x)),
            (ArgumentType::Integer, Some(x)) => V::from_i64_str(&x),
            (ArgumentType::IntegerOption, Some(x)) => {
                if x == "" {
                    Ok(V::none())
                } else {
                    V::from_i64_str(&x)
                }
            }
            (ArgumentType::Float, Some(x)) => V::from_f64_str(&x),
            (ArgumentType::FloatOption, Some(x)) => {
                if x == "" {
                    Ok(V::none())
                } else {
                    V::from_f64_str(&x)
                }
            }
            (ArgumentType::Boolean, Some(x)) => V::from_bool_str(&x),
            (ArgumentType::Enum(e), Some(x)) => {
                if let Some(x) = e.name_to_value(&x) {
                    match e.value_type {
                        EnumArgumentType::String => Ok(V::from_string(x)),
                        EnumArgumentType::Integer => V::from_i64_str(&x),
                        EnumArgumentType::IntegerOption => {
                            if x == "" {
                                Ok(V::none())
                            } else {
                                V::from_i64_str(&x)
                            }
                        }
                        EnumArgumentType::Float => V::from_f64_str(&x),
                        EnumArgumentType::FloatOption => {
                            if x == "" {
                                Ok(V::none())
                            } else {
                                V::from_f64_str(&x)
                            }
                        }
                        EnumArgumentType::Boolean => V::from_bool_str(&x),
                    }
                } else {
                    Err(Error::conversion_error(x, &e.name))
                }
            }
            (ArgumentType::Any, Some(x)) => Ok(V::from_string(x)),
            (ArgumentType::None, Some(_)) => Err(Error::NotSupported {
                message: "None not supported".into(),
            }),
        }
    }
    fn get_parameters(&mut self, command_metadata:&CommandMetadata) -> Result<(), Error> {
        self.arginfo_number = 0;
        self.resolved_parameters = ResolvedParameters::new();
        for (i, a) in command_metadata.arguments.iter().enumerate(){
            self.arginfo_number = i;
            let value = self.pop_value(a)?;
            self.resolved_parameters
                .parameters
                .push(value);
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Plan {
    query: Query,
    steps: Vec<Step>,
}

impl Plan {
    pub fn new() -> Self {
        Plan {
            query: Query::new(),
            steps: Vec::new(),
        }
    }
    fn info(&mut self, message: String) {
        self.steps.push(Step::Info(message));
    }
    fn warning(&mut self, message: String) {
        self.steps.push(Step::Warning(message));
    }
    fn error(&mut self, message: String) {
        self.steps.push(Step::Error(message));
    }
    fn has_error(&self) -> bool {
        self.steps.iter().any(|x| x.is_error())
    }
    fn has_warning(&self) -> bool {
        self.steps.iter().any(|x| x.is_warning())
    }
}

impl Display for Plan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Plan for {}:\n{}",
            self.query.encode(),
            self.steps.iter().map(|x| format!("  {x}")).join("\n")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

}
