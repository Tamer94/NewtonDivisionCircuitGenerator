use std::io;

use polyengine::{BPolynom, Monom, PolyEngine};

use crate::{data::{LevelizedCircuit, DEFAULT_LEVEL, Gate::{And, Or, Xor, Not}}, dividers::DivInfo, cli::parse, data::Bit};
 
#[test]
fn interactive() {
    'outer: loop {
        println!("Interactive NewtonDividerVerification\n");
        println!("Enter how many bits the divider should use as input size!");

        let mut input = String::new();

        io::stdin().read_line(&mut input).expect("Input should be written into stdin");
        let mut bits = input.trim().to_lowercase();

        while let Err(_) = bits.parse::<usize>() {
            println!("Try again to enter the number of bits only enter digits and press enter.\n
            The number should be in the range 1 to {}", u16::MAX);

            input.clear();
            io::stdin().read_line(&mut input).expect("Input should be written into stdin");
            bits = input.trim().to_lowercase();
        }

        
        let mut info = DivInfo::default_newton();
        info.number_bits = bits.parse().unwrap();
        println!("You entered {}\ndivision circuit with the given input sizes will be created", info.number_bits);
    
        let mut lc = LevelizedCircuit::get_divider_circuit(info);
        lc.circuit.remove_dead_ends();
        println!("division circuit was created");

        println!("\x1B[32m");
        println!("{:?}", lc.circuit.outputs);
        println!("{:?}", lc.circuit.inputs);
        println!("\x1B[0m");

        let mut en = PolyEngine::new(BPolynom::empty());
        println!("\x1B[34m");
        let mut d_poly = BPolynom::empty();
        let mut q_poly = BPolynom::empty();
        let mut r_poly = BPolynom::empty();
        let mut n_poly = BPolynom::empty();
        let d_name = String::from("D");
        let n_name = String::from("N");
        let r_name = String::from("R");
        let q_name = String::from("Q");
        for input in &lc.circuit.inputs {
            let vars: Vec<usize> = input.bits.iter().filter(|&& x| 
                {
                    match x { 
                        Bit::Var(l) => true,
                        _ => false

                }
            }
            ).map(|&x| {
                if let Bit::Var(l) = x {
                    l.n
                } else {
                    0
                }
            }).collect();
            let name = input.name.clone();
            let names = (0..vars.len()).map(|i| format!("{name}{i}")).collect();
            let poly = en.get_unsigned_poly(vars, names);
            if name == n_name {
                n_poly = poly.clone();
            }
            if name == d_name {
                d_poly = poly.clone();
            }
            println!("{}", poly.to_string(&en.var_names, ", "));
        }

        for output in &lc.circuit.outputs {
            let vars: Vec<usize> = output.bits.iter().filter(|&& x| 
                {
                    match x { 
                        Bit::Var(l) => true,
                        _ => false

                }
            }
            ).map(|&x| {
                if let Bit::Var(l) = x {
                    l.n
                } else {
                    0
                }
            }).collect();
            let name = output.name.clone();
            let names = (0..vars.len()).map(|i| format!("{name}{i}")).collect();
            let mut poly = BPolynom::empty();
            if name != String::from("Valid") {
                poly = en.get_2_compl_poly(vars, names);
            }
            if name == q_name {
                q_poly = poly.clone();
            }
            if name == r_name {
                r_poly =  poly.clone();
            }
            println!("{}", poly.to_string(&en.var_names, ", "));
        }

        let mut wires = lc.circuit.wires.clone();
        wires.sort_by(|w0, w1| {
            let ordering = lc.substitution_levels.get(&w0.out.n).unwrap_or(&DEFAULT_LEVEL).cmp(&lc.substitution_levels.get(&w1.out.n).unwrap_or(&DEFAULT_LEVEL));
            match ordering {
                std::cmp::Ordering::Equal => {
                    w1.out.level.cmp(&w0.out.level)
                }
                _ => {
                    ordering
                }
            }
        });

        wires = wires.into_iter().filter(|w| lc.substitution_levels.contains_key(&w.out.n) && *lc.substitution_levels.get(&w.out.n).unwrap_or(&DEFAULT_LEVEL) < DEFAULT_LEVEL).collect();

        wires.iter().enumerate().for_each(|(i, w)| println!("{}: {:?}", i, w));

        print!("\x1B[0m");

        println!("N Polynomial: {}", n_poly.to_string(&en.var_names, ", "));
        println!("D Polynomial: {}", d_poly.to_string(&en.var_names, ", "));
        println!("Q Polynomial: {}", q_poly.to_string(&en.var_names, ", "));
        println!("R Polynomial: {}", r_poly.to_string(&en.var_names, ", "));

        let spec_poly = n_poly + &(((&d_poly * &q_poly) + &r_poly) * -1);

        println!("\x1B[35m");
        print!("Spec Polynomial: ");
        println!("{}", spec_poly.to_string(&en.var_names, " ,"));
        println!("\x1B[0m");

        
        let mut sub_counter = 0;
        en.add_from_generates(spec_poly);
        println!("{}", en.p.to_string(&en.var_names, " "));

        println!("enter r to perform replacements step by step");
        while sub_counter < wires.len() {
            io::stdin().read_line(&mut input).expect("Input should be written into stdin");
            let mut command = input.trim().to_lowercase();
            if command.contains("r") {
                let wire = wires[sub_counter];
                let var = wire.out.n;
                let mut in_var1 = 0;
                let mut in_var2 = 0;
                match wire.gate {
                    And(in1, in2) => {
                        let in1_name = match lc.circuit.io_lines.get(&in1.n) {
                            Some(&nl) => { format!("{}{}", nl.name, nl.idx) }
                            _ => { format!("J{}", in1.n) }
                        };
                        let in2_name = match lc.circuit.io_lines.get(&in2.n) {
                            Some(&nl) => { format!("{}{}", nl.name, nl.idx) }
                            _ => { format!("J{}", in2.n) }
                        };
                        in_var1 = in1.n;
                        in_var2 = in2.n;
                        en.and_replace(var, in1.n, in1_name, in2.n, in2_name);
                    }
                    Or(in1, in2) => {
                        let in1_name = match lc.circuit.io_lines.get(&in1.n) {
                            Some(&nl) => { format!("{}{}", nl.name, nl.idx) }
                            _ => { format!("J{}", in1.n) }
                        };
                        let in2_name = match lc.circuit.io_lines.get(&in2.n) {
                            Some(&nl) => { format!("{}{}", nl.name, nl.idx) }
                            _ => { format!("J{}", in2.n) }
                        };
                        in_var1 = in1.n;
                        in_var2 = in2.n;
                        en.or_replace(var, in1.n, in1_name, in2.n, in2_name);
                    }
                    Xor(in1, in2) => {
                        let in1_name = match lc.circuit.io_lines.get(&in1.n) {
                            Some(&nl) => { format!("{}{}", nl.name, nl.idx) }
                            _ => { format!("J{}", in1.n) }
                        };
                        let in2_name = match lc.circuit.io_lines.get(&in2.n) {
                            Some(&nl) => { format!("{}{}", nl.name, nl.idx) }
                            _ => { format!("J{}", in2.n) }
                        };
                        in_var1 = in1.n;
                        in_var2 = in2.n;
                        en.xor_replace(var, in1.n, in1_name, in2.n, in2_name);
                    }
                    Not(in1) => {
                        let in1_name = match lc.circuit.io_lines.get(&in1.n) {
                            Some(&nl) => { format!("{}{}", nl.name, nl.idx) }
                            _ => { format!("J{}", in1.n) }
                        };
                        in_var1 = in1.n;
                        en.not_replace(var, in1.n, in1_name);
                    }
                }
                if in_var1 == usize::MAX || in_var2 == usize::MAX {
                    en.const_0_replace(usize::MAX);
                }
                if in_var1 == usize::MAX-1 || in_var2 == usize::MAX-1 {
                    en.const_1_replace(usize::MAX-1);
                }
                println!("After replacement: {}", en.p.to_string(&en.var_names, " "));
                println!("Number of Monoms: {}", en.p.poly.len());
                sub_counter += 1;
            }
            if command.contains("exit") { break; }
        }

        // println!("Final replacement of constants");
        // en.const_0_replace(usize::MAX);
        // en.const_1_replace(usize::MAX-1);
        println!("After final replacement: {}", en.p.to_string(&en.var_names, " "));


        
        println!("Would you like to exit then enter exit");
        println!("or enter restart to restart the program");

        io::stdin().read_line(&mut input).expect("Input should be written into stdin");
        let mut command = input.trim().to_lowercase();
        let exit = String::from("exit");
        let restart = String::from("restart");
        println!("Exit as {:?}", exit.as_bytes());
        loop {
            println!("Command as {:?}", command.as_bytes());
            
            if command.contains("exit") {
                println!("Leaving now... Bye");
                break 'outer;
            }
            if command.contains("restart") { break; } else {
                println!("enter a valid command either exit or restart!");
                io::stdin().read_line(&mut input).expect("Input should be written into stdin");
                command = input.trim().to_lowercase();
            }
        }

    }
        
}

#[test]
fn interactive_subtraction() {
    'outer: loop {
        println!("Interactive NewtonDividerVerification\n");
        println!("Enter how many bits the divider should use as input size!");

        let mut input = String::new();

        io::stdin().read_line(&mut input).expect("Input should be written into stdin");
        let mut bits = input.trim().to_lowercase();

        while let Err(_) = bits.parse::<usize>() {
            println!("Try again to enter the number of bits only enter digits and press enter.\n
            The number should be in the range 1 to {}", u16::MAX);

            input.clear();
            io::stdin().read_line(&mut input).expect("Input should be written into stdin");
            bits = input.trim().to_lowercase();
        }

        
        let mut info = DivInfo::default_newton();
        info.number_bits = bits.parse().unwrap();
        println!("You entered {}\nsubtraction circuit with the given input sizes will be created", info.number_bits);
    
        let mut lc = LevelizedCircuit::get_sub(info.number_bits);
        lc.circuit.remove_dead_ends();
        println!("subtraction circuit was created");

        println!("\x1B[32m");
        println!("{:?}", lc.circuit.outputs);
        println!("{:?}", lc.circuit.inputs);
        println!("\x1B[0m");

        let mut en = PolyEngine::new(BPolynom::empty());
        println!("\x1B[34m");
        let mut a_poly = BPolynom::empty();
        let mut b_poly = BPolynom::empty();
        let mut c_poly = BPolynom::empty();
        let a_name = String::from("A");
        let b_name = String::from("B");
        let c_name = String::from("C");
        for input in &lc.circuit.inputs {
            let vars: Vec<usize> = input.bits.iter().filter(|&& x| 
                {
                    match x { 
                        Bit::Var(l) => true,
                        _ => false

                }
            }
            ).map(|&x| {
                if let Bit::Var(l) = x {
                    l.n
                } else {
                    0
                }
            }).collect();
            let name = input.name.clone();
            let names = (0..vars.len()).map(|i| format!("{name}{i}")).collect();
            let poly = en.get_unsigned_poly(vars, names);
            if name == a_name {
                a_poly = poly.clone();
            }
            if name == b_name {
                b_poly = poly.clone();
            }
            println!("{}", poly.to_string(&en.var_names, ", "));
        }

        for output in &lc.circuit.outputs {
            let vars: Vec<usize> = output.bits.iter().filter(|&& x| 
                {
                    match x { 
                        Bit::Var(l) => true,
                        _ => false

                }
            }
            ).map(|&x| {
                if let Bit::Var(l) = x {
                    l.n
                } else {
                    0
                }
            }).collect();
            let name = output.name.clone();
            let names = (0..vars.len()).map(|i| format!("{name}{i}")).collect();
            let mut poly = BPolynom::empty();
            if name != String::from("Valid") {
                poly = en.get_2_compl_poly(vars, names);
            }
            if name == c_name {
                c_poly = poly.clone();
            }
            println!("{}", poly.to_string(&en.var_names, ", "));
        }

        let mut wires = lc.circuit.wires.clone();
        wires.sort_by(|w0, w1| {
            let ordering = lc.substitution_levels.get(&w0.out.n).unwrap_or(&DEFAULT_LEVEL).cmp(&lc.substitution_levels.get(&w1.out.n).unwrap_or(&DEFAULT_LEVEL));
            match ordering {
                std::cmp::Ordering::Equal => {
                    w1.out.level.cmp(&w0.out.level)
                }
                _ => {
                    ordering
                }
            }
        });

        wires = wires.into_iter().filter(|w| lc.substitution_levels.contains_key(&w.out.n) && *lc.substitution_levels.get(&w.out.n).unwrap_or(&DEFAULT_LEVEL) < DEFAULT_LEVEL).collect();

        wires.iter().enumerate().for_each(|(i, w)| println!("{}: {:?}", i, w));

        print!("\x1B[0m");

        println!("A Polynomial: {}", a_poly.to_string(&en.var_names, ", "));
        println!("B Polynomial: {}", b_poly.to_string(&en.var_names, ", "));
        println!("C Polynomial: {}", c_poly.to_string(&en.var_names, ", "));

        let spec_poly = c_poly + &(b_poly + &(a_poly * -1));

        println!("\x1B[35m");
        print!("Spec Polynomial: ");
        println!("{}", spec_poly.to_string(&en.var_names, " ,"));
        println!("\x1B[0m");

        
        let mut sub_counter = 0;
        en.add_from_generates(spec_poly);
        println!("{}", en.p.to_string(&en.var_names, " "));

        println!("enter r to perform replacements step by step");
        while sub_counter < wires.len() {
            io::stdin().read_line(&mut input).expect("Input should be written into stdin");
            let mut command = input.trim().to_lowercase();
            if command.contains("r") {
                let wire = wires[sub_counter];
                let var = wire.out.n;
                let mut in_var1 = 0;
                let mut in_var2 = 0;
                match wire.gate {
                    And(in1, in2) => {
                        let in1_name = match lc.circuit.io_lines.get(&in1.n) {
                            Some(&nl) => { format!("{}{}", nl.name, nl.idx) }
                            _ => { format!("J{}", in1.n) }
                        };
                        let in2_name = match lc.circuit.io_lines.get(&in2.n) {
                            Some(&nl) => { format!("{}{}", nl.name, nl.idx) }
                            _ => { format!("J{}", in2.n) }
                        };
                        in_var1 = in1.n;
                        in_var2 = in2.n;
                        en.and_replace(var, in1.n, in1_name, in2.n, in2_name);
                    }
                    Or(in1, in2) => {
                        let in1_name = match lc.circuit.io_lines.get(&in1.n) {
                            Some(&nl) => { format!("{}{}", nl.name, nl.idx) }
                            _ => { format!("J{}", in1.n) }
                        };
                        let in2_name = match lc.circuit.io_lines.get(&in2.n) {
                            Some(&nl) => { format!("{}{}", nl.name, nl.idx) }
                            _ => { format!("J{}", in2.n) }
                        };
                        in_var1 = in1.n;
                        in_var2 = in2.n;
                        en.or_replace(var, in1.n, in1_name, in2.n, in2_name);
                    }
                    Xor(in1, in2) => {
                        let in1_name = match lc.circuit.io_lines.get(&in1.n) {
                            Some(&nl) => { format!("{}{}", nl.name, nl.idx) }
                            _ => { format!("J{}", in1.n) }
                        };
                        let in2_name = match lc.circuit.io_lines.get(&in2.n) {
                            Some(&nl) => { format!("{}{}", nl.name, nl.idx) }
                            _ => { format!("J{}", in2.n) }
                        };
                        in_var1 = in1.n;
                        in_var2 = in2.n;
                        en.xor_replace(var, in1.n, in1_name, in2.n, in2_name);
                    }
                    Not(in1) => {
                        let in1_name = match lc.circuit.io_lines.get(&in1.n) {
                            Some(&nl) => { format!("{}{}", nl.name, nl.idx) }
                            _ => { format!("J{}", in1.n) }
                        };
                        in_var1 = in1.n;
                        en.not_replace(var, in1.n, in1_name);
                    }
                }
                if in_var1 == usize::MAX || in_var2 == usize::MAX {
                    en.const_0_replace(usize::MAX);
                }
                if in_var1 == usize::MAX-1 || in_var2 == usize::MAX-1 {
                    en.const_1_replace(usize::MAX-1);
                }
                println!("After replacement: {}", en.p.to_string(&en.var_names, " "));
                println!("Number of Monoms: {}", en.p.poly.len());
                println!("\x1B[32m");
                en.print_var_occurences();
                println!("\x1B[0m");
                sub_counter += 1;
            }
            if command.contains("exit") { break; }
        }

        // println!("Final replacement of constants");
        // en.const_0_replace(usize::MAX);
        // en.const_1_replace(usize::MAX-1);
        println!("After final replacement: {}", en.p.to_string(&en.var_names, " "));


        
        println!("Would you like to exit then enter exit");
        println!("or enter restart to restart the program");

        io::stdin().read_line(&mut input).expect("Input should be written into stdin");
        let mut command = input.trim().to_lowercase();
        let exit = String::from("exit");
        let restart = String::from("restart");
        println!("Exit as {:?}", exit.as_bytes());
        loop {
            println!("Command as {:?}", command.as_bytes());
            
            if command.contains("exit") {
                println!("Leaving now... Bye");
                break 'outer;
            }
            if command.contains("restart") { break; } else {
                println!("enter a valid command either exit or restart!");
                io::stdin().read_line(&mut input).expect("Input should be written into stdin");
                command = input.trim().to_lowercase();
            }
        }

    }
        
}

#[test]
fn interactive_multplication() {
    'outer: loop {
        println!("Interactive NewtonDividerVerification\n");
        println!("Enter how many bits the multiplier should use as input size!");

        let mut input = String::new();

        io::stdin().read_line(&mut input).expect("Input should be written into stdin");
        let mut bits = input.trim().to_lowercase();

        while let Err(_) = bits.parse::<usize>() {
            println!("Try again to enter the number of bits only enter digits and press enter.\n
            The number should be in the range 1 to {}", u16::MAX);

            input.clear();
            io::stdin().read_line(&mut input).expect("Input should be written into stdin");
            bits = input.trim().to_lowercase();
        }

        
        let mut info = DivInfo::default_newton();
        info.number_bits = bits.parse().unwrap();
        println!("You entered {}\nmultiplication circuit with the given input sizes will be created", info.number_bits);
    
        let mut lc = LevelizedCircuit::get_mul(info.number_bits);
        lc.circuit.remove_dead_ends();
        println!("multiplication circuit was created");

        println!("\x1B[32m");
        println!("{:?}", lc.circuit.outputs);
        println!("{:?}", lc.circuit.inputs);
        println!("\x1B[0m");

        let mut en = PolyEngine::new(BPolynom::empty());
        println!("\x1B[34m");
        let mut a_poly = BPolynom::empty();
        let mut b_poly = BPolynom::empty();
        let mut c_poly = BPolynom::empty();
        let a_name = String::from("A");
        let b_name = String::from("B");
        let c_name = String::from("C");
        for input in &lc.circuit.inputs {
            let vars: Vec<usize> = input.bits.iter().filter(|&& x| 
                {
                    match x { 
                        Bit::Var(l) => true,
                        _ => false

                }
            }
            ).map(|&x| {
                if let Bit::Var(l) = x {
                    l.n
                } else {
                    0
                }
            }).collect();
            let name = input.name.clone();
            let names = (0..vars.len()).map(|i| format!("{name}{i}")).collect();
            let poly = en.get_unsigned_poly(vars, names);
            if name == a_name {
                a_poly = poly.clone();
            }
            if name == b_name {
                b_poly = poly.clone();
            }
            println!("{}", poly.to_string(&en.var_names, ", "));
        }

        for output in &lc.circuit.outputs {
            let vars: Vec<usize> = output.bits.iter().filter(|&& x| 
                {
                    match x { 
                        Bit::Var(l) => true,
                        _ => false

                }
            }
            ).map(|&x| {
                if let Bit::Var(l) = x {
                    l.n
                } else {
                    0
                }
            }).collect();
            let name = output.name.clone();
            let names = (0..vars.len()).map(|i| format!("{name}{i}")).collect();
            let mut poly = BPolynom::empty();
            if name != String::from("Valid") {
                poly = en.get_unsigned_poly(vars, names);
            }
            if name == c_name {
                c_poly = poly.clone();
            }
            println!("{}", poly.to_string(&en.var_names, ", "));
        }

        let mut wires = lc.circuit.wires.clone();
        wires.sort_by(|w0, w1| {
            let ordering = lc.substitution_levels.get(&w0.out.n).unwrap_or(&DEFAULT_LEVEL).cmp(&lc.substitution_levels.get(&w1.out.n).unwrap_or(&DEFAULT_LEVEL));
            match ordering {
                std::cmp::Ordering::Equal => {
                    w1.out.level.cmp(&w0.out.level)
                }
                _ => {
                    ordering
                }
            }
        });

        wires = wires.into_iter().filter(|w| lc.substitution_levels.contains_key(&w.out.n) && *lc.substitution_levels.get(&w.out.n).unwrap_or(&DEFAULT_LEVEL) < DEFAULT_LEVEL).collect();

        wires.iter().enumerate().for_each(|(i, w)| println!("{}: {:?}", i, w));

        print!("\x1B[0m");

        println!("A Polynomial: {}", a_poly.to_string(&en.var_names, ", "));
        println!("B Polynomial: {}", b_poly.to_string(&en.var_names, ", "));
        println!("C Polynomial: {}", c_poly.to_string(&en.var_names, ", "));

        let spec_poly = c_poly;

        println!("\x1B[35m");
        print!("Spec Polynomial: ");
        println!("{}", spec_poly.to_string(&en.var_names, " ,"));
        println!("\x1B[0m");

        
        let mut sub_counter = 0;
        en.add_from_generates(spec_poly);
        println!("{}", en.p.to_string(&en.var_names, " "));

        println!("enter r to perform replacements step by step");
        while sub_counter < wires.len() {
            io::stdin().read_line(&mut input).expect("Input should be written into stdin");
            let mut command = input.trim().to_lowercase();
            if command.contains("r") {
                let wire = wires[sub_counter];
                let var = wire.out.n;
                let mut in_var1 = 0;
                let mut in_var2 = 0;
                match wire.gate {
                    And(in1, in2) => {
                        let in1_name = match lc.circuit.io_lines.get(&in1.n) {
                            Some(&nl) => { format!("{}{}", nl.name, nl.idx) }
                            _ => { format!("J{}", in1.n) }
                        };
                        let in2_name = match lc.circuit.io_lines.get(&in2.n) {
                            Some(&nl) => { format!("{}{}", nl.name, nl.idx) }
                            _ => { format!("J{}", in2.n) }
                        };
                        in_var1 = in1.n;
                        in_var2 = in2.n;
                        en.and_replace(var, in1.n, in1_name, in2.n, in2_name);
                    }
                    Or(in1, in2) => {
                        let in1_name = match lc.circuit.io_lines.get(&in1.n) {
                            Some(&nl) => { format!("{}{}", nl.name, nl.idx) }
                            _ => { format!("J{}", in1.n) }
                        };
                        let in2_name = match lc.circuit.io_lines.get(&in2.n) {
                            Some(&nl) => { format!("{}{}", nl.name, nl.idx) }
                            _ => { format!("J{}", in2.n) }
                        };
                        in_var1 = in1.n;
                        in_var2 = in2.n;
                        en.or_replace(var, in1.n, in1_name, in2.n, in2_name);
                    }
                    Xor(in1, in2) => {
                        let in1_name = match lc.circuit.io_lines.get(&in1.n) {
                            Some(&nl) => { format!("{}{}", nl.name, nl.idx) }
                            _ => { format!("J{}", in1.n) }
                        };
                        let in2_name = match lc.circuit.io_lines.get(&in2.n) {
                            Some(&nl) => { format!("{}{}", nl.name, nl.idx) }
                            _ => { format!("J{}", in2.n) }
                        };
                        in_var1 = in1.n;
                        in_var2 = in2.n;
                        en.xor_replace(var, in1.n, in1_name, in2.n, in2_name);
                    }
                    Not(in1) => {
                        let in1_name = match lc.circuit.io_lines.get(&in1.n) {
                            Some(&nl) => { format!("{}{}", nl.name, nl.idx) }
                            _ => { format!("J{}", in1.n) }
                        };
                        in_var1 = in1.n;
                        en.not_replace(var, in1.n, in1_name);
                    }
                }
                if in_var1 == usize::MAX || in_var2 == usize::MAX {
                    en.const_0_replace(usize::MAX);
                }
                if in_var1 == usize::MAX-1 || in_var2 == usize::MAX-1 {
                    en.const_1_replace(usize::MAX-1);
                }
                println!("After replacement: {}", en.p.to_string(&en.var_names, " "));
                println!("Number of Monoms: {}", en.p.poly.len());
                sub_counter += 1;
            }
            if command.contains("exit") { break; }
        }

        // println!("Final replacement of constants");
        // en.const_0_replace(usize::MAX);
        // en.const_1_replace(usize::MAX-1);
        println!("After final replacement: {}", en.p.to_string(&en.var_names, " "));


        
        println!("Would you like to exit then enter exit");
        println!("or enter restart to restart the program");

        io::stdin().read_line(&mut input).expect("Input should be written into stdin");
        let mut command = input.trim().to_lowercase();
        let exit = String::from("exit");
        let restart = String::from("restart");
        println!("Exit as {:?}", exit.as_bytes());
        loop {
            println!("Command as {:?}", command.as_bytes());
            
            if command.contains("exit") {
                println!("Leaving now... Bye");
                break 'outer;
            }
            if command.contains("restart") { break; } else {
                println!("enter a valid command either exit or restart!");
                io::stdin().read_line(&mut input).expect("Input should be written into stdin");
                command = input.trim().to_lowercase();
            }
        }

    }
        
}

#[test]
fn interactive_2_sub() {
    'outer: loop {
        println!("Interactive NewtonDividerVerification\n");
        println!("Enter how many bits the 2sub should use as input size!");

        let mut input = String::new();

        io::stdin().read_line(&mut input).expect("Input should be written into stdin");
        let mut bits = input.trim().to_lowercase();

        while let Err(_) = bits.parse::<usize>() {
            println!("Try again to enter the number of bits only enter digits and press enter.\n
            The number should be in the range 1 to {}", 32);

            input.clear();
            io::stdin().read_line(&mut input).expect("Input should be written into stdin");
            bits = input.trim().to_lowercase();
        }

        
        let mut info = DivInfo::default_newton();
        info.number_bits = bits.parse().unwrap();
        println!("You entered {}\n2sub circuit with the given input sizes will be created", info.number_bits);
    
        let mut lc = LevelizedCircuit::get_2_sub(info.number_bits);
        lc.circuit.remove_dead_ends();
        println!("2sub circuit was created");

        println!("\x1B[32m");
        println!("{:?}", lc.circuit.outputs);
        println!("{:?}", lc.circuit.inputs);
        println!("\x1B[0m");

        let mut en = PolyEngine::new(BPolynom::empty());
        println!("\x1B[34m");
        let mut a_poly = BPolynom::empty();
        let mut b_poly = BPolynom::empty();
        let mut b2_poly = BPolynom::empty();
        let mut c_poly = BPolynom::empty();
        let a_name = String::from("N");
        let b_name = String::from("D");
        let b2_name = String::from("DQ");
        let c_name = String::from("R˘");
        for input in &lc.circuit.inputs {
            let vars: Vec<usize> = input.bits.iter().filter(|&& x| 
                {
                    match x { 
                        Bit::Var(l) => true,
                        _ => false

                }
            }
            ).map(|&x| {
                if let Bit::Var(l) = x {
                    l.n
                } else {
                    0
                }
            }).collect();
            let name = input.name.clone();
            let names = (0..vars.len()).map(|i| format!("{name}{i}")).collect();
            let poly = en.get_unsigned_poly(vars, names);
            if name == a_name {
                a_poly = poly.clone();
            }
            if name == b_name {
                b_poly = poly.clone();
            }
            if name == b2_name {
                b2_poly = poly.clone();
            }
            println!("{}", poly.to_string(&en.var_names, ", "));
        }

        for output in &lc.circuit.outputs {
            let vars: Vec<usize> = output.bits.iter().filter(|&& x| 
                {
                    match x { 
                        Bit::Var(l) => true,
                        _ => false

                }
            }
            ).map(|&x| {
                if let Bit::Var(l) = x {
                    l.n
                } else {
                    0
                }
            }).collect();
            let name = output.name.clone();
            let names = (0..vars.len()).map(|i| format!("{name}{i}")).collect();
            let mut poly = BPolynom::empty();
            if name != String::from("Valid") {
                poly = en.get_2_compl_poly(vars, names);
            }
            if name == c_name {
                c_poly = poly.clone();
            }
            println!("{}", poly.to_string(&en.var_names, ", "));
        }

        let mut wires = lc.circuit.wires.clone();
        wires.sort_by(|w0, w1| {
            let ordering = lc.substitution_levels.get(&w0.out.n).unwrap_or(&DEFAULT_LEVEL).cmp(&lc.substitution_levels.get(&w1.out.n).unwrap_or(&DEFAULT_LEVEL));
            match ordering {
                std::cmp::Ordering::Equal => {
                    w1.out.level.cmp(&w0.out.level)
                }
                _ => {
                    ordering
                }
            }
        });

        wires = wires.into_iter().filter(|w| lc.substitution_levels.contains_key(&w.out.n) && *lc.substitution_levels.get(&w.out.n).unwrap_or(&DEFAULT_LEVEL) < DEFAULT_LEVEL).collect();

        wires.iter().enumerate().for_each(|(i, w)| println!("{}: {:?}", i, w));

        print!("\x1B[0m");

        println!("N Polynomial: {}", a_poly.to_string(&en.var_names, ", "));
        println!("D Polynomial: {}", b_poly.to_string(&en.var_names, ", "));
        println!("DQ Polynomial: {}", b2_poly.to_string(&en.var_names, ", "));
        println!("R˘ Polynomial: {}", c_poly.to_string(&en.var_names, ", "));

        let spec_poly = c_poly;

        println!("\x1B[35m");
        print!("Spec Polynomial: ");
        println!("{}", spec_poly.to_string(&en.var_names, " ,"));
        println!("\x1B[0m");

        
        let mut sub_counter = 0;
        en.add_from_generates(spec_poly);
        println!("{}", en.p.to_string(&en.var_names, " "));

        println!("enter r to perform replacements step by step");
        while sub_counter < wires.len() {
            io::stdin().read_line(&mut input).expect("Input should be written into stdin");
            let mut command = input.trim().to_lowercase();
            if command.contains("r") {
                let wire = wires[sub_counter];
                let var = wire.out.n;
                let mut in_var1 = 0;
                let mut in_var2 = 0;
                match wire.gate {
                    And(in1, in2) => {
                        let in1_name = match lc.circuit.io_lines.get(&in1.n) {
                            Some(&nl) => { format!("{}{}", nl.name, nl.idx) }
                            _ => { format!("J{}", in1.n) }
                        };
                        let in2_name = match lc.circuit.io_lines.get(&in2.n) {
                            Some(&nl) => { format!("{}{}", nl.name, nl.idx) }
                            _ => { format!("J{}", in2.n) }
                        };
                        in_var1 = in1.n;
                        in_var2 = in2.n;
                        en.and_replace(var, in1.n, in1_name, in2.n, in2_name);
                    }
                    Or(in1, in2) => {
                        let in1_name = match lc.circuit.io_lines.get(&in1.n) {
                            Some(&nl) => { format!("{}{}", nl.name, nl.idx) }
                            _ => { format!("J{}", in1.n) }
                        };
                        let in2_name = match lc.circuit.io_lines.get(&in2.n) {
                            Some(&nl) => { format!("{}{}", nl.name, nl.idx) }
                            _ => { format!("J{}", in2.n) }
                        };
                        in_var1 = in1.n;
                        in_var2 = in2.n;
                        en.or_replace(var, in1.n, in1_name, in2.n, in2_name);
                    }
                    Xor(in1, in2) => {
                        let in1_name = match lc.circuit.io_lines.get(&in1.n) {
                            Some(&nl) => { format!("{}{}", nl.name, nl.idx) }
                            _ => { format!("J{}", in1.n) }
                        };
                        let in2_name = match lc.circuit.io_lines.get(&in2.n) {
                            Some(&nl) => { format!("{}{}", nl.name, nl.idx) }
                            _ => { format!("J{}", in2.n) }
                        };
                        in_var1 = in1.n;
                        in_var2 = in2.n;
                        en.xor_replace(var, in1.n, in1_name, in2.n, in2_name);
                    }
                    Not(in1) => {
                        let in1_name = match lc.circuit.io_lines.get(&in1.n) {
                            Some(&nl) => { format!("{}{}", nl.name, nl.idx) }
                            _ => { format!("J{}", in1.n) }
                        };
                        in_var1 = in1.n;
                        en.not_replace(var, in1.n, in1_name);
                    }
                }
                if in_var1 == usize::MAX || in_var2 == usize::MAX {
                    en.const_0_replace(usize::MAX);
                }
                if in_var1 == usize::MAX-1 || in_var2 == usize::MAX-1 {
                    en.const_1_replace(usize::MAX-1);
                }
                println!("After replacement: {}", en.p.to_string(&en.var_names, " "));
                println!("Number of Monoms: {}", en.p.poly.len());
                println!("Number of Monoms: {}", en.p.poly.len());
                println!("\x1B[32m");
                en.print_var_occurences();
                println!("\x1B[0m");
                sub_counter += 1;
            }
            if command.contains("exit") { break; }
        }

        // println!("Final replacement of constants");
        // en.const_0_replace(usize::MAX);
        // en.const_1_replace(usize::MAX-1);
        println!("\nAfter final replacement: {}", en.p.to_string(&en.var_names, " "));


        
        println!("Would you like to exit then enter exit");
        println!("or enter restart to restart the program");

        io::stdin().read_line(&mut input).expect("Input should be written into stdin");
        let mut command = input.trim().to_lowercase();
        let exit = String::from("exit");
        let restart = String::from("restart");
        println!("Exit as {:?}", exit.as_bytes());
        loop {
            println!("Command as {:?}", command.as_bytes());
            
            if command.contains("exit") {
                println!("Leaving now... Bye");
                break 'outer;
            }
            if command.contains("restart") { break; } else {
                println!("enter a valid command either exit or restart!");
                io::stdin().read_line(&mut input).expect("Input should be written into stdin");
                command = input.trim().to_lowercase();
            }
        }

    }
        
}