use {
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

const DATA_FILE_PREFIX: &str = concat!(os_id!(), "-", arch_id!());

const BENCH_DATA_DIR_NAME: &str = "bench-data";

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

pub fn measure_results() -> Result<(), BfError> {
    fs::create_dir_all(BENCH_DATA_DIR_NAME)?;

    // See https://github.com/eliben/code-for-blog/tree/master/2017/bfjit/bf-programs.
    for (short_title, title, input) in [
        ("mandelbrot", "mandelbrot generator", ""),
        ("factor", "factorization", "179424691\n"),
    ] {
        let filepath = format!("corpus/{short_title}.bf");
        println!("Measuring file {filepath}...");
        let source_code = fs::read_to_string(filepath)?;

        measure_results_for_file(title, short_title, input, &source_code)?;
    }

    Ok(())
}

fn measure_results_for_file(
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

    output_data(title, short_title, impl_infos)?;

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
    let num_runs_u128: u128 = NUM_RUNS.try_into()?;

    let result = sum / num_runs_u128;
    println!("Completed in {result}ms on average over {NUM_RUNS} runs.");

    Ok(result)
}

fn output_data<const N: usize>(
    title: &str,
    short_title: &str,
    impl_infos: [ImplInfo; N],
) -> Result<(), BfError> {
    let mut output_json = String::new();
    output_json.push_str("{\n");

    output_json.push_str(&format!(
        "    \"title\": \"BF JIT {title} ({OS_NAME} {ARCH_NAME})\",\n"
    ));

    output_json.push_str("    \"data\": [\n");

    let num_implementations = impl_infos.len();
    for (i, info) in impl_infos.into_iter().enumerate() {
        let comma = if i == num_implementations - 1 {
            ""
        } else {
            ","
        };
        output_json.push_str(&format!(
            "        {{\"implementation\": \"{name}\", \"milliseconds\": {millis}}}{comma}\n",
            name = info.name,
            millis = info.millis
        ));
    }

    output_json.push_str("    ]\n");
    output_json.push_str("}\n");

    let output_filename = format!("{BENCH_DATA_DIR_NAME}/{DATA_FILE_PREFIX}-{short_title}.json");
    fs::write(&output_filename, output_json)?;
    println!("Benchmark data has been saved to {output_filename}.");

    Ok(())
}
