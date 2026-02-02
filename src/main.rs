use rand::Rng;

fn main() {
    let mut cp: ControlPlane = ControlPlane::new();

    cp.desired_vms.push(DesiredVm {
        id: 1,
        cpu: 2,
        memory_mb: 2048,
        target_state: VmState::Running,
    });

    cp.actual_vms.push(ActualVm {
        id: 1,
        state: VmState::Requested,
        host_id: None,
    });

    loop {
        for desired in &cp.desired_vms {
            if let Some(actual) = cp.actual_vms.iter_mut().find(|v| v.id == desired.id) {
                ControlPlane::reconcile_vm(desired, actual);
                println!("VM {} -> {:?}", actual.id, actual.state);
            }
        }

        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

struct ControlPlane {
    desired_vms: Vec<DesiredVm>,
    actual_vms: Vec<ActualVm>,
    hosts: Vec<Host>,
}

impl ControlPlane {
    fn new() -> Self {
        Self {
            desired_vms: Vec::new(),
            actual_vms: Vec::new(),
            hosts: vec![
                Host {
                    id: 1,
                    total_cpu: 8,
                    used_cpu: 0,
                    total_memory_mb: 16384,
                    used_memory_mb: 0,
                    is_alive: true,
                },
            ],
        }
    }

    fn reconcile_vm(desired: &DesiredVm, actual: &mut ActualVm) {
        if actual.state == desired.target_state {
            return;
        }

        match (actual.state, desired.target_state) {
            (VmState::Requested, VmState::Running) => {
                actual.state = VmState::Allocated;
            }
            (VmState::Allocated, VmState::Running) => {
                actual.state = VmState::Booting;
            }
            (VmState::Booting, VmState::Running) => {
                let mut rng = rand::thread_rng();
                if rng.gen_bool(0.3) {
                    actual.state = VmState::Failed;
                } else {
                    actual.state = VmState::Running;
                }
            }
            (VmState::Failed, VmState::Running) => {
                actual.state = VmState::Allocated;
            }
            (_, VmState::Destroyed) => {
                actual.state = VmState::Destroyed;
            }
            _ => {}
        }
    }

}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VmState {
    Requested,
    Allocated,
    Booting,
    Running,
    Stopping,
    Stopped,
    Destroyed,
    Failed
}

#[derive(Debug, Clone)]
struct DesiredVm {
    id: u64,
    cpu: u8,
    memory_mb: u32,
    target_state: VmState,
}

#[derive(Debug)]
struct ActualVm {
    id: u64,
    state: VmState,
    host_id: Option<u64>,
}


struct Host {
    id: u64,
    total_cpu: u8,
    used_cpu: u8,
    total_memory_mb: u32,
    used_memory_mb: u32,
    is_alive: bool
}