use std::cell::RefCell;

use crate::{
    command_metadata::CommandMetadataRegistry, commands::CommandExecutor, interpreter::Environment, metadata::MetadataRecord, query::Key, store::Store, value::ValueInterface
};

pub struct MasterContext<I: Environment<V>, V: ValueInterface, CE: CommandExecutor<I, V>, S: Store>
{
    command_metadata_registry: CommandMetadataRegistry,
    command_executor: CE,
    store: S,
    phantom_value: std::marker::PhantomData<V>,
    phantom_injection: std::marker::PhantomData<I>,
}

impl<I: Environment<V>, V: ValueInterface, CE: CommandExecutor<I, V>, S: Store> MasterContext<I,V,CE,S>{
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