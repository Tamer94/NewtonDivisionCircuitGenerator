use crate::data::{Bit, Circuit, Gate};
use varisat::{self, CnfFormula, ExtendFormula, Lit};
use std::collections::HashMap;

impl Circuit {
    pub fn into_cnf_tseitin(&self) -> (CnfFormula, Vec<Lit>) {
        let mut formula = CnfFormula::new();
        let count_input_bits = self.inputs.iter().fold(0, |acc, io| acc + io.bits.len());
        let number_vars = self.wires.len() + count_input_bits;
        // println!("{:?}", self.wires);
        let vars: Vec<_> = formula.new_lit_iter(number_vars).collect();
        for wire in self.wires.iter() {
            match wire.gate {
                Gate::And(l1, l2) => {
                    formula.add_clause(&[!vars[wire.out.n], vars[l1.n]]);
                    formula.add_clause(&[!vars[wire.out.n], vars[l2.n]]);
                    formula.add_clause(&[vars[wire.out.n], !vars[l1.n], !vars[l2.n]]);
                }
                Gate::Or(l1, l2) => {
                    formula.add_clause(&[vars[wire.out.n], !vars[l1.n]]);
                    formula.add_clause(&[vars[wire.out.n], !vars[l2.n]]);
                    formula.add_clause(&[!vars[wire.out.n], vars[l1.n], vars[l2.n]]);
                }
                Gate::Xor(l1, l2) => {
                    formula.add_clause(&[!vars[wire.out.n], vars[l1.n], vars[l2.n]]);
                    formula.add_clause(&[!vars[wire.out.n], !vars[l1.n], !vars[l2.n]]);
                    formula.add_clause(&[vars[wire.out.n], vars[l1.n], !vars[l2.n]]);
                    formula.add_clause(&[vars[wire.out.n], !vars[l1.n], vars[l2.n]]);
                }
                Gate::Not(l1) => {
                    formula.add_clause(&[vars[wire.out.n], vars[l1.n]]);
                    formula.add_clause(&[!vars[wire.out.n], !vars[l1.n]]);
                }
            }
            // println!("{:?}", formula);
        }
        (formula, vars)
    }
}