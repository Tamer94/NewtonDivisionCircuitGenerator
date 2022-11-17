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
        let idx = self.wires.len();
        self.wires.push(Wire::new(
            Line {
                level: gate.get_next_level(),
                n: self.stats.add_line(),
            },
            gate,
        ));
        idx
    }

    #[inline(always)]
    pub fn and(&mut self, i1: usize, i2: usize) -> usize {
        let mut gate = Gate::And(i1, i2);
        let idx = self.wires.len();
        self.wires.push(Wire::new(
            Line {
                level: gate.get_next_level(),
                n: self.stats.add_line(),
            },
            gate,
        ));
        idx
    }

    #[inline(always)]
    pub fn or(&mut self, i1: usize, i2: usize) -> usize {
        let mut gate = Gate::Or(i1, i2);
        let idx = self.wires.len();
        self.wires.push(Wire::new(
            Line {
                level: gate.get_next_level(),
                n: self.add_line(),
            },
            gate,
        ));
        idx
    }

    #[inline(always)]
    pub fn not(&mut self, i1: usize) -> usize {
        let mut gate = Gate::Not(i1);
        let idx = self.wires.len();
        self.wires.push(Wire::new(
            Line {
                level: gate.get_next_level(),
                n: self.stats.add_line(),
            },
            gate,
        ));
        idx
    }

    #[inline(always)]
    pub fn half_adder(&mut self, i1: Line, i2: Line) -> Sum2 {
        let s_idx = self.xor(i1, i2);
        let c_idx = self.and(i1, i2);
        Sum2 { c: c_idx, s: s_idx }
    }

    #[inline(always)]
    pub fn full_adder(&mut self, i1: Line, i2: Line, c: Line) -> Sum2 {
        let sum1 = half_adder(i1, i2);
        let sum2 = half_adder(self.out(sum1), c);
        let c_idx = self.or(self.out(sum1.c), self.out(sum2.c));
        Sum2 { c: c_idx, sum2.s }
    }

    #[inline(always)]
    pub fn a_dot_operator(&mut self, p: Line, g: Line, p_prev: Line, g_prev: Line) -> PropagateGenerate {
        let p_idx = self.and(p, p_prev);
        let temp = self.out(self.and(p, g_prev));
        let g_idx = self.or(g, temp);
        PropagateGenerate { p: p_idx, g: g_idx }
    }

    #[inline(always)]
    pub fn a_final(&mut self, p: Line, g_idx: usize, c_in: Line) -> Sum2 {
      let s_idx = self.xor(p, c_in);
      Sum2 { p: p_idx, g: g_idx }
    }

    #[inline(always)]
    pub fn half_sub(&mut self, minuend: Line, subtrahend: Line) -> Sum2 {
        let s_idx = self.xor(minuend, subtrahend);
        let temp = self.out(self.not(minuend));
        let c_idx = self.and(temp, subtrahend);
        Sum2 { c: c_idx, s: s_idx }
    }
}
