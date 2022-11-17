mod data;
use data::data::{Circuit, Gate, Line, Out, Output, Stats, Wire, IO};

pub struct Sum2 {
    pub c: usize,
    pub s: usize,
}

pub struct PropagetGenerate {
    pub p: usize,
    pub g: usize,
}

pub struct Sum3 {
    pub c1: usize,
    pub c0: usize,
    pub s: usize,
}
impl Circuit {
    #[inline(always)]
    pub fn xor(&mut self, i1: usize, i2: usize) -> usize {
        let mut gate = Gate::Xor(i1, i2);
        let idx = self.add_line(self.get_next_level(i1, i2));
        self.wires.push(Wire::new(
            gate,
            idx
        ));
        idx
    }

    #[inline(always)]
    pub fn and(&mut self, i1: usize, i2: usize) -> usize {
        let mut gate = Gate::And(i1, i2);
        let idx = self.add_line(self.get_next_level(i1, i2));
        self.wires.push(Wire::new(
            gate,
            idx
        ));
        idx
    }

    #[inline(always)]
    pub fn or(&mut self, i1: usize, i2: usize) -> usize {
        let mut gate = Gate::Or(i1, i2);
        let idx = self.add_line(self.get_next_level(i1, i2));
        self.wires.push(Wire::new(
            gate,
            idx
        ));
        idx
    }

    #[inline(always)]
    pub fn not(&mut self, i1: usize) -> usize {
        let mut gate = Gate::Not(i1);
        let idx = self.add_line(self.get_next_level(i1, i1));
        self.wires.push(Wire::new(
            gate,
            idx
        ));
        idx
    }

    #[inline(always)]
    pub fn half_adder(&mut self, i1: usize, i2: usize) -> Sum2 {
        let s_idx = self.xor(i1, i2);
        let c_idx = self.and(i1, i2);
        Sum2 { c: c_idx, s: s_idx }
    }

    #[inline(always)]
    pub fn full_adder(&mut self, i1: usize, i2: usize, c: usize) -> Sum2 {
        let sum1 = half_adder(i1, i2);
        let sum2 = half_adder(sum1, c);
        let c_idx = self.or(sum1.c, sum2.c);
        Sum2 { c: c_idx, sum2.s }
    }

    #[inline(always)]
    pub fn a_dot_operator(&mut self, p: usize, g: usize, p_prev: usize, g_prev: usize) -> PropagateGenerate {
        let p_idx = self.and(p, p_prev);
        let temp = self.and(p, g_prev;
        let g_idx = self.or(g, temp);
        PropagateGenerate { p: p_idx, g: g_idx }
    }

    #[inline(always)]
    pub fn a_final(&mut self, p: usize, g_idx: usize, c_in: usize) -> Sum2 {
      let s_idx = self.xor(p, c_in);
      Sum2 { p: p_idx, g: g_idx }
    }

    #[inline(always)]
    pub fn half_sub(&mut self, minuend: Line, subtrahend: Line) -> Sum2 {
        let s_idx = self.xor(minuend, subtrahend);
        let temp = self.not(minuend);
        let c_idx = self.and(temp, subtrahend);
        Sum2 { c: c_idx, s: s_idx }
    }
}
