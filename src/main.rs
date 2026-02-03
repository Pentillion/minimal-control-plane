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
                ControlPlane::reconcile_vm(desired, actual, &mut cp.hosts);
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

    fn reconcile_vm(desired: &DesiredVm, actual: &mut ActualVm, hosts: &mut [Host]) {
        if actual.state == desired.target_state {
            return;
        }

        if actual.state == VmState::Requested {
            if try_allocate(desired, actual, hosts) {
                actual.state = VmState::Allocated;
            } else {
                println!(
                    "vm={} actual={:?} desired={:?} - Unable to allocate vm, not proceeding further...",
                    actual.id, actual, desired
                );
                return;
            }
        }

        let valid_states = get_valid_states(&actual.state);
        let next_state = choose_by_policy(valid_states, &desired.target_state);
        match next_state {
            None => {
                println!(
                    "vm={} actual={:?} desired={:?} - Unable to move to desired state, not proceeding further...",
                    actual.id, actual, desired
                );
            }
            Some(next) => {
                actual.state = *next;
            }
        }
    }
}

fn try_allocate(
    desired: &DesiredVm,
    actual: &mut ActualVm,
    hosts: &mut [Host]
) -> bool {
    for host in hosts {
        if host.is_alive && host.total_cpu - host.used_cpu >= desired.cpu && host.total_memory_mb - host.used_memory_mb >= desired.memory_mb {
            actual.host_id = Some(host.id);
            host.used_cpu += desired.cpu;
            host.used_memory_mb += desired.memory_mb;
            return true;
        }
    }
    false
}

fn choose_by_policy<'a>(allowed: &'a [VmState], desired: &'a VmState) -> Option<&'a VmState> {
    let next_states: &[VmState] = match desired {
        VmState::Running => {
            &[VmState::Booting, VmState::Allocated, VmState::Running]
        }
        VmState::Stopped => {
            &[VmState::Stopped, VmState::Stopping]
        }
        VmState::Destroyed => {
            &[VmState::Stopped, VmState::Stopping, VmState::Destroyed]
        }
        _ => {
            &[]
        }
    };

    let next_state = allowed.iter().find(|state| next_states.contains(state));

    match next_state {
        None => {
            return None;
        },
        Some(state) => {
            return Some(state);
        }
    }
}

fn get_valid_states(actual: &VmState) -> &[VmState] {   
    match actual {
        VmState::Requested => &[VmState::Allocated, VmState::Failed],
        VmState::Allocated => &[VmState::Booting, VmState::Failed],
        VmState::Booting => &[VmState::Running, VmState::Failed],
        VmState::Running => &[VmState::Stopping, VmState::Failed],
        VmState::Stopping => &[VmState::Stopped, VmState::Failed],
        VmState::Stopped => &[VmState::Destroyed, VmState::Failed],
        VmState::Failed => &[VmState::Allocated, VmState::Failed],
        _ => &[]
    } 
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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