use lay::{Layer, Measured, OpsVec, operations::{opid, OpArgs, Operation}, gates::{PauliGate, HGate, SGate, CXGate}};
use lay_simulator_gk::{ GottesmanKnillSimulator, DefaultRng };

use num_traits::cast::{cast, NumCast};

macro_rules! cast { ($n:expr) => { cast($n).unwrap() } }

const PHYSQUBIT_PER_LOGQUBIT: u32 = 7;
const MEASURE_ANCILLA_QUBITS: u32 = 6;
// const MEASURE_MASK: u32 = 127;

pub struct SteaneLayer<L> {
    pub instance: L,
    n_logical_qubits: u32,
}

const ERR_TABLE_X: [u32;8] = [999 /* dummy */, 0, 1, 6, 2, 4, 3, 5];
const ERR_TABLE_Z: [u32;8] = [999 /* dummy */, 3, 4, 6, 5, 0, 1, 2];

pub fn required_physical_qubits(n_logical_qubits: u32) -> u32 {
    PHYSQUBIT_PER_LOGQUBIT * n_logical_qubits + MEASURE_ANCILLA_QUBITS
}

impl<L> SteaneLayer<L> {
    pub fn from_instance(instance: L, n_logical_qubits: u32) -> Self {
        Self { instance, n_logical_qubits }
    }
}

impl SteaneLayer<GottesmanKnillSimulator<DefaultRng>> {
    pub fn from_seed_with_gk(seed: u64, n_logical_qubits: u32) -> Self {
        SteaneLayer::from_instance(
            GottesmanKnillSimulator::from_seed(required_physical_qubits(n_logical_qubits) as _, seed),
            n_logical_qubits)
    }
}

pub struct Buf();
impl Measured for Buf {
    type Slot = u32;
    fn get(&self, _: u32) -> bool { false }
}

impl<L: Layer + PauliGate + HGate + SGate + CXGate> Layer for SteaneLayer<L>
where
        L : Layer + PauliGate + HGate + SGate + CXGate,
        L::Operation: Operation<L>,
        L::Qubit: NumCast,
        L::Slot : NumCast,
{
    type Operation = OpArgs<Self>;
    type Qubit = u32;
    type Slot = u32;
    type Buffer = Buf;
    type Requested = L::Requested;
    type Response = L::Response;

    fn send(&mut self, ops: &[Self::Operation]) -> L::Requested {
        let mut lowlevel_ops = OpsVec::new();
        for op in ops {
            match op {
                OpArgs::Empty(id) if *id == opid::INIT => {
                    self.initialize(&mut lowlevel_ops);
                },
                OpArgs::Empty(id) if *id == opid::USERDEF => {
                    self.syndrome_measure_and_recover(&mut lowlevel_ops);
                },
                OpArgs::Q(id, q) => {
                    match *id {
                        opid::X => {
                            self.x(*q, &mut lowlevel_ops);
                        },
                        opid::Y => {
                            self.y(*q, &mut lowlevel_ops);
                        },
                        opid::Z => {
                            self.z(*q, &mut lowlevel_ops);
                        },
                        opid::H => {
                            self.h(*q, &mut lowlevel_ops);
                        },
                        opid::S => {
                            self.s(*q, &mut lowlevel_ops);
                        },
                        opid::SDG => {
                            self.sdg(*q, &mut lowlevel_ops);
                        },
                        _ => {
                            unimplemented!("Unknown 1 qubit op");
                        }
                    }
                },
                OpArgs::QQ(id, c, t) if *id == opid::CX => {
                    self.cx(*c, *t, &mut lowlevel_ops);
                },
                _ => unimplemented!("Unknown op")
            }
        }
        self.instance.send(lowlevel_ops.as_ref())
    }

    fn receive(&mut self, _: &mut Buf) -> L::Response {
        let mut buf = self.instance.make_buffer();
        self.instance.receive(&mut buf)
    }

    fn send_receive(&mut self, ops: &[Self::Operation], buf: &mut Buf) -> L::Response {
        self.send(&ops);
        self.receive(buf)
    }

    fn make_buffer(&self) -> Buf {
        Buf()
    }
}

impl<L: Layer + PauliGate + HGate + SGate + CXGate> PauliGate for SteaneLayer<L>
where
        L : Layer + PauliGate + HGate + SGate + CXGate,
        L::Operation: Operation<L>,
        L::Qubit : NumCast,
        L::Slot : NumCast {}

impl<L: Layer + PauliGate + HGate + SGate + CXGate> HGate for SteaneLayer<L>
where
        L : Layer + PauliGate + HGate + CXGate,
        L::Operation: Operation<L>,
        L::Qubit : NumCast,
        L::Slot : NumCast {}

impl<L: Layer + PauliGate + HGate + SGate + CXGate> SGate for SteaneLayer<L>
where
        L : Layer + PauliGate + HGate + CXGate,
        L::Operation: Operation<L>,
        L::Qubit : NumCast,
        L::Slot : NumCast {}

impl<L: Layer + PauliGate + HGate + SGate + CXGate> CXGate for SteaneLayer<L>
where
        L : Layer + PauliGate + HGate + CXGate,
        L::Operation: Operation<L>,
        L::Qubit : NumCast,
        L::Slot : NumCast {}

impl<L: Layer + PauliGate + HGate + SGate + CXGate> SteaneLayer<L>
where
        L : Layer + PauliGate + HGate + CXGate,
        L::Operation: Operation<L>,
        L::Qubit : NumCast,
        L::Slot : NumCast,
{
    fn measure_ancilla(&self) -> u32 {
        //self.sim.n_qubits() as Qubit - MEASURE_ANCILLA_QUBITS
        self.n_logical_qubits * PHYSQUBIT_PER_LOGQUBIT
    }

    fn initialize(&mut self, ops: &mut OpsVec<L>) {
        ops.initialize();
        self.syndrome_measure_and_recover(ops);
    }

    fn syndrome_measure_and_recover(&mut self, ops: &mut OpsVec<L>) {
        eprintln!("START syndrome_measure_and_recover");
        let m0 = self.measure_ancilla();
        for i in 0..self.n_logical_qubits {
            let offset = i * PHYSQUBIT_PER_LOGQUBIT;
            for j in 0..PHYSQUBIT_PER_LOGQUBIT {
                ops.h(cast!(offset + j));
            }
            ops.cx(cast!(offset), cast!(m0));
            ops.cx(cast!(offset + 1), cast!(m0 + 1));
            ops.cx(cast!(offset + 2), cast!(m0 + 2));
            ops.cx(cast!(offset + 3), cast!(m0 + 1));
            ops.cx(cast!(offset + 3), cast!(m0 + 2));
            ops.cx(cast!(offset + 4), cast!(m0));
            ops.cx(cast!(offset + 4), cast!(m0 + 2));
            ops.cx(cast!(offset + 5), cast!(m0));
            ops.cx(cast!(offset + 5), cast!(m0 + 1));
            ops.cx(cast!(offset + 5), cast!(m0 + 2));
            ops.cx(cast!(offset + 6), cast!(m0));
            ops.cx(cast!(offset + 6), cast!(m0 + 1));

            for j in 0..PHYSQUBIT_PER_LOGQUBIT {
                ops.h(cast!(offset + j));
            }
            ops.cx(cast!(offset), cast!(m0 + 3));
            ops.cx(cast!(offset), cast!(m0 + 5));
            ops.cx(cast!(offset + 1), cast!(m0 + 4));
            ops.cx(cast!(offset + 1), cast!(m0 + 5));
            ops.cx(cast!(offset + 2), cast!(m0 + 3));
            ops.cx(cast!(offset + 2), cast!(m0 + 4));
            ops.cx(cast!(offset + 2), cast!(m0 + 5));
            ops.cx(cast!(offset + 3), cast!(m0 + 3));
            ops.cx(cast!(offset + 4), cast!(m0 + 4));
            ops.cx(cast!(offset + 5), cast!(m0 + 5));
            ops.cx(cast!(offset + 6), cast!(m0 + 3));
            ops.cx(cast!(offset + 6), cast!(m0 + 4));
            for j in 0..MEASURE_ANCILLA_QUBITS {
                ops.measure(cast!(m0 + j), cast!(j));
            }
            let mut buf = self.instance.make_buffer();
            self.instance.send_receive(ops.as_ref(), &mut buf);
            let measured = buf.get_range_u8(0, MEASURE_ANCILLA_QUBITS as usize);
            eprintln!("logical qubit: {}, measured: {:b}", i, measured);
            ops.clear();
            for j in 0..MEASURE_ANCILLA_QUBITS {
                // reset
                if measured & (1 << j) != 0 {
                    ops.x(cast!(m0 + j));
                }
            }
            if measured & 7 > 0 {
                let err_x = ERR_TABLE_X[(measured & 7) as usize] + i * PHYSQUBIT_PER_LOGQUBIT;
                eprintln!("Z Err on {}", err_x);
                ops.z(cast!(err_x));
            }
            if (measured >> 3) > 0 {
                let err_z = ERR_TABLE_Z[(measured >> 3) as usize] + i * PHYSQUBIT_PER_LOGQUBIT;
                eprintln!("X Err on {}", err_z);
                ops.x(cast!(err_z));
            }
        }
        eprintln!("END   syndrome_measure_and_recover");
    }

    fn x(&mut self, q: u32, ops: &mut OpsVec<L>) {
        for i in (q * PHYSQUBIT_PER_LOGQUBIT)..(q * PHYSQUBIT_PER_LOGQUBIT + PHYSQUBIT_PER_LOGQUBIT) {
            ops.x(cast!(i));
        }
    }

    fn y(&mut self, q: u32, ops: &mut OpsVec<L>) {
        for i in (q * PHYSQUBIT_PER_LOGQUBIT)..(q * PHYSQUBIT_PER_LOGQUBIT + PHYSQUBIT_PER_LOGQUBIT) {
            ops.y(cast!(i));
        }
    }

    fn z(&mut self, q: u32, ops: &mut OpsVec<L>) {
        for i in (q * PHYSQUBIT_PER_LOGQUBIT)..(q * PHYSQUBIT_PER_LOGQUBIT + PHYSQUBIT_PER_LOGQUBIT) {
            ops.z(cast!(i));
        }
    }

    fn h(&mut self, q: u32, ops: &mut OpsVec<L>) {
        for i in (q * PHYSQUBIT_PER_LOGQUBIT)..(q * PHYSQUBIT_PER_LOGQUBIT + PHYSQUBIT_PER_LOGQUBIT) {
            ops.h(cast!(i));
        }
    }

    fn s(&mut self, q: u32, ops: &mut OpsVec<L>) {
        for i in (q * PHYSQUBIT_PER_LOGQUBIT)..(q * PHYSQUBIT_PER_LOGQUBIT + PHYSQUBIT_PER_LOGQUBIT) {
            ops.s(cast!(i));
        }
    }

    fn sdg(&mut self, q: u32, ops: &mut OpsVec<L>) {
        for i in (q * PHYSQUBIT_PER_LOGQUBIT)..(q * PHYSQUBIT_PER_LOGQUBIT + PHYSQUBIT_PER_LOGQUBIT) {
            ops.sdg(cast!(i));
        }
    }

    fn cx(&mut self, c: u32, t: u32, ops: &mut OpsVec<L>) {
        for i in 0..PHYSQUBIT_PER_LOGQUBIT {
            ops.cx(cast!(c * PHYSQUBIT_PER_LOGQUBIT + i), cast!(t * PHYSQUBIT_PER_LOGQUBIT + i));
        }
    }
}

pub trait Syndrome<L>
where
        L : Layer + PauliGate + HGate + SGate + CXGate,
        L::Operation : Operation<L>,
        L::Qubit : NumCast,
        L::Slot : NumCast
{
    fn syndrome(&mut self);
}

impl<L> Syndrome<L> for OpsVec<SteaneLayer<L>>
where
        L : Layer + PauliGate + HGate + SGate + CXGate,
        L::Operation : Operation<L>,
        L::Qubit : NumCast,
        L::Slot : NumCast
{
    fn syndrome(&mut self) {
        self.as_mut_vec().push(OpArgs::Empty(opid::USERDEF));
    }
}




#[cfg(test)]
mod tests {
    use lay::{Layer, OpsVec};
    use lay_simulator_gk::{ GottesmanKnillSimulator, DefaultRng };
    use crate::{SteaneLayer, Syndrome};

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn it_works2() {
        let mut steane = SteaneLayer::from_seed_with_gk(1, 16);
        let mut ops = OpsVec::<SteaneLayer<_>>::new();
        ops.initialize();
        steane.send(ops.as_ref());
        /*
        ops.syndrome_measure_and_recover();
        ops.syndrome_measure_and_recover();
        ops.syndrome_measure_and_recover();
        ops.syndrome_measure_and_recover();
        ops.syndrome_measure_and_recover();
        eprintln!("END First syndrome measurement 5 times");
        eprintln!("Expected: not shown");
        ops.syndrome_measure_and_recover();
        ops.x(0);
        eprintln!("Expected: not shown");
        ops.syndrome_measure_and_recover();
        ops.x(12);
        eprintln!("Expected: 12");
        ops.syndrome_measure_and_recover();
        eprintln!("Expected: not shown");
        ops.syndrome_measure_and_recover();
        ops.sim.z(8);
        eprintln!("Expected: 8");
        ops.syndrome_measure_and_recover();
        eprintln!("Expected: not shown");
        ops.syndrome_measure_and_recover();
        */
    }
}
