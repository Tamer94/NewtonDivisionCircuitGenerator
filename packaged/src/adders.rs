use crate::data::{Bit, Bit::One, Bit::Zero, Circuit};
use crate::primitives::*;

#[derive(Clone, Copy, Debug)]
pub struct Sum2 {
    pub c: Bit,
    pub s: Bit,
}

#[derive(Clone, Copy, Debug)]
pub struct SumO2 {
    pub c: Bit,
    pub s: Bit,
}

#[derive(Clone, Copy, Debug)]
pub struct PropagateGenerate {
    pub p: Bit,
    pub g: Bit,
}

#[allow(dead_code)]
pub struct Sum3 {
    pub c1: Bit,
    pub c0: Bit,
    pub s: Bit,
}

// note: all adders fail when both input vectors are empty
impl Circuit {
    #[inline(always)]
    pub fn half_adder(&mut self, i1: Bit, i2: Bit) -> Sum2 {
        let s = self.xor(i1, i2);
        let c = self.and(i1, i2);
        Sum2 { c: c, s: s }
    }

    #[inline(always)]
    pub fn full_adder(&mut self, i1: Bit, i2: Bit, c: Bit) -> Sum2 {
        let sum1 = self.half_adder(i1, i2);
        let sum2 = self.half_adder(sum1.s, c);
        let c = self.or(sum1.c, sum2.c);
        Sum2 { c: c, s: sum2.s }
    }

    #[inline(always)]
    pub fn a_dot_operator(
        &mut self,
        p: Bit,
        g: Bit,
        p_prev: Bit,
        g_prev: Bit,
    ) -> PropagateGenerate {
        let p_g = PropagateGenerate {
            p: self.and(p, p_prev),
            g: {
                let temp = self.and(p, g_prev);
                self.or(temp, g)
            },
        };
        p_g
    }

    #[inline(always)]
    pub fn a_final(&mut self, p: Bit, g: Bit, c_in: Bit) -> Sum2 {
        let sum = Sum2 {
            c: g,
            s: self.xor(p, c_in),
        };
        sum
    }

    #[inline(always)]
    pub fn half_sub(&mut self, minuend: Bit, subtrahend: Bit) -> Sum2 {
        let s = self.xor(minuend, subtrahend);
        let temp = self.not(minuend);
        let c = self.and(temp, subtrahend);
        Sum2 { c: c, s: s }
    }

    #[inline(always)]
    pub fn full_sub(&mut self, minuend: Bit, subtrahend: Bit, c_in: Bit) -> Sum2 {
        let sum1 = self.half_sub(minuend, subtrahend);
        let sum2 = self.half_sub(sum1.s, c_in);
        let c_out = self.or(sum2.c, sum1.c);
        Sum2 {
            c: c_out,
            s: sum2.s,
        }
    }

    pub fn cra(&mut self, s1: Vec<Bit>, s2: Vec<Bit>, c_in: Bit) -> Vec<Bit> {
        let mut c_in = c_in;
        let n = s1.len().max(s2.len());
        let mut s_vector = Vec::with_capacity(n);
        let mut last_carry = Zero;
        for idx in 0..n {
            let Sum2 { c, s } = self.full_adder(s1.get_or(idx, Zero), s2.get_or(idx, Zero), c_in);
            c_in = c;
            s_vector.push(s);
            last_carry = c;
        }
        s_vector.push(last_carry);
        s_vector
    }

    pub fn get_cra(bits: usize) -> Circuit {
        let mut circuit = Circuit::new();
        let mut a = vec![];
        let mut b = vec![];
        let select = circuit.new_line();

        for _ in 0..bits {
            a.push(circuit.new_line());
            b.push(circuit.new_line());
        }

        let result = circuit.cra(a.clone(), b.clone(), select);

        circuit.add_as_io(&a, "X", false);
        circuit.add_as_io(&b, "Y", false);
        circuit.add_as_io(&vec![select], "Cin", false);
        circuit.add_as_io(&result, "S", true);
        circuit
    }

    pub fn get_csa(bits: usize) -> Circuit {
        let mut circuit = Circuit::new();
        let mut a = vec![];
        let mut b = vec![];
        let select = circuit.new_line();

        for _ in 0..bits {
            a.push(circuit.new_line());
            b.push(circuit.new_line());
        }

        let result = circuit.csa(a.clone(), b.clone(), select);

        circuit.add_as_io(&a, "X", false);
        circuit.add_as_io(&b, "Y", false);
        circuit.add_as_io(&vec![select], "Cin", false);
        circuit.add_as_io(&result, "S", true);
        circuit
    }

    pub fn get_ksa(bits: usize) -> Circuit {
        let mut circuit = Circuit::new();
        let mut a = vec![];
        let mut b = vec![];
        let select = circuit.new_line();

        for _ in 0..bits {
            a.push(circuit.new_line());
            b.push(circuit.new_line());
        }

        let result = circuit.ksa(a.clone(), b.clone(), select);

        circuit.add_as_io(&a, "X", false);
        circuit.add_as_io(&b, "Y", false);
        circuit.add_as_io(&vec![select], "Cin", false);
        circuit.add_as_io(&result, "S", true);
        circuit
    }

    pub fn crs(&mut self, minuend: Vec<Bit>, subtrahend: Vec<Bit>, c_in: Bit) -> Vec<Bit> {
        let mut c_in = c_in;
        let n = minuend.len().max(subtrahend.len());
        let mut s_vector = Vec::with_capacity(n);
        let mut last_carry = Zero;
        for idx in 0..n {
            let Sum2 { c, s } = self.full_sub(minuend.get_or(idx, Zero), subtrahend.get_or(idx, Zero), c_in);
            c_in = c;
            s_vector.push(s);
            last_carry = c;
        }
        s_vector.push(last_carry);
        s_vector
    }

    pub fn csa(&mut self, s1: Vec<Bit>, s2: Vec<Bit>, c_in: Bit) -> Vec<Bit> {
        let mut c_in = c_in;
        let max_len = s1.len().max(s2.len());
        let mut s_vector = Vec::with_capacity(max_len + 1);
        let mut collector = Vec::with_capacity(((max_len * 2) as f64).sqrt() as usize + 1);
        let summands = (0..max_len)
            .map(|i| (s1.get_or(i, Zero), s2.get_or(i, Zero)))
            .collect::<Vec<(Bit, Bit)>>();

        let mut start_size = 2;
        let mut free_place = start_size;
        let mut done = 0;
        let mut first_iteration = true;
        for &pair in &summands {
            if free_place == 0 {
                start_size = (start_size + 1).min(summands.len() - done);
                free_place = start_size;
                let (collected1, collected2): (Vec<Bit>, Vec<Bit>) = collector.iter().cloned().unzip();
                if first_iteration {
                    let s1 = self.cra(collected1.clone(), collected2.clone(), c_in);
                    for &bit in s1.iter().take(s1.len() - 1) {
                        s_vector.push(bit);
                    }
                    c_in = *s1.iter().next_back().unwrap_or(&Zero);
                    first_iteration = false;
                } else {
                    let s1 = self.cra(collected1.clone(), collected2.clone(), One);
                    let s2 = self.cra(collected1.clone(), collected2.clone(), Zero);
                    let r = self.mux_n_1(&s1, &s2, c_in);
                    for &bit in r.iter().take(r.len() - 1) {
                        s_vector.push(bit);
                    }
                    c_in = *r.iter().next_back().unwrap_or(&Zero);
                }
                collector.clear();
            }
            collector.push(pair);
            free_place -= 1;
            done += 1;
        }

        let (collected1, collected2): (Vec<Bit>, Vec<Bit>) = collector.iter().cloned().unzip();
        if first_iteration {
            let s1 = self.cra(collected1.clone(), collected2.clone(), c_in);
            for &bit in s1.iter().take(s1.len() - 1) {
                s_vector.push(bit);
            }
            c_in = *s1.iter().next_back().unwrap_or(&Zero);
        } else {
            let s1 = self.cra(collected1.clone(), collected2.clone(), One);
            let s2 = self.cra(collected1.clone(), collected2.clone(), Zero);
            let r = self.mux_n_1(&s1, &s2, c_in);
            for &bit in r.iter().take(r.len() - 1) {
                s_vector.push(bit);
            }
            c_in = *r.last().unwrap_or(&Zero);
        }
        s_vector.push(c_in);
        s_vector
    }

    pub fn css(&mut self, s1: Vec<Bit>, s2: Vec<Bit>, c_in: Bit) -> Vec<Bit> {
        let mut c_in = c_in;
        let max_len = s1.len().max(s2.len());
        let mut s_vector = Vec::with_capacity(max_len + 1);
        let mut collector = Vec::with_capacity(((max_len * 2) as f64).sqrt() as usize + 1);
        let summands = (0..max_len)
            .map(|i| (s1.get_or(i, Zero), s2.get_or(i, Zero)))
            .collect::<Vec<(Bit, Bit)>>();

        let mut start_size = 2;
        let mut free_place = start_size;
        let mut done = 0;
        let mut first_iteration = true;
        for &pair in &summands {
            if free_place == 0 {
                start_size = (start_size + 1).min(summands.len() - done);
                free_place = start_size;
                let (collected1, collected2): (Vec<Bit>, Vec<Bit>) = collector.iter().cloned().unzip();
                if first_iteration {
                    let s1 = self.crs(collected1.clone(), collected2.clone(), c_in);
                    for &bit in s1.iter().take(s1.len() - 1) {
                        s_vector.push(bit);
                    }
                    c_in = *s1.iter().next_back().unwrap_or(&Zero);
                    first_iteration = false;
                } else {
                    let s1 = self.crs(collected1.clone(), collected2.clone(), One);
                    let s2 = self.crs(collected1.clone(), collected2.clone(), Zero);
                    let r = self.mux_n_1(&s1, &s2, c_in);
                    for &bit in r.iter().take(r.len() - 1) {
                        s_vector.push(bit);
                    }
                    c_in = *r.iter().next_back().unwrap_or(&Zero);
                }
                collector.clear();
            }
            collector.push(pair);
            free_place -= 1;
            done += 1;
        }

        let (collected1, collected2): (Vec<Bit>, Vec<Bit>) = collector.iter().cloned().unzip();
        if first_iteration {
            let s1 = self.crs(collected1.clone(), collected2.clone(), c_in);
            for &bit in s1.iter().take(s1.len() - 1) {
                s_vector.push(bit);
            }
            c_in = *s1.iter().next_back().unwrap_or(&Zero);
        } else {
            let s1 = self.crs(collected1.clone(), collected2.clone(), One);
            let s2 = self.crs(collected1.clone(), collected2.clone(), Zero);
            let r = self.mux_n_1(&s1, &s2, c_in);
            for &bit in r.iter().take(r.len() - 1) {
                s_vector.push(bit);
            }
            c_in = *r.last().unwrap_or(&Zero);
        }
        s_vector.push(c_in);
        s_vector
    }

    #[allow(dead_code)]
    pub fn get_negative_2s_complement(&mut self, mut number: Vec<Bit>) -> Vec<Bit> {
        for bit in &mut number {
            *bit = self.not(*bit);
        }

        let zeroes = Bit::zeroes(number.len());
        self.csa(number, zeroes, One)
    }

    pub fn ksa(&mut self, s1: Vec<Bit>, s2: Vec<Bit>, c_in: Bit) -> Vec<Bit> {
        let n = s1.len().max(s2.len());
        let mut s = Vec::with_capacity(n);
        let depth = (n as f32).log2().ceil() as usize;
        let mut p_g: Vec<Vec<PropagateGenerate>> = Vec::with_capacity(depth);
        for _ in 0..n {
            p_g.push(Vec::new());
        }
        let mut p;
        let mut g;
        let mut first_entry = true;

        // least significant bit
        for column in 0..n {
            let (i1, i2) = (s1.get_or(column, Zero), s2.get_or(column, Zero));
            p = self.xor(i1, i2);
            g = self.and(i1, i2);
            if c_in != Zero && first_entry {
                let temp = self.and(p, c_in);
                g = self.or(temp, g);
            }
            p_g[column].push(PropagateGenerate { p, g });
            first_entry = false;

            for row in 0..depth {
                let (c_idx, overflow) = column.overflowing_sub(2_usize.pow(row as u32));
                if overflow {
                    let copy = p_g[column][row];
                    p_g[column].push(copy);
                    continue;
                }
                let PropagateGenerate { p, g } = p_g[column][row];
                let PropagateGenerate {
                    p: p_prev,
                    g: g_prev,
                } = p_g[c_idx][row];
                p_g[column].push(self.a_dot_operator(p, g, p_prev, g_prev));
            }
        }

        let mut c_in = c_in;
        for column in 0..n {
            let PropagateGenerate { p: _, g } = p_g[column][depth];
            let PropagateGenerate { p, g: _ } = p_g[column][0];
            if column > 0 {
                c_in = p_g[column - 1][depth].g;
            } else {
                c_in = c_in;
            }
            s.push(self.a_final(p, g, c_in).s);
        }
        s.push(p_g[s.len() - 1][depth].g);

        s
    }

    #[allow(dead_code)]
    pub fn kss(&mut self, minuend: Vec<Bit>, subtrahend: Vec<Bit>, c_in: Bit) -> Vec<Bit> {
        let n = minuend.len().max(subtrahend.len());
        let mut s = Vec::with_capacity(n);
        let depth = (n as f32).log2().ceil() as usize;
        let mut p_g: Vec<Vec<PropagateGenerate>> = Vec::with_capacity(depth);
        for _ in 0..n {
            p_g.push(Vec::new());
        }
        let mut p;
        let mut g;
        let mut first_entry = true;

        // least significant bit
        for column in 0..n {
            let (i1, i2) = (minuend.get_or(column, Zero), subtrahend.get_or(column, Zero));
            let i1_not = self.not(i1);
            p = self.or(i1_not, i2); // unsure whether to use i1_not or i1 here
            g = self.and(i1_not, i2);
            if c_in == One && first_entry {
                let temp = self.and(p, One);
                g = self.or(temp, g);
            }
            p_g[column].push(PropagateGenerate { p, g });
            first_entry = false;

            for row in 0..depth {
                let (c_idx, overflow) = column.overflowing_sub(2_usize.pow(row as u32));
                if overflow {
                    let copy = p_g[column][row];
                    p_g[column].push(copy);
                    continue;
                }
                let PropagateGenerate { p, g } = p_g[column][row];
                let PropagateGenerate {
                    p: p_prev,
                    g: g_prev,
                } = p_g[c_idx][row];
                p_g[column].push(self.a_dot_operator(p, g, p_prev, g_prev));
            }
        }

        let mut c_in = c_in;
        for column in 0..n {
            let p = self.xor(minuend.get_or(column, Zero), subtrahend.get_or(column, Zero));
            if column > 0 {
                c_in = p_g[column - 1][depth].g;
            }
            let s_final = self.xor(p, c_in);
            s.push(s_final);
        }
        s.push(p_g[s.len() - 1][depth].g);

        s
    }
}
