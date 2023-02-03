#[cfg(test)]
mod tests {
    use num::{BigInt, FromPrimitive, Signed, traits::Pow};

    use crate::{
        data::{Bit, Bit::One, Bit::Zero, Circuit, Shift, Mul},
        dividers::{DivInfo, Estimate, IntDivResult, SubMethod},
    };
    use rand::random;
    
    const TEST_SIZE_SMALL: usize = 1_000;
    const TEST_SIZE_BIG: usize = 10_000;

    fn get_random_number_with_fixed_digits(mut digits: usize) -> (u128, Vec<Bit>) {
        let mut s1: u128 = random();
        digits = digits.clamp(1, 127);
        let mask: u128 = (1_u128 << (digits)).saturating_sub(1);
        s1 &= mask;
        let mut v1 = Bit::get_bits_vec_u128(s1);
        v1.truncate(digits as usize);
        //print!("{}", v1.len());
        (s1, v1)
    }

    fn get_random_number_with_random_number_of_digits() -> (u128, Vec<Bit>) {
        let mut s1: u128 = random();
        let mut digits: u8 = random();
        digits %= 128;
        if digits == 0 {
            digits = 1;
        }
        let mask: u128 = (0x1_u128 << (digits)).saturating_sub(1);
        s1 &= mask;
        let mut v1 = Bit::get_bits_vec_u128(s1);
        v1.truncate(digits as usize);
        //print!("{}", v1.len());
        (s1, v1)
    }

    #[test]
    fn test_bits_to_num_u8() {
        for n in 0..256 {
            let v = Bit::get_bits_vec_u8(n as u8);
            let translation = Bit::get_number_u(&v).unwrap();
            assert_eq!(translation, n);
        }
    }

    #[test]
    fn mux_n_2() {
        let mut circuit = Circuit::new();
            for _i in 0..100000 {
                let b1: usize = rand::random();
                let b2: usize = rand::random();
                let b3: usize = rand::random();

                let v1 = Bit::get_bits_vec_usize(b1);
                let v2 = Bit::get_bits_vec_usize(b2);
                let v3 = Bit::get_bits_vec_usize(b3);

                let r1 = circuit.mux_n_2(&v1, &v2, &v3, (Zero, Zero));
                let r2 = circuit.mux_n_2(&v1, &v2, &v3, (One, Zero));
                let r3 = circuit.mux_n_2(&v1, &v2, &v3, (Zero, One));
                let r4 = circuit.mux_n_2(&v1, &v2, &v3, (One, One));

                let calc1 = Bit::get_number_u(&r1).unwrap();
                let calc2 = Bit::get_number_u(&r2).unwrap();
                let calc3 = Bit::get_number_u(&r3).unwrap();
                let calc4 = Bit::get_number_u(&r4).unwrap();

                assert_eq!(calc1, b1);
                assert_eq!(calc2, b2);
                assert_eq!(calc3, b3);
                assert_eq!(calc4, b3);
            }
    }

    #[test]
    fn mux_n_4() {
        let mut circuit = Circuit::new();
        for _ in 0..TEST_SIZE_BIG {
            let (r1, b1) = get_random_number_with_fixed_digits(16);
            let (r2, b2) = get_random_number_with_fixed_digits(16);
            let (r3, b3) = get_random_number_with_fixed_digits(16);
            let (r4, b4) = get_random_number_with_fixed_digits(16);
            //println!("1.{}\n2.{}\n3.{}\n4.{}\n", r1, r2, r3, r4);

            let select = (Zero, Zero);
            let r_calc = circuit.mux_n_4(&b1, &b2, &b3, &b4, select);
            let r_calc = Bit::get_number_u128(&r_calc).unwrap();
            //println!("calculated {} =! {}", r_calc, r1);
            assert_eq!(r1, r_calc);

            let select = (One, Zero);
            let r_calc = circuit.mux_n_4(&b1, &b2, &b3, &b4, select);
            let r_calc = Bit::get_number_u128(&r_calc).unwrap();
            //println!("calculated {} =! {}", r_calc, r2);
            assert_eq!(r2, r_calc);

            let select = (Zero, One);
            let r_calc = circuit.mux_n_4(&b1, &b2, &b3, &b4, select);
            let r_calc = Bit::get_number_u128(&r_calc).unwrap();
            //println!("calculated {} =! {}", r_calc, r3);
            assert_eq!(r3, r_calc);

            let select = (One, One);
            let r_calc = circuit.mux_n_4(&b1, &b2, &b3, &b4, select);
            let r_calc = Bit::get_number_u128(&r_calc).unwrap();
            //println!("calculated {} =! {}", r_calc, r4);
            assert_eq!(r4, r_calc);
        }
    }

    fn test_adder_random_size(adder: fn(&mut Circuit, Vec<Bit>, Vec<Bit>, Bit) -> Vec<Bit>) {
        let mut circuit = Circuit::new();
        for _i in 0..TEST_SIZE_BIG {
            let (s1, v1) = get_random_number_with_random_number_of_digits();
            let (s2, v2) = get_random_number_with_random_number_of_digits();

            let (r, _) = s1.overflowing_add(s2);

            let r_calc = adder(&mut circuit, v1.clone(), v2.clone(), Zero);
            let r_calc = match Bit::get_number_u128(&r_calc) {
                Ok(r) => r,
                Err(_) => { continue; }
            };

            assert_eq!(r, r_calc);

            let r_calc_with_carry = adder(&mut circuit, v1.clone(), v2.clone(), One);
            let r_calc_with_carry = match Bit::get_number_u128(&r_calc_with_carry) {
                Ok(r) => r,
                Err(_) => { continue; }
            };
            assert_eq!(r.overflowing_add(1).0, r_calc_with_carry);
        }
    }

    #[test]
    fn adders_random_size() {
        test_adder_random_size(Circuit::cra);
        test_adder_random_size(Circuit::csa);
        test_adder_random_size(Circuit::ksa);
    }

    fn test_subs_random_size(sub: fn(&mut Circuit, Vec<Bit>, Vec<Bit>, Bit) -> Vec<Bit>) {
        let mut circuit = Circuit::new();
        for _i in 0..TEST_SIZE_BIG {
            let (mut s1, mut v1) = get_random_number_with_random_number_of_digits();
            let (mut s2, mut v2) = get_random_number_with_random_number_of_digits();

            if s1 < s2 {
                std::mem::swap(&mut s1, &mut s2);
                std::mem::swap(&mut v1, &mut v2);
            }

            let r = s1 - s2;

            let r_calc = sub(&mut circuit, v1.clone(), v2.clone(), Zero);
            let r_calc = Bit::get_number_u128(&r_calc).unwrap();

            assert_eq!(r, r_calc);
        }
    }

    #[test]
    fn subs_random_size() {
        test_subs_random_size(Circuit::crs);
        test_subs_random_size(Circuit::css);
        test_subs_random_size(Circuit::kss);
    }

    #[test]
    fn test_kss_one_example() {
        let mut circuit = Circuit::new();
        let s1 = 16;
        let s2 = 9;

        let v1 = Bit::get_bits_vec_u8(s1);
        let v2 = Bit::get_bits_vec_u8(s2);

        let r = s1 - s2;

        let r_calc = circuit.kss(v1.clone(), v2.clone(), Zero);
        let r_calc = Bit::get_number_u128(&r_calc).unwrap();

        assert_eq!(r, r_calc as u8);
    }

    #[test]
    fn test_shift_2() {
        let mut circuit = Circuit::new();
        for _i in 0..TEST_SIZE_SMALL {
            let n: usize = rand::random();
            let shift: u8 = rand::random::<u8>() & 0b0011_1111;

            let v_n = Bit::get_bits_vec_usize(n);
            let not_shift = !shift;
            let mut v_not_shift = Bit::get_bits_vec_u8(not_shift);
            v_not_shift.truncate(6);

            let r1 = n << shift;
            let r2 = n >> shift;

            let calc1 = circuit.shift(v_n.clone(), v_not_shift.clone(), Shift::Left, Zero);
            let calc2 = circuit.shift(v_n.clone(), v_not_shift.clone(), Shift::Right, Zero);

            let calc1 = Bit::get_number_u(&calc1).unwrap();
            let calc2 = Bit::get_number_u(&calc2).unwrap();

            if calc1 != r1 {
                println!(
                    "Left Shift by {}: {} = {} is {}",
                    shift,
                    r1,
                    calc1,
                    r1 == calc1
                );
            }
            if calc2 != r2 {
                println!(
                    "Right Shift by {}: {} = {} is {}",
                    shift,
                    r2,
                    calc2,
                    r2 == calc2
                );
            }
            assert_eq!(r1, calc1);
            assert_eq!(r2, calc2);
        }
    }

    #[test]
    fn test_shift() {
        let mut circuit = Circuit::new();
        for _i in 0..TEST_SIZE_SMALL {
            let (n, v) = get_random_number_with_random_number_of_digits();
            let digits = 7;
            let (shift_by, v_s) = get_random_number_with_fixed_digits(digits);
            let shift_by = shift_by as u8;
            let mut v_ns = v_s.clone();
            circuit.not_all(&mut v_ns);
            println!("shift_by {shift_by}, vec: {:?}", v_s);
            println!("shift_by {shift_by}, vec: {:?}", v_ns);

            let test = Bit::get_number_u128(&v).unwrap();
            assert_eq!(n, test);
            let mask: u128 = (0x1_u128 << (v.len())).saturating_sub(1);

            let r = (n << shift_by) & mask;
            let r_calc = circuit.shift(v.clone(), v_s.clone(), Shift::Left, One);
            let r_calc = Bit::get_number_u128(&r_calc).unwrap();
            assert_eq!(r, r_calc);

            let r = n >> shift_by;
            let r_calc = circuit.shift(v.clone(), v_s.clone(), Shift::Right, One);
            let r_calc = Bit::get_number_u128(&r_calc).unwrap();
            assert_eq!(r, r_calc);

            let r = (n << shift_by) & mask;
            println!("{n}, {r}");
            println!("number {:?}", v);
            let r_calc = circuit.shift(v.clone(), v_ns.clone(), Shift::Left, Zero);
            println!("after calc: {:?}", r_calc);
            let r_calc = Bit::get_number_u128(&r_calc).unwrap();
            assert_eq!(r, r_calc);

            let r = n >> shift_by;
            let r_calc = circuit.shift(v, v_ns.clone(), Shift::Right, Zero);
            let r_calc = Bit::get_number_u128(&r_calc).unwrap();
            assert_eq!(r, r_calc);
        }
    }

    #[test]
    fn test_lzc() {
        let mut circuit = Circuit::new();

        for n in 1..256 {
            let lz = (n as u8).leading_zeros();
            // println!("{} || {}", n, lz);
            let v = Bit::get_bits_vec_u8(n as u8);
            let mut lz_calc = circuit.lzc(v);
            lz_calc.truncate(3);
            circuit.not_all(&mut lz_calc);
            // println!("{:?}", lz_calc);
            let lz_calc = Bit::get_number_u(&lz_calc).unwrap();
            // println!("{}", lz_calc);
            assert_eq!(lz as usize, lz_calc);
        }

        for n in 1..(u16::MAX) {
            let lz = (n as u16).leading_zeros();
            // println!("{} || {}", n, lz);
            let v = Bit::get_bits_vec_u16(n as u16);
            let mut lz_calc = circuit.lzc(v);
            lz_calc.truncate(4);
            circuit.not_all(&mut lz_calc);
            // println!("{:?}", lz_calc);
            let lz_calc = Bit::get_number_u(&lz_calc).unwrap();
            // println!("{}", lz_calc);
            assert_eq!(lz as usize, lz_calc);
        }

        for _ in 0..TEST_SIZE_BIG {
            let (n, v) = get_random_number_with_random_number_of_digits();

            // the lzc doesnt need to give a correct answer when all bits are zero
            if n == 0 {
                continue;
            }
            println!("{:?}", v);
            let lz_count = n.leading_zeros() as usize - (128 - v.len());
            let mut lz_calc = circuit.lzc(v.clone());
            lz_calc.pop();
            println!(
                "n was {n} while {:?}, leading zeroes {}",
                lz_calc,
                n.leading_zeros()
            );
            circuit.not_all(&mut lz_calc);
            // fails for v.len() == 1 because log_2(1) == 0
            // assert_eq!(lz_calc.len(), (v.len() as f64).log2().ceil() as usize);

            let lz_calc = Bit::get_number_u(&lz_calc).unwrap();
            assert_eq!(lz_count, lz_calc);
        }
    }

    #[test]
    fn test_mul_u_dadda_fixed_size() {
        let mut circuit = Circuit::new();
        for i in 0..TEST_SIZE_BIG {
            let m1: u32 = rand::random();
            let m2: u32 = rand::random();
            let s: u64 = if i % 2 == 0 { rand::random() } else { 0 };
            let p = (m1 as u64) * (m2 as u64);
            let (sum, c) = s.overflowing_add(p);
            if c {
                continue;
            }
            let v1 = Bit::get_bits_vec_u32(m1);
            let v2 = Bit::get_bits_vec_u32(m2);
            let v3 = Bit::get_bits_vec_u64(s);
            let p_calc = circuit.mul_unsigned_clean(v1, v2, Some(v3), crate::data::Adder::KSA);
            let p_calc = Bit::get_number_u(&p_calc).unwrap();
            assert_eq!(sum as usize, p_calc);
            if sum as usize != p_calc {
                println!("{} * {} + {} = {}   =  {:?} ", m1, m2, s, sum, p_calc);
            }
        }
    }

    #[test]
    fn array_mul() {
        let mut circuit = Circuit::new();
        for i in 0..TEST_SIZE_BIG {
            let m1: u32 = rand::random();
            let m2: u32 = rand::random();
            let s: u64 = if i % 2 == 0 { rand::random() } else { 0 };
            let p = (m1 as u64) * (m2 as u64);
            let (sum, c) = s.overflowing_add(p);
            if c {
                continue;
            }
            let v1 = Bit::get_bits_vec_u32(m1);
            let v2 = Bit::get_bits_vec_u32(m2);
            let v3 = Bit::get_bits_vec_u64(s);
            let p_calc = circuit.array_mul(v1, v2, Some(v3), crate::data::Adder::CRA);
            let p_calc = Bit::get_number_u(&p_calc).unwrap();
            assert_eq!(sum as usize, p_calc);
            if sum as usize != p_calc {
                println!("{} * {} + {} = {}   =  {:?} ", m1, m2, s, sum, p_calc);
            }
        }
    }

    #[test]
    fn array_mul_random_sizes() {
        let mut circuit = Circuit::new();
        let mut counter = 0;
        while counter < TEST_SIZE_BIG {
            let (s1, v1) = get_random_number_with_random_number_of_digits();
            let (f1, v2) = get_random_number_with_random_number_of_digits();
            let (f2, v3) = get_random_number_with_random_number_of_digits();

            let (p, mul_overflow) = f1.overflowing_mul(f2);
            let (r, skip) = s1.overflowing_add(p);
            if skip || mul_overflow {
                continue;
            }

            let r_calc = circuit.array_mul(v3.clone(), v2.clone(), Some(v1.clone()), crate::data::Adder::CRA);
            let r_calc2 = circuit.array_mul(v3.clone(), v2.clone(), Some(v1.clone()), crate::data::Adder::CSA);
            let r_calc3 = circuit.array_mul(v3.clone(), v2.clone(), Some(v1.clone()), crate::data::Adder::KSA);
            let r_calc = match Bit::get_number_u128(&r_calc) {
                Ok(r) => r,
                Err(_) => { continue; }
            };
            let r_calc2= match Bit::get_number_u128(&r_calc2) {
                Ok(r) => r,
                Err(_) => { continue; }
            };
            let r_calc3= match Bit::get_number_u128(&r_calc3) {
                Ok(r) => r,
                Err(_) => { continue; }
            };

            assert_eq!(r, r_calc);
            assert_eq!(r, r_calc2);
            assert_eq!(r, r_calc3);
            counter += 1;
        }
    }

    #[test]
    fn test_fused_multiply_subtract_u8() {
        let mut circuit = Circuit::new();
        for minuend in 0..u8::MAX {
            for f1 in 0..16 {
                for f2 in 0..16 {
                    let v1 = Bit::get_bits_vec_u8(minuend);
                    let v2 = Bit::get_bits_vec_u8(f1);
                    let v3 = Bit::get_bits_vec_u8(f2);
            
                    let r_calc = circuit.fused_mul_subtraction(v1.clone(), v2.clone(), v3.clone(), false);
                    let r_calc = Bit::get_number_u(&r_calc).unwrap();
                    let r = minuend.overflowing_sub(f1 * f2).0;
            
                    assert_eq!(r, r_calc as u8);
                }
            }
        }
    }

    #[test]
    fn test_fused_multiply_subtract_example() {
        let mut circuit = Circuit::new();

        let minuend = u16::MAX - 256;
        let f1 = 255;
        let f2 = 256;

        let v1 = Bit::get_bits_vec_u16(minuend);
        let v2 = Bit::get_bits_vec_u16(f1);
        let v3 = Bit::get_bits_vec_u16(f2);

        let r_calc = circuit.fused_mul_subtraction(v1.clone(), v2.clone(), v3.clone(), false);
        let r_calc = Bit::get_number_u(&r_calc).unwrap();
        let r = minuend.overflowing_sub(f1 * f2).0;
        println!("{minuend} - {f1} * {f2} = {r}");

        assert_eq!(r, r_calc as u16);
    }

    #[test]
    fn test_fused_multiply_subtract_example_u128() {
        let mut circuit = Circuit::new();

        let minuend = 58385310019316736931060226292976234401;
        let f1 = 343968725963202977;
        let f2 = 1720773172939479196;

        let mut v1 = Bit::get_bits_vec_u128(minuend);
        let mut v2 = Bit::get_bits_vec_u64(f1 as u64);
        let mut v3 = Bit::get_bits_vec_u64(f2 as u64);

        v1.truncate(126);
        v2.truncate(63);
        v3.truncate(63);

        let r_calc = circuit.fused_mul_subtraction(v1.clone(), v2.clone(), v3.clone(), false);
        let r_calc = Bit::get_number_u128(&r_calc).unwrap();
        let r = minuend.overflowing_sub(f1 * f2).0;
        println!("{minuend} - {f1} * {f2} = {r}");

        assert_eq!(r, r_calc);
    }

    #[test]
    fn test_fused_multiply_subtract() {
        let mut circuit = Circuit::new();
        let mut counter = 0;
        while counter < TEST_SIZE_BIG {
            let (minuend, v1) = get_random_number_with_random_number_of_digits();
            let (f1, v2) = get_random_number_with_random_number_of_digits();
            let (f2, v3) = get_random_number_with_random_number_of_digits();

            let (p, mul_overflow) = f1.overflowing_mul(f2);
            let (r, skip) = minuend.overflowing_sub(p);
            if skip || mul_overflow {
                // println!("skip! {minuend} - {f1} * {f2} = {r}");
                continue;
            }
           // println!("{minuend} - {f1} * {f2} = {r}");

            let r_calc = circuit.fused_mul_subtraction(v1.clone(), v2.clone(), v3.clone(), false);
            // println!("expected_length: {}bits || new_length: {}bits", v1.len().max(v2.len() + v3.len()), r_calc.len());

            // error seems to occur in translation! why?
            let r_calc = match Bit::get_number_u128(&r_calc) {
                Ok(r) => r,
                Err(_) => { continue; }
            };

            assert_eq!(r, r_calc);
            counter += 1;
        }
    }

    #[test]
    fn test_div_simple_fixed_size() {
        let mut circuit = Circuit::new();
        for d_num in 0..=255 {
            for z_num in 1..=255 {
                let mut d = Bit::get_bits_vec_u8(d_num);
                let mut z = Bit::get_bits_vec_u8(z_num);

                let q = d_num / z_num;
                let r = d_num % z_num;
                d.truncate(8);
                z.truncate(8);
                // println!("dividend: {}={:?}, divisor: {}={:?}", d_num, d, z_num, z);

                let mut info = DivInfo::default_newton();
                info.estimator = Estimate::Flip5bit;
                info.defaultmult = Mul::DadaTree;
                let result = circuit.div_newton(d, z, info);
                let q_calc = Bit::get_number_u(&result.q).unwrap();
                let r_calc = Bit::get_number_u(&result.r).unwrap();

                if !(q_calc == q as usize && r_calc == r as usize) {
                    println!("some_thing_to_correct! d: {}, z: {}", d_num, z_num);
                }
            }
        }
    }

    #[test]
    fn test_div_fused_fixed_size_() {
        let mut circuit = Circuit::new();
        for d_num in 0..=255 {
            for z_num in 1..=255 {
                let mut d = Bit::get_bits_vec_u8(d_num);
                let mut z = Bit::get_bits_vec_u8(z_num);

                let q = d_num / z_num;
                let r = d_num % z_num;
                d.truncate(8);
                z.truncate(8);
                // println!("dividend: {}={:?}, divisor: {}={:?}", d_num, d, z_num, z);

                let mut info = DivInfo::default_newton();
                info.sub_method = SubMethod::Fused;
                info.estimator = Estimate::Table10bit;
                let result = circuit.div_newton(d, z, info);
                let q_calc = Bit::get_number_u(&result.q).unwrap();
                let r_calc = Bit::get_number_u(&result.r).unwrap();

                if !(q_calc == q as usize && r_calc == r as usize) {
                    println!("some_thing_to_correct! d: {}, z: {}", d_num, z_num);
                }
            }
        }
    }

    #[test]
    fn test_div_simple_fixed_size_10bit() {
        let mut circuit = Circuit::new();
        let mut wrong_count = 0;
        for _ in 64900..64901 {
            for z_num in 1..0x000f_ffff {
                let d_num = u32::MAX;
                let d = Bit::get_bits_vec_u32(d_num);
                let z = Bit::get_bits_vec_u32(z_num);

                let q = d_num / z_num;
                let r = d_num % z_num;
                // println!("dividend: {}={:?}, divisor: {}={:?}", d_num, d, z_num, z);

                let mut info = DivInfo::default_newton();
                info.estimator = Estimate::Table10bit;
                let result = circuit.div_newton(d, z, info);
                let q_calc = Bit::get_number_u(&result.q).unwrap();
                let r_calc = Bit::get_number_u(&result.r).unwrap();

                if !(q_calc == q as usize && r_calc == r as usize) {
                    println!("some_thing_to_correct! d: {}, z: {}", d_num, z_num);
                    wrong_count += 1;
                }
            }
        }
        println!("wrong!: {}", wrong_count);
    }

    #[test]
    fn test_div_simple_random_size() {
        let mut circuit = Circuit::new();
        for _i in 0..10000 {
            let (r0, v_r0) = get_random_number_with_random_number_of_digits();
            let (d, v_d) = get_random_number_with_fixed_digits(v_r0.len());

            if d == 0 {
                continue;
            }

            let q = r0 / d;
            let r = r0 % d;

            let mut info = DivInfo::default_newton();
            info.estimator = Estimate::Flip5bit;
            info.defaultmult = Mul::Array;
            let result = circuit.div_newton(v_r0, v_d, info);

            let q_calc = Bit::get_number_u128(&result.q).unwrap();
            let r_calc = Bit::get_number_u128(&result.r).unwrap();

            println!("{r0} / {d} = {q_calc}\n{r0} % {d} = {r_calc}\n");

            assert_eq!(q, q_calc);
            assert_eq!(r, r_calc);
        }
    }

    #[test]
    fn newt_div_test_divisors() {
        let mut circuit = Circuit::new();
        for _i in 0..10000 {
            let (r0, v_r0) = (u64::MAX, Bit::get_bits_vec_u64(u64::MAX));
            let (d, v_d) = get_random_number_with_fixed_digits(v_r0.len());
            let d = d as u64;

            if d == 0 {
                continue;
            }

            let q = r0 / d;
            let r = r0 % d;

            let mut info = DivInfo::default_newton();
            info.estimator = Estimate::Flip5bit;
            info.defaultmult = Mul::Array;
            let result = circuit.div_newton(v_r0, v_d, info);

            let q_calc = Bit::get_number_u128(&result.q).unwrap();
            let r_calc = Bit::get_number_u128(&result.r).unwrap();

            println!("{r0} / {d} = {q_calc}\n{r0} % {d} = {r_calc}\n");

            assert_eq!(q, q_calc as u64);
            assert_eq!(r, r_calc as u64);
        }
    }

    #[test]
    fn test_div_simple_random_size_worst_divisor() {
        let mut circuit = Circuit::new();
        for i in 6..128 {
            let number_bits = i;
            let biggest_int = 2u128.pow(number_bits) - 1;
            let (r0, mut v_r0) = (biggest_int, Bit::get_bits_vec_u128(biggest_int));
            let (d, mut v_d) = (3, Bit::get_bits_vec_u128(3));
            v_d.truncate(number_bits as usize);
            v_r0.truncate(number_bits as usize);
            //println!("{:?}", v_r0);
            println!("{r0} / {d}");

            let q = r0 / d;
            let r = r0 % d;
            //println!("{r0}  {d}");
            //println!("{q}, {r}");
            //println!("{:?}", v_r0);

            let mut info = DivInfo::default_newton();
            info.estimator = Estimate::Table10bit;
            info.sub_method = SubMethod::Fused;
            let result = circuit.div_newton(v_r0, v_d, info);

            let q_calc = Bit::get_number_u128(&result.q).unwrap();
            let r_calc = Bit::get_number_u128(&result.r).unwrap();

            //println!("{r0} / {d} = {q_calc}\n{r0} % {d} = {r_calc}\n");
            //println!("{r0} / {d} = {q}\n{r0} % {d} = {r}\n");

            assert_eq!(q, q_calc);
            assert_eq!(r, r_calc);
        }
    }

    #[test]
    fn test_errors_for_div_simple() {
        let mut circuit = Circuit::new();
        let r_0 = 2.pow(15u16) as u16 - 1;
        let d = 2.pow(15u16) as u16 - 2;
        let v_r0 = Bit::get_bits_vec_u16(r_0);
        let v_d = Bit::get_bits_vec_u16(d);

        println!("{} = {}", v_r0.len(), v_d.len());

        let mut info = DivInfo::default_newton();
        info.estimator = Estimate::None;
        let IntDivResult {
            q: q_calc,
            r: r_calc,
            ok: _,
        } = circuit.div_newton(v_r0, v_d, info);

        let q_calc = Bit::get_number_u(&q_calc).unwrap();
        let r_calc = Bit::get_number_u(&r_calc).unwrap();

        println!("quotient: {}, remainder: {}", q_calc, r_calc);
    }

    #[test]
    fn test_one_example_for_div_goldschmidt() {
        let mut circuit = Circuit::new();
        let r_0 = 205;
        let d = 1;
        let v_r0 = Bit::get_bits_vec_u8(r_0);
        let v_d = Bit::get_bits_vec_u8(d);

        println!("{} = {}", v_r0.len(), v_d.len());

        let IntDivResult {
            q: q_calc,
            r: r_calc,
            ok: _,
        } = circuit.goldschmidt_divider(v_r0, v_d, DivInfo::default_goldschmidt());

        let q_calc = Bit::get_number_u(&q_calc).unwrap();
        let r_calc = Bit::get_number_u(&r_calc).unwrap();

        println!("quotient: {}, remainder: {}", q_calc, r_calc);
    }

    #[test]
    fn test_div_goldschmidt_fixed_size() {
        let mut circuit = Circuit::new();
        for d_num in u16::MAX..=u16::MAX {
            for z_num in 1..=u16::MAX {
                let d = Bit::get_bits_vec_u16(d_num);
                let z = Bit::get_bits_vec_u16(z_num);

                let q = d_num / z_num;
                let r = d_num % z_num;
                // println!("dividend: {}={:?}, divisor: {}={:?}", d_num, d, z_num, z);

                let result = circuit.goldschmidt_divider(d, z, DivInfo::default_goldschmidt());
                let q_calc = Bit::get_number_u(&result.q).unwrap();
                let r_calc = Bit::get_number_u(&result.r).unwrap();

                if !(q_calc == q as usize && r_calc == r as usize) {
                    println!("some_thing_to_correct! d: {}, z: {}", d_num, z_num);
                }
                assert_eq!(q_calc, q as usize);
                assert_eq!(r_calc, r as usize);
            }
        }
    }

    #[test]
    fn test_div_goldschmidt_fixed_size_all_u8() {
        let mut circuit = Circuit::new();
        for d_num in 0..=u8::MAX {
            for z_num in 1..=u8::MAX {
                let d = Bit::get_bits_vec_u8(d_num);
                let z = Bit::get_bits_vec_u8(z_num);

                let q = d_num / z_num;
                let r = d_num % z_num;
                // println!("dividend: {}={:?}, divisor: {}={:?}", d_num, d, z_num, z);

                let result = circuit.goldschmidt_divider(d, z, DivInfo::default_goldschmidt());
                let q_calc = Bit::get_number_u(&result.q).unwrap();
                let r_calc = Bit::get_number_u(&result.r).unwrap();

                if !(q_calc == q as usize && r_calc == r as usize) {
                    println!("some_thing_to_correct! d: {}, z: {}", d_num, z_num);
                }
                assert_eq!(q_calc, q as usize);
                assert_eq!(r_calc, r as usize);
            }
        }
    }

    #[test]
    fn test_div_goldschmidt_random_size() {
        let mut circuit = Circuit::new();
        for _i in 0..10000 {
            let (r0, v_r0) = get_random_number_with_random_number_of_digits();
            let (d, v_d) = get_random_number_with_fixed_digits(v_r0.len());

            if d == 0 {
                continue;
            }

            let q = r0 / d;
            let r = r0 % d;

            let result = circuit.goldschmidt_divider(v_r0, v_d, DivInfo::default_goldschmidt());

            let q_calc = Bit::get_number_u128(&result.q).unwrap();
            let r_calc = Bit::get_number_u128(&result.r).unwrap();

            println!("{r0} / {d} = {q_calc}\n{r0} % {d} = {r_calc}\n");

            assert_eq!(q, q_calc);
            assert_eq!(r, r_calc);
        }
    }

    #[test]
    fn extra_tests() {

        if false {
            let mut circuit = Circuit::new();
            let mut dividend = Vec::new();
            let mut divisor = Vec::new();
            let mut m3 = Vec::new();
            let mut shift_by = Vec::new();
            for _ in 0..1024 {
                dividend.push(circuit.new_line());
                divisor.push(Zero);
                m3.push(circuit.new_line());
            }

            divisor[0] = Zero;

            for _ in 0..10 {
                shift_by.push(circuit.new_line());
            }
            println!("{:?}, {:?}", dividend, divisor);
            // let sel = (circuit.new_line(), circuit.new_line());

            circuit.csa(dividend, divisor, One);
            println!("{:?} \n {:?}", circuit.wires.last(), circuit.stats);
        }

        if false {
            use num::rational::BigRational;
            let mut circuit = Circuit::new();
            for bits in 18..19 {
                let mut errors = Vec::new();
                let mut problem = 0;
                for num in 1..2_u32.pow(bits) {
                    let leading_zeroes = num.leading_zeros() - (32 - bits);
                    let num_shifted = num << leading_zeroes;
                    let max_number = 2_u32.pow(bits) - 1;
                    let q = (BigRational::new(
                        BigInt::from_u32(max_number).unwrap(),
                        BigInt::from_u32(1).unwrap(),
                    ) / BigRational::new(
                        BigInt::from_u32(num).unwrap(),
                        BigInt::from_u32(1).unwrap(),
                    ))
                    .floor();
                    // println!("{q}");
                    let mut bits_vec = Bit::get_bits_vec_u32(num_shifted);
                    bits_vec.truncate(bits as usize);
                    // println!("number: {num} :: {bits_vec:?}");
                    // println!("lz: {leading_zeroes}");
                    let estimate = circuit.flip_estimate(&bits_vec);
                    // println!("{estimate:?}");
                    let shift_right_by = bits as usize - leading_zeroes as usize;
                    // println!("{shift_right_by}");
                    let mut estimate = estimate[shift_right_by..(estimate.len() - 2)].to_vec();
                    estimate.append(&mut Bit::zeroes(shift_right_by));
                    let q_est = {
                        let mut q_est = BigRational::from_u32(0).unwrap();
                        for digit in 0..estimate.len() {
                            let value = 0x1 << (estimate.len() - digit - 1);
                            if estimate[digit] == One {
                                q_est += BigRational::new(
                                    BigInt::from_u32(1).unwrap(),
                                    BigInt::from_u32(value).unwrap(),
                                );
                            }
                        }
                        q_est
                    };
                    // println!("estimate as ratio: {q_est}");
                    let q_est = (q_est
                        * BigRational::new(
                            BigInt::from_u32(max_number).unwrap(),
                            BigInt::from_u32(1).unwrap(),
                        ))
                    .floor();
                    // println!("estimated q as ratio: {q_est}");
                    let error = (q - q_est).abs();
                    if error >= BigRational::from_f32(2f32).unwrap() {
                        errors.push((error.clone(), num));
                        problem += 1;
                    }
                    //println!("error: {error}");
                    //println!("after shifting{estimate:?} \n");
                }
                errors.sort_unstable_by(|a, b| a.0.cmp(&b.0));

                println!("to far off {problem}");
                println!("{errors:?}");
            }
        }

        if true {
            use num::rational::BigRational;
            let mut circuit = Circuit::new();
            for bits in 19..20 {
                let mut errors = Vec::new();
                let mut problem = 0;
                for num in 1..2_u32.pow(bits) {
                    let leading_zeroes = num.leading_zeros() - (32 - bits);
                    let num_shifted = num << leading_zeroes;
                    let max_number = 2_u32.pow(bits) - 1;
                    let q = (BigRational::new(
                        BigInt::from_u32(max_number).unwrap(),
                        BigInt::from_u32(1).unwrap(),
                    ) / BigRational::new(
                        BigInt::from_u32(num).unwrap(),
                        BigInt::from_u32(1).unwrap(),
                    ))
                    .floor();
                    // println!("{q}");
                    let mut bits_vec = Bit::get_bits_vec_u32(num_shifted);
                    bits_vec.truncate(bits as usize);
                    // println!("number: {num} :: {bits_vec:?}");
                    // println!("lz: {leading_zeroes}");
                    let estimate = circuit.flip_estimate(&bits_vec);
                    // println!("{num} : {estimate:?}");
                    let shift_right_by = bits as usize - leading_zeroes as usize;
                    // println!("{shift_right_by}");
                    let mut estimate = estimate[shift_right_by..(estimate.len() - 2)].to_vec();
                    estimate.append(&mut Bit::zeroes(shift_right_by));
                    let q_est = {
                        let mut q_est = BigRational::from_u32(0).unwrap();
                        for digit in 0..estimate.len() {
                            let value = 0x1 << (estimate.len() - digit - 1);
                            if estimate[digit] == One {
                                q_est += BigRational::new(
                                    BigInt::from_u32(1).unwrap(),
                                    BigInt::from_u32(value).unwrap(),
                                );
                            }
                        }
                        q_est
                    };
                    // println!("estimate as ratio: {q_est}");
                    let q_est = (q_est
                        * BigRational::new(
                            BigInt::from_u32(max_number).unwrap(),
                            BigInt::from_u32(1).unwrap(),
                        ))
                    .floor();
                    // println!("estimated q as ratio: {q_est}");
                    let error = (q - q_est).abs();
                    if error >= BigRational::from_f32(2f32).unwrap() {
                        errors.push((error.clone(), num));
                        problem += 1;
                    }
                    //println!("error: {error}");
                    //println!("after shifting{estimate:?} \n");
                }
                errors.sort_unstable_by(|a, b| a.0.cmp(&b.0));
                println!("to far off {problem}");
                println!("{errors:?}");
                println!("\n\n\n\n");
            }
        }
    }

    #[test]
    fn number_its() {
        for n in 0..129 {
            let temp1 = (0.max(n as i32 - 2) as f64 + 1_f64).ceil().log2().ceil() as usize;

            println!("{n} := {temp1}");
        }
    }

    #[test]
    fn test_depth_adders() {
        for exp in 0..10 {
            let mut circuit = Circuit::new();
            let bits = 2_usize.pow(exp);
            let mut s1 = vec![];
            let mut s2= vec![];
            let mut not_shift = vec![];
            for i in 0..bits {
                s1.push(circuit.new_line());
                s2.push(circuit.new_line());
                if i < (bits as f64).log2().ceil() as usize {
                    not_shift.push(circuit.new_line());
                }
            }

            let sum = circuit.shift(s1.clone(), not_shift, Shift::Left, Zero);
            circuit.add_as_io(&s1, "s1", false);
            circuit.add_as_io(&s2, "s2", false);
            circuit.add_as_io(&sum, "sum", true);
            circuit.remove_dead_ends();
            println!("{bits} : depth: {}", circuit.stats.level_count);
        }
    }
}
