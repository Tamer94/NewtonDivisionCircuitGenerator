use crate::data::{Adder, Mul};
use crate::data::{Bit, Bit::One, Bit::Zero, Circuit, Shift};
use crate::primitives::*;
use clap::ValueEnum;

pub struct IntDivResult {
    pub q: Vec<Bit>,
    pub r: Vec<Bit>,
    pub ok: Bit,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, ValueEnum)]
pub enum Method {
    Newton,
    Goldschmidt,
    Restoring,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, ValueEnum)]
pub enum DividendSize {
    DividendDouble,
    Equal,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, ValueEnum)]
pub enum Precision {
    Fixed,
    Boundless,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, ValueEnum)]
pub enum Estimate {
    Table10bit,
    Flip5bit,
    None,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, ValueEnum)]
pub enum SubMethod {
    Seperate,
    Fused,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct DivInfo {
    pub division_method: Method,
    pub defaultadder: Adder,
    pub defaultmult: Mul,
    pub estimator: Estimate,
    pub dividend_size: DividendSize,
    pub sub_method: SubMethod,
    pub number_bits: usize,
}

impl DivInfo {
    #[allow(dead_code)]
    pub fn default_goldschmidt() -> DivInfo {
        DivInfo {
            division_method: Method::Goldschmidt, 
            defaultadder: Adder::CRA,
            defaultmult: Mul::DadaTree,
            estimator: Estimate::None,
            dividend_size: DividendSize::Equal,
            sub_method: SubMethod::Seperate,
            number_bits: 0
        }
    }

    #[allow(dead_code)]
    pub fn default_newton() -> DivInfo {
        DivInfo {
            division_method: Method::Newton, 
            defaultadder: Adder::CRA,
            defaultmult: Mul::DadaTree,
            estimator: Estimate::None,
            dividend_size: DividendSize::Equal,
            sub_method: SubMethod::Seperate,
            number_bits: 0
        }
    }
}

fn shift_right<T: Clone>(array: &mut [T], shift: usize) {
    let len = array.len();
    if shift > 0 && len > 0 {
        let shift = shift % len;
        let mut i = len - 1;
        while i >= shift {
            array[i] = array[i - shift].clone();
            i -= 1;
        }
    }
}

impl Circuit {
    pub fn restoring_division(&mut self,
    dividend: Vec<Bit>,
    divisor: Vec<Bit>,
    info: DivInfo
) -> IntDivResult {
    let ok = self.or_of_all(divisor.clone());
    if dividend.len() != divisor.len() {
        panic!("Divisor and Dividend do not have the same number of bits");
    }
    if divisor.len() == 0 || dividend.len() == 0 {
        panic!("One of the input bit vectors was empty!");
    }

    let n = divisor.len();
    let mut q = Bit::zeroes(n);
    let mut r = Bit::zeroes(n);
    for i in 0..n {
        r[0] = dividend.get_or(n - 1 - i, Zero);
        let mut r_copy = r.clone();
        r_copy.reverse();
        // println!("{:?}", r_copy);
        let intermediate = info.defaultadder.sub(self, r.clone(), divisor.clone(), Zero);
        let mut r_copy = intermediate.clone();
        r_copy.reverse();
        // println!("After sub: {:?}", r_copy);
        let decide_bit = *intermediate.last().unwrap_or(&Zero);
        q[n - i - 1] = self.not(decide_bit);
        // intermediate.iter_mut().for_each(|x| *x = self.and(*x, decide_bit));
        let mut readd = divisor.clone();
        readd.iter_mut().for_each(|x| *x = self.and(*x, decide_bit));
        let mut r_copy = readd.clone();
        r_copy.reverse();
        // println!("readd: {:?}", r_copy);
        r = info.defaultadder.add(self, readd, intermediate, Zero);
        r.truncate(n);
        let mut r_copy = r.clone();
        r_copy.reverse();
        // println!("Remainder after restoring: {:?}", r_copy);
        if i < (n - 1) {
            shift_right(&mut r, 1);
        }
        let mut r_copy = q.clone();
        r_copy.reverse();
        // println!("Quotient: {:?}", r_copy);
    }
    IntDivResult { q, r, ok }
}

    pub fn get_divider_circuit(info: DivInfo) -> Circuit {
        // create a new circuit object to store the circuit and its stats
        let mut circuit = Circuit::new();

        let bits = info.number_bits;

        let mut divisor = vec![];
        let mut dividend = vec![];
        for _ in 0..bits {
            dividend.push(circuit.new_line());
        }
        if info.dividend_size == DividendSize::DividendDouble {
            for _ in 0..(bits - 2) {
                dividend.push(circuit.new_line());
            }
        }

        for _ in 0..bits {
            divisor.push(circuit.new_line());
        }
        if info.dividend_size == DividendSize::DividendDouble {
            divisor.pop();
            divisor.push(Zero);
            for _ in 0..(bits - 2) {
                divisor.push(Zero);
            }
        }

        let IntDivResult { mut q, mut r, ok } = match info.division_method {
            Method::Newton => { circuit.div_newton(dividend.clone(), divisor.clone(), info) }
            Method::Goldschmidt => { circuit.goldschmidt_divider(dividend.clone(), divisor.clone(), info) }
            Method::Restoring => { circuit.restoring_division(dividend.clone(), divisor.clone(), info) }
        };

        if info.dividend_size == DividendSize::DividendDouble {
            q.truncate(bits);
            r.truncate(bits);
            divisor.truncate(bits - 1);
        }

        circuit.add_as_io(&dividend, "R_0", false);
        circuit.add_as_io(&divisor, "D", false);
        circuit.add_as_io(&q, "Q", true);
        circuit.add_as_io(&r, "R_n1", true);
        circuit.add_as_io(&vec![ok], "Valid", true);
        circuit
    }

    #[inline(always)]
    fn div_newton_precalculations(
        &mut self,
        dividend: &Vec<Bit>,
        divisor: &Vec<Bit>,
        info: &DivInfo) -> (Vec<Bit>, Vec<Bit>, Vec<Bit>, Vec<Bit>, Vec<Bit>, Bit, usize, usize) {
        let ok = self.or_of_all(divisor.clone());
        if dividend.len() != divisor.len() {
            panic!("Divisor and Dividend do not have the same number of bits");
        }
        if divisor.len() == 0 || dividend.len() == 0 {
            panic!("One of the input bit vectors was empty!");
        }

        let n = divisor.len();
        // println!{"before shifting {:?}", divisor};
        let d_plus = info.defaultadder.add(self, dividend.clone(), divisor.clone(), Zero);
        let d_minus = info.defaultadder.sub(self, dividend.clone(), divisor.clone(), Zero);
        // println!("dividend_alt_1 {:?}, dividend_alt2 {:?}", d_plus, d_minus);
        let mut shift_left_by = self.lzc(divisor.clone());
        // we dont need the all or bit
        shift_left_by.pop();
        // println!{"shifting left by {:?}", shift_left_by};
        let shifted_divisor = self.shift(divisor.clone(), shift_left_by.clone(), Shift::Left, Zero);
        let _shifted_dividend =
            self.shift(dividend.clone(), shift_left_by.clone(), Shift::Left, Zero);
        // println!{"after shifting {:?}", shifted_divisor};

        let estimate = match info.estimator {
            Estimate::Flip5bit => self.flip_estimate(&shifted_divisor),
            Estimate::Table10bit => self.table_estimate(&shifted_divisor),
            _ => {
                let mut estimate = Bit::zeroes(n - 1);
                estimate.append(&mut vec![One, One, Zero, Zero]);
                estimate
            }
        };

        // println!{"after appending {:?}", shifted_divisor};
        // println!{"estimate {:?}", estimate};
        // let mut negative_estimate = self.get_negative(estimate.clone());
        // // println!{"negative_estimate {:?}", negative_estimate};
        let necessary_its = match info.estimator {
            Estimate::Flip5bit => (0.max(n as i32 - 6) as f64 / 5_f64 + 1_f64).ceil().log2().ceil() as usize,
            // Estimate::Linear => /* ((n + 1) as f64 / (27f64).log2()).log2().ceil() as usize */ (0.max(n as i32 - 2) as f64 + 1_f64).ceil().log2().ceil() as usize,
            Estimate::Table10bit => if n < 11 { 0 } else { ((0.max(n as i32) as f64).ceil().log2().ceil() as usize) - 2 },
            Estimate::None => /* (0.max(n as i32 - 2) as f64 + 1_f64).ceil().log2().ceil() as usize */ (0.max(n as i32) as f64).ceil().log2().ceil() as usize,
        };
        println!("{necessary_its} :: {n}");

        (shift_left_by, estimate, d_plus, d_minus, shifted_divisor, ok, necessary_its, n)
    }

    #[inline(always)]
    fn div_newton_iterations(&mut self, mut estimate: Vec<Bit>, shifted_divisor: Vec<Bit>, n: usize, necessary_its: usize, info: &DivInfo) -> Vec<Bit> {
        for _i in 0..necessary_its {
            let mut squared = info.defaultmult.square_u(self,estimate.clone(), 3, info.defaultadder);
            let mut shifted_estimate = vec![Zero];
            for i in 0..(n + 2) {
                shifted_estimate.push(estimate.get_or(i, Zero));
            }
            // println!("squared estimate: {:?}", squared);
            // println!("shifted estimate: {:?}", shifted_estimate);

            /*
            if pos_digits > 3 {
                for _ in 0..(pos_digits - 3) {
                    squared.pop();
                    pos_digits -= 1;
                }
            }
            squared.drain(0..(neg_digits - n));
            */

            squared.drain(0..n);
            // println!("squared after discard: {:?}", squared);

            match info.sub_method {
                SubMethod::Seperate => {
                    let mut p = info.defaultmult.mul_u(self, squared, shifted_divisor.clone(), None, info.defaultadder);
                    p.drain(0..n);
                    // println!("mult: {:?}", p);
        
                    estimate = info.defaultadder.sub(self, shifted_estimate, p, Zero);
                    let failure = estimate.pop().unwrap_or(Zero);
                    if failure == One {
                        // println!("Got negative number in round {}", i);
                    }
                },
                SubMethod::Fused => {
                    estimate = self.fused_mul_subtraction(shifted_estimate, squared, shifted_divisor.clone(), true);
                    estimate.drain(0..n);
                }
                
            }


            // println!("new estimate: {:?}", estimate);

            // negative_estimate = self.get_negative(estimate.clone());
        }
        estimate
    }

    fn div_newton_correction_step(&mut self, divisor: Vec<Bit>, dividend: Vec<Bit>, estimate: Vec<Bit>, d_plus: Vec<Bit>, d_minus: Vec<Bit>, mut shift_left_by: Vec<Bit>, ok: Bit, n: usize, info: &DivInfo) -> IntDivResult {
        let mut one = vec![One];
        one.append(&mut Bit::zeroes(shift_left_by.len() - 1));
        self.not_all(&mut shift_left_by);
        let mut v_n = Bit::get_bits_vec_usize(divisor.len());
        let num_bits = (divisor.len() as f64).log2().floor() as usize + 1;
        v_n.truncate(num_bits);
        // println!("n: {:?}", v_n);
        let mut shift_right_by = info.defaultadder.sub(self, v_n, shift_left_by, Zero);
        shift_right_by.truncate(num_bits);
        // println!("right_shift_by {:?}", shift_right_by);
        let mut restored_estimate = self.shift(estimate.clone(), shift_right_by, Shift::Right, One);
        // restored_estimate can be 1 caution!
        restored_estimate.truncate(n + 1);
        // println!("restored_estimate {:?}", restored_estimate);

        let mut q0 = self.umul_dadda(restored_estimate.clone(), dividend.clone(), None, info.defaultadder);
        // let mut q0 = self.mul_unsigned_clean(estimate.clone(), shifted_dividend.clone(), None);

        // println!("q {:?}", q0);

        q0.drain(0..n);
        q0.truncate(n);

        // println!("q {:?}", q0);

        let mut q_plus = info.defaultadder.add(self,q0.clone(), vec![], One);
        let mut q_minus = info.defaultadder.sub(self,q0.clone(), vec![One], Zero);

        q_plus.truncate(n);
        q_minus.truncate(n);

        // println!("q_plus {:?}, q_minus {:?}", q_plus, q_minus);

        let mut qz = self.umul_dadda(q0.clone(), divisor.clone(), None, info.defaultadder);

        // println!("qz {:?}", qz);

        qz.truncate(n);

        // println!("qz {:?}", qz);

        let mut r0 = info.defaultadder.sub(self,dividend.clone(), qz.clone(), Zero);
        let mut r_plus = info.defaultadder.sub(self,d_minus.clone(), qz.clone(), Zero);
        let mut r_minus = info.defaultadder.sub(self,d_plus.clone(), qz.clone(), Zero);

        // println!("r {:?}, r_plus {:?}, r_minus {:?}", r0, r_plus, r_minus);

        let s1 = r0.get_or(n, Zero);
        let s0 = r_plus.get_or(n, Zero);

        // println!("s0: {:?}, s1: {:?}", s0, s1);

        r0.truncate(n);
        r_plus.truncate(n);
        r_minus.truncate(n);

        // println!("r {:?}, r_plus {:?}, r_minus {:?}", r0, r_plus, r_minus);

        let q_final = self.mux_n_4(&q_plus, &q0, &vec![Zero], &q_minus, (s0, s1));
        let r_final = self.mux_n_4(&r_plus, &r0, &vec![Zero], &r_minus, (s0, s1));
        // println!("q_final {:?}, r_final {:?}, result is valid: {:?}", q_final, r_final, ok);

        IntDivResult {
            q: q_final,
            r: r_final,
            ok,
        }
    }

    pub fn div_newton(
        &mut self,
        dividend: Vec<Bit>,
        divisor: Vec<Bit>,
        info: DivInfo
    ) -> IntDivResult {
        let (shift_left_by, estimate, d_plus, d_minus, shifted_divisor, result_valid, necessary_its, n) = self.div_newton_precalculations(&dividend, &divisor, &info);
        let estimate = self.div_newton_iterations(estimate, shifted_divisor.clone(), n, necessary_its, &info);
        self.div_newton_correction_step(divisor, dividend, estimate, d_plus, d_minus, shift_left_by, result_valid, n, &info)
    }

    #[allow(dead_code)]
    pub fn goldschmidt_divider(
        &mut self,
        dividend: Vec<Bit>,
        divisor: Vec<Bit>,
        info: DivInfo,
    ) -> IntDivResult {
        if dividend.len() != divisor.len() {
            panic!("Divisor and Dividend do not have the same number of bits");
        }
        if divisor.len() == 0 || dividend.len() == 0 {
            panic!("One of the input bit vectors was empty!");
        }

        let n = divisor.len();
        let ok = self.or_of_all(divisor.clone());
        let divisor_is_one = self.or_of_all(divisor[1..n].to_vec());
        let divisor_is_one = self.not(divisor_is_one);
        let divisor_is_one = self.and(divisor[0], divisor_is_one);
        let constant_zero = Bit::zeroes(n);

        let d_plus = info.defaultadder.add(self, dividend.clone(), divisor.clone(), Zero);
        let d_minus = info.defaultadder.sub(self, dividend.clone(), divisor.clone(), Zero);
        let mut shift_left_by = self.lzc(divisor.clone());
        let mut dividend_shift_left_by = self.lzc(dividend.clone());
        // we dont need the <all bits are zero> bit
        shift_left_by.pop();
        dividend_shift_left_by.pop();
        let shifted_divisor = self.shift(divisor.clone(), shift_left_by.clone(), Shift::Left, Zero);
        let shifted_dividend = self.shift(dividend.clone(), dividend_shift_left_by.clone(), Shift::Left, Zero);

        let mut constant_one = Bit::zeroes(n);
        constant_one.push(One);
        let mut x = info.defaultadder.sub(self, constant_one, shifted_divisor, Zero);
        x.truncate(n);
        let necessary_iters = ((n) as f64).log2().ceil() as usize;

        let mut factors = vec![];
        factors.push(x);
        for i in 0..necessary_iters {
            let current_factor = factors[i].clone();
            let mut new_factor = self.usquare_dadda(current_factor, 0, info.defaultadder);
            new_factor.drain(0..n);
            factors.push(new_factor);
        }

        let mut first_factor = factors.get(0).unwrap_or(&vec![Zero]).clone();
        first_factor.push(One);
        let mut p = self.umul_dadda(first_factor, shifted_dividend, None, info.defaultadder);
        p.drain(0..n);

        if factors.len() > 1 {
            for i in 1..factors.len() {
                let mut factor = factors[i].clone();
                factor.push(One);
                p = self.umul_dadda(p, factor, None, info.defaultadder);
                p.drain(0..(n));
                p.truncate(n+1);
            }
        }

        let mut one = vec![One];
        one.append(&mut Bit::zeroes(shift_left_by.len() - 1));
        self.not_all(&mut shift_left_by);
        let mut v_n = Bit::get_bits_vec_usize(divisor.len());
        let num_bits = (divisor.len() as f64).log2().floor() as usize + 1;
        v_n.truncate(num_bits);
        let mut shift_additional = info.defaultadder.sub(self, v_n, shift_left_by, Zero);
        shift_additional.truncate(num_bits);

        let mut shift_right_by = dividend_shift_left_by;
        self.not_all(&mut shift_right_by);
        let mut shift_right_by = info.defaultadder.add(self, shift_right_by.clone(), shift_additional, Zero);
        self.not_all(&mut shift_right_by);
        let mut q0 = self.shift(p.clone(), shift_right_by, Shift::Right, Zero);
        q0.truncate(n);

        let mut q_plus = info.defaultadder.add(self, q0.clone(), vec![], One);
        let mut q_minus = info.defaultadder.sub(self, q0.clone(), vec![], One);
        q_plus.truncate(n);
        q_minus.truncate(n);

        let mut qz = self.umul_dadda(q0.clone(), divisor.clone(), None, info.defaultadder);
        qz.truncate(n);

        let mut r0 = info.defaultadder.sub(self, dividend.clone(), qz.clone(), Zero);
        let mut r_plus = info.defaultadder.sub(self, d_minus.clone(), qz.clone(), Zero);
        let mut r_minus = info.defaultadder.sub(self, d_plus.clone(), qz.clone(), Zero);

        let s1 = r0.get_or(n, Zero);
        let s1 = self.or(divisor_is_one, s1);
        let temp = self.not(divisor_is_one);
        let s0 = r_plus.get_or(n, Zero);
        let s0 = self.and(temp, s0);

        r0.truncate(n);
        r_plus.truncate(n);
        r_minus.truncate(n);

        let q_final = self.mux_n_4(&q_plus, &q0, &dividend, &q_minus, (s0, s1));
        let r_final = self.mux_n_4(&r_plus, &r0, &constant_zero,&r_minus, (s0, s1));

        IntDivResult {
            q: q_final,
            r: r_final,
            ok,
        }
    }
}
