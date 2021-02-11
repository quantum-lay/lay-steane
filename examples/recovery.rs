use lay::{Layer, OpsVec};
use lay_simulator_gk::GottesmanKnillSimulator;
use lay_steane::{SteaneLayer, Syndrome};

fn main() {
    let mut steane = SteaneLayer::from_seed_with_gk(16, 1);
    let mut ops = OpsVec::new();
    let mut inner_ops = OpsVec::<GottesmanKnillSimulator<_>>::new();
    ops.initialize();
    steane.send(ops.as_ref());
    eprintln!("END Initialize");
    eprintln!("Expected: error not shown");
    ops.clear();
    ops.syndrome();
    ops.x(0);
    ops.syndrome();
    eprintln!("Expected: error not shown");
    steane.send(ops.as_ref());
    ops.clear();
    inner_ops.x(12);
    steane.instance.send(inner_ops.as_ref());
    ops.syndrome();
    eprintln!("Expected: 12");
    steane.send(ops.as_ref());
    eprintln!("Expected: not shown");
    steane.send(ops.as_ref());
    inner_ops.clear();
    inner_ops.z(8);
    steane.instance.send(inner_ops.as_ref());
    eprintln!("Expected: 8");
    steane.send(ops.as_ref());
    eprintln!("Expected: not shown");
}
