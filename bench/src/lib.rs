use {
    plotters::{
        backend::BitMapBackend,
        chart::ChartBuilder,
        coord::ranged1d::{IntoSegmentedCoord, SegmentValue},
        drawing::IntoDrawingArea,
        series::Histogram,
        style::{Color, RED, WHITE},
    },
    std::{fs, time::Instant},
    util::BfError,
};

const RESOLUTION: (u32, u32) = (1280, 720);

trait RunFunction: FnOnce(&str) -> Result<(), BfError> {}
impl<T> RunFunction for T where T: FnOnce(&str) -> Result<(), BfError> {}

struct ImplInfo {
    name: &'static str,
    millis: u128,
}

impl ImplInfo {
    fn new(name: &'static str, millis: u128) -> Self {
        Self { name, millis }
    }
}

pub fn graph_results() -> Result<(), BfError> {
    // See https://github.com/eliben/code-for-blog/tree/master/2017/bfjit/bf-programs.
    for (short_title, title) in [
        ("mandelbrot", "mandelbrot generator"),
        ("factor", "factorization"),
    ] {
        let filepath = format!("corpus/{short_title}.bf");
        let source_code = fs::read_to_string(filepath)?;
        graph_results_for_file(title, short_title, &source_code)?;
    }

    Ok(())
}

fn graph_results_for_file(
    title: &str,
    short_title: &str,
    source_code: &str,
) -> Result<(), BfError> {
    let perf_millis = [ImplInfo::new(
        "simpleinterp",
        benchmark(simpleinterp::run, source_code)?,
    )];

    create_graph(title, short_title, perf_millis)?;

    Ok(())
}

fn benchmark(run_function: impl RunFunction, source_code: &str) -> Result<u128, BfError> {
    let start = Instant::now();

    run_function(source_code)?;

    Ok(start.elapsed().as_millis())
}

fn segmented_value_to_inner<T>(value: &SegmentValue<T>) -> &T {
    match value {
        SegmentValue::Exact(t) => t,
        SegmentValue::CenterOf(t) => t,
        SegmentValue::Last => unreachable!(),
    }
}

fn create_graph<const N: usize>(
    title: &str,
    short_title: &str,
    perf_millis: [ImplInfo; N],
) -> Result<(), BfError> {
    let output_filename = format!("perf-graph-{short_title}.png");

    let root = BitMapBackend::new(&output_filename, RESOLUTION).into_drawing_area();
    root.fill(&WHITE)?;

    let names = perf_millis.iter().map(|x| x.name).collect::<Vec<_>>();
    let max_value = perf_millis.iter().map(|x| x.millis).max().unwrap();

    let mut chart = ChartBuilder::on(&root)
        .x_label_area_size(65)
        .y_label_area_size(60)
        .margin(20)
        .caption(
            format!("BF JIT performance comparison ({title})"),
            ("sans-serif", 50.0),
        )
        .build_cartesian_2d(names.into_segmented(), 0u128..(max_value + 100))?;
    chart
        .configure_mesh()
        .disable_x_mesh()
        .bold_line_style(WHITE.mix(0.3))
        .y_desc("Runtime (ms)")
        .x_desc("Implementation")
        .axis_desc_style(("sans-serif", 25))
        .label_style(("sans-serif", 20))
        .x_label_formatter(&|value| segmented_value_to_inner(value).to_string())
        .draw()?;

    chart.draw_series(
        Histogram::vertical(&chart)
            .style(RED.mix(0.5).filled())
            .data(names.iter().map(|name| (name, 1))),
    )?;

    root.present()?;
    println!("Performance graph has been saved to {output_filename}.");

    Ok(())
}
