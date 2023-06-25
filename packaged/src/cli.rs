use clap::{Parser, ValueEnum};
use crate::dividers::{Method, DividendSize, Precision, Estimate, SubMethod, DivInfo};
use crate::data::{Adder, Mul};

#[derive(Debug, PartialEq, Eq, Clone, Copy, ValueEnum)]
pub enum CircuitKind {
    CRA,
    CSA,
    KSA,
    Mux1,
    Mux2,
    LZC,
    LT,
    MulDadda,
    SquareDadda,
    ArrayMul,
    DIVIDER,
}

#[derive(Parser, Debug, Clone)]
#[command(name = "NewtonDivisionCircuitGenerator")]
#[command(author = "Tamer Bouz El-Jedi <tbejtf@gmail.com>")]
#[command(version = "1.0")]
#[command(about = "Outputs a division circuit for unsigned integers specified by it's verilog file", long_about = None)]
pub struct Args {
    #[arg(value_enum, short, long, default_value_t = CircuitKind::DIVIDER)]
    pub circuit_kind: CircuitKind,
    #[arg(short, long, default_value_t = 16, value_parser = clap::value_parser!(u16).range(1..=0xffff))]
    pub bits: u16,
    #[arg(long, default_value_t = false)]
    pub remove_dead_ends: bool,
    #[arg(value_enum, short = 'd', long, default_value_t = Method::Newton)]
    pub division_method: Method,
    #[arg(value_enum, short = 'a', long, default_value_t = Adder::CRA)]
    pub preferred_adder: Adder,
    #[arg(value_enum, short = 'm', long, default_value_t = Mul::DadaTree)]
    pub preferred_multiplier: Mul,
    #[arg(value_enum, short, long, default_value_t = Estimate::None)]
    pub estimator: Estimate,
    #[arg(value_enum, short = 'z', long, default_value_t = DividendSize::Equal)]
    pub dividend_size: DividendSize,
    #[arg(value_enum, short, long, default_value_t = Precision::Fixed)]
    pub precision: Precision,
    #[arg(value_enum, short, long, default_value_t = SubMethod::Seperate)]
    pub sub_method: SubMethod,
    #[arg(short, long)]
    pub outputfile: Option<String>,
}

pub fn parse() -> (DivInfo, bool, Args) {
    let args = Args::parse();

    let info = DivInfo {
        division_method: args.division_method,
        number_bits: args.bits as usize,
        defaultadder: args.preferred_adder,
        defaultmult: args.preferred_multiplier,
        estimator: args.estimator,
        sub_method: args.sub_method,
        dividend_size: args.dividend_size,
    };

    (info, args.remove_dead_ends, args)
}

use std::path::Path;
pub fn get_file_and_module_name(args: Args) -> (String, String) {
    let mut file_name = String::from("");
    
    match args.division_method {
        Method::Newton => { file_name.push_str("NewtDiv"); }
        Method::Goldschmidt => { file_name.push_str("GoldDiv"); }
        Method::Restoring => { file_name.push_str("RestoringDiv"); }
    }

    match args.dividend_size {
        DividendSize::DividendDouble => { file_name.push_str("DDouble");},
        DividendSize::Equal => { file_name.push_str("");}
    }

    if args.division_method == Method::Newton {
        match args.estimator {
            Estimate::Flip5bit => { file_name.push_str("FlipEst");},
            Estimate::Table10bit => { file_name.push_str("TableEst");},
            _ => (),
        }
    }

    let module_name = file_name.clone();

    if let Some(name) = args.outputfile {
        return (name, module_name);
    }

    file_name.push_str(&format!("_{}bit.v", args.bits));

    let mut num = 1;
    let mut file_name_copy = file_name.clone();
    while Path::exists(Path::new(&file_name_copy)) {
        let (file_name_head, file_name_data_type) = file_name.split_at(file_name.find('.').unwrap());
        file_name_copy = format!("{}_{num}{}", file_name_head, file_name_data_type);
        num += 1;
    }
    file_name = file_name_copy;

    (file_name, module_name)
}