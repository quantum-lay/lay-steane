use lay::Operations;
use lay::gates::{CliffordGate};
use lay_simulator_gk::{ GottesmanKnillSimulator, DefaultRng, Qubit };

pub struct SteaneLayer {
    sim: GottesmanKnillSimulator<DefaultRng>,
}

impl SteaneLayer {
    pub fn new(n: Qubit) -> Self {
        Self { sim: GottesmanKnillSimulator::from_seed(7 * n + 6, 0) }
    }
}

impl Operations for SteaneLayer {
    type Qubit = Qubit;
    type Slot = Qubit;
    fn initialize(&mut self) { todo!() }
    fn measure(&mut self, _: <Self as lay::Operations>::Qubit, _: <Self as lay::Operations>::Slot) { todo!() }
}

impl CliffordGate for SteaneLayer {
   fn x(&mut self, _: <Self as lay::Operations>::Qubit) { todo!() }
   fn y(&mut self, _: <Self as lay::Operations>::Qubit) { todo!() }
   fn z(&mut self, _: <Self as lay::Operations>::Qubit) { todo!() }
   fn h(&mut self, _: <Self as lay::Operations>::Qubit) { todo!() }
   fn s(&mut self, _: <Self as lay::Operations>::Qubit) { todo!() }
   fn sdg(&mut self, _: <Self as lay::Operations>::Qubit) { todo!() }
   fn cx(&mut self, _: <Self as lay::Operations>::Qubit, _: <Self as lay::Operations>::Qubit) { todo!() }
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
