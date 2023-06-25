use crate::data::{Bit, Circuit, Gate};
use biodivine_lib_bdd::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct NamedBdd {
    pub bdd : Bdd,
    pub name : String,
}

impl Circuit {
    pub fn get_bdds(&self, input_bits: usize) -> Vec<NamedBdd> {
        let mut gates = self.wires.clone();
        gates.sort_by(|a, b| a.out.level.cmp(&b.out.level));
        println!("{:?}", gates);
        let count_input_bits = self.inputs.iter().fold(0, |acc, io| acc + io.bits.len());
        let set = BddVariableSet::new_anonymous(count_input_bits as u16);
        let v = set.variables();
        let mut bdds = HashMap::<usize, Bdd>::new();
        // let mut input_constrain = set.mk_false();
        for input in self.inputs.iter() {
            for b in input.bits.iter() {
                if let Bit::Var(l) = b {
                    let bdd = set.mk_var(v[l.n]);
                    println!("{}", bdd);
                    bdds.insert(l.n, bdd);
                }
            }
        }
        // for i in 0..input_bits {
        //     // let bdd = set.mk_var(v[input_bits - i - 1]);
        //     let bdd = set.mk_var(v[i]);
        //     // input_constrain = input_constrain.or(&bdd);
        //     println!("{}", bdd);
        //     bdds.insert(i, bdd);
        // }



        for gate in gates {
            match gate.gate {
                Gate::And(l1, l2) => {
                    let b1 = bdds.get(&l1.n).unwrap();
                    let b2 = bdds.get(&l2.n).unwrap();
                    let b_result = b1.and(b2);
                    bdds.insert(gate.out.n, b_result);
                }
                Gate::Or(l1, l2) => {
                    let b1 = bdds.get(&l1.n).unwrap();
                    let b2 = bdds.get(&l2.n).unwrap();
                    let b_result = b1.or(b2);
                    bdds.insert(gate.out.n, b_result);
                }
                Gate::Xor(l1, l2) => {
                    let b1 = bdds.get(&l1.n).unwrap();
                    let b2 = bdds.get(&l2.n).unwrap();
                    let b_result = b1.xor(b2);
                    bdds.insert(gate.out.n, b_result);
                }
                Gate::Not(l1) => {
                    let b1 = bdds.get(&l1.n).unwrap();
                    let b_result = b1.not();
                    bdds.insert(gate.out.n, b_result);
                }
            }
        }

        let mut output_bdds = vec![];
        for io in self.outputs.iter() {
            for (idx, bit) in io.bits.iter().enumerate() {
                if let Bit::Var(l) = bit {
                    let b = bdds.remove(&l.n).unwrap();
                    // b = b.and(&input_constrain);
                    let name = format!("{}{}", io.name, idx);
                    output_bdds.push(NamedBdd {bdd: b, name});
                }
            }
        }
        output_bdds
    }

    pub fn get_lzc_bdd(&self, bit_idx: usize, out_active_low: bool, apply_not_all_zero_constrain: bool) -> NamedBdd {
        let count_input_bits = self.inputs.iter().fold(0, |acc, io| acc + io.bits.len());
        let set = BddVariableSet::new_anonymous(count_input_bits as u16);
        let v = set.variables();
        let mut bdds = vec![];
        let mut input_constrain = set.mk_false();
        for i in 0..count_input_bits {
            let bdd = set.mk_var(v[i]);
            input_constrain = input_constrain.or(&bdd);
            bdds.push(bdd);
        }
        input_constrain = input_constrain.not();

        let count_that_many_zeros = 2_usize.pow(bit_idx as u32);
        // bdd for most significant input bit
        let mut bdd_stack = vec![];
        let mut count_zeros;
        //for i in (0..(count_input_bits - 1)).rev() {
        //    let current_var_index = i;
        //    decision_stack.push((current_var_index, 0));
        //    while !decision_stack.is_empty() {
        //        let (var_idx, counted_zeros) = decision_stack.pop().unwrap();
        //    }
        //}

        for i in 0..count_input_bits {
            count_zeros = count_input_bits - i - 1;
            let bdd = set.mk_var(v[i]);
            let mut f;
            let mut g;
            if (count_zeros / count_that_many_zeros) % 2 == 1 && (count_zeros / count_that_many_zeros) != 0 {
                f = set.mk_true();

            } else {
                f = set.mk_false();
            }

            f = f.and(&bdd);

            if i == 0 && count_that_many_zeros != count_input_bits {
                g = set.mk_false();
            } else if count_that_many_zeros == count_input_bits && i == 0 {
                g = set.mk_true();
            } else {
                g = bdd_stack.pop().unwrap();
            }

            g = g.and(&bdd.not());


            let bdd = g.or(&f);

            bdd_stack.push(bdd);
            
        }

        println!("constrain {}", input_constrain);

        let mut return_bdd = bdd_stack.pop().unwrap();
        if apply_not_all_zero_constrain {
            return_bdd = return_bdd.or(&input_constrain);
        }

        if out_active_low {
            return_bdd = return_bdd.not();
        }

        NamedBdd { bdd: return_bdd, name: format!("lzc{}", bit_idx) }
    }
}