use {
    plotters::{
        backend::BitMapBackend,
        chart::ChartBuilder,
        coord::ranged1d::IntoSegmentedCoord,
        drawing::IntoDrawingArea,
        series::Histogram,
        style::{Color, RED, WHITE},
    },
    std::time::Instant,
    util::BfError,
};

const OUT_FILE_NAME: &str = "perf-graph.png";
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
    let perf_millis = [ImplInfo::new("simpleinterp", benchmark(simpleinterp::run)?)];

    create_graph(perf_millis)?;

    Ok(())
}

fn benchmark(run_function: impl RunFunction) -> Result<u128, BfError> {
    let start = Instant::now();

    run_function("")?;

    Ok(start.elapsed().as_millis())
}

fn create_graph<const N: usize>(perf_millis: [ImplInfo; N]) -> Result<(), BfError> {
    let root = BitMapBackend::new(OUT_FILE_NAME, RESOLUTION).into_drawing_area();
    root.fill(&WHITE)?;

    let names = perf_millis.iter().map(|x| x.name).collect::<Vec<_>>();
    let max_value = perf_millis.iter().map(|x| x.millis).max().unwrap();

    let mut chart = ChartBuilder::on(&root)
        .x_label_area_size(65)
        .y_label_area_size(60)
        .margin(20)
        .caption("BF JIT performance comparison", ("sans-serif", 50.0))
        .build_cartesian_2d(names.into_segmented(), 0u128..(max_value + 100))?;
    chart
        .configure_mesh()
        .disable_x_mesh()
        .bold_line_style(&WHITE.mix(0.3))
        .y_desc("Runtime (ms)")
        .x_desc("Implementation")
        .axis_desc_style(("sans-serif", 25))
        .label_style(("sans-serif", 20))
        .draw()?;

    chart.draw_series(
        Histogram::vertical(&chart)
            .style(RED.mix(0.5).filled())
            .data(names.iter().map(|name| (name, 1))),
    )?;

    root.present()?;
    println!("Performance graph has been saved to {}.", OUT_FILE_NAME);

    Ok(())
}
