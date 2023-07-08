use crate::{data::{LevelizedCircuit, Bit::Zero, Bit}, adders::Sum2, primitives::Get};

impl LevelizedCircuit {
    pub fn sign_extension(number: &Vec<Bit>, new_size: usize) -> Vec<Bit> {
        let n = number.len();
        let sign = *number.last().unwrap_or(&Zero);
        let mut sign_extended = number.clone();

        for _ in 0..(new_size.saturating_sub(n)) {
            sign_extended.push(sign);
        }
        println!("sign extended {:?}", sign_extended);
        sign_extended
    }

    pub fn get_sub(n: usize) -> LevelizedCircuit {
        let mut lc = LevelizedCircuit::new(1 << 30);
        let mut a = vec![];
        let mut b = vec![];
        for _ in 0..n {
            a.push(lc.circuit.new_line());
            b.push(lc.circuit.new_line());
        }

        let difference = lc.sub(&a, &b, Zero);

        lc.circuit.add_as_io(&a, "A", false);
        lc.circuit.add_as_io(&b, "B", false);
        lc.circuit.add_as_io(&difference, "C", true);
        lc
    }

    pub fn get_mul(n: usize) -> LevelizedCircuit {
        let mut lc = LevelizedCircuit::new(1 << 30);
        let mut a = vec![];
        let mut b = vec![];
        for _ in 0..n {
            a.push(lc.circuit.new_line());
            b.push(lc.circuit.new_line());
        }

        let difference = lc.mul(a.clone(), b.clone(), None);

        lc.circuit.add_as_io(&a, "A", false);
        lc.circuit.add_as_io(&b, "B", false);
        lc.circuit.add_as_io(&difference, "C", true);
        lc
    }

    pub fn get_2_sub(n: usize) -> LevelizedCircuit {
        let mut lc = LevelizedCircuit::new(1 << 30);
        let mut a = vec![];
        let mut b = vec![];
        let mut b2 = vec![];
        for _ in 0..n {
            a.push(lc.circuit.new_line());
            b.push(lc.circuit.new_line());
        }

        for _ in 0..(2*n) {
            b2.push(lc.circuit.new_line());
        }

        let mut difference = lc.sub(&a, &b, Zero);
        
        let mut c = vec![];

        difference = LevelizedCircuit::sign_extension(&difference, n*2 + 1);

        let mut borrow = Zero;
        for i in 0..b2.len() {
            let Sum2 { c: temp, s } = lc.full_sub(difference.get_or(i, Zero), b2.get_or(i, Zero), borrow);
            borrow = temp;
            c.push(s);
        }

        let Sum2 { c: borrow, s } = lc.full_sub(b2.get_or(2*n, Zero), difference.get_or(2*n, Zero), borrow);
        c.push(s);
        c.push(borrow);
        println!("\x1B[5m");
        println!("N-D: {:?}", difference);
        println!("R˘: {:?}", c);
        println!("DQ: {:?}", b2);
        println!("\x1B[0m");

        lc.circuit.add_as_io(&a, "N", false);
        lc.circuit.add_as_io(&b, "D", false);
        lc.circuit.add_as_io(&b2, "DQ", false);
        lc.circuit.add_as_io(&c, "R˘", true);
        lc
    }
}