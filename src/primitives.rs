use crate::data::data::{Circuit, Gate, Line, Out, Output, Stats, Wire, IO};

pub struct Sum2 {
    pub c: Line,
    pub s: Line,
}

pub struct PropagateGenerate {
    pub p: Line,
    pub g: Line,
}

pub struct Sum3 {
    pub c1: Line,
    pub c0: Line,
    pub s: Line,
}
impl Circuit {
    #[inline(always)]
    pub fn xor(&mut self, i1: Line, i2: Line) -> Line {
        let mut gate = Gate::Xor(i1, i2);
        let line = Line {
                level: gate.get_next_level(),
                n: self.stats.add_line(),
            };
        self.wires.push(Wire::new(line, gate));
        line
    }

    #[inline(always)]
    pub fn and(&mut self, i1: Line, i2: Line) -> Line {
        let mut gate = Gate::And(i1, i2);
        let line = Line {
                level: gate.get_next_level(),
                n: self.stats.add_line(),
            };
        self.wires.push(Wire::new(line, gate));
        line
    }

    #[inline(always)]
    pub fn or(&mut self, i1: Line, i2: Line) -> Line {
        let mut gate = Gate::Or(i1, i2);
        let line = Line {
                level: gate.get_next_level(),
                n: self.stats.add_line(),
            };
        self.wires.push(Wire::new(line, gate));
        line
    }

    #[inline(always)]
    pub fn not(&mut self, i1: Line) -> Line {
        let mut gate = Gate::Not(i1);
        let line = Line {
                level: gate.get_next_level(),
                n: self.stats.add_line(),
            };
        self.wires.push(Wire::new(line, gate));
        line
    }

    #[inline(always)]
    pub fn half_adder(&mut self, i1: Line, i2: Line) -> Sum2 {
        let s = self.xor(i1, i2);
        let c = self.and(i1, i2);
        Sum2 { c: c, s: s }
    }

    #[inline(always)]
    pub fn full_adder(&mut self, i1: Line, i2: Line, c: Line) -> Sum2 {
        let sum1 = self.half_adder(i1, i2);
        let sum2 = self.half_adder(sum1.s, c);
        let c = self.or(sum1.c, sum2.c);
        Sum2 { c: c, s: sum2.s }
    }

    #[inline(always)]
    pub fn a_dot_operator(&mut self, p: Line, g: Line, p_prev: Line, g_prev: Line) -> PropagateGenerate {
        let p = self.and(p, p_prev);
        let temp = self.and(p, g_prev);
        let g = self.or(g, temp);
        PropagateGenerate { p: p, g: g }
    }

    #[inline(always)]
    pub fn a_final(&mut self, p: Line, g: Line, c_in: Line) -> Sum2 {
      let s = self.xor(p, c_in);
      Sum2 { s, c: g }
    }

    #[inline(always)]
    pub fn half_sub(&mut self, minuend: Line, subtrahend: Line) -> Sum2 {
        let s = self.xor(minuend, subtrahend);
        let temp = self.not(minuend);
        let c = self.and(temp, subtrahend);
        Sum2 { c: c, s: s }
    }

    #[inline(always)]
    pub fn full_sub(&mut self, minuend: Line, subtrahend: Line, c_in: Line) -> Sum2 {
        let sum1 = self.half_sub(minuend, subtrahend);
        let sum2 = self.half_sub(sum1.s, c_in);
        let c_out = self.or(sum2.c, sum1.c); 
        Sum2 { c: c_out, s: sum2.s }
    }
}
