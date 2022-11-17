pub mod data {
    #[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
    pub struct Line {
        pub level: usize,
        pub n: usize,
    }

    impl Line {
        pub fn new(level: usize, n: usize) -> Self {
            Line { level, n }
        }
    }

    #[derive(Debug, PartialEq, Eq)]
    pub struct IO {
        pub name: String,
        pub lines: Vec<usize>,
    }

    impl IO {
        pub fn new(name: &str, lines: Vec<usize>) -> Self {
            IO {
                name: String::from(name),
                lines,
            }
        }
    }

    #[derive(Debug, PartialEq, Eq)]
    pub struct Output {
        pub name: String,
        pub lines: Vec<usize>,
    }

    impl Output {
        pub fn new(name: &str, lines: Vec<usize>) -> Self {
            Output {
                name: String::from(name),
                lines,
            }
        }
    }

    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub enum Gate {
        Not(usize),
        And(usize, usize),
        Or(usize, usize),
        Xor(usize, usize),
    }

    use Gate::{And, Not, Or, Xor};
    impl Gate {
    }

    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub struct Wire {
        pub gate: Gate,
        pub out: usize,
    }

    impl Wire {
        pub fn new(out: usize, gate: Gate) -> Self {
            Wire { gate, out }
        }
    }

    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub enum Out {
        Input,
        Wire,
    }

    #[derive(Debug, PartialEq, Eq)]
    pub struct Circuit {
        pub inputs: Vec<IO>,
        pub outputs: Vec<Output>,
        pub lines: Vec<Line>,
        pub wires: Vec<Wire>,
        pub stats: Stats,
    }

    impl Circuit {
        pub fn new() -> Self {
            Circuit {
                inputs: { Vec::new() },
                outputs: { Vec::new() },
                lines: { Vec::new() },
                wires: { Vec::new() },
                stats: { Stats::new() },
            }
        }

              pub fn get_next_level(&self, i1: usize, i2: usize) -> usize {
                self.lines[i1].level.max(self.lines[i2].level) + 1

        }

        pub fn out(&self, idx: usize) -> usize {
            self.lines[idx].n
        }

       pub fn add_line(&mut self, level: usize) -> usize {
         let line_count = self.stats.line_count;
            self.stats.line_count += 1;
            self.lines.push(Line { level, n: line_count });
            
            line_count
        }

        pub fn all_labels(&self) -> String {
            let mut s = String::new();
            for input in &self.inputs {
                if input.lines.len() > 1 {
                    s.push_str(&format!(
                        "input [{}:0] {};\n",
                        (input.lines.len() - 1),
                        input.name
                    ));
                } else {
                    s.push_str(&format!("input {};\n", input.name));
                }
            }
            for output in &self.outputs {
                if output.lines.len() > 1 {
                    s.push_str(&format!(
                        "output [{}:0] {};\n",
                        (output.lines.len() - 1),
                        output.name
                    ));
                } else {
                    s.push_str(&format!("output {};\n", output.name));
                }
            }

            for wire in &self.wires {
                s.push_str(&format!("wire _{}_;\n", self.lines[wire.out].n));
            }
            s
        }

        pub fn verilog_str(&self, module_name: &str) -> String {
            use std::collections::HashMap;
            let mut s = String::new();
            let mut input_lines = HashMap::new();
            s.push_str(&format!("module {}(", module_name));
            for input in &self.inputs {
                s.push_str(&format!("{}, ", input.name));
                for (i, line) in input.lines.iter().enumerate() {
                    if input.lines.len() > 1 {
                        input_lines.insert(line, format!("{}[{}]", input.name, i));
                    } else {
                        input_lines.insert(line, format!("{}", input.name));
                    }
                }
            }
            s.push_str(&format!("S"));
            s.push_str(");\n");
            s.push_str(&self.all_labels());

            let mut index = 0;
            for (i, wire) in self.wires.iter().enumerate() {
                let output_to_test = self.lines[self.outputs[0].lines[index]].n;
                if i == output_to_test {
                    s.push_str(&format!("assign S[{}] = ", index));
                    index += 1;
                } else {
                    s.push_str(&format!("assign _{}_ = ", self.lines[wire.out].n));
                }
                match wire.gate {
                    And(i1, i2) => {
                        if input_lines.contains_key(&i1) {
                            s.push_str(&format!("{} & ", input_lines[&i1]));
                        } else {
                            s.push_str(&format!("_{}_ & ", i1.n));
                        }
                        if input_lines.contains_key(&i2) {
                            s.push_str(&format!("{};\n", input_lines[&i2]));
                        } else {
                            s.push_str(&format!("_{}_;\n", i2.n));
                        }
                    }

                    Or(i1, i2) => {
                        if input_lines.contains_key(&i1) {
                            s.push_str(&format!("{} | ", input_lines[&i1]));
                        } else {
                            s.push_str(&format!("_{}_ | ", i1.n));
                        }
                        if input_lines.contains_key(&i2) {
                            s.push_str(&format!("{};\n", input_lines[&i2]));
                        } else {
                            s.push_str(&format!("_{}_;\n", i2.n));
                        }
                    }

                    Xor(i1, i2) => {
                        if input_lines.contains_key(&i1) {
                            s.push_str(&format!("{} ^ ", input_lines[&i1]));
                        } else {
                            s.push_str(&format!("_{}_ ^ ", i1.n));
                        }
                        if input_lines.contains_key(&i2) {
                            s.push_str(&format!("{};\n", input_lines[&i2]));
                        } else {
                            s.push_str(&format!("_{}_;\n", i2.n));
                        }
                    }

                    Not(i1) => {
                        if input_lines.contains_key(&i1) {
                            s.push_str(&format!("~{}", input_lines[&i1]));
                        } else {
                            s.push_str(&format!("~_{}_;\n", i1.n));
                        }
                    }
                }
            }

            s.push_str("endmodule");
            s
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
    }
}
