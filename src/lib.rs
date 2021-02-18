use lay::{Layer, Measured, OpsVec, operations::{opid, OpArgs, Operation, PauliOperation, HOperation, SOperation, CXOperation}, gates::{PauliGate, HGate, SGate, CXGate}};
use lay_simulator_gk::{ GottesmanKnillSimulator, DefaultRng };

use num_traits::cast::{cast, NumCast};

macro_rules! cast { ($n:expr) => { cast($n).unwrap() } }

const PHYSQUBIT_PER_LOGQUBIT: u32 = 7;
const MEASURE_ANCILLA_QUBITS: u32 = 6;

pub struct SteaneLayer<L: Layer> {
    pub instance: L,
    n_logical_qubits: u32,
    instance_buf: L::Buffer,
    measured: Vec<bool>,
}

const ERR_TABLE_X: [u32;8] = [999 /* dummy */, 0, 1, 6, 2, 4, 3, 5];
const ERR_TABLE_Z: [u32;8] = [999 /* dummy */, 3, 4, 6, 5, 0, 1, 2];

pub fn required_physical_qubits(n_logical_qubits: u32) -> u32 {
    PHYSQUBIT_PER_LOGQUBIT * n_logical_qubits + MEASURE_ANCILLA_QUBITS
}

impl<L: Layer> SteaneLayer<L> {
    pub fn from_instance(instance: L, n_logical_qubits: u32) -> Self {
        let instance_buf = instance.make_buffer();
        Self { instance,
               n_logical_qubits,
               instance_buf,
               measured: vec![false; n_logical_qubits as usize]
        }
    }
}

impl SteaneLayer<GottesmanKnillSimulator<DefaultRng>> {
    pub fn from_seed_with_gk(n_logical_qubits: u32, seed: u64) -> Self {
        SteaneLayer::from_instance(
            GottesmanKnillSimulator::from_seed(required_physical_qubits(n_logical_qubits) as _, seed),
            n_logical_qubits)
    }
}

pub struct SteaneBuffer(Vec<bool>);

impl Measured for SteaneBuffer
{
    type Slot = u32;
    fn get(&self, s: u32) -> bool {
        (self.0)[s as usize]
    }
}

impl<L: Layer + PauliGate + HGate + SGate + CXGate> Layer for SteaneLayer<L>
where
        L : Layer + PauliGate + HGate + SGate + CXGate,
        L::Operation: Operation<L> + PauliOperation<L> + HOperation<L> + SOperation<L> + CXOperation<L>,
        L::Qubit: NumCast,
        L::Slot : NumCast,
{
    type Operation = OpArgs<Self>;
    type Qubit = u32;
    type Slot = u32;
    type Buffer = SteaneBuffer;
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
                OpArgs::QS(id, q, s) if *id == opid::MEAS => {
                    self.measure(*q, *s, &mut lowlevel_ops);
                },
                OpArgs::QQ(id, c, t) if *id == opid::CX => {
                    self.cx(*c, *t, &mut lowlevel_ops);
                },
                _ => unimplemented!("Unknown op")
            }
        }
        self.instance.send(lowlevel_ops.as_ref())
    }

    fn receive(&mut self, buf: &mut Self::Buffer) -> L::Response {
        let res = self.instance.receive(&mut self.instance_buf);
        std::mem::swap(&mut self.measured, &mut buf.0);
        res
    }

    fn send_receive(&mut self, ops: &[Self::Operation], buf: &mut Self::Buffer) -> L::Response {
        self.send(&ops);
        self.receive(buf)
    }

    fn make_buffer(&self) -> Self::Buffer {
        SteaneBuffer(vec![false; self.n_logical_qubits as usize])
    }
}

impl<L: Layer + PauliGate + HGate + SGate + CXGate> PauliGate for SteaneLayer<L>
where
        L : Layer + PauliGate + HGate + SGate + CXGate,
        L::Operation: Operation<L> + PauliOperation<L> + HOperation<L> + SOperation<L> + CXOperation<L>,
        L::Qubit : NumCast,
        L::Slot : NumCast {}

impl<L: Layer + PauliGate + HGate + SGate + CXGate> HGate for SteaneLayer<L>
where
        L : Layer + PauliGate + HGate + SGate + CXGate,
        L::Operation: Operation<L> + PauliOperation<L> + HOperation<L> + SOperation<L> + CXOperation<L>,
        L::Qubit : NumCast,
        L::Slot : NumCast {}

impl<L: Layer + PauliGate + HGate + SGate + CXGate> SGate for SteaneLayer<L>
where
        L : Layer + PauliGate + HGate + SGate + CXGate,
        L::Operation: Operation<L> + PauliOperation<L> + HOperation<L> + SOperation<L> + CXOperation<L>,
        L::Qubit : NumCast,
        L::Slot : NumCast {}

impl<L: Layer + PauliGate + HGate + SGate + CXGate> CXGate for SteaneLayer<L>
where
        L : Layer + PauliGate + HGate + SGate + CXGate,
        L::Operation: Operation<L> + PauliOperation<L> + HOperation<L> + SOperation<L> + CXOperation<L>,
        L::Qubit : NumCast,
        L::Slot : NumCast {}

impl<L: Layer + PauliGate + HGate + SGate + CXGate> SteaneLayer<L>
where
        L : Layer + PauliGate + HGate + SGate + CXGate,
        L::Operation: Operation<L> + PauliOperation<L> + HOperation<L> + SOperation<L> + CXOperation<L>,
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
            println!("X {}", {let i: u32 = cast!(i); i});
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

    fn measure(&mut self, q: u32, s: u32, ops: &mut OpsVec<L>) {
        let m0 = self.measure_ancilla();
        for i in 0..PHYSQUBIT_PER_LOGQUBIT {
            ops.cx(cast!(q * PHYSQUBIT_PER_LOGQUBIT + i), cast!(m0));
        }
        ops.measure(cast!(m0), cast!(0));
        self.instance.send_receive(ops.as_ref(), &mut self.instance_buf);
        let result = self.instance_buf.get(cast!(0));
        self.measured[s as usize] = result;
        if result {
            ops.x(cast!(m0));
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
        L::Operation : Operation<L> + PauliOperation<L> + HOperation<L> + SOperation<L> + CXOperation<L>,
        L::Qubit : NumCast,
        L::Slot : NumCast
{
    fn syndrome(&mut self) {
        self.as_mut_vec().push(OpArgs::Empty(opid::USERDEF));
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use lay::{Layer, OpsVec, Measured};
    #[allow(unused_imports)]
    use lay_simulator_gk::{ GottesmanKnillSimulator, DefaultRng };
    #[allow(unused_imports)]
    use crate::{SteaneLayer, Syndrome};

    #[test]
    fn initialize() {
        let mut steane = SteaneLayer::from_seed_with_gk(16, 1);
        let mut ops = OpsVec::<SteaneLayer<_>>::new();
        ops.initialize();
        steane.send(ops.as_ref());
    }

    #[test]
    fn initialize_and_measure() {
        let mut steane = SteaneLayer::from_seed_with_gk(16, 1);
        let mut ops = steane.opsvec();
        let mut buf = steane.make_buffer();
        ops.initialize();
        ops.x(1);
        ops.measure(0, 0);
        ops.measure(1, 1);
        steane.send_receive(ops.as_ref(), &mut buf);
        assert_eq!(buf.get(0), false);
        assert_eq!(buf.get(1), true);
    }

    #[test]
    fn cx() {
        let mut steane = SteaneLayer::from_seed_with_gk(4, 4);
        let mut ops = steane.opsvec();
        let mut buf = steane.make_buffer();
        ops.initialize();
        ops.x(1);
        ops.cx(1, 0);
        ops.measure(0, 0);
        for i in 0..10 {
            steane.send_receive(ops.as_ref(), &mut buf);
            assert!(buf.get(0));
        }
    }

    #[test]
    fn bell() {
        let mut steane = SteaneLayer::from_seed_with_gk(4, 4);
        let mut ops = steane.opsvec();
        let mut buf = steane.make_buffer();
        ops.initialize();
        ops.h(1);
        ops.cx(1, 0);
        ops.measure(0, 0);
        ops.measure(1, 1);
        for i in 0..10 {
            steane.send_receive(ops.as_ref(), &mut buf);
            eprintln!("try: {}, |{}{}>", i, buf.get(0) as u8, buf.get(1) as u8);
            assert_eq!(buf.get(0), buf.get(1));
        }
    }

    #[test]
    fn ghz() {
        let mut steane = SteaneLayer::from_seed_with_gk(4, 4);
        let mut ops = steane.opsvec();
        let mut buf = steane.make_buffer();
        ops.initialize();
        ops.h(1);
        ops.cx(1, 0);
        ops.cx(1, 2);
        ops.measure(0, 0);
        ops.measure(1, 1);
        ops.measure(2, 2);
        for i in 0..10 {
            steane.send_receive(ops.as_ref(), &mut buf);
            let m0 = buf.get(0);
            let m1 = buf.get(1);
            let m2 = buf.get(2);
            eprintln!("try: {}, |{}{}{}>", i, m0 as u8, m1 as u8, m2 as u8);
            assert_eq!(m0, m1);
            assert_eq!(m0, m2);
        }
    }
}
