use crate::{
    command_metadata::CommandMetadataRegistry,
    commands::{CommandExecutor, CommandRegistry},
    error::Error,
    metadata::MetadataRecord,
    query::{Key, Query},
    state::State,
    store::Store,
    value::ValueInterface,
};

pub trait Environment: Sized{
    type Value: ValueInterface;
    type CommandExecutor: CommandExecutor<Self, Self::Value>;

    fn evaluate(&mut self, _query: &Query) -> Result<State<Self::Value>, Error> {
        Err(Error::not_supported("evaluate not implemented".to_string()))
    }
    fn get_command_metadata_registry(&self) -> &CommandMetadataRegistry;
    fn get_mut_command_metadata_registry(&mut self) -> &mut CommandMetadataRegistry;
    fn get_command_executor(&self) -> &Self::CommandExecutor;
    fn get_mut_command_executor(&mut self) -> &mut Self::CommandExecutor;
}

pub struct SimpleEnvironment<V: ValueInterface> {
    command_registry: CommandRegistry<Self,V>
}

impl<V: ValueInterface> SimpleEnvironment<V> {
    pub fn new() -> Self {
        SimpleEnvironment {
            command_registry: CommandRegistry::new()
        }
    }
}

impl<V: ValueInterface> Environment for SimpleEnvironment<V> {
    type Value=V;
    type CommandExecutor=CommandRegistry<Self,V>;

    fn get_mut_command_metadata_registry(&mut self) -> &mut CommandMetadataRegistry {
        &mut self.command_registry.command_metadata_registry
    }

    fn get_command_metadata_registry(&self) -> &CommandMetadataRegistry {
        &self.command_registry.command_metadata_registry
    }

    fn get_command_executor(&self) -> &Self::CommandExecutor {
        &self.command_registry
    }
    fn get_mut_command_executor(&mut self) -> &mut Self::CommandExecutor {
        &mut self.command_registry
    }
}

// TODO: Add cache support
/// MasterContext is owns the global data structures/objects that are required to create a [Context].
/// This entails [CommandMetadataRegistry], [CommandExecutor], Cache and [Store].
/// [MasterContext] is not meant to be used directly but rather through a [Context] object,
/// which will have a reference to the [MasterContext].
/// Though [MasterContext] is (kind of) a singleton, there might be multiple instances of them:
///
/// - Each thread may have its own master context. They should be equivalent.
/// - There could be multiple master contexts to support multiple realms.
pub struct MasterContext<I, V: ValueInterface, CE: CommandExecutor<I, V>, S: Store> {
    command_metadata_registry: CommandMetadataRegistry,
    command_executor: CE,
    store: S,
    phantom_value: std::marker::PhantomData<V>,
    phantom_injection: std::marker::PhantomData<I>,
}

impl<I, V: ValueInterface, CE: CommandExecutor<I, V>, S: Store> MasterContext<I, V, CE, S> {
    /// Construct a new [MasterContext]
    pub fn new(cmr: CommandMetadataRegistry, ce: CE, store: S) -> Self {
        MasterContext {
            command_metadata_registry: cmr,
            command_executor: ce,
            store: store,
            phantom_value: std::marker::PhantomData,
            phantom_injection: std::marker::PhantomData,
        }
    }
}

pub struct Context<'c, I: Environment, V: ValueInterface, CE: CommandExecutor<I, V>, S: Store> {
    master_context: &'c MasterContext<I, V, CE, S>,
    metadata: MetadataRecord,
    cwd: Key,
}

impl<'c, I: Environment, V: ValueInterface, CE: CommandExecutor<I, V>, S: Store>
    Context<'c, I, V, CE, S>
{
    pub fn new(master_context: &'c MasterContext<I, V, CE, S>) -> Self {
        Context {
            master_context: master_context,
            metadata: MetadataRecord::new(),
            cwd: Key::new(),
        }
    }
    pub fn get_store(&self) -> &S {
        &self.master_context.store
    }
    pub fn get_command_executor(&self) -> &CE {
        &self.master_context.command_executor
    }
    pub fn get_command_metadata_registry(&self) -> &CommandMetadataRegistry {
        &self.master_context.command_metadata_registry
    }
    pub fn new_context(&self) -> Context<I, V, CE, S> {
        Context::new(self.master_context)
    }
}
