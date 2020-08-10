use lay::Operations;
use lay::gates::{CliffordGate};
use lay_simulator_gk::{ GottesmanKnillSimulator, DefaultRng, Qubit };

const PHISQUBIT_PER_LOGQUBIT: Qubit = 7;
const MEASURE_ANCILLA_QUBITS: Qubit = 6;
const MEASURE_MASK: u32 = 127;

pub struct SteaneLayer {
    sim: GottesmanKnillSimulator<DefaultRng>,
    n_logical_qubits: Qubit
}

const ERR_TABLE_X: [u32;8] = [999 /* dummy */, 0, 1, 6, 2, 4, 3, 5];
const ERR_TABLE_Z: [u32;8] = [999 /* dummy */, 3, 4, 6, 5, 0, 1, 2];

impl SteaneLayer {
    pub fn new(n_qubits: Qubit) -> Self {
        Self {
            sim: GottesmanKnillSimulator::from_seed(PHISQUBIT_PER_LOGQUBIT * n_qubits + MEASURE_ANCILLA_QUBITS, 0),
            n_logical_qubits: n_qubits }
    }

    fn measure_ancilla(&self) -> Qubit {
        self.sim.n_qubits() as Qubit - 6
    }

    fn syndrome_measure_and_recover(&mut self) {
        let m0 = self.measure_ancilla();
        for i in 0..m0 {
            self.sim.h(i);
        }
        for i in 0..self.n_logical_qubits {
            for j in i * PHISQUBIT_PER_LOGQUBIT..(i + 1) * PHISQUBIT_PER_LOGQUBIT {
                self.sim.h(j);
            }
            self.cx(i * PHISQUBIT_PER_LOGQUBIT, m0);
            self.cx(i * PHISQUBIT_PER_LOGQUBIT + 1, m0 + 1);
            self.cx(i * PHISQUBIT_PER_LOGQUBIT + 2, m0 + 2);
            self.cx(i * PHISQUBIT_PER_LOGQUBIT + 3, m0 + 1);
            self.cx(i * PHISQUBIT_PER_LOGQUBIT + 3, m0 + 2);
            self.cx(i * PHISQUBIT_PER_LOGQUBIT + 4, m0);
            self.cx(i * PHISQUBIT_PER_LOGQUBIT + 4, m0 + 2);
            self.cx(i * PHISQUBIT_PER_LOGQUBIT + 5, m0);
            self.cx(i * PHISQUBIT_PER_LOGQUBIT + 5, m0 + 1);
            self.cx(i * PHISQUBIT_PER_LOGQUBIT + 5, m0 + 2);
            self.cx(i * PHISQUBIT_PER_LOGQUBIT + 6, m0);
            self.cx(i * PHISQUBIT_PER_LOGQUBIT + 6, m0 + 1);
            for j in i * PHISQUBIT_PER_LOGQUBIT..(i + 1) * PHISQUBIT_PER_LOGQUBIT {
                self.sim.h(j);
            }
            self.cx(i * PHISQUBIT_PER_LOGQUBIT, m0 + 3);
            self.cx(i * PHISQUBIT_PER_LOGQUBIT, m0 + 5);
            self.cx(i * PHISQUBIT_PER_LOGQUBIT + 1, m0 + 4);
            self.cx(i * PHISQUBIT_PER_LOGQUBIT + 1, m0 + 5);
            self.cx(i * PHISQUBIT_PER_LOGQUBIT + 2, m0 + 4);
            self.cx(i * PHISQUBIT_PER_LOGQUBIT + 2, m0 + 5);
            self.cx(i * PHISQUBIT_PER_LOGQUBIT + 2, m0 + 6);
            self.cx(i * PHISQUBIT_PER_LOGQUBIT + 3, m0 + 4);
            self.cx(i * PHISQUBIT_PER_LOGQUBIT + 4, m0 + 5);
            self.cx(i * PHISQUBIT_PER_LOGQUBIT + 5, m0 + 6);
            self.cx(i * PHISQUBIT_PER_LOGQUBIT + 6, m0 + 4);
            self.cx(i * PHISQUBIT_PER_LOGQUBIT + 6, m0 + 5);
            for j in 0..PHISQUBIT_PER_LOGQUBIT {
                self.sim.measure(i * PHISQUBIT_PER_LOGQUBIT + j, j);
            }
            let measured = self.sim.measured.inner[0] & MEASURE_MASK;
            for j in 0..PHISQUBIT_PER_LOGQUBIT {
                // reset
                if measured & (1 << j) != 0 {
                    self.x(i * PHISQUBIT_PER_LOGQUBIT + j);
                }
            }
            if measured & 7 > 0 {
                let err_x = ERR_TABLE_X[(measured & 7) as usize];
                self.x(err_x);
            }
            if (measured >> 3) > 0 {
                let err_z = ERR_TABLE_X[(measured >> 3) as usize];
                self.z(err_z);
            }
        }
    }
}

impl Operations for SteaneLayer {
    type Qubit = Qubit;
    type Slot = Qubit;
    fn initialize(&mut self) {
        self.sim.initialize();
    }
    fn measure(&mut self, q: <Self as lay::Operations>::Qubit, c: <Self as lay::Operations>::Slot) {

    }
}

impl CliffordGate for SteaneLayer {
   fn x(&mut self, q: <Self as lay::Operations>::Qubit) {
       for i in (q * PHISQUBIT_PER_LOGQUBIT)..(q * PHISQUBIT_PER_LOGQUBIT + PHISQUBIT_PER_LOGQUBIT) {
           self.sim.x(i);
       }
   }
   fn y(&mut self, q: <Self as lay::Operations>::Qubit) {
       for i in (q * PHISQUBIT_PER_LOGQUBIT)..(q * PHISQUBIT_PER_LOGQUBIT + PHISQUBIT_PER_LOGQUBIT) {
           self.sim.y(i);
       }
   }
   fn z(&mut self, q: <Self as lay::Operations>::Qubit) {
       for i in (q * PHISQUBIT_PER_LOGQUBIT)..(q * PHISQUBIT_PER_LOGQUBIT + PHISQUBIT_PER_LOGQUBIT) {
           self.sim.z(i);
       }
   }
   fn h(&mut self, q: <Self as lay::Operations>::Qubit) {
       for i in (q * PHISQUBIT_PER_LOGQUBIT)..(q * PHISQUBIT_PER_LOGQUBIT + PHISQUBIT_PER_LOGQUBIT) {
           self.sim.h(i);
       }
   }
   fn s(&mut self, q: <Self as lay::Operations>::Qubit) {
       for i in (q * PHISQUBIT_PER_LOGQUBIT)..(q * PHISQUBIT_PER_LOGQUBIT + PHISQUBIT_PER_LOGQUBIT) {
           self.sim.s(i);
       }
   }
   fn sdg(&mut self, q: <Self as lay::Operations>::Qubit) {
       for i in (q * PHISQUBIT_PER_LOGQUBIT)..(q * PHISQUBIT_PER_LOGQUBIT + PHISQUBIT_PER_LOGQUBIT) {
           self.sim.sdg(i);
       }
   }
   fn cx(&mut self, c: <Self as lay::Operations>::Qubit, t: <Self as lay::Operations>::Qubit) {
       for i in 0..PHISQUBIT_PER_LOGQUBIT {
           self.sim.cx(c * PHISQUBIT_PER_LOGQUBIT + i, t * PHISQUBIT_PER_LOGQUBIT + i);
       }
   }
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
