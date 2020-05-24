
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ActionParameter{
    String(String),
    Link(String)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ActionRequest{
    pub name:String,
    pub parameters: Vec<ActionParameter>
}

