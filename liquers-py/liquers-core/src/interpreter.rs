use std::fs::Metadata;

use crate::command_metadata::CommandMetadataRegistry;
use crate::commands::{CommandArguments, CommandExecutor};
use crate::error::Error;
use crate::metadata::MetadataRecord;
use crate::parse::parse_query;
use crate::plan::{Plan, PlanBuilder};
use crate::state::{self, State};
use crate::value::ValueInterface;

pub struct PlanInterpreter<I, V: ValueInterface, CE: CommandExecutor<I, V>> {
    plan: Option<Plan>,
    command_metadata_registry: CommandMetadataRegistry,
    command_executor: CE,
    injection: I,
    step_number: usize,
    metadata: Option<MetadataRecord>,
    state: Option<State<V>>,
    phantom_value: std::marker::PhantomData<V>,
}

impl<I, V: ValueInterface, CE: CommandExecutor<I, V>> PlanInterpreter<I, V, CE> {
    pub fn new(cmr: CommandMetadataRegistry, injection: I, ce: CE) -> Self {
        PlanInterpreter {
            plan: None,
            command_metadata_registry: cmr,
            injection: injection,
            step_number: 0,
            metadata: None,
            command_executor: ce,
            state: None,
            phantom_value: std::marker::PhantomData,
        }
    }
    pub fn with_plan(&mut self, plan: Plan) -> &mut Self {
        self.plan = Some(plan);
        self.step_number = 0;
        self.metadata.replace(MetadataRecord::new());
        self
    }

    pub fn with_query(&mut self, query: &str) -> Result<&mut Self, Error> {
        let query = parse_query(query)?;
        let mut pb = PlanBuilder::new(query, &self.command_metadata_registry);
        let plan = pb.build()?;
        Ok(self.with_plan(plan))
    }
    pub fn step(&mut self) -> Result<(), Error> {
        if let Some(plan) = &mut self.plan {
            if let Some(step) = plan.steps.get(self.step_number) {
                match step {
                    crate::plan::Step::GetResource(_) => todo!(),
                    crate::plan::Step::GetResourceMetadata(_) => todo!(),
                    crate::plan::Step::GetNamedResource(_) => todo!(),
                    crate::plan::Step::GetNamedResourceMetadata(_) => todo!(),
                    crate::plan::Step::Evaluate(_) => todo!(),
                    crate::plan::Step::Action {
                        realm,
                        ns,
                        action_name,
                        position,
                        parameters,
                    } => {
                        let mut arguments =
                            CommandArguments::new(parameters.clone(), &self.injection);
                        arguments.action_position = position.clone();
                        let input_state = self.state.take().unwrap_or(State::new());
                        let result = self.command_executor.execute(
                            &realm,
                            ns,
                            &action_name,
                            &input_state,
                            &mut arguments,
                        )?;
                        let state = State::new()
                            .with_data(result)
                            .with_metadata(self.metadata.take().unwrap().into());
                        self.state.replace(state);
                    }
                    crate::plan::Step::Filename(name) => {
                        self.metadata
                            .as_mut()
                            .unwrap()
                            .with_filename(name.name.clone());
                    }
                    crate::plan::Step::Info(m) => {
                        self.metadata.as_mut().unwrap().info(&m);
                    }
                    crate::plan::Step::Warning(m) => {
                        self.metadata.as_mut().unwrap().warning(&m);
                    }
                    crate::plan::Step::Error(m) => {
                        self.metadata.as_mut().unwrap().error(&m);
                    }
                    crate::plan::Step::Plan(_) => todo!(),
                }
                self.step_number += 1;
            } else {
                return Err(Error::general_error("No more steps".to_string()));
            }
        } else {
            return Err(Error::general_error("No plan".to_string()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::command_metadata::ArgumentInfo;
    use crate::command_metadata::CommandKey;
    use crate::command_metadata::CommandMetadata;
    use crate::command_metadata::CommandMetadataRegistry;
    use crate::commands::*;
    use crate::value::Value;
    struct TestExecutor;
    impl CommandExecutor<NoInjection, Value> for TestExecutor {
        fn execute(
            &mut self,
            realm: &str,
            namespace: &str,
            command_name: &str,
            state: &State<Value>,
            arguments: &mut CommandArguments<'_, NoInjection>,
        ) -> Result<Value, Error> {
            assert_eq!(realm, "");
            assert_eq!(namespace, "root");
            assert_eq!(command_name, "test");
            (|| -> String { "Hello".into() }).execute(state, arguments)
        }
    }
    #[test]
    fn test_plan_interpreter() -> Result<(), Error> {
        let mut cmr = CommandMetadataRegistry::new();
        cmr.add_command(&CommandMetadata::new("test"));
        let ce = TestExecutor;
        let mut pi = PlanInterpreter::new(cmr, NoInjection, ce);
        pi.with_query("test").unwrap();
        //println!("{:?}", pi.plan);
        pi.step()?;
        assert_eq!(pi.state.as_ref().unwrap().data.try_into_string()?, "Hello");
        Ok(())
    }
    #[test]
    fn test_hello_world_interpreter() -> Result<(), Error> {
        let mut cmr = CommandMetadataRegistry::new();
        cmr.add_command(&CommandMetadata::new("hello"));
        cmr.add_command(&CommandMetadata::new("greet")
        .with_state_argument(ArgumentInfo::string_argument("greeting"))
        .with_argument(ArgumentInfo::string_argument("who"))
        );

        let mut ce = HashMap::<CommandKey, Box<dyn Command<NoInjection, Value>>>::new();
        let f = || { "Hello".to_string()};
        let f1:Box<dyn Command::<NoInjection, Value>> = Box::new(f);
        ce.insert(CommandKey::new_name("hello"), f1);

        let f =|state:State<Value>, who:&str| -> String {
            let greeting = state.data.try_into_string().unwrap();
            format!("{} {}!", greeting, who)
        };

//        let f2:Box<dyn Command::<NoInjection,Value>>=Box::new(
//            Command::<NoInjection,Value>::from(f)
//        );
//        ce.insert(CommandKey::new_name("greet"), f2);

        let mut pi = PlanInterpreter::new(cmr, NoInjection, ce);
        pi.with_query("hello/greeting-world").unwrap();
        //println!("{:?}", pi.plan);
        pi.step()?;
        assert_eq!(pi.state.as_ref().unwrap().data.try_into_string()?, "Hello");
        pi.step()?;
        assert_eq!(pi.state.as_ref().unwrap().data.try_into_string()?, "Hello, world!");
        Ok(())
    }
}
