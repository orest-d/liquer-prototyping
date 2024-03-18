use liquers_core::command_metadata::ArgumentInfo;
use liquers_core::commands::*;
use liquers_core::context::*;
use liquers_core::state::*;
use liquers_core::value::*;
use polars::prelude::*;

use plotly::common::{
    ColorScale, ColorScalePalette, DashType, Fill, Font, Line, LineShape, Marker, Mode, Title,
};
use plotly::layout::{Axis, BarMode, Layout, Legend, TicksDirection};
use plotly::{Bar, Plot, Scatter};

use crate::value::ExtValue;

fn lower(state: &State<ExtValue>, postfix: String) -> String {
    let input: String = state.data.try_into_string().unwrap();
    format!("{} {}", input.to_lowercase(), postfix)
}

fn csv2polars(state: &State<ExtValue>) -> DataFrame {
    let input: String = state.data.try_into_string().unwrap();
    let df = CsvReader::new(std::io::Cursor::new(input.as_bytes()))
        .finish()
        .unwrap();
    df
}

fn fmt(state: &State<ExtValue>) -> String {
    match *(state.data) {
        ExtValue::None => "None".to_owned(),
        ExtValue::Bool(b) => (if b { "true" } else { "false" }).to_owned(),
        ExtValue::I32(x) => format!("{}", x),
        ExtValue::I64(x) => format!("{}", x),
        ExtValue::F64(x) => format!("{}", x),
        ExtValue::Text(ref x) => x.clone(),
        ExtValue::Array(ref x) => format!("{:?}", x),
        ExtValue::Object(ref x) => format!("{:?}", x),
        ExtValue::Bytes(_) => "Bytes".to_owned(),
        ExtValue::DataFrame(ref df) => format!("{}", df),
    }
}

fn plot(state: &State<ExtValue>) -> String {
    let df = match *(state.data) {
        ExtValue::DataFrame(ref df) => df.clone(),
        _ => {
            return format!("Not a DataFrame");
        },
    };
    let vx = df.get_columns()[0].cast(&DataType::Float64).unwrap().f64().unwrap().to_vec();
    let vy = df.get_columns()[1].cast(&DataType::Float64).unwrap().f64().unwrap().to_vec();

    let trace = Scatter::new(vx, vy).mode(Mode::Markers);
    let mut plot = Plot::new();
    plot.add_trace(trace);
    format!("{}", plot.to_inline_html(Some("simple_scatter_plot")))
}

pub fn make_command_executor(mut env: SimpleEnvironment<ExtValue>) -> SimpleEnvironment<ExtValue> {
    let mut cr = env.get_mut_command_executor();
    cr.register_command("lower", Command2::from(lower))
        .expect("Failed to register command")
        .with_state_argument(ArgumentInfo::string_argument("text"))
        .with_argument(ArgumentInfo::string_argument("postfix"));

    cr.register_command(
        "testpolars",
        Command2::from(|state: &State<ExtValue>, postfix: String| -> String {
            let input: String = state.data.try_into_string().unwrap();
            let df = CsvReader::new(std::io::Cursor::new(input.as_bytes()))
                .finish()
                .unwrap();
            format!("testpolars {} {}\n{:?}", input, postfix, df)
        }),
    )
    .expect("Failed to register command")
    .with_state_argument(ArgumentInfo::string_argument("text"))
    .with_argument(ArgumentInfo::string_argument("arg"));

    cr.register_command("csv2polars", Command1::from(csv2polars))
        .expect("Failed to register command")
        .with_state_argument(ArgumentInfo::string_argument("csv"));

    cr.register_command("fmt", Command1::from(fmt))
        .expect("Failed to register command")
        .with_state_argument(ArgumentInfo::string_argument("csv"));

    cr.register_command("plot", Command1::from(plot))
        .expect("Failed to register command")
        .with_state_argument(ArgumentInfo::string_argument("df"));

    env
}
