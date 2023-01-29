use crate::query::{Query, ActionRequest};

pub enum Step{
    Get(String),
    GetMeta(String),
    Evaluate(ActionRequest)
}


pub struct Plan{
    query:Query,
    steps:Vec<Step>
}
#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_step(){
        let a:Step = Step::Get("abc".to_owned());
    }
}