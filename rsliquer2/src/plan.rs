use crate::query::{Query, ActionRequest, ResourceName, QuerySegment};

pub enum Step{
    Get(String),
    GetRes(Vec<ResourceName>),
    GetMeta(String),
    GetResMeta(Vec<ResourceName>),
    Evaluate(Query),
    ApplyAction(ActionRequest),
    Filename(ResourceName),
    Info(String),
    Warning(String),
    Error(String)
}


pub struct Plan{
    query:Query,
    steps:Vec<Step>
}

impl Plan {
    fn from(query:&Query) -> Plan {
        let mut plan = Plan{query:query.clone(), steps:vec![]};        
        let (p,r)=query.predecessor();
        match r{
            Some(QuerySegment::Resource(res)) => {
                if let Some(p) = p{
                    if !p.is_empty() {
                        plan.warning(format!("Query '{}' before resource at {} is ignored", p.encode(), res.position()));
                    }
                }
                if let Some(head) = res.header{
                    if !head.name.is_empty() {
                        plan.warning(format!("Resource segment name '{}' at {} is ignored", head.name, head.position));
                    }
                    if head.parameters.is_empty(){
                        plan.steps.push(Step::GetRes(res.query.clone()));
                    }
                    else{
                        if head.parameters[0].value == "meta"{
                            if head.parameters.len()>1{
                                plan.warning(format!("Resource segment '{}...' parameters after meta at {} ignored", head.encode(), head.parameters[2]));
                            }
                            plan.steps.push(Step::GetResMeta(res.query.clone()));
                        }
                        else {
                            plan.warning(format!("Resource segment '{}...' parameters at {} ignored", head.encode(), head.parameters[0]));
                            plan.steps.push(Step::GetRes(res.query.clone()));
                        }
                    }
                }
                else{
                    plan.steps.push(Step::GetRes(res.query.clone()));
                }
            },
            Some(QuerySegment::Transform(tqs)) => {
                if let Some(p) = p{
                    if !p.is_empty() {
                        plan.steps.push(Step::Evaluate(p.clone()));
                    }
                }
                if let Some(action) = tqs.action(){
                    plan.steps.push(Step::ApplyAction(action.clone()));   
                }
                else{
                    if tqs.is_filename(){
                        plan.steps.push(Step::Filename(tqs.filename.unwrap().clone()));
                    }
                    else{
                        plan.error(format!("Unrecognized remainder {:?}",tqs));
                    }
                }

            }
            None => {
                if let Some(p) = p{
                    if !p.is_empty() {
                        plan.steps.push(Step::Evaluate(p.clone()));
                    }
                }
                plan.warning(format!("Empty remainder"));
            }
        }
        plan
    }
    fn info(&mut self, message:String){
        self.steps.push(Step::Info(message));
    }
    fn warning(&mut self, message:String){
        self.steps.push(Step::Warning(message));
    }
    fn error(&mut self, message:String){
        self.steps.push(Step::Error(message));
    }
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_step(){
        let a:Step = Step::Get("abc".to_owned());
    }

    #[test]
    fn test_plan()-> Result<(), Box<dyn std::error::Error>>{
        let query = crate::parse::parse_query("query")?;
        let plan = Plan::from(&query);
        Ok(())
    }
}