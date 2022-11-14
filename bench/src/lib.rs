use {
    plotters::{
        backend::BitMapBackend,
        chart::ChartBuilder,
        coord::ranged1d::{IntoSegmentedCoord, SegmentValue},
        drawing::IntoDrawingArea,
        series::Histogram,
        style::{Color, RGBColor, WHITE},
    },
    std::{fs, time::Instant},
    util::{BfError, RunFunction},
};

#[cfg(target_os = "linux")]
macro_rules! os_id {
    () => {
        "linux"
    };
}
#[cfg(target_os = "macos")]
macro_rules! os_id {
    () => {
        "macos"
    };
}
#[cfg(target_os = "windows")]
macro_rules! os_id {
    () => {
        "windows"
    };
}

#[cfg(target_os = "linux")]
const OS_NAME: &str = "Linux";
#[cfg(target_os = "macos")]
const OS_NAME: &str = "macOS";
#[cfg(target_os = "windows")]
const OS_NAME: &str = "Windows";

#[cfg(target_arch = "x86_64")]
macro_rules! arch_id {
    () => {
        "x86_64"
    };
}
#[cfg(target_arch = "x86")]
macro_rules! arch_id {
    () => {
        "x86"
    };
}
#[cfg(target_arch = "aarch64")]
macro_rules! arch_id {
    () => {
        "aarch64"
    };
}

#[cfg(target_arch = "x86_64")]
const ARCH_NAME: &str = "x86-64";
#[cfg(target_arch = "x86")]
const ARCH_NAME: &str = "x86";
#[cfg(target_arch = "aarch64")]
const ARCH_NAME: &str = "aarch64";

const GRAPH_NAME_PREFIX: &str = concat!(os_id!(), "-", arch_id!());

const RESOLUTION: (u32, u32) = (1280, 720);

const BAR1_COLOR: RGBColor = RGBColor(76, 114, 176);
const BAR2_COLOR: RGBColor = RGBColor(191, 191, 0);
const BAR3_COLOR: RGBColor = RGBColor(255, 165, 0);
const BAR4_COLOR: RGBColor = RGBColor(255, 0, 0);
const BAR5_COLOR: RGBColor = RGBColor(0, 128, 0);
const BAR6_COLOR: RGBColor = RGBColor(144, 238, 144);

struct ImplInfo {
    name: &'static str,
    millis: u128,
}

impl ImplInfo {
    fn new(
        name: &'static str,
        run_function: &dyn RunFunction,
        source_code: &str,
        input: &str,
    ) -> Result<Self, BfError> {
        let millis = benchmark(name, run_function, source_code, input)?;
        Ok(Self { name, millis })
    }
}

pub fn graph_results() -> Result<(), BfError> {
    fs::create_dir_all("graphs")?;

    // See https://github.com/eliben/code-for-blog/tree/master/2017/bfjit/bf-programs.
    for (short_title, title, input) in [
        ("mandelbrot", "mandelbrot generator", ""),
        ("factor", "factorization", "179424691\n"),
    ] {
        let filepath = format!("corpus/{short_title}.bf");
        println!("Measuring file {filepath}...");
        let source_code = fs::read_to_string(filepath)?;

        graph_results_for_file(title, short_title, input, &source_code)?;
    }

    Ok(())
}

fn graph_results_for_file(
    title: &str,
    short_title: &str,
    input: &str,
    source_code: &str,
) -> Result<(), BfError> {
    let impl_infos = [
        ImplInfo::new("simpleinterp", &simpleinterp::run, source_code, input)?,
        ImplInfo::new("opinterp", &opinterp::run, source_code, input)?,
        ImplInfo::new("opinterp2", &opinterp2::run, source_code, input)?,
        ImplInfo::new("opinterp3", &opinterp3::run, source_code, input)?,
        ImplInfo::new("simplejit", &simplejit::run, source_code, input)?,
        ImplInfo::new("opjit", &opjit::run, source_code, input)?,
    ];

    create_graph(title, short_title, impl_infos)?;

    Ok(())
}

fn benchmark(
    name: &str,
    run_function: impl RunFunction,
    source_code: &str,
    input: &str,
) -> Result<u128, BfError> {
    println!("Benchmarking {name}...");

    const NUM_RUNS: usize = 10;
    let mut times = [0; NUM_RUNS];

    for item in times.iter_mut() {
        let start = Instant::now();
        run_function(source_code, &mut input.as_bytes(), &mut vec![])?;
        *item = start.elapsed().as_millis();
    }

    let sum = times.into_iter().sum::<u128>();
    let num_runs_u128 = <usize as TryInto<u128>>::try_into(NUM_RUNS)?;
    Ok(sum / num_runs_u128)
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
    impl_infos: [ImplInfo; N],
) -> Result<(), BfError> {
    let output_filename = format!("graphs/{GRAPH_NAME_PREFIX}-{short_title}.png");

    let root = BitMapBackend::new(&output_filename, RESOLUTION).into_drawing_area();
    root.fill(&WHITE)?;

    let names = impl_infos.iter().map(|x| x.name).collect::<Vec<_>>();
    let max_value = impl_infos.iter().map(|x| x.millis).max().unwrap();
    let max_value_more_10_percent = ((max_value as f64) * 1.1) as u128;

    let mut chart = ChartBuilder::on(&root)
        .x_label_area_size(65)
        .y_label_area_size(90)
        .right_y_label_area_size(90)
        .margin(20)
        .caption(
            format!("BF JIT {title} ({OS_NAME} {ARCH_NAME})"),
            ("sans-serif", 50.0),
        )
        .build_cartesian_2d(names.into_segmented(), 0u128..(max_value_more_10_percent))?;
    chart
        .configure_mesh()
        .disable_x_mesh()
        .bold_line_style(WHITE.mix(0.3))
        .y_desc("Average runtime (ms)")
        .x_desc("Implementation")
        .axis_desc_style(("sans-serif", 25))
        .label_style(("sans-serif", 20))
        .x_label_formatter(&|value| segmented_value_to_inner(value).to_string())
        .draw()?;

    chart.draw_series(
        Histogram::vertical(&chart)
            .style_func(|value, _millis| match **segmented_value_to_inner(value) {
                "simpleinterp" => BAR1_COLOR.filled(),
                "opinterp" => BAR2_COLOR.filled(),
                "opinterp2" => BAR3_COLOR.filled(),
                "opinterp3" => BAR4_COLOR.filled(),
                "simplejit" => BAR5_COLOR.filled(),
                "opjit" => BAR6_COLOR.filled(),
                _ => unreachable!(),
            })
            .data(impl_infos.iter().map(|x| (&x.name, x.millis))),
    )?;

    root.present()?;
    println!("Performance graph has been saved to {output_filename}.");

    Ok(())
}
