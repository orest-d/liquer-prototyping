use std::cell::RefCell;

use crate::{
    command_metadata::CommandMetadataRegistry, commands::CommandExecutor, interpreter::Environment, metadata::MetadataRecord, query::Key, store::Store, value::ValueInterface
};

// TODO: Add cache support
/// MasterContext is owns the global data structures/objects that are required to create a [Context].
/// This entails [CommandMetadataRegistry], [CommandExecutor], Cache and [Store].
/// [MasterContext] is not meant to be used directly but rather through a [Context] object,
/// which will have a reference to the [MasterContext].
/// Though [MasterContext] is (kind of) a singleton, there might be multiple instances of them:
///
/// - Each thread may have its own master context. They should be equivalent.
/// - There could be multiple master contexts to support multiple realms.
pub struct MasterContext<I: Environment<V>, V: ValueInterface, CE: CommandExecutor<I, V>, S: Store>
{
    command_metadata_registry: CommandMetadataRegistry,
    command_executor: CE,
    store: S,
    phantom_value: std::marker::PhantomData<V>,
    phantom_injection: std::marker::PhantomData<I>,
}

impl<I: Environment<V>, V: ValueInterface, CE: CommandExecutor<I, V>, S: Store> MasterContext<I,V,CE,S>{
    /// Construct a new [MasterContext]
    pub fn new(cmr: CommandMetadataRegistry, ce: CE, store:S) -> Self {
        MasterContext {
            command_metadata_registry: cmr,
            command_executor: ce,
            store: store,
            phantom_value: std::marker::PhantomData,
            phantom_injection: std::marker::PhantomData,
        }
    }
    pub fn new_context(&self)->Context<I, V, CE, S>{
        Context::new(self)
    }
}

pub struct Context<'c, I: Environment<V>, V: ValueInterface, CE: CommandExecutor<I, V>, S: Store>{
    master_context: &'c MasterContext<I,V,CE,S>,
    metadata: MetadataRecord,
    cwd: Key
}

impl<'c, I: Environment<V>, V: ValueInterface, CE: CommandExecutor<I, V>, S: Store> Context<'c, I, V, CE,S>{
    fn new(master_context:&'c MasterContext<I,V,CE,S>)->Self{
        Context{
            master_context:master_context,
            metadata: MetadataRecord::new(),
            cwd:Key::new()
        }
    }
}