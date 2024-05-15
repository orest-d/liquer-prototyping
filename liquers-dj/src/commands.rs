use liquers_core::context::*;
use liquers_core::error::*;
use liquers_core::state::*;
use liquers_core::value::*;
use liquers_core::*;
use polars::prelude::*;

use plotly::common::{
    ColorScale, ColorScalePalette, DashType, Fill, Font, Line, LineShape, Marker, Mode, Title,
};
use plotly::layout::{Axis, BarMode, Layout, Legend, TicksDirection};
use plotly::{Bar, Plot, Scatter};

use crate::value::ExtValue;

fn lower(state: &State<ExtValue>, postfix: String) -> Result<ExtValue, Error> {
    let input: String = state.data.try_into_string().unwrap();
    Ok(ExtValue::from_string(format!(
        "{} {}",
        input.to_lowercase(),
        postfix
    )))
}

fn csv2polars(state: &State<ExtValue>) -> Result<ExtValue, Error> {
    let input: String = state.data.try_into_string().unwrap();
    let df = CsvReader::new(std::io::Cursor::new(input.as_bytes()))
        .finish()
        .unwrap();
    Ok(ExtValue::DataFrame(df))
}

fn fmt(state: &State<ExtValue>) -> Result<ExtValue, Error> {
    let res = match *(state.data) {
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
    };
    Ok(ExtValue::from_string(res))
}

fn plot(state: &State<ExtValue>) -> Result<ExtValue, Error> {
    let df = match *(state.data) {
        ExtValue::DataFrame(ref df) => df.clone(),
        _ => {
            return Ok(ExtValue::from_string(format!("Not a DataFrame")));
        }
    };
    let vx = df.get_columns()[0]
        .cast(&DataType::Float64)
        .unwrap()
        .f64()
        .unwrap()
        .to_vec();
    let vy = df.get_columns()[1]
        .cast(&DataType::Float64)
        .unwrap()
        .f64()
        .unwrap()
        .to_vec();

    let trace = Scatter::new(vx, vy).mode(Mode::Markers);
    let mut plot = Plot::new();
    plot.add_trace(trace);
    let res = format!(
        "<!doctype html>
    <html lang=\"en\">
    
    <head>
        <!-- snip -->
        <script src=\"https://cdn.plot.ly/plotly-2.14.0.min.js\"></script>
    </head>
    
    <body>
    <h1>Simple Scatter Plot test</h1>
    {}
    </body>
    </html>
    ",
        plot.to_inline_html(Some("simple_scatter_plot"))
    );
    Ok(ExtValue::from_string(res))
}

pub fn testpolars(state: &State<ExtValue>, postfix: String) -> Result<ExtValue, Error> {
    let input: String = state.data.try_into_string().unwrap();
    let df = CsvReader::new(std::io::Cursor::new(input.as_bytes()))
        .finish()
        .unwrap();
    let res = format!("testpolars {} {}\n{:?}", input, postfix, df);
    Ok(ExtValue::from_string(res))
}

pub fn make_command_executor(mut env: SimpleEnvironment<ExtValue>) -> Result<SimpleEnvironment<ExtValue>,Error> {
    let mut cr = env.get_mut_command_executor();
    register_command!(cr, lower(state, postfix: String));
    //println!(stringify!(register_command!(cr, lower(state, postfix: String))));
    register_command!(cr, testpolars(state, postfix: String));
    register_command!(cr, csv2polars(state));
    register_command!(cr, fmt(state));
    register_command!(cr, plot(state));

    Ok(env)
}
