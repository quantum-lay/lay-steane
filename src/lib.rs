use lay::{Layer, Operation, OpsVec, gates::{PauliGate, HGate, CXGate}};
use lay_simulator_gk::{ GottesmanKnillSimulator, DefaultRng, Qubit };

const PHYSQUBIT_PER_LOGQUBIT: usize = 7;
const MEASURE_ANCILLA_QUBITS: usize = 6;
const MEASURE_MASK: u32 = 127;

/*
pub struct SteaneLayer {
    // TODO: not pub.
    pub sim: GottesmanKnillSimulator<DefaultRng>,
    n_logical_qubits: Qubit
}
*/

pub struct SteaneLayer<L> {
    instance: L,
    n_logical_qubits: usize,
}

const ERR_TABLE_X: [u32;8] = [999 /* dummy */, 0, 1, 6, 2, 4, 3, 5];
const ERR_TABLE_Z: [u32;8] = [999 /* dummy */, 3, 4, 6, 5, 0, 1, 2];

pub fn required_physical_qubits(n_logical_qubits: usize) -> usize {
    PHYSQUBIT_PER_LOGQUBIT * n_logical_qubits + MEASURE_ANCILLA_QUBITS
}

impl<L> SteaneLayer<L> {
    pub fn from_instance(instance: L, n_logical_qubits: usize) -> Self {
        Self { instance, n_logical_qubits }
    }
}

impl SteaneLayer<GottesmanKnillSimulator<DefaultRng>> {
    pub fn from_seed_with_gk(seed: u64, n_logical_qubits: usize) -> Self {
        SteaneLayer::from_instance(
            GottesmanKnillSimulator::from_seed(required_physical_qubits(n_logical_qubits) as Qubit, seed),
            n_logical_qubits)
    }
}

impl<L: Layer + PauliGate + HGate + CXGate> Layer for SteaneLayer<L> {
    type Qubit = L::Qubit;
    type Slot = L::Slot;
    type Buffer = L::Buffer;
    type Requested = L::Requested;
    type Response = L::Response;

    fn send(&mut self, ops: &[Operation<SteaneLayer<L>>]) -> L::Requested {
        todo!();
    }

    fn receive(&mut self, buf: &mut L::Buffer) -> L::Response {
        todo!();
    }

    fn send_receive(&mut self, ops: &[Operation<SteaneLayer<L>>], buf: &mut L::Buffer) -> L::Response {
        todo!();
    }
}

impl<L: Layer + PauliGate + HGate + CXGate> PauliGate for SteaneLayer<L> {}
impl<L: Layer + PauliGate + HGate + CXGate> HGate for SteaneLayer<L> {}
impl<L: Layer + PauliGate + HGate + CXGate> CXGate for SteaneLayer<L> {}

impl<L: Layer + PauliGate + HGate + CXGate> SteaneLayer<L> {
    fn measure_ancilla(&self) -> usize {
        //self.sim.n_qubits() as Qubit - MEASURE_ANCILLA_QUBITS
        self.n_logical_qubits * PHYSQUBIT_PER_LOGQUBIT
    }

    fn syndrome_measure_and_recover(&mut self) {
        eprintln!("START syndrome_measure_and_recover");
        let ops = OpsVec::new();
        let m0 = self.measure_ancilla();
        for i in 0..self.n_logical_qubits {
            for j in 0..PHYSQUBIT_PER_LOGQUBIT {
                ops.h(i * PHYSQUBIT_PER_LOGQUBIT + j);
            }
            ops.cx(i * PHYSQUBIT_PER_LOGQUBIT, m0);
            ops.cx(i * PHYSQUBIT_PER_LOGQUBIT + 1, m0 + 1);
            ops.cx(i * PHYSQUBIT_PER_LOGQUBIT + 2, m0 + 2);
            ops.cx(i * PHYSQUBIT_PER_LOGQUBIT + 3, m0 + 1);
            ops.cx(i * PHYSQUBIT_PER_LOGQUBIT + 3, m0 + 2);
            ops.cx(i * PHYSQUBIT_PER_LOGQUBIT + 4, m0);
            ops.cx(i * PHYSQUBIT_PER_LOGQUBIT + 4, m0 + 2);
            ops.cx(i * PHYSQUBIT_PER_LOGQUBIT + 5, m0);
            ops.cx(i * PHYSQUBIT_PER_LOGQUBIT + 5, m0 + 1);
            ops.cx(i * PHYSQUBIT_PER_LOGQUBIT + 5, m0 + 2);
            ops.cx(i * PHYSQUBIT_PER_LOGQUBIT + 6, m0);
            ops.cx(i * PHYSQUBIT_PER_LOGQUBIT + 6, m0 + 1);

            for j in 0..PHYSQUBIT_PER_LOGQUBIT {
                ops.h(i * PHYSQUBIT_PER_LOGQUBIT + j);
            }
            ops.cx(i * PHYSQUBIT_PER_LOGQUBIT, m0 + 3);
            ops.cx(i * PHYSQUBIT_PER_LOGQUBIT, m0 + 5);
            ops.cx(i * PHYSQUBIT_PER_LOGQUBIT + 1, m0 + 4);
            ops.cx(i * PHYSQUBIT_PER_LOGQUBIT + 1, m0 + 5);
            ops.cx(i * PHYSQUBIT_PER_LOGQUBIT + 2, m0 + 3);
            ops.cx(i * PHYSQUBIT_PER_LOGQUBIT + 2, m0 + 4);
            ops.cx(i * PHYSQUBIT_PER_LOGQUBIT + 2, m0 + 5);
            ops.cx(i * PHYSQUBIT_PER_LOGQUBIT + 3, m0 + 3);
            ops.cx(i * PHYSQUBIT_PER_LOGQUBIT + 4, m0 + 4);
            ops.cx(i * PHYSQUBIT_PER_LOGQUBIT + 5, m0 + 5);
            ops.cx(i * PHYSQUBIT_PER_LOGQUBIT + 6, m0 + 3);
            ops.cx(i * PHYSQUBIT_PER_LOGQUBIT + 6, m0 + 4);
            for j in 0..MEASURE_ANCILLA_QUBITS {
                ops.measure(m0 + j, j);
            }
            // TODO: バックエンド変わったときダメ。まともな方法
            let measured = self.sim.measured.inner[0] & MEASURE_MASK;
            self.sim.measured.inner[0] = 0;
            eprintln!("logical qubit: {}, measured: {:b}", i, measured);
            for j in 0..MEASURE_ANCILLA_QUBITS {
                // reset
                if measured & (1 << j) != 0 {
                    self.sim.x(m0 + j);
                }
            }
            if measured & 7 > 0 {
                let err_x = ERR_TABLE_X[(measured & 7) as usize] + i * PHYSQUBIT_PER_LOGQUBIT;
                eprintln!("Z Err on {}", err_x);
                self.sim.z(err_x);
            }
            if (measured >> 3) > 0 {
                let err_z = ERR_TABLE_Z[(measured >> 3) as usize] + i * PHYSQUBIT_PER_LOGQUBIT;
                eprintln!("X Err on {}", err_z);
                self.sim.x(err_z);
            }
        }
        eprintln!("END   syndrome_measure_and_recover");
    }
}

/*
impl Operations for SteaneLayer {
    fn initialize(&mut self) {
        self.sim.initialize();
        //self.syndrome_measure_and_recover();
    }
    fn measure(&mut self, q: <Self as lay::Operations>::Qubit, c: <Self as lay::Operations>::Slot) {

    }
}

impl CliffordGate for SteaneLayer {
   fn x(&mut self, q: <Self as lay::Operations>::Qubit) {
       for i in (q * PHYSQUBIT_PER_LOGQUBIT)..(q * PHISQUBIT_PER_LOGQUBIT + PHISQUBIT_PER_LOGQUBIT) {
           self.sim.x(i);
       }
   }
   fn y(&mut self, q: <Self as lay::Operations>::Qubit) {
       for i in (q * PHYSQUBIT_PER_LOGQUBIT)..(q * PHISQUBIT_PER_LOGQUBIT + PHISQUBIT_PER_LOGQUBIT) {
           self.sim.y(i);
       }
   }
   fn z(&mut self, q: <Self as lay::Operations>::Qubit) {
       for i in (q * PHYSQUBIT_PER_LOGQUBIT)..(q * PHISQUBIT_PER_LOGQUBIT + PHISQUBIT_PER_LOGQUBIT) {
           self.sim.z(i);
       }
   }
   fn h(&mut self, q: <Self as lay::Operations>::Qubit) {
       for i in (q * PHYSQUBIT_PER_LOGQUBIT)..(q * PHISQUBIT_PER_LOGQUBIT + PHISQUBIT_PER_LOGQUBIT) {
           self.sim.h(i);
       }
   }
   fn s(&mut self, q: <Self as lay::Operations>::Qubit) {
       for i in (q * PHYSQUBIT_PER_LOGQUBIT)..(q * PHISQUBIT_PER_LOGQUBIT + PHISQUBIT_PER_LOGQUBIT) {
           self.sim.s(i);
       }
   }
   fn sdg(&mut self, q: <Self as lay::Operations>::Qubit) {
       for i in (q * PHYSQUBIT_PER_LOGQUBIT)..(q * PHISQUBIT_PER_LOGQUBIT + PHISQUBIT_PER_LOGQUBIT) {
           self.sim.sdg(i);
       }
   }
   fn cx(&mut self, c: <Self as lay::Operations>::Qubit, t: <Self as lay::Operations>::Qubit) {
       for i in 0..PHYSQUBIT_PER_LOGQUBIT {
           self.sim.cx(c * PHYSQUBIT_PER_LOGQUBIT + i, t * PHISQUBIT_PER_LOGQUBIT + i);
       }
   }
}
*/


#[cfg(test)]
mod tests {
    use lay::Operations;
    use lay::gates::{CliffordGate};
    use lay_simulator_gk::{ GottesmanKnillSimulator, DefaultRng, Qubit };
    use crate::SteaneLayer;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn it_works2() {
        let mut steane = SteaneLayer::new(3);
        steane.initialize();
        eprintln!("First syndrome measurement 5 times");
        steane.syndrome_measure_and_recover();
        steane.syndrome_measure_and_recover();
        steane.syndrome_measure_and_recover();
        steane.syndrome_measure_and_recover();
        steane.syndrome_measure_and_recover();
        eprintln!("END First syndrome measurement 5 times");
        eprintln!("Expected: not shown");
        steane.syndrome_measure_and_recover();
        steane.x(0);
        eprintln!("Expected: not shown");
        steane.syndrome_measure_and_recover();
        steane.sim.x(12);
        eprintln!("Expected: 12");
        steane.syndrome_measure_and_recover();
        eprintln!("Expected: not shown");
        steane.syndrome_measure_and_recover();
        steane.sim.z(8);
        eprintln!("Expected: 8");
        steane.syndrome_measure_and_recover();
        eprintln!("Expected: not shown");
        steane.syndrome_measure_and_recover();
    }
}
