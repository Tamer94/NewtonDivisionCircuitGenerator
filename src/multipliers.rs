use crate::{data::{Bit, Bit::Zero, Circuit, Adder, LevelizedCircuit}, cli::CircuitKind, dividers::SubMethod};
use std::{collections::VecDeque};

impl Circuit {
    pub fn umul_dadda(
        &mut self,
        m1: Vec<Bit>,
        m2: Vec<Bit>,
        s: Option<Vec<Bit>>,
        adder: Adder
    ) -> Vec<Bit> {
        let s = s.unwrap_or(Vec::new());
        let (n1, n2) = (m1.len(), m2.len());
        let max_rows = n1.min(n2);
        let columns = if s.len() >= n1 + n2 {
            s.len()
        } else {
            n1 + n2 - 1
        };

        let mut summands: Vec<VecDeque<Bit>> = Vec::with_capacity((columns) as usize);
        for _ in 0..(columns) {
            summands.push(VecDeque::with_capacity(max_rows));
        }

        for a in 0..n1 {
            for b in 0..n2 {
                let s = self.and(m1[a], m2[b]);
                summands[a + b].push_back(s);
            }
        }

        for idx in 0..s.len() {
            summands[idx].push_back(s[idx]);
        }

        let mut sorted = self
            .info
            .mul_numbers_map
            .clone()
            .into_iter()
            .collect::<Vec<(usize, usize)>>();
        sorted.sort();

        let mut b_idx = *match self.info.mul_numbers_map.get(&max_rows) {
            Some(u) => u,
            None => {
                self.info.update_for_value(max_rows);
                self.info
                    .mul_numbers_map
                    .get(&max_rows)
                    .expect("this number should have been already inserted into the map!")
            }
        };
        let mut boundry = self.info.list_mul_numbers[b_idx];

        loop {
            for i in 0..summands.len() {
                while summands[i].len() > boundry {
                    let remaining = summands[i].len() - boundry;
                    let s1 = summands[i].pop_back().expect("VecDeque shouldnt be empty");
                    let s2 = summands[i].pop_back().expect("VecDeque shouldnt be empty");
                    let s_new = if remaining > 1 {
                        let s3 = summands[i].pop_back().expect("VecDeque shouldnt be empty");
                        self.full_adder(s1, s2, s3)
                    } else {
                        self.half_adder(s1, s2)
                    };

                    summands[i].push_front(s_new.s);
                    summands[i + 1].push_front(s_new.c);
                }
            }

            if b_idx == 0 {
                break;
            }
            b_idx -= 1;
            boundry = self.info.list_mul_numbers[b_idx];
        }

        let mut s1 = Vec::new();
        let mut s2 = Vec::new();
        for summand in &mut summands {
            let l1 = summand.pop_front().unwrap_or(Zero);
            let l2 = summand.pop_front().unwrap_or(Zero);
            s1.push(l1);
            s2.push(l2);
        }

        // call adder here
        adder.add(self, s1, s2, Zero)
    }

    pub fn usquare_dadda(&mut self, m1: Vec<Bit>, ignore: usize, adder: Adder) -> Vec<Bit> {
        let n1 = m1.len();
        let new_len = 2 * n1;
        let max_rows = (n1 / 2) + 1;
        let mut summands: Vec<VecDeque<Bit>> = Vec::with_capacity((new_len) as usize);
        for _ in 0..(new_len - 1) {
            summands.push(VecDeque::with_capacity(max_rows));
        }

        for a in 0..n1 {
            for b in a..n1 {
                let s = self.and(m1[a], m1[b]);
                if a == b {
                    summands[a + b].push_back(s);
                } else {
                    summands[a + b + 1].push_back(s);
                }
            }
        }

        for i in (new_len - ignore)..(new_len - 1) {
            summands[i].clear();
        }

        let mut sorted = self
            .info
            .mul_numbers_map
            .clone()
            .into_iter()
            .collect::<Vec<(usize, usize)>>();
        sorted.sort();

        let mut b_idx = *match self.info.mul_numbers_map.get(&max_rows) {
            Some(u) => u,
            None => {
                self.info.update_for_value(max_rows);
                self.info
                    .mul_numbers_map
                    .get(&max_rows)
                    .expect("this number should have been already inserted into the map!")
            }
        };
        let mut boundry = self.info.list_mul_numbers[b_idx];
        loop {
            for i in 0..(summands.len() - 1) {
                while summands[i].len() > boundry {
                    let remaining = summands[i].len() - boundry;
                    let s1 = summands[i].pop_back().expect("VecDeque shouldnt be empty");
                    let s2 = summands[i].pop_back().expect("VecDeque shouldnt be empty");
                    let s_new = if remaining > 1 {
                        let s3 = summands[i].pop_back().expect("VecDeque shouldnt be empty");
                        self.full_adder(s1, s2, s3)
                    } else {
                        self.half_adder(s1, s2)
                    };

                    summands[i].push_front(s_new.s);
                    summands[i + 1].push_front(s_new.c);
                }
            }

            if b_idx == 0 {
                break;
            }
            b_idx -= 1;
            boundry = self.info.list_mul_numbers[b_idx];
        }

        let mut s1 = Vec::new();
        let mut s2 = Vec::new();
        for summand in &mut summands {
            let l1 = summand.pop_front().unwrap_or(Zero);
            let l2 = summand.pop_front().unwrap_or(Zero);
            s1.push(l1);
            s2.push(l2);
        }

        // call adder here
        let mut r = adder.add(self, s1, s2, Zero);

        // needs to get fixed inside adder Option to not compute last carry
        if ignore > 0 {
            r.truncate(new_len - ignore);
        }
        r
    }

    // computes: 
    // result = minuend - factor1 * factor2 
    // in a fused multiplication circuit
    // analog to fused-multiply-add
    #[allow(dead_code)]
    pub fn fused_mul_subtraction(&mut self, mut minuend: Vec<Bit>, f1: Vec<Bit>, f2: Vec<Bit>, fill_from_least_significant: bool) -> Vec<Bit> {
        let final_number_bits = f1.len() + f2.len();
        // if minuend.len() > final_number_bits {
        //     panic!("minuend has not the right number of bits! to many!");
        // }
        let (mut number, overflowed) = final_number_bits.overflowing_sub( minuend.len());
        if overflowed {
            number = 0;
        }
        let mut tail = Bit::zeroes(number);
        if fill_from_least_significant {
            tail.append(&mut minuend);
            minuend = tail;
        } else {
            minuend.append(&mut tail);
        }
        self.not_all(&mut minuend);
        let mut p = self.umul_dadda(f1, f2, Some(minuend), Adder::KSA);
        p.pop(); //#fixme! there seem to be certain bits lengths of the input vectors where the last bit is messed up
        self.not_all(&mut p);
        p
    } 

    // naive version of an array multiplier depth O(n)
    #[allow(dead_code)]
    pub fn array_mul(&mut self, m1: Vec<Bit>, m2: Vec<Bit>, s: Option<Vec<Bit>>, adder: Adder) -> Vec<Bit> {
        let n = m1.len() + m2.len();
        let s = s.unwrap_or(Vec::new());
        let mut product = Bit::zeroes(n);

        let mut sum;
        let mut c_in1;
        let mut c_in2;
        let mut sums = Bit::zeroes(n);
        let mut carrys = Vec::with_capacity(n);
        for _ in 0..n {
            carrys.push(Vec::with_capacity(2));
        }

        // counter counts how many of the summand's s bits we have used in case a summand s was given as input
        // let mut counter = 0;
        for idx1 in 0..m1.len() {
            for idx2 in 0..m2.len() {
                sum = sums[idx1 + idx2];
                c_in1 = carrys[idx1 + idx2].pop().unwrap_or(Zero);
                c_in2 = carrys[idx1 + idx2].pop().unwrap_or(Zero);
                if sum == Zero && c_in2 != Zero {
                    sum = c_in2;
                }

                let partial_product = self.and(m1[idx1], m2[idx2]);
                let intermediate_sum = self.full_adder(partial_product, sum, c_in1);
                sums[idx1 + idx2] = intermediate_sum.s;
                carrys[idx1 + idx2 + 1].push(intermediate_sum.c);
            }
        }

        // collect result bits and carry_out
        for idx in 0..n {
            product[idx] = sums[idx];
        }
        product[n - 1] = carrys[n - 1].pop().unwrap_or(Zero);
        if s.len() > 0 {
            let new_len = n.max(s.len());
            let mut s1 = Vec::with_capacity(new_len);
            let mut s2 = Vec::with_capacity(new_len);
            for idx in 0..new_len {
                s1.push(*s.get(idx).unwrap_or(&Zero));
                s2.push(*product.get(idx).unwrap_or(&Zero));
            }
            product = adder.add(self, s1, s2, Zero);
        }

        product
    }

    pub fn get_umul(bits: usize, kind: CircuitKind, fused: SubMethod, adder: Adder) -> Circuit {
        let mut circuit = Circuit::new();
        let mut fac1 = Vec::with_capacity(bits);
        let mut fac2 = Vec::with_capacity(bits);
        let mut summand = Vec::with_capacity(bits);

        for _ in 0..bits {
            fac1.push(circuit.new_line());
        }

        if kind != CircuitKind::SquareDadda {
            for _ in 0..bits {
                fac2.push(circuit.new_line());
            }
        }

        if fused == SubMethod::Fused {
            for _ in 0..bits {
                summand.push(circuit.new_line());
            }
        }
        let mut result = vec![];

        match kind {
            CircuitKind::SquareDadda => { 
                result = circuit.usquare_dadda(fac1.clone(), 0, adder);
            }
            CircuitKind::MulDadda => {
                if fused == SubMethod::Fused {
                    result = circuit.umul_dadda(fac1.clone(), fac2.clone(), Some(summand.clone()), adder);
                } else {
                    result = circuit.umul_dadda(fac1.clone(), fac2.clone(), None, adder);
                }
            }
            CircuitKind::ArrayMul => {
                if fused == SubMethod::Fused {
                    result = circuit.array_mul(fac1.clone(), fac2.clone(), Some(summand.clone()), adder);
                } else {
                    result = circuit.array_mul(fac1.clone(), fac2.clone(), None, adder);
                }
            }

            _ => {}
        }

        circuit.add_as_io(&fac1, "X", false);
        if kind != CircuitKind::SquareDadda {
            circuit.add_as_io(&fac2, "Y", false);
        }
        if fused == SubMethod::Fused {
            circuit.add_as_io(&summand, "Z", false);
        }

        circuit.add_as_io(&result, "P", true);

        circuit
    }
}

impl LevelizedCircuit {
    // naive version of an array multiplier depth O(n)
    #[allow(dead_code)]
    pub fn mul(&mut self, m1: Vec<Bit>, m2: Vec<Bit>, s: Option<Vec<Bit>>, adder: Adder) -> Vec<Bit> {
        let n = m1.len() + m2.len();
        let s = s.unwrap_or(Vec::new());
        let mut product = Bit::zeroes(n);

        let mut sum;
        let mut c_in1;
        let mut c_in2;
        let mut sums = Bit::zeroes(n);
        let mut carrys = Vec::with_capacity(n);
        for _ in 0..n {
            carrys.push(Vec::with_capacity(2));
        }

        // counter counts how many of the summand's s bits we have used in case a summand s was given as input
        // let mut counter = 0;
        for idx1 in 0..m1.len() {
            for idx2 in 0..m2.len() {
                sum = sums[idx1 + idx2];
                c_in1 = carrys[idx1 + idx2].pop().unwrap_or(Zero);
                c_in2 = carrys[idx1 + idx2].pop().unwrap_or(Zero);
                if sum == Zero && c_in2 != Zero {
                    sum = c_in2;
                }

                let partial_product = self.and(m1[idx1], m2[idx2]);
                let intermediate_sum = self.full_adder(partial_product, sum, c_in1);
                sums[idx1 + idx2] = intermediate_sum.s;
                carrys[idx1 + idx2 + 1].push(intermediate_sum.c);
            }
        }

        // collect result bits and carry_out
        for idx in 0..n {
            product[idx] = sums[idx];
        }
        product[n - 1] = carrys[n - 1].pop().unwrap_or(Zero);

        product
    }
}
