use std::collections::VecDeque;

use crate::data::{
    Bit, Bit::One, Bit::Var, Bit::Zero, Circuit, Gate, Line,
    Wire, LevelizedCircuit
};

pub trait Get<T> {
    fn get_or(&self, idx: usize, default: T) -> T;
}

impl<T> Get<T> for Vec<T>
where
    T: Copy,
{
    fn get_or(&self, idx: usize, default: T) -> T {
        *self.get(idx).unwrap_or(&default)
    }
}

impl Circuit {
    #[inline(always)]
    pub fn xor(&mut self, i1: Bit, i2: Bit) -> Bit {
        let bit;
        match (i1, i2) {
            (Var(j1), Var(j2)) => {
                if j1 != j2 {
                    let gate = Gate::Xor(j1, j2);
                    self.stats.gatter_count += 1;
                    let line = Line {
                        level: gate.get_next_level(),
                        n: self.stats.add_line(),
                    };
                    self.wires.push(Wire::new(line, gate));
                    bit = Var(line);
                }
                else {
                    bit = Zero;
                }
            }
            (Var(_), Zero) => {
                bit = i1;
            }
            (Zero, Var(_)) => {
                bit = i2;
            }
            (Var(j1), One) => {
                let gate = Gate::Not(j1);
                self.stats.gatter_count += 1;
                let line = Line {
                    level: gate.get_next_level(),
                    n: self.stats.add_line(),
                };
                self.wires.push(Wire::new(line, gate));
                bit = Var(line);
            }
            (One, Var(j2)) => {
                let gate = Gate::Not(j2);
                self.stats.gatter_count += 1;
                let line = Line {
                    level: gate.get_next_level(),
                    n: self.stats.add_line(),
                };
                self.wires.push(Wire::new(line, gate));
                bit = Var(line);
            }
            (Zero, Zero) => {
                bit = Zero;
            }
            (Zero, One) => {
                bit = One;
            }
            (One, Zero) => {
                bit = One;
            }
            (One, One) => {
                bit = Zero;
            }
        }
        bit
    }

    #[inline(always)]
    pub fn and(&mut self, i1: Bit, i2: Bit) -> Bit {
        let bit;
        match (i1, i2) {
            (Var(j1), Var(j2)) => {
                if j1 != j2 {
                    let gate = Gate::And(j1, j2);
                    self.stats.gatter_count += 1;
                    let line = Line {
                        level: gate.get_next_level(),
                        n: self.stats.add_line(),
                    };
                    self.wires.push(Wire::new(line, gate));
                    bit = Var(line);
                } else {
                    bit = i1;
                }
            }
            (Var(_), Zero) => {
                bit = Zero;
            }
            (Zero, Var(_)) => {
                bit = Zero;
            }
            (Var(_), One) => {
                bit = i1;
            }
            (One, Var(_)) => {
                bit = i2;
            }
            (Zero, ..) => {
                bit = Zero;
            }
            (One, Zero) => {
                bit = Zero;
            }
            (One, One) => {
                bit = One;
            }
        }
        bit
    }

    #[inline(always)]
    pub fn or(&mut self, i1: Bit, i2: Bit) -> Bit {
        let bit;
        match (i1, i2) {
            (Var(j1), Var(j2)) => {
                if j1 != j2 {
                    let gate = Gate::Or(j1, j2);
                    self.stats.gatter_count += 1;
                    let line = Line {
                        level: gate.get_next_level(),
                        n: self.stats.add_line(),
                    };
                    self.wires.push(Wire::new(line, gate));
                    bit = Var(line);
                } else {
                    bit = i1;
                }
            }
            (Var(_), Zero) => {
                bit = i1;
            }
            (Zero, Var(_)) => {
                bit = i2;
            }
            (Var(_), One) => {
                bit = One;
            }
            (One, Var(_)) => {
                bit = One;
            }
            (Zero, Zero) => {
                bit = Zero;
            }
            (Zero, One) => {
                bit = One;
            }
            (One, ..) => {
                bit = One;
            }
        }
        bit
    }

    #[inline(always)]
    pub fn not(&mut self, i1: Bit) -> Bit {
        let bit;
        match i1 {
            Var(j1) => {
                let gate = Gate::Not(j1);
                self.stats.gatter_count += 1;
                let line = Line {
                    level: gate.get_next_level(),
                    n: self.stats.add_line(),
                };
                self.wires.push(Wire::new(line, gate));
                bit = Var(line);
            }
            One => {
                bit = Zero;
            }
            Zero => {
                bit = One;
            }
        }
        bit
    }

    pub fn or_of_all(&mut self, bits: Vec<Bit>) -> Bit {
        let mut bits = VecDeque::from(bits);
        while bits.len() > 1 {
            let bit1 = bits.pop_front().unwrap_or(Zero);
            let bit2 = bits.pop_front().unwrap_or(Zero);
            let r = self.or(bit1, bit2);
            bits.push_back(r);
        }
        bits.pop_front().unwrap_or(Zero)
    }

    pub fn not_all(&mut self, bits: &mut Vec<Bit>) {
        for bit in bits {
            *bit = self.not(*bit);
        }
    }
}

impl LevelizedCircuit {
    #[inline(always)]
    pub fn not(&mut self, i1: Bit) -> Bit {
        let bit;
        let gate;
        match i1 {
            Var(j1) => {
                gate = Gate::Not(j1);
            }
            One => {
                gate = Gate::Not(self.circuit.get_one_wire());
            }
            Zero => {
                gate = Gate::Not(self.circuit.get_zero_wire());
            }
        }

        self.circuit.stats.gatter_count += 1;
        let line = Line {
            level: gate.get_next_level(),
            n: self.circuit.stats.add_line(),
        };
        self.circuit.wires.push(Wire::new(line, gate));
        bit = Var(line);
        self.substitution_levels.insert(line.n, self.current_level);
        self.current_level -= 1;
        bit
    }

    #[inline(always)]
    pub fn or(&mut self, i1: Bit, i2: Bit) -> Bit {
        let bit;
        let gate;
        let in1 = match i1 {
            Zero => self.circuit.get_zero_wire(),
            One => self.circuit.get_one_wire(),
            Var(l) => l,
        };

        let in2 = match i2 {
            Zero => self.circuit.get_zero_wire(),
            One => self.circuit.get_one_wire(),
            Var(l) => l,
        };

        gate = Gate::Or(in1, in2);
        self.circuit.stats.gatter_count += 1;
        let line = Line {
            level: gate.get_next_level(),
            n: self.circuit.stats.add_line(),
        };
        self.circuit.wires.push(Wire::new(line, gate));
        bit = Var(line);
        self.substitution_levels.insert(line.n, self.current_level);
        self.current_level -= 1;
        bit
    }

    #[inline(always)]
    pub fn and(&mut self, i1: Bit, i2: Bit) -> Bit {
        let bit;
        let gate;
        let in1 = match i1 {
            Zero => self.circuit.get_zero_wire(),
            One => self.circuit.get_one_wire(),
            Var(l) => l,
        };

        let in2 = match i2 {
            Zero => self.circuit.get_zero_wire(),
            One => self.circuit.get_one_wire(),
            Var(l) => l,
        };

        gate = Gate::And(in1, in2);
        self.circuit.stats.gatter_count += 1;
        let line = Line {
            level: gate.get_next_level(),
            n: self.circuit.stats.add_line(),
        };
        self.circuit.wires.push(Wire::new(line, gate));
        bit = Var(line);
        self.substitution_levels.insert(line.n, self.current_level);
        self.current_level -= 1;
        bit
    }

    #[inline(always)]
    pub fn xor(&mut self, i1: Bit, i2: Bit) -> Bit {
        let bit;
        let gate;
        let in1 = match i1 {
            Zero => self.circuit.get_zero_wire(),
            One => self.circuit.get_one_wire(),
            Var(l) => l,
        };

        let in2 = match i2 {
            Zero => self.circuit.get_zero_wire(),
            One => self.circuit.get_one_wire(),
            Var(l) => l,
        };

        gate = Gate::Or(in1, in2);
        self.circuit.stats.gatter_count += 1;
        let line = Line {
            level: gate.get_next_level(),
            n: self.circuit.stats.add_line(),
        };
        self.circuit.wires.push(Wire::new(line, gate));
        bit = Var(line);
        self.substitution_levels.insert(line.n, self.current_level);
        self.current_level -= 1;
        bit
    }
}
