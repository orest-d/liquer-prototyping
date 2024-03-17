use leptos::*;
use liquers_core::{command_metadata::ArgumentInfo, commands::Command2, interpreter::PlanInterpreter, metadata::Metadata, parse::{self, parse_key}, query::Key, state::State, store::MemoryStore};
use std::sync::Mutex;
use liquers_core::context::*;
use liquers_core::value::ValueInterface;

use polars::prelude::*;

mod value;
use value::*;

#[component]
fn Hello() -> impl IntoView {
    let body = create_resource(|| (), |_| async move {
        log::info!("fetching...");
        let resp = reqwest::get("http://127.0.0.1:8080/api/test.txt").await.unwrap();
        let text = resp.text().await.unwrap();
        log::info!("fetched: {}", text);
        let envref = use_context::<ReadSignal<LocalEnvRef>>().expect("No EvvRef");
        envref.with_untracked(|e| {
            let mut store = e.get_store().clone();
            store.lock().unwrap().set(&parse_key("test.txt").unwrap(), text.as_bytes(), &Metadata::new());
            log::info!("stored");
        });

        log::info!("fetching CSV...");
        let resp = reqwest::get("http://127.0.0.1:8080/api/test.csv").await.unwrap();
        let csv_text = resp.text().await.unwrap();
        log::info!("fetched CSV: {}", csv_text);
        envref.with_untracked(|e| {
            let mut store = e.get_store().clone();
            store.lock().unwrap().set(&parse_key("test.csv").unwrap(), csv_text.as_bytes(), &Metadata::new());
            log::info!("stored CSV");
        });

        text 
    });

    view! { <p>"Hello, world!"</p><p>{body}</p> }
}

#[component]
fn Interpreter(query:String) -> impl IntoView {
    let (result, set_result) = create_signal("No result".to_string());
    view!{
        <button on:click={move |_| {
            let envref = use_context::<ReadSignal<LocalEnvRef>>().expect("No EvvRef");
            log::info!("Evaluate");
            //set_result("Fake result".to_string());
            envref.with_untracked(|env| {
                let mut pi = PlanInterpreter::new(env.clone());
                let res = pi.evaluate(&query).unwrap();
                //let result = pi.state.as_ref().unwrap().data.try_into_string().unwrap();
                set_result(format!("{:?}\n{}", res, res.data.try_into_string().unwrap()));
            });
            }}>
            "Evaluate"
        </button>
        <pre>{result}</pre>
    }
}

type LocalValue = ExtValue;
type LocalEnvRef = ArcEnvRef<SimpleEnvironment<LocalValue>>;
fn main() {
    _ = console_log::init_with_level(log::Level::Debug);
    let mut env:SimpleEnvironment<LocalValue> = SimpleEnvironment::new();
    env.with_store(Box::new(MemoryStore::new(&Key::new())));
    let mut cr = env.get_mut_command_executor();
    cr.register_command(
        "lower",
        Command2::from(|state: &State<LocalValue>, postfix: String| -> String {
            let input:String = state.data.try_into_string().unwrap();
            format!("{} {}", input.to_lowercase(), postfix)
        }),
    ).expect("Failed to register command")
    .with_state_argument(ArgumentInfo::string_argument("text"))
    .with_argument(ArgumentInfo::string_argument("postfix"));
    cr.register_command(
        "testpolars",
        Command2::from(|state: &State<LocalValue>, postfix: String| -> String {
            let input:String = state.data.try_into_string().unwrap();
            let df = CsvReader::new(std::io::Cursor::new(input.as_bytes())).finish().unwrap();
            format!("testpolars {} {}\n{:?}", input, postfix, df)
        }),
    ).expect("Failed to register command")
    .with_state_argument(ArgumentInfo::string_argument("text"))
    .with_argument(ArgumentInfo::string_argument("arg"));

    let (envref, _):(ReadSignal<LocalEnvRef>,_) = create_signal(env.to_ref());
    provide_context(envref);
    

    console_error_panic_hook::set_once();
    log::info!("Hello, world??");
    
    mount_to_body(|| view! {
        <h1>"Hello"</h1>
        <Hello/>
        <Interpreter query="test.txt/-/lower-xxx".to_owned()/>
        <Interpreter query="test.csv/-/testpolars-yyy".to_owned()/>
    });
}