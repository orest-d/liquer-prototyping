use std::fmt::Display;

use itertools::Itertools;

use crate::query::{ActionParameter, ActionRequest, Query, QuerySegment, ResourceName};

#[derive(Serialize, Deserialize, Debug, Clone)]

pub enum Step {
    Get(String),
    GetRes(Vec<ResourceName>),
    GetMeta(String),
    GetResMeta(Vec<ResourceName>),
    Evaluate(Query),
    ApplyAction {
        ns: Option<Vec<ActionParameter>>,
        action: ActionRequest,
    },
    Filename(ResourceName),
    Info(String),
    Warning(String),
    Error(String),
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
            Step::Get(s) => write!(f, "GET             {s}"),
            Step::GetRes(s) => write!(
                f,
                "GET RES         {}",
                s.iter().map(|x| x.encode()).join("/")
            ),
            Step::GetMeta(s) => write!(f, "GET META        {s}"),
            Step::GetResMeta(s) => {
                write!(
                    f,
                    "GET RES META    {}",
                    s.iter().map(|x| x.encode()).join("/")
                )
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
                        plan.steps.push(Step::GetRes(res.query.clone()));
                    } else {
                        if head.parameters[0].value == "meta" {
                            if head.parameters.len() > 1 {
                                plan.warning(format!(
                                    "Resource segment '{}...' parameters after meta at {} ignored",
                                    head.encode(),
                                    head.parameters[2]
                                ));
                            }
                            plan.steps.push(Step::GetResMeta(res.query.clone()));
                        } else {
                            plan.warning(format!(
                                "Resource segment '{}...' parameters at {} ignored",
                                head.encode(),
                                head.parameters[0]
                            ));
                            plan.steps.push(Step::GetRes(res.query.clone()));
                        }
                    }
                } else {
                    plan.steps.push(Step::GetRes(res.query.clone()));
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
                }
                plan.warning(format!("Empty remainder"));
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
    fn test_step() {
        let _a: Step = Step::Get("abc".to_owned());
    }

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
