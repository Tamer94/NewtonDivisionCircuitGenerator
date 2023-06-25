use std::collections::VecDeque;

use num::Integer;

// Here the helping circuits for Divison with the Newton Method are contained among them MUXes, LZC (LeadingZeroCounter) and
// BarrelShifter
use crate::data::{
    Bit, Bit::One, Bit::Zero, Circuit, Shift, Shift::Left,
    Shift::Right,
};
use crate::primitives::*;

impl Circuit {
    #[inline(always)]
    pub fn mux_1(&mut self, s1: Bit, s2: Bit, select: Bit) -> Bit {
        let high = self.and(s1, select);
        let select = self.not(select);
        let low = self.and(s2, select);
        self.or(high, low)
    }

    #[inline(always)]
    pub fn mux_n_1(&mut self, s1: &Vec<Bit>, s2: &Vec<Bit>, select: Bit) -> Vec<Bit> {
        let n = s1.len().max(s2.len());
        let mut selected = Vec::with_capacity(n);
        for idx in 0..n {
            let high = self.and(s1[idx], select);
            let select = self.not(select);
            let low = self.and(s2[idx], select);
            selected.push(self.or(high, low));
        }
        selected
    }

    pub fn get_mux_n_1(bits: usize) -> Circuit {
        let mut circuit = Circuit::new();
        let mut a = vec![];
        let mut b = vec![];
        let select = circuit.new_line();

        for _ in 0..bits {
            a.push(circuit.new_line());
            b.push(circuit.new_line());
        }

        let result = circuit.mux_n_1(&a, &b, select);

        circuit.add_as_io(&a, "X", false);
        circuit.add_as_io(&b, "Y", false);
        circuit.add_as_io(&vec![select], "SEL", false);
        circuit.add_as_io(&result, "S", true);
        circuit
    }

    // for choosing between 3 n-bit vectors
    // select (s0, s1) => input_i
    // select (0,0) => input1
    // select (1,0) => input2
    // select (0,1) => input3
    // select (1,1) => input3
    #[allow(dead_code)]
    pub fn mux_n_2(
        &mut self,
        b1: &Vec<Bit>,
        b2: &Vec<Bit>,
        b3: &Vec<Bit>,
        select: (Bit, Bit),
    ) -> Vec<Bit> {
        let intermediate = self.mux_n_1(b2, b1, select.0);
        self.mux_n_1(b3, &intermediate, select.1)
    }

    // for choosing between 4 n-bit vectors
    // its expected that all 4 bit vectors have the same number of bits
    // otherwise this function will crash
    // select (s0, s1) => input_i
    // select (0,0) => input1
    // select (1,0) => input2
    // select (0,1) => input3
    // select (1,1) => input4
    pub fn mux_n_4(
        &mut self,
        b1: &Vec<Bit>,
        b2: &Vec<Bit>,
        b3: &Vec<Bit>,
        b4: &Vec<Bit>,
        select: (Bit, Bit),
    ) -> Vec<Bit> {
        let n = b1.len();
        let mut selected = Vec::with_capacity(n);
        let s0_not = self.not(select.0);
        let s1_not = self.not(select.1);

        for i in 0..n {
            let b1_temp = self.and(b1.get_or(i, Zero), s0_not);
            let b2_temp = self.and(b2.get_or(i, Zero), select.0);
            let b3_temp = self.and(b3.get_or(i, Zero), s0_not);
            let b4_temp = self.and(b4.get_or(i, Zero), select.0);

            let b1_a = self.and(b1_temp, s1_not);
            let b2_a = self.and(b2_temp, s1_not);
            let b3_a = self.and(b3_temp, select.1);
            let b4_a = self.and(b4_temp, select.1);

            let temp1 = self.or(b1_a, b2_a);
            let temp2 = self.or(b3_a, b4_a);
            let temp3 = self.or(temp1, temp2);

            selected.push(temp3);
        }

        selected
    }

    pub fn get_mux_n_2(bits: usize) -> Circuit {
        let mut circuit = Circuit::new();
        let mut a = vec![];
        let mut b = vec![];
        let mut c = vec![];
        let mut d = vec![];
        let select_0 = vec![circuit.new_line()];
        let select_1 = vec![circuit.new_line()];

        for _ in 0..bits {
            a.push(circuit.new_line());
            b.push(circuit.new_line());
            c.push(circuit.new_line());
            d.push(circuit.new_line());
        }

        let result = circuit.mux_n_4(&a, &b, &c, &d, (select_0[0], select_1[0]));

        circuit.add_as_io(&a, "X", false);
        circuit.add_as_io(&b, "Y", false);
        circuit.add_as_io(&c, "P", false);
        circuit.add_as_io(&d, "Q", false);
        circuit.add_as_io(&select_0, "SEL0", false);
        circuit.add_as_io(&select_1, "SEL1", false);
        circuit.add_as_io(&result, "S", true);
        circuit
    }

    // implements a barrel shifter non circular, both left and rightshift is possible
    // if >not_shift< represents a number bigger than the number of digits of >number<
    // >number< will effectivly made all Zeros
    pub fn shift(
        &mut self,
        mut number: Vec<Bit>,
        not_shift: Vec<Bit>,
        direction: Shift,
        shift_for: Bit,
    ) -> Vec<Bit> {
        let n = number.len();
        let m = not_shift.len();
        let mut intermediate = number.clone();
        let mut next = 1;
        let mut neighbor;
        for j in 0..m {
            for i in 0..n {
                match direction {
                    Right => {
                        neighbor = i + next;
                    }
                    Left => {
                        // this is a potential bug in case n is almost usize::MAX
                        // very unlikely memory will run out before this the case
                        neighbor = i.wrapping_sub(next);
                    }
                }

                let n1;
                let n2;
                let shift;
                if shift_for == Zero {
                    n1 = number[i];
                    n2 = *number.get(neighbor).unwrap_or(&Zero);
                    shift = *not_shift.get(j).unwrap_or(&One);
                } else {
                    n1 = *number.get(neighbor).unwrap_or(&Zero);
                    n2 = number[i];
                    shift = *not_shift.get(j).unwrap_or(&Zero);
                }

                intermediate[i] = self.mux_1(n1, n2, shift);
            }
            next *= 2;
            number = intermediate.clone();
        }
        number
    }

    pub fn lzc(&mut self, number: Vec<Bit>) -> Vec<Bit> {
        let mut containers = self.lzc_init(number);
        let dummy_container = Vec::new();
        while containers.len() > 1 {
            let mut counter = 0;
            for i in (0..containers.len()).step_by(2) {
                let left = containers.get(i).unwrap_or(&dummy_container);
                let right = containers.get(i + 1).unwrap_or(&dummy_container);
                containers[counter] = self.lzc_merge(left, right);
                counter += 1;
            }
            containers.truncate(counter);
        }

        let mut result = containers.swap_remove(0);
        result.reverse();
        result
    }

    #[inline]
    fn lzc_init(&mut self, number: Vec<Bit>) -> Vec<Vec<Bit>> {
        let mut containers = Vec::with_capacity(number.len() / 2 + 1);
        for i in (0..number.len()).rev().step_by(2) {
            let mut container = Vec::with_capacity(2);
            let first = *number.get(i).unwrap_or(&One);
            // note wrapping_sub is a potential bug if vec contains usize::MAX elements
            let second = *number.get(i.wrapping_sub(1)).unwrap_or(&One);
            let all = self.or(first, second);
            container.push(all);
            container.push(first);
            containers.push(container);
        }
        containers
    }

    #[inline(always)]
    fn lzc_merge(&mut self, left: &Vec<Bit>, right: &Vec<Bit>) -> Vec<Bit> {
        let mut new_container = Vec::with_capacity(left.len() + 1);
        let left_all = left.get_or(0, One);
        let right_all = right.get_or(0, One);
        let all = self.or(left_all, right_all);
        let extra = left_all;

        new_container.push(all);
        new_container.push(extra);

        for i in 1..left.len() {
            // take right choice for bit at position n - i (^= 2^(n-2))?
            let take_right = self.not(extra);
            let filter_right = self.and(take_right, right.get_or(i, One));
            let decide = self.or(left.get_or(i, One), filter_right);
            new_container.push(decide);
        }
        new_container
    }

    #[allow(dead_code)]
    pub fn flip_est(&mut self, number: &Vec<Bit>) -> Vec<Bit> {
        let n = number.len();
        let mut estimate = vec![Zero, Zero, One];
        let mut number_reverse = number.clone();
        number_reverse.reverse();
        for i in 0..n {
            estimate.push(self.not(number[i]));
        }
        estimate.reverse();
        estimate
    }

    // this fn expects the input number to represent a number within the range [0.5, 1)
    // in little endian format
    // it outputs a number represented by its BitsVector to be in [1, 2) with 3 Bits leading
    // the "decimal" point e.g. 1.5 = 001.100...
    pub fn flip_estimate(&mut self, number: &Vec<Bit>) -> Vec<Bit> {
        let n = number.len();
        let mut estimate = vec![Zero, Zero, One];
        let mut number_reverse = number.clone();
        number_reverse.reverse();
        let mut first_six = self.flip_estimate_first_six(&number_reverse, 1);
        estimate.append(&mut first_six);
        if number.len() > 6 {
            for i in 7..(n + 1) {
                estimate.push(self.not(number_reverse.get_or(i, Zero)));
            }
        }
        estimate.truncate(n + 3);
        estimate.reverse();
        estimate
    }

    #[inline]
    fn flip_estimate_first_six(&mut self, number: &Vec<Bit>, idx: usize) -> Vec<Bit> {
        let s0 = number.get_or(idx, Zero);
        let s0_not = self.not(s0);
        let s1 = number.get_or(idx + 1, Zero);
        let s1_not = self.not(s1);
        let s2 = number.get_or(idx + 2, Zero);
        let s2_not = self.not(s2);
        let s3 = number.get_or(idx + 3, Zero);
        let s3_not = self.not(s3);
        let s4 = number.get_or(idx + 4, Zero);
        let s4_not = self.not(s4);
        let s5 = number.get_or(idx + 5, Zero);
        let s6_not = self.not(number.get_or(idx + 6, Zero));


        let e0 = self.and(s1, s2);
        // let select = e0;
        let e0 = self.not(e0);
        let e0 = self.and(s0_not, e0);

        let e1 = self.xor(s1, s2);
        let e1 = self.not(e1);
        let e1_temp = self.and(s0, s1);
        let e1_temp = self.not(e1_temp);
        let e1 = self.and(e1, e1_temp);

        let e2 = self.and(s0, s1);
        let select = e2;
        let e2 = self.not(e2);
        let e2 = self.and(s2, e2);
        let e2_temp = self.and(s0_not, s1_not);
        let e2_temp2 = self.and(s2_not, s3_not);
        let e2_temp = self.and(e2_temp, e2_temp2);
        let e2 = self.or(e2, e2_temp);

        let e3 = self.and(s4, s3);
        let e3 = self.not(e3);
        let e3 = self.and(s3_not, e3);
        let e3 = self.mux_1(s2_not, e3, select);

        let e4 = self.xor(s4, s5);
        let e4 = self.not(e4);
        let e4_temp = self.or(s3, s4);
        let e4_temp = self.not(e4_temp);
        let e4 = self.and(e4, e4_temp);
        let e4 = self.mux_1(s3_not, e4, select);

        let e5 = self.and(s3, s4);
        let e5 = self.not(e5);
        let e5_temp = self.or(s5, s6_not);
        let e5 = self.and(e5, e5_temp);
        let e5 = self.mux_1(s4_not, e5, select);
        vec![e0, e1, e2, e3, e4, e5]
    }

    pub fn table_estimate(&mut self, number: &Vec<Bit>) -> Vec<Bit> {
        let n = number.len();
        let mut number_reverse = number.clone();
        number_reverse.reverse();
        let mut estimate = vec![Zero, Zero, One];
        let s0 = number_reverse.get_or(1, Zero);
        let s1 = number_reverse.get_or(2, Zero);
        let s2 = number_reverse.get_or(3, Zero);
        let s3 = number_reverse.get_or(4, Zero);

        if n > 0 {
            let s0_not = self.not(s0);
            let temp = self.and(s1, s2);
            let temp = self.not(temp);
            let e1 = self.and(temp, s0_not);
            estimate.push(e1);

            if n > 1 {
                let s1_not = self.not(s1);
                let s2_not = self.not(s2);
                let s3_not = self.not(s3);
                let temp1 = self.and(s1_not, s2_not);

                let temp2 = self.and(s0_not, s2);
                let temp3 = self.or(s1, s3_not);
                let temp3 = self.and(temp2, temp3);
                
                let e2 = self.or(temp1, temp3);
                estimate.push(e2);

                if n > 2 {
                    let temp1 = self.xor(s1, s2);
                    let temp1 = self.not(temp1);
                    let temp1 = self.and(s0_not, temp1);

                    let temp2 = self.and(s0, s2);
                    let temp2 = self.and(temp2, s1_not);
                    let temp7 = self.and(s0, s1); // changed from temp3 to temp7 to reuse
                    let temp9 = self.and(s2_not, s3_not); // changed from temp4 to temp9 to reuse later
                    let temp3 = self.and(temp7, temp9);
                    let temp8 = self.and(s0_not, s1_not); // changed from temp4 to temp8 to reuse
                    let temp4 = self.and(s2, s3);
                    let temp4 = self.and(temp8, temp4);

                    let temp1 = self.or(temp1, temp2);            
                    let temp3 = self.or(temp3, temp4);

                    let e3 = self.or(temp1, temp3);
                    estimate.push(e3);

                    if n > 3 {
                        let temp1 = self.and(s0, s1);
                        let temp2 = self.and(s2_not, s3);
                        let temp1 = self.and(temp1, temp2);

                        let temp3 = self.and(s0, s2);
                        let temp3 = self.and(temp3, s3_not);
                        let temp1 = self.or(temp1, temp3);

                        let temp5 = self.and(s0_not, s3_not);
                        let temp5 = self.and(temp5, s1);

                        let temp10 = self.and(s1_not, s2_not);
                        let temp10 = self.and(temp10, s3_not);
                        let temp5 = self.or(temp5, temp10);
                        
                        let e4 = self.or(temp1, temp5);
                        estimate.push(e4);

                        if n > 4 {
                            let temp1 = self.and(s0_not, s2_not);
                            let temp1 = self.and(temp1, s3_not);

                            let temp2 = self.or(s0, s3);
                            let temp3 = self.and(s1_not, s2);
                            let temp2 = self.and(temp2, temp3);
                            let temp1 = self.or(temp1, temp2);

                            let temp2 = self.and(s0, s1);
                            let temp2 = self.and(temp2, s3);

                            let e5 = self.or(temp1, temp2);
                            estimate.push(e5);

                            if n > 5 {
                                let temp1 = self.and(s2_not, s3_not);
                                let temp2 = self.and(s1_not, s3);
                                let temp1 = self.or(temp1, temp2);
                                let temp1 = self.and(s0, temp1);

                                let temp2 = self.and(s1, s3);
                                let temp2 = self.or(temp2, s2);
                                let temp3 = self.and(s1_not, s3_not);
                                let temp2 = self.or(temp2, temp3);
                                let temp2 = self.and(s0_not, temp2);
                                
                                let e6= self.or(temp1, temp2);
                                estimate.push(e6);

                                if n > 6 {
                                    let temp1 = self.and(s0_not, s3_not);
                                    let temp2 = self.and(s0, s3);
                                    let temp2 = self.or(s2, temp2);
                                    let temp1 = self.or(temp1, temp2);
                                    let temp1 = self.and(s1_not, temp1);

                                    let temp2 = self.and(s1, s3);
                                    let temp2 = self.and(temp2, s2_not);

                                    let e7 = self.or(temp1, temp2);
                                    estimate.push(e7);

                                    if n > 7 {
                                        let temp1 = self.xor(s0, s2);
                                        let temp1 = self.not(temp1);
                                        let temp1 = self.and(s3_not, temp1);
                                        let e8 = self.or(temp1, s1_not);
                                        estimate.push(e8);

                                        if n > 8 {
                                            let temp1 = self.and(s1_not, s2_not);
                                            let temp1 = self.and(temp1, s3);

                                            let temp2 = self.and(s1, s2_not);
                                            let temp2 = self.and(temp2, s3_not);

                                            let temp3 = self.and(s0_not, s1_not);
                                            let temp3 = self.and(temp3, s2_not);

                                            let temp1 = self.or(temp1, temp2);
                                            let e9= self.or(temp1, temp3);
                                            estimate.push(e9);

                                            if n > 9 {
                                                let temp1 = self.and(s0_not, s1);
                                                let temp2 = self.and(s2, s3_not);
                                                let temp1 = self.and(temp1, temp2);

                                                let temp2 = self.and(s1_not, s2_not);
                                                let temp3 = self.or(s0_not, s3_not);
                                                let temp2 = self.and(temp2, temp3);

                                                let temp1 = self.or(temp1, temp2);

                                                let temp2 = self.and(s0, s2);
                                                let temp2 = self.and(temp2, s3);
                                                
                                                let temp3 = self.and(s0, s1);
                                                let temp3 = self.and(temp3, s3);

                                                let temp2 = self.or(temp2, temp3);

                                                let e10 = self.or(temp1, temp2);
                                                estimate.push(e10);

                                                for i in 10..(10.max(n)) {
                                                    if i.is_odd() {
                                                        estimate.push(One);
                                                    } else {
                                                        estimate.push(Zero);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            
                        }
                    }
                }
            }
        }
        estimate.reverse();
        estimate
    }

    // comperator circuit for a < b
    // has two outputs LT (a < b ?) and EQ (a = b ?)
    // if both outputs are low you'll get GT (a > b ?) obviously
    pub fn less_than(&mut self, a: &Vec<Bit>, b: &Vec<Bit>) -> (Bit, Bit) {
        let max_bits = a.len().max(b.len());
        // on the first level we compare for each bit if a[i] is less than b[i] and also if 
        // a[i] is equal to b[i]

        // we need this que to later merge the outputs from the first level and further down
        // the road each time for the previous level we merge the corresponding two output
        // signals
        let mut previous = VecDeque::with_capacity(max_bits * 2);
        let mut current = VecDeque::with_capacity(2 * max_bits);
        for i in (0..max_bits).rev() {
            let a_i = a.get_or(i, Zero);
            let b_i = b.get_or(i, Zero);

            // EQ for two bits is the XNOR (≡)
            let eq = self.xor(a_i, b_i);
            let eq = self.not(eq);

            // LT is !a[i] & b[i]
            let lt = self.not(a_i);
            let lt = self.and(lt, b_i);

            previous.push_back(lt);
            previous.push_back(eq);
            // println!("{previous:?}");
        }

        while previous.len() > 2 {
            while !previous.is_empty() {
                let left_lt = previous.pop_front().unwrap();
                let left_eq = previous.pop_front().unwrap();
                let right_lt = previous.pop_front().unwrap_or(Zero);
                let right_eq = previous.pop_front().unwrap_or(One);
    
                // now merge the the two LT signals
                // the new LT = LT_left || (LT_right & EQ_left)
                let lt = self.and(right_lt, left_eq);
                let lt = self.or(left_lt, lt);
    
                // merging EQ_left and EQ_right is straight forward
                // new EQ = (EQ_left & EQ_right)
                let eq = self.and(left_eq, right_eq);
    
                current.push_back(lt);
                current.push_back(eq);
                // println!("{current:?}");
            }
            previous = current.clone();
            current.clear();
        }

        (previous.pop_front().unwrap(), previous.pop_front().unwrap())
    }
}

impl Circuit {
    pub fn get_lzc_circuit(bits: usize, remove_dead_ends: bool) -> Circuit {
        let mut circuit = Circuit::new();
        let mut number = vec![];
        // create input vector of bit variables
        for _ in 0..bits {
            number.push(circuit.new_line());
        }

        let leading_zero_count = circuit.lzc(number.clone());

        circuit.add_as_io(&number, "n", false);
        circuit.add_as_io(&leading_zero_count, "lzc", true);
        if remove_dead_ends {
            circuit.remove_dead_ends();
        }
        circuit
    }

    pub fn get_lt(bits: usize) -> Circuit {
        let mut circuit = Circuit::new();
        let mut a = vec![];
        let mut b = vec![];

        for _ in 0..bits {
            a.push(circuit.new_line());
        }

        for _ in 0..bits {
            b.push(circuit.new_line());
        }

        let (lt, eq) = circuit.less_than(&a, &b);

        circuit.add_as_io(&a, "X", false);
        circuit.add_as_io(&b, "Y", false);
        circuit.add_as_io(&vec![lt], "LT", true);
        circuit.add_as_io(&vec![eq], "EQ", true);
        circuit
    }
}