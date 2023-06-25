mod adders;
mod circuit_tests;
mod data;
mod dividers;
mod helpers;
mod multipliers;
mod primitives;
mod cli;
mod bdd;
mod bdd_test;
mod sat;
mod sat_test;
use data::{Circuit};
use std::time::Instant;

use crate::cli::CircuitKind;

fn main() -> std::io::Result<()> {
    use std::env;
    env::set_var("RUST_BACKTRACE", "1");

    let (divider_builder, remove_dead_ends, additional_args) = cli::parse();
    let (output_filename, module_name) = cli::get_file_and_module_name(additional_args.clone());


    match additional_args.circuit_kind {

        CircuitKind::Mux1 => {
            let mut circuit = Circuit::get_mux_n_1(divider_builder.number_bits);
            circuit.write_to_file(&format!("MUX_1_ {}bit.v", divider_builder.number_bits), "MUX")?;
        }
        CircuitKind::Mux2 => {
            let mut circuit = Circuit::get_mux_n_2(divider_builder.number_bits);
            circuit.write_to_file(&format!("MUX_2_{}bit.v", divider_builder.number_bits), "MUX")?;
        }
        CircuitKind::LT => {
            let mut circuit = Circuit::get_lt(divider_builder.number_bits);
            circuit.write_to_file(&format!("LT_{}bit.v", divider_builder.number_bits), "LT")?;
        }
        CircuitKind::CRA => {
            let mut circuit = Circuit::get_cra(divider_builder.number_bits);
            circuit.write_to_file(&format!("CRA_{}bit.v", divider_builder.number_bits), "ADD")?;
        }
        CircuitKind::CSA => {
            let mut circuit = Circuit::get_csa(divider_builder.number_bits);
            circuit.write_to_file(&format!("CSA_{}bit.v", divider_builder.number_bits), "ADD")?;
        }
        CircuitKind::KSA => {
            let mut circuit = Circuit::get_ksa(divider_builder.number_bits);
            circuit.write_to_file(&format!("KSA_{}bit.v", divider_builder.number_bits), "ADD")?;
        }
        CircuitKind::MulDadda => {
            let mut circuit = Circuit::get_umul(divider_builder.number_bits, additional_args.circuit_kind, additional_args.sub_method, additional_args.preferred_adder);
            circuit.write_to_file(&format!("UMUL_DADDA_{}bit.v", divider_builder.number_bits), "UMUL")?;
        }
        CircuitKind::ArrayMul => {
            let mut circuit = Circuit::get_umul(divider_builder.number_bits, additional_args.circuit_kind, additional_args.sub_method, additional_args.preferred_adder);
            circuit.write_to_file(&format!("UMUL_ARRAY_{}bit.v", divider_builder.number_bits), "UMUL")?;
        }
        CircuitKind::SquareDadda => {
            let mut circuit = Circuit::get_umul(divider_builder.number_bits, additional_args.circuit_kind, additional_args.sub_method, additional_args.preferred_adder);
            circuit.write_to_file(&format!("USQUARE_DADDA_{}bit.v", divider_builder.number_bits), "SQUARE")?;
        }
        _ => {
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
            circuit.update_stats();
            println!("Gatter count: {}, Max depth: {}", circuit.stats.gatter_count, circuit.stats.level_count);
        
            let mut circuit = Circuit::get_lzc_circuit(divider_builder.number_bits
                , remove_dead_ends);
            circuit.write_to_file(&format!("LZC_{}bit.v", divider_builder.number_bits), "LZC")?;
        }
    }

    Ok(())
}
