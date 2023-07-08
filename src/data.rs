use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::File;
use std::io::prelude::*;
use std::ops::Range;
use clap::ValueEnum;

const ONE: usize = usize::MAX - 1;
pub const DEFAULT_LEVEL: usize = (1 << 31) - 1;

#[macro_export]
macro_rules! stringify_enum {
    (
        $(#[$meta:meta])*
        $pub:vis enum $name:ident {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident
            ),*,
        }
    ) => {
        $(#[$meta])*
        $pub enum $name {
            $(
                $(#[$variant_meta])*
                $variant
            ),*
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                match *self {
                    $(
                        $name::$variant => write!(f, stringify!($variant)),
                    )*
                }
            }
        }
    };
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Ignore {
    pub lower_bits: Range<usize>,
    pub higher_bits: Range<usize>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, ValueEnum)]
pub enum Adder {
    CRA,
    CSA,
    KSA,
}

impl Adder {
    pub fn add(&self, circuit: &mut Circuit, s1: Vec<Bit>, s2: Vec<Bit>, c_in: Bit) -> Vec<Bit> {
        match *self {
            Self::CRA => Circuit::cra(circuit, s1, s2, c_in),
            Self::CSA => Circuit::csa(circuit, s1, s2, c_in),
            Self::KSA => Circuit::ksa(circuit, s1, s2, c_in),
        }
    }

    pub fn sub(&self, circuit: &mut Circuit, minuend: Vec<Bit>, subtrahend: Vec<Bit>, c_in: Bit) -> Vec<Bit> {
        match *self {
            Self::CRA => Circuit::crs(circuit, minuend, subtrahend, c_in),
            Self::CSA => Circuit::css(circuit, minuend, subtrahend, c_in),
            Self::KSA => Circuit::kss(circuit, minuend, subtrahend, c_in),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, ValueEnum)]
pub enum Mul {
    Array,
    DadaTree,
}

impl Mul {
    pub fn mul_u(&self, circuit: &mut Circuit, f1: Vec<Bit>, f2: Vec<Bit>, s: Option<Vec<Bit>>, adder: Adder) -> Vec<Bit> {
        match *self {
            Self::DadaTree => Circuit::umul_dadda(circuit, f1, f2, s, adder),
            Self::Array => Circuit::array_mul(circuit, f1, f2, s, adder),
        }
    }

    pub fn square_u(&self, circuit: &mut Circuit, f1: Vec<Bit>, ignore: usize, adder: Adder) -> Vec<Bit> {
        match *self {
            Self::DadaTree => Circuit::usquare_dadda(circuit, f1, ignore, adder),
            Self::Array => Circuit::array_mul(circuit, f1.clone(), f1, None, adder),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Shift {
    Left,
    Right,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Bit {
    Var(Line),
    One,
    Zero,
}

impl Bit {
    #[inline]
    pub fn zeroes(size: usize) -> Vec<Bit> {
        let mut zeroes = Vec::with_capacity(size);
        for _ in 0..size {
            zeroes.push(Bit::Zero);
        }
        zeroes
    }

    #[inline]
    #[allow(dead_code)]
    pub fn ones(size: usize) -> Vec<Bit> {
        let mut ones = Vec::with_capacity(size);
        for _ in 0..size {
            ones.push(Bit::One);
        }
        ones
    }

    #[cfg(test)]
    pub fn get_number_u(bits: &Vec<Bit>) -> Result<usize, CastingError> {
        let n = bits.len();
        if n > 65 {
            Err(CastingError::ToMany(
                "the number represented might be to big to fit into a usize",
            ))
        } else {
            let mut number = 0;
            let mut current_bit_high = 0x1;
            for bit in bits {
                if let &Bit::Var(_) = bit {
                    return Err(CastingError::VariableContained(
                        "The vector of bits contained a bit which was variable",
                    ));
                }
                if *bit == Bit::One {
                    number |= current_bit_high;
                }
                current_bit_high = current_bit_high << 1;
            }
            Ok(number)
        }
    }

    #[cfg(test)]
    pub fn get_number_u128(bits: &Vec<Bit>) -> Result<u128, CastingError> {
        let n = bits.len();
        if n > 128 {
            Err(CastingError::ToMany(
                "the number represented might be to big to fit into a u128",
            ))
        } else {
            let mut number = 0;
            let mut current_bit_high = 0x1;
            for bit in bits {
                if let &Bit::Var(_) = bit {
                    return Err(CastingError::VariableContained(
                        "The vector of bits contained a bit which was variable",
                    ));
                }
                if *bit == Bit::One {
                    number |= current_bit_high;
                }
                current_bit_high = current_bit_high << 1;
            }
            Ok(number)
        }
    }

    #[cfg(test)]
    pub fn get_bits_vec_u8(mut n: u8) -> Vec<Bit> {
        let mut bits = Vec::with_capacity(u8::BITS as usize);
        for _ in 0..(u8::BITS as usize) {
            if (n & 0x1) != 0 {
                bits.push(Bit::One);
            } else {
                bits.push(Bit::Zero);
            }
            n = n >> 1;
        }
        bits
    }

    #[cfg(test)]
    pub fn get_bits_vec_u16(mut n: u16) -> Vec<Bit> {
        let mut bits = Vec::with_capacity(u16::BITS as usize);
        for _ in 0..(u16::BITS as usize) {
            if (n & 0x1) != 0 {
                bits.push(Bit::One);
            } else {
                bits.push(Bit::Zero);
            }
            n = n >> 1;
        }
        bits
    }

    #[cfg(test)]
    pub fn get_bits_vec_u32(mut n: u32) -> Vec<Bit> {
        let mut bits = Vec::with_capacity(u32::BITS as usize);
        for _ in 0..(u32::BITS as usize) {
            if (n & 0x1) != 0 {
                bits.push(Bit::One);
            } else {
                bits.push(Bit::Zero);
            }
            n = n >> 1;
        }
        bits
    }

    #[cfg(test)]
    pub fn get_bits_vec_u64(mut n: u64) -> Vec<Bit> {
        let mut bits = Vec::with_capacity(u64::BITS as usize);
        for _ in 0..(u64::BITS as usize) {
            if (n & 0x1) != 0 {
                bits.push(Bit::One);
            } else {
                bits.push(Bit::Zero);
            }
            n = n >> 1;
        }
        bits
    }

    pub fn get_bits_vec_usize(mut n: usize) -> Vec<Bit> {
        let mut bits = Vec::with_capacity(usize::BITS as usize);
        for _ in 0..(usize::BITS as usize) {
            if (n & 0x1) != 0 {
                bits.push(Bit::One);
            } else {
                bits.push(Bit::Zero);
            }
            n = n >> 1;
        }
        bits
    }

    #[cfg(test)]
    pub fn get_bits_vec_u128(mut n: u128) -> Vec<Bit> {
        let mut bits = Vec::with_capacity(u128::BITS as usize);
        for _ in 0..(u128::BITS as usize) {
            if (n & 0x1) != 0 {
                bits.push(Bit::One);
            } else {
                bits.push(Bit::Zero);
            }
            n = n >> 1;
        }
        bits
    }
}

#[cfg(test)]
#[derive(Debug)]
pub enum CastingError {
    ToMany(&'static str),
    VariableContained(&'static str),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Line {
    pub level: usize,
    pub n: usize,
}

impl Line {
    #[allow(dead_code)]
    pub fn new(level: usize, n: usize) -> Self {
        Line { level, n }
    }

    #[inline]
    pub fn to_verilog(&self, io_lines: &HashMap<usize, NamedLine>) -> String {
        match self.n {
            // usize::MAX means zeroWire
            usize::MAX => {
                String::from("zeroWire")
            }
            // usize::MAX - 1 means oneWire
            ONE => {
                String::from("oneWire")
            }
            _ => {
                io_lines.get(&self.n).map_or(format!("_{}_", self.n), |l| {
                    format!("{}[{}]", l.name, l.idx)
                })
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct IO {
    pub name: String,
    pub bits: Vec<Bit>,
}

impl IO {
    pub fn new(name: &str, bits: Vec<Bit>) -> Self {
        IO {
            name: String::from(name),
            bits,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Gate {
    Not(Line),
    And(Line, Line),
    Or(Line, Line),
    Xor(Line, Line),
}

use Gate::{And, Not, Or, Xor};
impl Gate {
    pub fn get_next_level(&self) -> usize {
        match *self {
            Not(line) => line.level + 1,
            And(l1, l2) => l1.level.max(l2.level) + 1,
            Or(l1, l2) => l1.level.max(l2.level) + 1,
            Xor(l1, l2) => l1.level.max(l2.level) + 1,
        }
    }

    pub fn to_verilog(&self, io_lines: &HashMap<usize, NamedLine>) -> String {
        let op;
        match self {
            &Not(l) => {
                op = format!("~{};", l.to_verilog(io_lines));
            }
            And(l1, l2) => {
                op = format!("{} & {};", l1.to_verilog(io_lines), l2.to_verilog(io_lines));
            }
            Or(l1, l2) => {
                op = format!("{} | {};", l1.to_verilog(io_lines), l2.to_verilog(io_lines));
            }
            Xor(l1, l2) => {
                op = format!("{} ^ {};", l1.to_verilog(io_lines), l2.to_verilog(io_lines));
            }
        }
        op
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Wire {
    pub gate: Gate,
    pub out: Line,
}

impl Wire {
    pub fn new(out: Line, gate: Gate) -> Self {
        Wire { gate, out }
    }

    pub fn to_verilog(&self, io_lines: &HashMap<usize, NamedLine>) -> String {
        format!(
            "assign {} = {}\n",
            self.out.to_verilog(io_lines),
            self.gate.to_verilog(io_lines)
        )
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Info {
    pub list_mul_numbers: Vec<usize>,
    pub mul_numbers_map: HashMap<usize, usize>,
}

impl Info {
    pub fn new() -> Self {
        let mut l = Vec::with_capacity(0x100);
        let mut m = HashMap::with_capacity(0x400);
        let mut n = 2;
        for _ in 0..0x40 {
            l.push(n);
            n = (n * 3) / 2;
        }
        let mut idx = 0;
        for number in 0..0x100 {
            m.insert(number, idx);
            if number >= l[idx + 1] {
                idx += 1;
            }
        }
        Info {
            list_mul_numbers: l,
            mul_numbers_map: m,
        }
    }

    pub fn update_for_value(&mut self, v: usize) {
        for (idx, x) in self.list_mul_numbers.iter().enumerate() {
            if *x >= v {
                self.mul_numbers_map.insert(v, idx - 1);
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct NamedLine {
    pub idx: usize,
    pub name: &'static str,
    pub is_output: bool,
}

impl Default for NamedLine {
    fn default() -> Self {
        NamedLine {
            idx: usize::MAX,
            name: "",
            is_output: true,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Circuit {
    pub io_lines: HashMap<usize, NamedLine>,
    pub inputs: Vec<IO>,
    pub outputs: Vec<IO>,
    pub wires: Vec<Wire>,
    pub stats: Stats,
    pub info: Info,
    pub zero_wire: Option<Line>,
    pub one_wire: Option<Line>,
}

impl Circuit {
    pub fn new() -> Self {
        Circuit {
            io_lines: { HashMap::new() },
            inputs: { Vec::new() },
            outputs: { Vec::new() },
            wires: { Vec::new() },
            stats: { Stats::new() },
            info: { Info::new() },
            zero_wire: None,
            one_wire: None,
        }
    }

    #[allow(dead_code)]
    pub fn out(&self, idx: usize) -> Line {
        self.wires[idx].out
    }

    #[inline(always)]
    pub fn new_line(&mut self) -> Bit {
        let line = Line {
            level: 0,
            n: self.stats.add_line(),
        };
        Bit::Var(line)
    }

    #[inline(always)]
    pub fn new_line_constant_value(&mut self, n: usize) -> Line {
        let line = Line {
            level: 0,
            n,
        };
        line
    }

    pub fn update_stats(&mut self) {
        self.stats.gatter_count = self.wires.len();
        let mut max_depth = 0;
        for out in &self.outputs {
            for bit in &out.bits {
                match bit {
                    &Bit::Var(l) => {
                        max_depth = max_depth.max(l.level);
                    }
                    _ => (),
                }
            }
        }
        self.stats.level_count = max_depth;
    }

    pub fn add_as_io(&mut self, bits: &Vec<Bit>, name: &'static str, is_output: bool) {
        for (idx, bit) in bits.iter().enumerate() {
            if let Bit::Var(l) = bit {
                if !self.io_lines.contains_key(&l.n) {
                    self.io_lines.insert(
                        l.n,
                        NamedLine {
                            idx,
                            name,
                            is_output,
                        },
                    );
                }
            }
        }
        if is_output {
            self.outputs.push(IO::new(name, bits.clone()));
        } else {
            self.inputs.push(IO::new(name, bits.clone()));
        }
    }

    pub fn set_zero_wire(&mut self) -> Line {
        let mut line = self.new_line_constant_value(usize::MAX);
        self.zero_wire = Some(line);
        line
    }

    pub fn set_one_wire(&mut self) -> Line {
        let mut line = self.new_line_constant_value(usize::MAX - 1);
        self.one_wire = Some(line);
        line
    }

    pub fn get_zero_wire(&mut self) -> Line {
        self.zero_wire.unwrap_or(self.set_zero_wire())
    }

    pub fn get_one_wire(&mut self) -> Line {
        self.one_wire.unwrap_or(self.set_one_wire())
    }

    pub fn verilog_header(&self, name: &str) -> String {
        let mut s = String::new();
        s.push_str(&format!("module {name}({});\n", self.get_io_names()));
        s
    }

    pub fn get_io_names(&self) -> String {
        let mut s = String::new();
        for io in &self.inputs {
            s.push_str(&format!("{}, ", io.name));
        }

        for io in &self.outputs {
            s.push_str(&format!("{}, ", io.name));
        }
        
        // delete last white space and comma
        s.pop();
        s.pop();
        s
    }

    pub fn all_labels(&mut self) -> String {
        let mut s = String::new();
        let mut zero = false;
        let mut one = false;
        for o in &self.outputs {
            if o.bits.iter().any(|b| b == &Bit::Zero) {
                zero = true;
                break;
            }
        }

        for o in &self.outputs {
            if o.bits.iter().any(|b| b == &Bit::One) {
                one = true;
                break;
            }
        }
        if let Some(l) = self.one_wire {
            one = true;
        }
        if let Some(l) = self.zero_wire {
            zero = true;
        }

        for input in &self.inputs {
            if input.bits.len() > 0 {
                s.push_str(&format!(
                    "input [{}:0] {};\n",
                    (input.bits.len() - 1),
                    input.name
                ));
            } else {
                s.push_str(&format!("input {};\n", input.name));
            }
        }

        for output in &self.outputs {
            if output.bits.len() > 0 {
                s.push_str(&format!(
                    "output [{}:0] {};\n",
                    (output.bits.len() - 1),
                    output.name
                ));
            } else {
                s.push_str(&format!("output {};\n", output.name));
            }
        }

        if zero {
            s.push_str(&format!{"wire zeroWire;\n"});
        }

        if one {
            s.push_str(&format!{"wire oneWire;\n"});
        }

        for wire in &self.wires {
            if !self.io_lines.contains_key(&wire.out.n) {
                s.push_str(&format!("wire _{}_;\n", wire.out.n));
            }
        }

        s
    }

    pub fn all_assigns_statements(&mut self) -> String {
        let mut s = String::new();
        if let Some(_) = self.zero_wire {
            s.push_str(&format!("assign zeroWire = 1'b0 /*0*/;\n"))
        }
        if let Some(_) = self.one_wire {
            s.push_str(&format!("assign oneWire = 1'b1 /*0*/;\n"))
        }

        for wire in &self.wires {
            s.push_str(&wire.to_verilog(&self.io_lines));
        }

        for out in &mut self.outputs.clone() {
            for (idx, bit) in out.bits.iter().enumerate() {
                match bit {
                    &Bit::Zero => {
                        s.push_str(&format!(
                            "assign {}[{}] = zeroWire;\n",
                            out.name,
                            idx,
                        ))
                    }
                    &Bit::One => {
                        s.push_str(&format!(
                            "assign {}[{}] = oneWire;\n",
                            out.name,
                            idx,
                        ))
                    }
                    &Bit::Var(l) => {
                        if !self
                            .io_lines
                            .get(&l.n)
                            .unwrap_or(&NamedLine::default())
                            .is_output
                        {
                            s.push_str(&format!(
                                "assign {}[{}] = {};\n",
                                out.name,
                                idx,
                                l.to_verilog(&self.io_lines)
                            ))
                        }
                    }
                }
            }
        }
        s
    }

    pub fn to_verilog(&mut self, name: &str) -> String {
        let mut s = String::new();
        s.push_str(&self.verilog_header(name));
        s.push_str(&self.all_labels());
        s.push_str(&&self.all_assigns_statements());
        s.push_str("endmodule");
        s
    }

    pub fn write_to_file(&mut self, file_name: &str, module_name: &str) -> std::io::Result<()> {
        let mut file = File::create(file_name)?;
        write!(file, "{}", self.to_verilog(module_name))?;
        Ok(())
    }

    pub fn remove_dead_ends(&mut self) {
        let input_lines_count = self.stats.line_count - self.stats.gatter_count;

        let mut wire_indexes = HashSet::new();
        let mut lines_to_traverse = VecDeque::new();

        for out in &self.outputs {
            for bit in &out.bits {
                match bit {
                    &Bit::Var(l) => {
                        lines_to_traverse.push_back(l.n);
                    }
                    _ => (),
                }
            }
        }

        while !lines_to_traverse.is_empty() {
            let w_idx = lines_to_traverse.pop_front().unwrap();
            if !wire_indexes.contains(&w_idx) {
                if w_idx < input_lines_count || w_idx == ONE || w_idx == usize::MAX {
                    continue;
                }
                let w = self.wires[w_idx - input_lines_count];

                match w.gate {
                    Not(l) => {
                        lines_to_traverse.push_front(l.n);
                    }
                    And(l1, l2) => {
                        lines_to_traverse.push_front(l1.n);
                        lines_to_traverse.push_front(l2.n);
                    }
                    Or(l1, l2) => {
                        lines_to_traverse.push_front(l1.n);
                        lines_to_traverse.push_front(l2.n);
                    }
                    Xor(l1, l2) => {
                        lines_to_traverse.push_front(l1.n);
                        lines_to_traverse.push_front(l2.n);
                    }
                }
            }

            wire_indexes.insert(w_idx);
        }

        let mut reduced_wires = vec![];
        for w in &self.wires {
            if wire_indexes.contains(&w.out.n) {
                reduced_wires.push(*w);
            }
        }
        self.wires = reduced_wires;
        self.update_stats();
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Stats {
    pub line_count: usize,
    pub gatter_count: usize,
    pub level_count: usize,
}

impl Stats {
    pub fn new() -> Self {
        Stats {
            line_count: 0,
            gatter_count: 0,
            level_count: 0,
        }
    }

    pub fn add_line(&mut self) -> usize {
        self.line_count += 1;
        self.line_count - 1
    }
}

pub struct LevelizedCircuit {
    pub circuit: Circuit,
    pub substitution_levels: HashMap<usize, usize>,
    pub current_level: usize,
}

impl LevelizedCircuit {
    pub fn new(current_level_start: usize) -> Self {
        LevelizedCircuit {
            circuit: Circuit::new(),
            substitution_levels: HashMap::new(),
            current_level: current_level_start,
        }
    }

    pub fn write_to_file(&mut self, file_name: &str, module_name: &str) -> std::io::Result<()> {
        let mut file = File::create(file_name)?;
        write!(file, "{}", self.to_verilog(module_name))?;
        Ok(())
    }

    pub fn to_verilog(&mut self, name: &str) -> String {
        let mut s = String::new();
        s.push_str(&self.circuit.verilog_header(name));
        s.push_str(&self.circuit.all_labels());
        s.push_str(&self.all_assigns_statements());
        s.push_str("endmodule");
        s
    }

    pub fn all_assigns_statements(&mut self) -> String {
        let mut s = String::new();
        if let Some(_) = self.circuit.zero_wire {
            s.push_str(&format!("assign zeroWire = 1'b0 /*0*/;\n"))
        }
        if let Some(_) = self.circuit.one_wire {
            s.push_str(&format!("assign oneWire = 1'b1 /*0*/;\n"))
        }

        for wire in &self.circuit.wires {
            let mut assign_statement = wire.to_verilog(&self.circuit.io_lines);
            assign_statement.pop();
            assign_statement.pop();
            let new_assign_statement = format!("{} /*{}*/;\n", assign_statement, self.substitution_levels.get(&wire.out.n).unwrap_or(&(DEFAULT_LEVEL)));
            s.push_str(&new_assign_statement);
        }

        for out in &mut self.circuit.outputs.clone() {
            for (idx, bit) in out.bits.iter().enumerate() {
                match bit {
                    &Bit::Zero => {
                        s.push_str(&format!(
                            "assign {}[{}] = zeroWire;\n",
                            out.name,
                            idx,
                        ))
                    }
                    &Bit::One => {
                        s.push_str(&format!(
                            "assign {}[{}] = oneWire;\n",
                            out.name,
                            idx,
                        ))
                    }
                    &Bit::Var(l) => {
                        if !self
                            .circuit
                            .io_lines
                            .get(&l.n)
                            .unwrap_or(&NamedLine::default())
                            .is_output
                        {
                            s.push_str(&format!(
                                "assign {}[{}] = {} /*{}*/;\n",
                                out.name,
                                idx,
                                l.to_verilog(&self.circuit.io_lines),
                                self.substitution_levels.get(&l.n).unwrap_or(&(DEFAULT_LEVEL))
                            ))
                        }
                    }
                }
            }
        }
        s
    }
}
