use std::{cell::RefCell, rc::Rc, sync::{Arc, Mutex}};

use crate::{
    command_metadata::CommandMetadataRegistry,
    commands::{CommandExecutor, CommandRegistry},
    error::Error,
    metadata::{self, MetadataRecord},
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
    metadata: Rc<RefCell<MetadataRecord>>
}

impl <'e, E: Environment> Context<'e, E> {
    pub fn new(environment: &'e E) -> Self {
        Context {
            environment,
            metadata: Rc::new(RefCell::new(MetadataRecord::new()))
        }
    }
    pub fn get_environment(&self) -> &'e E {
        self.environment
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
    pub fn get_metadata(&self) -> MetadataRecord {
        self.metadata.borrow().clone()
    }
    pub fn set_filename(&mut self, filename: String) {
        self.metadata.borrow_mut().with_filename(filename);
    }
    pub fn debug(&mut self, message:&str){
        self.metadata.borrow_mut().debug(message);
    }
    pub fn info(&mut self, message:&str){
        self.metadata.borrow_mut().info(message);
    }
    pub fn warning(&mut self, message:&str){
        self.metadata.borrow_mut().warning(message);
    }
    pub fn error(&mut self, message:&str){
        self.metadata.borrow_mut().error(message);
    }
    pub fn reset(&mut self){
        self.metadata = Rc::new(RefCell::new(MetadataRecord::new()));
    }
}

impl<'e, E:Environment> Clone for Context<'e, E>{
    fn clone(&self) -> Self {
        Context {
            environment: self.environment,
            metadata: self.metadata.clone()
        }
    }
}

#[derive(Clone)]
pub struct ActionContext{
    metadata: Rc<RefCell<MetadataRecord>>,
    store: Arc<Mutex<Box<dyn Store>>>
}

impl ActionContext{
    pub fn new(metadata: Rc<RefCell<MetadataRecord>>, store: Arc<Mutex<Box<dyn Store>>>) -> Self {
        ActionContext {
            metadata,
            store
        }
    }
    pub fn get_metadata(&self) -> MetadataRecord {
        self.metadata.borrow().clone()
    }
    pub fn set_filename(&mut self, filename: String) {
        self.metadata.borrow_mut().with_filename(filename);
    }
    pub fn debug(&mut self, message:&str){
        self.metadata.borrow_mut().debug(message);
    }
    pub fn info(&mut self, message:&str){
        self.metadata.borrow_mut().info(message);
    }
    pub fn warning(&mut self, message:&str){
        self.metadata.borrow_mut().warning(message);
    }
    pub fn error(&mut self, message:&str){
        self.metadata.borrow_mut().error(message);
    }
    pub fn reset(&mut self){
        self.metadata = Rc::new(RefCell::new(MetadataRecord::new()));
    }
    pub fn get_store(&self) -> Arc<Mutex<Box<dyn Store>>> {
        self.store.clone()
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

