use std::fmt::Display;

use itertools::Itertools;
use nom::Err;

use crate::command_registry::{
    self, ArgumentInfo, ArgumentType, CommandMetadata, CommandRegistry, EnumArgumentType,
};
use crate::error::Error;
use crate::query::{ActionParameter, ActionRequest, Key, Query, QuerySegment, ResourceName};
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
        }
    }

    fn build(mut self) -> Result<Plan, Error> {
        let mut plan = Plan {
            query: self.query.clone(),
            steps: Vec::new(),
        };
        for (q, qs) in self.query.all_predecessor_tuples() {
            if qs.is_empty() {
                continue;
            }
            if let Some(filename) = qs.filename() {
                plan.steps.push(Step::Filename(filename));
                continue;
            }
            if let Some(action) = qs.action().as_ref() {
                self.action_request = action.clone();
                self.arginfo_number = 0;
                self.parameter_number = 0;
                let realm = qs.name();
                let mut namespaces = if let Some(ns) = q.last_ns() {
                    ns.iter()
                        .filter_map(|x| match x {
                            ActionParameter::String(s, _) => Some(s.clone()),
                            _ => None,
                        })
                        .collect::<Vec<String>>()
                } else {
                    vec![]
                };
                namespaces.push("".to_string());
                namespaces.push("root".to_string());

                if let Some(command_metadata) = self.command_registry.find_command_in_namespaces(
                    &realm,
                    &namespaces,
                    &action.name,
                ) {
                    self.command_metadata = command_metadata.clone();
                } else {
                    return Err(Error::ActionNotRegistered {
                        message: format!(
                            "Action '{}' not registered in namespaces {}",
                            action.name, namespaces.iter().map(|ns| format!("'{}'",ns)).join(", ")
                        ),
                    });
                }
            }
        }
        Ok(plan)
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
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Plan {
    query: Query,
    steps: Vec<Step>,
}

impl Plan {
    fn from(query: &Query) -> Plan {
        let mut plan = Plan {
            query: query.clone(),
            steps: vec![],
        };
        let (p, r) = query.predecessor();
        match r {
            Some(QuerySegment::Resource(res)) => {
                if let Some(p) = p.as_ref() {
                    if !p.is_empty() {
                        plan.warning(format!(
                            "Query '{}' before resource at {} is ignored",
                            p.encode(),
                            res.position()
                        ));
                    }
                }
                if let Some(head) = res.header {
                    if !head.name.is_empty() {
                        plan.warning(format!(
                            "Resource segment name '{}' at {} is ignored",
                            head.name, head.position
                        ));
                    }
                    if head.parameters.is_empty() {
                        plan.steps.push(Step::GetResource(res.key.clone()));
                    } else {
                        if head.parameters[0].value == "meta" {
                            if head.parameters.len() > 1 {
                                plan.warning(format!(
                                    "Resource segment '{}...' parameters after meta at {} ignored",
                                    head.encode(),
                                    head.parameters[2]
                                ));
                            }
                            plan.steps.push(Step::GetResourceMetadata(res.key.clone()));
                        } else {
                            plan.warning(format!(
                                "Resource segment '{}...' parameters at {} ignored",
                                head.encode(),
                                head.parameters[0]
                            ));
                            plan.steps.push(Step::GetResource(res.key.clone()));
                        }
                    }
                } else {
                    plan.steps.push(Step::GetResource(res.key.clone()));
                }
            }
            Some(QuerySegment::Transform(tqs)) => {
                if let Some(p) = p.as_ref() {
                    if !p.is_empty() {
                        plan.steps.push(Step::Evaluate(p.clone()));
                    }
                }
                if let Some(action) = tqs.action() {
                    let ns = p.and_then(|x| x.last_ns());
                    if let Some(ns) = ns.as_ref() {
                        for par in ns.iter() {
                            if !par.is_string() {
                                plan.error(format!(
                                    "Unsuported namespace {} at {}",
                                    par.encode(),
                                    par.position()
                                ));
                            }
                        }
                    }
                    plan.steps.push(Step::ApplyAction {
                        ns,
                        action: action.clone(),
                    });
                } else {
                    if tqs.is_filename() {
                        plan.steps
                            .push(Step::Filename(tqs.filename.unwrap().clone()));
                    } else {
                        plan.error(format!("Unrecognized remainder {:?}", tqs));
                    }
                }
            }
            None => {
                if let Some(p) = p {
                    if !p.is_empty() {
                        plan.steps.push(Step::Evaluate(p.clone()));
                    }
                } else {
                    plan.warning(format!("Empty remainder"));
                }
            }
        }
        plan
    }
    fn expand_evaluate(&mut self) -> bool {
        for i in 0..self.steps.len() {
            if let Step::Evaluate(query) = &self.steps[i] {
                let mut plan = Plan::from(query);
                self.steps.remove(i);
                let mut i = i;
                for x in plan.steps.drain(..) {
                    self.steps.insert(i, x);
                    i += 1;
                }
                return true;
            }
        }
        false
    }
    fn expand(&mut self) {
        while self.expand_evaluate() {}
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

    #[test]
    fn test_plan() -> Result<(), Box<dyn std::error::Error>> {
        let query = crate::parse::parse_query("query")?;
        let plan = Plan::from(&query);
        assert!(!plan.has_error());
        assert!(!plan.has_warning());
        assert_eq!(plan.steps.len(), 1);
        if let Step::ApplyAction { ns, action } = &plan.steps[0] {
            assert!(ns.is_none());
            assert_eq!(action.name, "query");
        } else {
            assert!(false);
        }
        Ok(())
    }
    #[test]
    fn test_plan_expand_evaluate() -> Result<(), Box<dyn std::error::Error>> {
        let query = crate::parse::parse_query("a/b/c")?;
        let mut plan = Plan::from(&query);
        let p1 = format!("{}", &plan);
        assert_eq!(
            p1,
            r#"Plan for a/b/c:
  EVALUATE        a/b
  APPLY ACTION    (root): c"#
        );
        assert!(plan.expand_evaluate());
        let p2 = format!("{}", &plan);
        assert_eq!(
            p2,
            r#"Plan for a/b/c:
  EVALUATE        a
  APPLY ACTION    (root): b
  APPLY ACTION    (root): c"#
        );
        Ok(())
    }
    #[test]
    fn test_plan_expand() -> Result<(), Box<dyn std::error::Error>> {
        let query = crate::parse::parse_query("a/b/c")?;
        let mut plan = Plan::from(&query);
        plan.expand();
        let p = format!("{}", &plan);
        assert_eq!(
            p,
            r#"Plan for a/b/c:
  APPLY ACTION    (root): a
  APPLY ACTION    (root): b
  APPLY ACTION    (root): c"#
        );
        println!("{}", &plan);
        Ok(())
    }
    #[test]
    fn test_plan_res_expand() -> Result<(), Box<dyn std::error::Error>> {
        let query = crate::parse::parse_query("-R/a/b/-/c/d")?;
        let mut plan = Plan::from(&query);
        plan.expand();
        let p = format!("{}", &plan);
        println!("{}", p);
        assert_eq!(
            p,
            r#"Plan for -R/a/b/-/c/d:
  GET RES         a/b
  APPLY ACTION    (root): c
  APPLY ACTION    (root): d"#
        );
        Ok(())
    }
    #[test]
    fn test_plan_res_expand1() -> Result<(), Box<dyn std::error::Error>> {
        let query = crate::parse::parse_query("a/b/-/c/d")?;
        let mut plan = Plan::from(&query);
        plan.expand();
        let p = format!("{}", &plan);
        println!("{}", p);
        assert_eq!(
            p,
            r#"Plan for a/b/-/c/d:
  GET RES         a/b
  APPLY ACTION    (root): c
  APPLY ACTION    (root): d"#
        );
        Ok(())
    }
}
