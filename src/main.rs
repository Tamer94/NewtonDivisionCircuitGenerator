mod adders;
mod circuit_tests;
mod data;
mod dividers;
mod helpers;
mod multipliers;
mod primitives;
mod cli;
use data::{Circuit};
use std::time::Instant;

fn main() -> std::io::Result<()> {
    use std::env;
    env::set_var("RUST_BACKTRACE", "1");

    let (divider_builder, remove_dead_ends, additional_args) = cli::parse();
    let (output_filename, module_name) = cli::get_file_and_module_name(additional_args);

    let mut time = Instant::now();
    let mut circuit = Circuit::get_divider_circuit(divider_builder);
    println!("Generating circuit took {:#?} µs", time.elapsed().as_micros());

    if remove_dead_ends {
        time = Instant::now();
        circuit.remove_dead_ends();
        println!("Removing dead ends took {:#?} µs", time.elapsed().as_micros());
    }


    time = Instant::now();
    circuit.write_to_file(&output_filename, &module_name)?;
    println!("Writing circuit to file took {:#?} µs saved as <{}>", time.elapsed().as_micros(), output_filename);
    println!("Gatter count: {}, Max depth: {}", circuit.stats.gatter_count, circuit.stats.level_count);

    Ok(())
}
