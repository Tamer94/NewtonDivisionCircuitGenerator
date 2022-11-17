mod data;
use data::data::{Circuit, Gate, Line, Out, Stats, Wire, IO};
use std::fs::File;
use std::io::prelude::*;

fn main() -> std::io::Result<()> {
    let mut file = File::create("Carry_Ripple.v")?;
  file.write_all(        create_carry_ripple(512, &mut Stats::new())
            .0
            .verilog_str("carry_ripple_4bit").as_bytes())?;
  /*
    println!(
        "{}",
        create_carry_ripple(4, &mut Stats::new())
            .0
            .verilog_str("carry_ripple_4bit")
    );
    println!("{:?}", create_carry_ripple(4, &mut Stats::new()));
    */
  Ok(())
}

fn create_carry_ripple(bits: u32, stats: &mut Stats) -> (Circuit, Stats) {
    let mut circuit = Circuit::new();
    if bits == 0 {
        return (circuit, *stats);
    }

    let mut x_v = {
        let mut lines = Vec::with_capacity(bits as usize);
        for x in 0..(bits as usize) {
            lines.push(Line { level: 0, n: x });
        }
        stats.line_count += bits as usize;
        lines
    };
    let mut y_v = {
        let mut lines = Vec::with_capacity(bits as usize);
        for x in 0..(bits as usize) {
            lines.push(Line {
                level: 0,
                n: x + stats.line_count,
            });
        }
        stats.line_count += bits as usize;
        lines
    };
    let mut c_v = {
        let mut lines = Vec::with_capacity(1);

        lines.push(Line {
            level: 0,
            n: stats.line_count,
        });
        lines
    };
    let x = IO::new("X", x_v);
    let y = IO::new("Y", y_v);
    let c_in = IO::new("C_in", c_v);
    stats.line_count += 1;

    let mut carry_line = c_in.lines[0];
    let mut first_xor_out;
    let mut first_and_out;
    let mut second_and_out;
    let mut sum: Vec<(Out, usize)> = Vec::new();

    for i in 0..(bits as usize - 1) {
        let mut gate = Gate::Xor(x.lines[i], y.lines[i]);
        circuit.wires.push(Wire::new(
            Line {
                level: gate.get_next_level(),
                n: stats.add_line(),
            },
            gate,
        ));
        first_xor_out = circuit.wires[circuit.wires.len() - 1].out;

        gate = Gate::And(x.lines[i], y.lines[i]);
        circuit.wires.push(Wire::new(
            Line {
                level: gate.get_next_level(),
                n: stats.add_line(),
            },
            gate,
        ));
        first_and_out = circuit.wires[circuit.wires.len() - 1].out;

        gate = Gate::Xor(first_xor_out, carry_line);
        circuit.wires.push(Wire::new(
            Line {
                level: gate.get_next_level(),
                n: stats.add_line(),
            },
            gate,
        ));
        sum.push((Out::Wire, circuit.wires.len() - 1));

        gate = Gate::And(first_xor_out, carry_line);
        circuit.wires.push(Wire::new(
            Line {
                level: gate.get_next_level(),
                n: stats.add_line(),
            },
            gate,
        ));
        second_and_out = circuit.wires[circuit.wires.len() - 1].out;

        gate = Gate::Or(first_and_out, second_and_out);
        circuit.wires.push(Wire::new(
            Line {
                level: gate.get_next_level(),
                n: stats.add_line(),
            },
            gate,
        ));
        carry_line = circuit.wires[circuit.wires.len() - 1].out;
    }

    let mut gate = Gate::Xor(x.lines[x.lines.len() - 1], y.lines[x.lines.len() - 1]);
    circuit.wires.push(Wire::new(
        Line {
            level: gate.get_next_level(),
            n: stats.add_line(),
        },
        gate,
    ));
    first_xor_out = circuit.wires[circuit.wires.len() - 1].out;

    gate = Gate::And(x.lines[x.lines.len() - 1], y.lines[x.lines.len() - 1]);
    circuit.wires.push(Wire::new(
        Line {
            level: gate.get_next_level(),
            n: stats.add_line(),
        },
        gate,
    ));
    first_and_out = circuit.wires[circuit.wires.len() - 1].out;

    gate = Gate::Xor(first_xor_out, carry_line);
    circuit.wires.push(Wire::new(
        Line {
            level: gate.get_next_level(),
            n: stats.add_line(),
        },
        gate,
    ));
    sum.push((Out::Wire, circuit.wires.len() - 1));

    gate = Gate::And(first_xor_out, carry_line);
    circuit.wires.push(Wire::new(
        Line {
            level: gate.get_next_level(),
            n: stats.add_line(),
        },
        gate,
    ));
    second_and_out = circuit.wires[circuit.wires.len() - 1].out;

    gate = Gate::Or(first_and_out, second_and_out);
    circuit.wires.push(Wire::new(
        Line {
            level: gate.get_next_level(),
            n: stats.add_line(),
        },
        gate,
    ));
    sum.push((Out::Wire, circuit.wires.len() - 1));

    circuit.inputs.push(x);
    circuit.inputs.push(y);
    circuit.inputs.push(c_in);
    circuit.outputs = sum;
    (circuit, *stats)
}
