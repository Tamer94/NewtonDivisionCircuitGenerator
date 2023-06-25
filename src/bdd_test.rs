
#[cfg(test)]
use crate::{data::Circuit};
#[test]
fn print_gates_in_order() {
    use std::time::Instant;
    let bits = 8;
    let circuit = Circuit::get_umul(bits, crate::cli::CircuitKind::MulDadda, crate::dividers::SubMethod::Seperate, crate::data::Adder::CRA);
    let now = Instant::now();
    let bdds = circuit.get_bdds(bits);
    println!("computing bdds took {}µs", now.elapsed().as_micros());
    for b in bdds {
        println!("{} {}", b.name, b.bdd);
    }
}

#[test]
fn bdd_of_cra() {
    use std::time::Instant;
    let bits = 64;
    let circuit = Circuit::get_mux_n_2(bits);
    let now = Instant::now();
    let bdds = circuit.get_bdds(bits);
    println!("computing bdds took {}µs", now.elapsed().as_micros());
    for b in bdds {
        println!("{} {}", b.name, b.bdd);
    }
}

#[test]
fn spec_lzc_bdd() {
    let bits = 4;
    let circuit = Circuit::get_lzc_circuit(bits, false);
    let bdd = circuit.get_lzc_bdd(2, false, false);
    println!("{}: {}", bdd.name, bdd.bdd);
}

#[test]
fn verify_lzc_by_bdd() {
    let bits = 64;
    let circuit = Circuit::get_lzc_circuit(bits, false);
    let circuit_bdds = circuit.get_bdds(bits);


    for i in 0..(((bits as f64).log2().floor() as usize) + 1) {
        let circuit_bdd = &circuit_bdds[i];
        println!("{}: {}", circuit_bdd.name, circuit_bdd.bdd);
        let spec_bdd = circuit.get_lzc_bdd(i, true, true);
        println!("{}: {}", spec_bdd.name, spec_bdd.bdd);
        assert_eq!(circuit_bdds[i].bdd, spec_bdd.bdd);
    }
}

#[test]
fn verify_csa() {
    use std::time::Instant;
    let bits = 128;
    let circuit1 = Circuit::get_cra(bits);
    let circuit2 = Circuit::get_csa(bits);
    let now = Instant::now();
    let bdds1 = circuit1.get_bdds(bits);
    let bdds2 = circuit2.get_bdds(bits);
    // println!("computing bdds took {}µs", now.elapsed().as_micros());
    // for b in bdds {
    //     println!("{} {}", b.name, b.bdd);
    // }
    for (i, bdd1) in bdds1.iter().enumerate() {
        let bdd2 = &bdds2[i];
        println!("cra {} {}", bdd1.name, bdd1.bdd);
        println!("csa {} {}", bdd2.name, bdd2.bdd);

        assert_eq!(bdd1.bdd, bdd2.bdd);
    }
}

#[test]
fn verify_ksa() {
    let bits = 1024;
    let circuit1 = Circuit::get_cra(bits);
    let circuit2 = Circuit::get_ksa(bits);
    let bdds1 = circuit1.get_bdds(bits);
    let bdds2 = circuit2.get_bdds(bits);
    // println!("computing bdds took {}µs", now.elapsed().as_micros());
    // for b in bdds {
    //     println!("{} {}", b.name, b.bdd);
    // }
    for (i, bdd1) in bdds1.iter().enumerate() {
        let bdd2 = &bdds2[i];
        // println!("cra {} {}", bdd1.name, bdd1.bdd);
        // println!("ksa {} {}", bdd2.name, bdd2.bdd);

        assert_eq!(bdd1.bdd, bdd2.bdd);
    }
}

#[test]
fn find_difference_of_cra_ksa() {
    let bits = 2;
    let circuit1 = Circuit::get_cra(bits);
    let circuit2 = Circuit::get_ksa(bits);
    let bdds1 = circuit1.get_bdds(bits);
    let bdds2 = circuit2.get_bdds(bits);
    for (i, bdd1) in bdds1.iter().enumerate() {
        let bdd2 = &bdds2[i];
        println!("cra {} {}", bdd1.name, bdd1.bdd);
        println!("ksa {} {}", bdd2.name, bdd2.bdd);

        let xor_bdd = bdd1.bdd.xor(&bdd2.bdd);
        let all_xor_clauses = xor_bdd.sat_clauses();
        for clause in all_xor_clauses {
            print!("{:?}", clause.to_values());
        }
        //println!("difference {}", xor_bdd);

        //assert_eq!(bdd1.bdd, bdd2.bdd);
    }

}