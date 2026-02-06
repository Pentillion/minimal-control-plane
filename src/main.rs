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
        cpu: 0,
        memory_mb: 0
    });

    loop {
        for desired in &cp.desired_vms {
            if let Some(actual) = cp.actual_vms.iter_mut().find(|v| v.id == desired.id) {
                let action = ControlPlane::reconcile_vm(desired, actual, &mut cp.hosts);
                ControlPlane::apply_action(action, actual, desired, &mut cp.hosts);
                println!("VM {} -> {:?} via {:?}", actual.id, actual.state, action);
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

#[derive(Debug, Clone, Copy)]
enum Action {
    AllocateHost { host_id: u64 },
    BootVm,
    StopVm,
    ReleaseResources,
    NoOp,
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

    fn reconcile_vm(desired: &DesiredVm, actual: &mut ActualVm, hosts: &mut [Host]) -> Action {
        if actual.state == desired.target_state {
            return Action::NoOp;
        }

        if actual.state == VmState::Requested || (actual.state == VmState::Failed && desired.target_state == VmState::Running) {
            if let Some(host) = hosts.iter().find(|h| h.is_alive && h.has_resources_for(desired)) {
                return Action::AllocateHost { host_id: host.id };
            } else {
                println!(
                    "vm={} actual={:?} desired={:?} hosts={:?} - Unable to allocate vm, not proceeding further...",
                    actual.id, actual, desired, hosts
                );
                return Action::NoOp;
            }
        }

        if actual.state == VmState::Allocated && desired.target_state == VmState::Running {
            return Action::BootVm;
        }

        if actual.state == VmState::Running && desired.target_state == VmState::Stopped {
            return Action::StopVm;
        }

        if desired.target_state == VmState::Destroyed {
            return Action::ReleaseResources;
        }

        Action::NoOp
    }

    fn apply_action(action: Action, actual: &mut ActualVm, desired: &DesiredVm, hosts: &mut [Host]) {
        match action {
            Action::AllocateHost { host_id } => {
                if actual.host_id.is_none() {
                    actual.host_id = Some(host_id);
                    let host = hosts.iter_mut().find(|h| h.id == host_id).unwrap();
                    host.used_cpu += desired.cpu;
                    host.used_memory_mb += desired.memory_mb;
                    actual.cpu = desired.cpu;
                    actual.memory_mb = desired.memory_mb;
                    actual.state = VmState::Allocated;
                }
            }
            Action::BootVm => {
                actual.state = VmState::Booting;
                let mut rng = rand::thread_rng();
                if rng.gen_bool(0.3) {
                    actual.state = VmState::Failed;
                } else {
                    actual.state = VmState::Running;
                }
            }
            Action::StopVm => {
                actual.state = VmState::Stopping;
                // instant stop for now
                actual.state = VmState::Stopped;
            }
            Action::ReleaseResources => {
                if let Some(host_id) = actual.host_id {
                    let host =  hosts.iter_mut().find(|h| h.id == host_id).unwrap();
                    host.used_cpu -= actual.cpu;
                    host.used_memory_mb -= actual.memory_mb;
                    
                }
                actual.host_id = None;
                actual.state = VmState::Destroyed;
                actual.cpu = 0;
                actual.memory_mb = 0;
            }
            Action::NoOp => {

            }
        }
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
    cpu: u8,
    memory_mb: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Host {
    id: u64,
    total_cpu: u8,
    used_cpu: u8,
    total_memory_mb: u32,
    used_memory_mb: u32,
    is_alive: bool
}

impl Host {
    fn has_resources_for(
        &self,
        desired: &DesiredVm,
    ) -> bool {
        self.total_cpu - self.used_cpu >= desired.cpu && self.total_memory_mb - self.used_memory_mb >= desired.memory_mb
    }
}