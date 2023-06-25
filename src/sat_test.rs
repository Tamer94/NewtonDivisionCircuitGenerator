use num::Zero;
use varisat::{Solver, ExtendFormula};

use crate::{data::{Bit, Adder}, dividers::{IntDivResult, Method, DivInfo, Estimate, SubMethod}};
#[cfg(test)]
use crate::{data::Circuit};
use std::time::Instant;

#[test]
fn construct_cnf_for_circuit() {
    let bits = 3;
    let circuit = Circuit::get_cra(bits);
    let formula = circuit.into_cnf_tseitin();
}

#[test]
fn verify_adder() {
    let bits = 128; 
    let mut circuit = Circuit::new();
    let mut a = vec![];
    let mut b = vec![];
    let select = circuit.new_line();

    for _ in 0..bits {
        a.push(circuit.new_line());
        b.push(circuit.new_line());
    }

    // let c_in = circuit.new_line();

    let result1 = circuit.cra(a.clone(), b.clone(), select);
    let result2 = circuit.ksa(a.clone(), b.clone(), select);
    let mut result = vec![];
    for (i, bit) in result1.iter().enumerate() {
        let xored = circuit.xor(*bit, result2[i]);
        result.push(xored);
    }

    circuit.add_as_io(&a, "X", false);
    circuit.add_as_io(&b, "Y", false);
    circuit.add_as_io(&vec![select], "Cin", false);
    circuit.add_as_io(&result, "S", true);
    let (mut formula, vars) = circuit.into_cnf_tseitin();
    let mut last_clause = vec![];
    for outputs in circuit.outputs {
        for output in outputs.bits {
            if let Bit::Var(l) = output {
                // formula.add_clause(&[vars[l.n]]);
                last_clause.push(vars[l.n]);
            }
        }
    }
    formula.add_clause(&last_clause);
    let mut solver = Solver::new();
    solver.add_formula(&formula);
    let now = Instant::now();
    let result = solver.solve().unwrap();
    let elapsed = now.elapsed().as_micros();
    println!("took {elapsed} µs to solve the SAT problem");
    println!("satisfiable? {result}!");
}

#[test]
fn verify_adder_mathamatical() {
    let bits = 128; 
    let mut circuit = Circuit::new();
    let mut a = vec![];
    let mut b = vec![];
    let select = circuit.new_line();

    for _ in 0..bits {
        a.push(circuit.new_line());
        b.push(circuit.new_line());
    }

    let result1 = circuit.csa(a.clone(), b.clone(), Bit::Zero);
    let result2 = circuit.css(result1.clone(), a.clone(), Bit::Zero);
    let result = circuit.css(result2.clone(), b.clone(), Bit::Zero);

    circuit.add_as_io(&a, "X", false);
    circuit.add_as_io(&b, "Y", false);
    circuit.add_as_io(&vec![select], "Cin", false);
    circuit.add_as_io(&result, "S", true);
    let (mut formula, vars) = circuit.into_cnf_tseitin();
    let mut last_clause = vec![];
    for outputs in circuit.outputs {
        for output in outputs.bits {
            if let Bit::Var(l) = output {
                // formula.add_clause(&[vars[l.n]]);
                last_clause.push(vars[l.n]);
            }
        }
    }
    formula.add_clause(&last_clause);
    let mut solver = Solver::new();
    solver.add_formula(&formula);
    let now = Instant::now();
    let result = solver.solve().unwrap();
    let elapsed = now.elapsed();
    println!("satisfiable? {result}!");
    println!("took {} µs to solve the the satisfiability problem", elapsed.as_micros());
}

#[test]
fn verify_div() {
            // create a new circuit object to store the circuit and its stats
            
            for bits in 11..=20/*.map(|x| 0x1 << x)*/ {
                let mut circuit = Circuit::new();
                let mut info = DivInfo::default_goldschmidt();
                info.number_bits = bits;
                info.defaultadder = Adder::CRA;
                info.estimator = Estimate::Table10bit;
                info.sub_method = SubMethod::Seperate;
                info.division_method = Method::Newton;
                
                let mut divisor = vec![];
                let mut dividend = vec![];
                for _ in 0..bits {
                    dividend.push(circuit.new_line());
                }
            
                for _ in 0..bits {
                    divisor.push(circuit.new_line());
                }
            
                let IntDivResult { q, r, ok } = match info.division_method {
                    Method::Newton => { circuit.div_newton(dividend.clone(), divisor.clone(), info) }
                    Method::Goldschmidt => { circuit.goldschmidt_divider(dividend.clone(), divisor.clone(), info) }
                    Method::Restoring => { circuit.restoring_division(dividend.clone(), divisor.clone(), info) }
                };

                let result = circuit.array_mul(q, divisor.clone(), Some(r.clone()), crate::data::Adder::CRA);
                //let result = circuit.cra(result, Bit::zeroes(bits), Bit::One);
                let result2 = circuit.crs(dividend.clone(), result, Bit::Zero);
                let (c_lt, c_eq) = circuit.less_than(&r, &divisor);
                let (constraint2_temp_lt, constraint2_temp_eq) = circuit.less_than(&divisor, &dividend);
                let constraint2 = circuit.or(constraint2_temp_lt, constraint2_temp_eq);
            
                circuit.add_as_io(&dividend, "R_0", false);
                circuit.add_as_io(&divisor, "D", false);
                let mut result = vec![];
                for bit in result2 {
                   let r = circuit.and(bit, ok);
                   // let r = circuit.and(r, constraint2);
                    result.push(r);
                }
                let constraint = circuit.not(c_lt);
                let constraint = circuit.and(constraint, ok);
                let constraint = circuit.and(constraint, constraint2);
                result.push(constraint);
                // let constraint2 = circuit.and(constraint2, ok);
                // result.push(constraint2);
                circuit.add_as_io(&result, "sat", true);

                let (mut formula, vars) = circuit.into_cnf_tseitin();
                //for outputs in circuit.outputs.iter() {
                //    for output in outputs.bits.iter() {
                //        if let Bit::Var(l) = output {
                //            formula.add_clause(&[vars[l.n]]);
                //        }
                //    }
                //}
                let mut last_clause = vec![];
                for outputs in circuit.outputs {
                    for output in outputs.bits {
                        if let Bit::Var(l) = output {
                            // formula.add_clause(&[vars[l.n]]);
                            last_clause.push(vars[l.n]);
                        }
                    }
                }
                formula.add_clause(&last_clause);

                circuit.wires.clear();
                circuit.wires.shrink_to_fit();

                // println!("{:?}", formula);

                let mut solver = Solver::new();
                solver.add_formula(&formula);
                let now = Instant::now();
                let result = solver.solve().unwrap();
                let elapsed = now.elapsed();
                println!("satisfiable? {result}!");
                println!("took {} µs to solve the the satisfiability problem of size {}", elapsed.as_micros(), bits);
                println!("formula had {} clauses\n", formula.len());
                if result {
                    let input = solver.model().unwrap();
                    println!("{:?}", input);

                }
        }
}

#[test]
fn verify_div_inequality() {
            // create a new circuit object to store the circuit and its stats
            
            for bits in 1..=20/*.map(|x| 0x1 << x)*/ {
                let mut circuit = Circuit::new();
                let mut info = DivInfo::default_goldschmidt();
                info.number_bits = bits;
                info.defaultadder = Adder::CRA;
                info.estimator = Estimate::Table10bit;
                info.sub_method = SubMethod::Seperate;
                info.division_method = Method::Newton;
                
                let mut divisor = vec![];
                let mut dividend = vec![];
                for _ in 0..bits {
                    dividend.push(circuit.new_line());
                }
            
                for _ in 0..bits {
                    divisor.push(circuit.new_line());
                }
            
                let IntDivResult { q, r, ok } = match info.division_method {
                    Method::Newton => { circuit.div_newton(dividend.clone(), divisor.clone(), info) }
                    Method::Goldschmidt => { circuit.goldschmidt_divider(dividend.clone(), divisor.clone(), info) }
                    Method::Restoring => { circuit.restoring_division(dividend.clone(), divisor.clone(), info) }
                };

                let result = circuit.array_mul(q, divisor.clone(), Some(r.clone()), crate::data::Adder::CRA);
                //let result = circuit.cra(result, Bit::zeroes(bits), Bit::One);
                let result2 = circuit.crs(dividend.clone(), result, Bit::Zero);
                let (c_lt, c_eq) = circuit.less_than(&divisor, &r);
            
                circuit.add_as_io(&dividend, "R_0", false);
                circuit.add_as_io(&divisor, "D", false);
                let c_lt = circuit.and(c_lt, ok);
                let c_eq = circuit.and(c_eq, ok);
                let mut result = vec![c_lt, c_eq];

                // let constraint2 = circuit.and(constraint2, ok);
                // result.push(constraint2);
                circuit.add_as_io(&result, "sat", true);

                let (mut formula, vars) = circuit.into_cnf_tseitin();
                
                let mut last_clause = vec![];
                for outputs in circuit.outputs {
                    for output in outputs.bits {
                        if let Bit::Var(l) = output {
                            // formula.add_clause(&[vars[l.n]]);
                            last_clause.push(vars[l.n]);
                        }
                    }
                }
                formula.add_clause(&last_clause);

                circuit.wires.clear();
                circuit.wires.shrink_to_fit();

                // println!("{:?}", formula);

                let mut solver = Solver::new();
                solver.add_formula(&formula);
                let now = Instant::now();
                let result = solver.solve().unwrap();
                let elapsed = now.elapsed();
                println!("satisfiable? {result}!");
                println!("took {} µs to solve the the satisfiability problem of size {}", elapsed.as_micros(), bits);
                println!("formula had {} clauses\n", formula.len());
                if result {
                    let input = solver.model().unwrap();
                    println!("{:?}", input);

                }
        }
}

#[test]
fn verify_div_xor() {
            // create a new circuit object to store the circuit and its stats
            
            for bits in (1..=12)/*.map(|x| 0x1 << x)*/ {
                let mut circuit = Circuit::new();
                let mut info = DivInfo::default_goldschmidt();
                info.number_bits = bits;
                info.defaultadder = Adder::CRA;
                info.estimator = Estimate::Table10bit;
                info.sub_method = SubMethod::Seperate;
                info.division_method = Method::Newton;
                
                let mut divisor = vec![];
                let mut dividend = vec![];
                for _ in 0..bits {
                    dividend.push(circuit.new_line());
                }
            
                for _ in 0..bits {
                    divisor.push(circuit.new_line());
                }
            
                let IntDivResult { q: q1, r: r1, ok: ok1 } = match info.division_method {
                    Method::Newton => { circuit.div_newton(dividend.clone(), divisor.clone(), info) }
                    Method::Goldschmidt => { circuit.goldschmidt_divider(dividend.clone(), divisor.clone(), info) }
                    Method::Restoring => { circuit.restoring_division(dividend.clone(), divisor.clone(), info) }
                };

                info.division_method = Method::Restoring;

                let IntDivResult { q: q2, r: r2, ok: ok2 } = match info.division_method {
                    Method::Newton => { circuit.div_newton(dividend.clone(), divisor.clone(), info) }
                    Method::Goldschmidt => { circuit.goldschmidt_divider(dividend.clone(), divisor.clone(), info) }
                    Method::Restoring => { circuit.restoring_division(dividend.clone(), divisor.clone(), info) }
                };
            
                circuit.add_as_io(&dividend, "R_0", false);
                circuit.add_as_io(&divisor, "D", false);

                let q1 = q1.iter().map(|x| circuit.and(ok1, *x)).collect::<Vec<_>>();
                let r1 = r1.iter().map(|x| circuit.and(ok1, *x)).collect::<Vec<_>>();
                let q2 = q2.iter().map(|x| circuit.and(ok2, *x)).collect::<Vec<_>>();
                let r2 = r2.iter().map(|x| circuit.and(ok2, *x)).collect::<Vec<_>>();
                
                let mut result = vec![];
                for (i, bit) in q1.iter().enumerate() {
                    let temp = circuit.xor(*bit, q2[i]);
                    result.push(temp);
                }
                for (i, bit) in r1.iter().enumerate() {
                    let mut temp = circuit.xor(*bit, r2[i]);
                    result.push(temp);
                }
                circuit.add_as_io(&result, "sat", true);

                let (mut formula, vars) = circuit.into_cnf_tseitin();
                let mut last_clause = vec![];
                for outputs in circuit.outputs {
                    for output in outputs.bits {
                        if let Bit::Var(l) = output {
                            // formula.add_clause(&[vars[l.n]]);
                            last_clause.push(vars[l.n]);
                        }
                    }
                }
                formula.add_clause(&last_clause);

                circuit.wires.clear();
                circuit.wires.shrink_to_fit();

                // println!("{:?}", formula);

                let mut solver = Solver::new();
                solver.add_formula(&formula);
                let now = Instant::now();
                let result = solver.solve().unwrap();
                let elapsed = now.elapsed();
                println!("satisfiable? {result}!");
                println!("took {} µs to solve the the satisfiability problem of size {}", elapsed.as_micros(), bits);
                println!("formula had {} clauses\n", formula.len());
                if result {
                    let input = solver.model().unwrap();
                    println!("{:?}", input);

                }
        }
}