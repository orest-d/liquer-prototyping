use std::{sync::{Arc, Mutex}};

use crate::{
    command_metadata::CommandMetadataRegistry,
    commands::{CommandExecutor, CommandRegistry},
    error::Error,
    metadata::MetadataRecord,
    query::{Key, Query},
    state::State,
    store::{NoStore, Store},
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
    fn get_store(&self) -> Arc<Mutex<Box<dyn Store>>>;
    fn new_context(&self) -> Context<Self> {
        Context::new(self)
    }
}

pub struct Context<'e, E: Environment> {
    environment: &'e E,
    metadata: MetadataRecord
}

impl <'e, E: Environment> Context<'e, E> {
    pub fn new(environment: &'e E) -> Self {
        Context {
            environment,
            metadata: MetadataRecord::new()
        }
    }
    pub fn get_command_metadata_registry(&self) -> &CommandMetadataRegistry {
        self.environment.get_command_metadata_registry()
    }
    pub fn get_command_executor(&self) -> &E::CommandExecutor {
        self.environment.get_command_executor()
    }
    pub fn get_store(&self) -> Arc<Mutex<Box<dyn Store>>> {
        self.environment.get_store()
    }
    pub fn get_metadata(&self) -> &MetadataRecord {
        &self.metadata
    }
    pub fn set_filename(&mut self, filename: String) {
        self.metadata.with_filename(filename);
    }
    pub fn debug(&mut self, message:&str){
        &self.metadata.debug(message);
    }
    pub fn info(&mut self, message:&str){
        &self.metadata.info(message);
    }
    pub fn warning(&mut self, message:&str){
        &self.metadata.warning(message);
    }
    pub fn error(&mut self, message:&str){
        &self.metadata.error(message);
    }
    pub fn reset(&mut self){
        self.metadata = MetadataRecord::new();
    }
}

pub struct SimpleEnvironment<V: ValueInterface> {
    store: Arc<Mutex<Box<dyn Store>>>,
    command_registry: CommandRegistry<Self,V>
}

impl<V: ValueInterface> SimpleEnvironment<V> {
    pub fn new() -> Self {
        SimpleEnvironment {
            store: Arc::new(Mutex::new(Box::new(NoStore))),
            command_registry: CommandRegistry::new()
        }
    }
    pub fn with_store(&mut self, store: Box<dyn Store>) -> &mut Self {
        self.store = Arc::new(Mutex::new(store));
        self
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
    fn get_store(&self) -> Arc<Mutex<Box<dyn Store>>>{
        self.store.clone()
    }
}

