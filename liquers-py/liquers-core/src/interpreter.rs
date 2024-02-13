use crate::command_metadata::CommandMetadataRegistry;
use crate::commands::{CommandArguments, CommandExecutor};
use crate::error::Error;
use crate::metadata::MetadataRecord;
use crate::parse::parse_query;
use crate::plan::{Plan, PlanBuilder};
use crate::state::State;
use crate::value::ValueInterface;
use crate::query::Query;

pub trait Environment<V: ValueInterface> {
    fn evaluate(&mut self, _query: &Query) -> Result<State<V>, Error>{
        Err(Error::not_supported("evaluate not implemented".to_string()))
    }
}

pub struct PlanInterpreter<I:Environment<V>, V: ValueInterface, CE: CommandExecutor<I, V>> {
    plan: Option<Plan>,
    command_metadata_registry: CommandMetadataRegistry,
    command_executor: CE,
    injection: I,
    step_number: usize,
    metadata: Option<MetadataRecord>,
    state: Option<State<V>>,
    phantom_value: std::marker::PhantomData<V>,
}

impl<I:Environment<V>, V: ValueInterface, CE: CommandExecutor<I, V>> PlanInterpreter<I, V, CE> {
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
        println!("with plan {:?}", plan);
        self.plan = Some(plan);
        self.step_number = 0;
        self.metadata.replace(MetadataRecord::new());
        self
    }

    pub fn with_query(&mut self, query: &str) -> Result<&mut Self, Error> {
        let query = parse_query(query)?;
        println!("Query: {}", query);
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
                            CommandArguments::new(parameters.clone(), &mut self.injection);
                        arguments.action_position = position.clone();
                        let input_state = self.state.take().unwrap_or(State::new());
                        let result = self.command_executor.execute(
                            &realm,
                            ns,
                            &action_name,
                            &input_state,
                            &mut arguments,
                        )?;
                        let state = State::new().with_data(result).with_metadata(
                            self.metadata.take().unwrap_or(MetadataRecord::new()).into(),
                        );
                        //TODO: Make sure metadata is correctly filled - now empty metadata is created.
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
    use std::cell::RefCell;
    use std::rc::Rc;

    use super::*;
    use crate::command_metadata::ArgumentInfo;
    use crate::command_metadata::CommandMetadata;
    use crate::command_metadata::CommandMetadataRegistry;
    use crate::commands::*;
    use crate::value::{Value, ValueInterface};
    struct TestExecutor;

    #[derive(Clone)]
    struct InjectedVariable(String);
    struct InjectionTest {
        variable: InjectedVariable,
    }

    impl Environment<Value> for InjectionTest{

    }

    impl Environment<Value> for NoInjection{

    }

    struct MutableInjectionTest {
        variable: Rc<RefCell<InjectedVariable>>,
    }

    impl Environment<Value> for MutableInjectionTest{

    }

    impl CommandExecutor<NoInjection, Value> for TestExecutor {
        fn execute(
            &self,
            realm: &str,
            namespace: &str,
            command_name: &str,
            state: &State<Value>,
            arguments: &mut CommandArguments<'_, NoInjection>,
        ) -> Result<Value, Error> {
            assert_eq!(realm, "");
            assert_eq!(namespace, "root");
            assert_eq!(command_name, "test");
            Command0::from(|| -> String { "Hello".into() }).execute(state, arguments)
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
        let mut cr: CommandRegistry<NoInjection, Value> = CommandRegistry::new();

        cr.register_command("hello", Command0::from(|| "Hello".to_string()))?;
        cr.register_command(
            "greet",
            Command2::from(|state: &State<Value>, who: String| -> String {
                let greeting = state.data.try_into_string().unwrap();
                format!("{} {}!", greeting, who)
            }),
        )?
        .with_state_argument(ArgumentInfo::string_argument("greeting"))
        .with_argument(ArgumentInfo::string_argument("who"));

        let cmr = cr.command_metadata_registry.clone();

        let mut pi = PlanInterpreter::new(cmr, NoInjection, cr);
        pi.with_query("hello/greet-world").unwrap();
        //println!("{:?}", pi.plan);
        println!(
            "{}",
            serde_yaml::to_string(pi.plan.as_ref().unwrap()).unwrap()
        );
        pi.step()?;
        assert_eq!(pi.state.as_ref().unwrap().data.try_into_string()?, "Hello");
        pi.step()?;
        assert_eq!(
            pi.state.as_ref().unwrap().data.try_into_string()?,
            "Hello world!"
        );
        Ok(())
    }

    #[test]
    fn test_interpreter_with_value_injection() -> Result<(), Error> {
        let mut cr: CommandRegistry<InjectionTest, Value> = CommandRegistry::new();
        impl FromCommandArguments<InjectedVariable, InjectionTest> for InjectedVariable {
            fn from_arguments(
                args: &mut CommandArguments<'_, InjectionTest>,
            ) -> Result<InjectedVariable, Error> {
                Ok(args.injection.variable.to_owned())
            }

            fn is_injected() -> bool {
                true
            }
        }

        cr.register_command(
            "injected",
            Command2::from(|_state: &State<Value>, what: InjectedVariable| {
                format!("Hello {}", what.0)
            }),
        )?
        .with_state_argument(ArgumentInfo::string_argument("nothing"));

        let cmr = cr.command_metadata_registry.clone();

        let mut pi = PlanInterpreter::new(
            cmr,
            InjectionTest {
                variable: InjectedVariable("injected string".to_string()),
            },
            cr,
        );
        pi.with_query("injected")?;
        println!(
            "{}",
            serde_yaml::to_string(pi.plan.as_ref().unwrap()).unwrap()
        );
        pi.step()?;
        assert_eq!(
            pi.state.as_ref().unwrap().data.try_into_string()?,
            "Hello injected string"
        );
        Ok(())
    }
    #[test]
    fn test_interpreter_with_mutable_injection() -> Result<(), Error> {
        let mut cr: CommandRegistry<MutableInjectionTest, Value> = CommandRegistry::new();
        impl<'v> FromCommandArguments<Rc<RefCell<InjectedVariable>>, MutableInjectionTest>
            for Rc<RefCell<InjectedVariable>>
        {
            fn from_arguments<'i>(
                args: &mut CommandArguments<'i, MutableInjectionTest>,
            ) -> Result<Rc<RefCell<InjectedVariable>>, Error> {
                Ok(args.injection.variable.clone())
            }

            fn is_injected() -> bool {
                true
            }
        }

        cr.register_command(
            "injected",
            Command2::from(
                |_state: &State<Value>, what: Rc<RefCell<InjectedVariable>>| {
                    let res = format!("Hello {}", what.borrow().0);
                    what.borrow_mut().0 = "changed".to_owned();
                    res
                },
            ),
        )?
        .with_state_argument(ArgumentInfo::string_argument("nothing"));

        let cmr = cr.command_metadata_registry.clone();
        let injection = MutableInjectionTest {
            variable: Rc::new(RefCell::new(InjectedVariable(
                "injected string".to_string(),
            ))),
        };
        let mut pi = PlanInterpreter::new(cmr, injection, cr);
        pi.with_query("injected")?;
        println!(
            "{}",
            serde_yaml::to_string(pi.plan.as_ref().unwrap()).unwrap()
        );
        pi.step()?;
        assert_eq!(
            pi.state.as_ref().unwrap().data.try_into_string()?,
            "Hello injected string"
        );
        assert_eq!(pi.injection.variable.borrow().0, "changed");
        Ok(())
    }
}
