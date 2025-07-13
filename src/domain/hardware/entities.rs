use super::value_objects::*;
use crate::domain::shared::Entity;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Board {
    pub model: BoardModel,
    pub kernel_modules: Vec<KernelModule>,
    pub usb_otg_available: bool,
}

impl Board {
    pub fn new(model: BoardModel) -> Self {
        Self {
            model,
            kernel_modules: vec![KernelModule::dwc2(), KernelModule::libcomposite()],
            usb_otg_available: false,
        }
    }

    pub fn with_kernel_modules(mut self, modules: Vec<KernelModule>) -> Self {
        self.kernel_modules = modules;
        self
    }

    pub fn with_usb_otg_status(mut self, available: bool) -> Self {
        self.usb_otg_available = available;
        self
    }

    pub fn can_setup_gadget(&self) -> bool {
        self.model.supports_usb_otg() && self.usb_otg_available
    }

    pub fn required_modules(&self) -> Vec<&KernelModule> {
        self.kernel_modules.iter().filter(|m| !m.loaded).collect()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UsbGadget {
    pub id: String,
    pub state: UsbGadgetState,
    pub descriptor: UsbDeviceDescriptor,
    pub gadget_path: String,
    pub udc_name: Option<String>,
}

impl UsbGadget {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            state: UsbGadgetState::NotConfigured,
            descriptor: UsbDeviceDescriptor::default(),
            gadget_path: "/sys/kernel/config/usb_gadget".to_string(),
            udc_name: None,
        }
    }

    pub fn nintendo_controller() -> Self {
        Self::new("nintendo_controller")
    }

    pub fn with_descriptor(mut self, descriptor: UsbDeviceDescriptor) -> Self {
        self.descriptor = descriptor;
        self
    }

    pub fn with_state(mut self, state: UsbGadgetState) -> Self {
        self.state = state;
        self
    }

    pub fn is_active(&self) -> bool {
        self.state == UsbGadgetState::Active
    }

    pub fn full_path(&self) -> String {
        format!("{}/{}", self.gadget_path, self.id)
    }
}

impl Entity for UsbGadget {
    type Id = String;

    fn id(&self) -> &Self::Id {
        &self.id
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SystemdService {
    pub name: String,
    pub state: SystemdServiceState,
    pub unit_file_path: String,
    pub exec_start: String,
}

impl SystemdService {
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            name: name.clone(),
            state: SystemdServiceState::NotInstalled,
            unit_file_path: format!("/etc/systemd/system/{name}.service"),
            exec_start: String::new(),
        }
    }

    pub fn nintendo_controller_service() -> Self {
        Self::new("nintendo-controller")
            .with_exec_start("/usr/local/bin/setup-nintendo-controller.sh")
    }

    pub fn with_exec_start(mut self, exec_start: impl Into<String>) -> Self {
        self.exec_start = exec_start.into();
        self
    }

    pub fn with_state(mut self, state: SystemdServiceState) -> Self {
        self.state = state;
        self
    }

    pub fn is_running(&self) -> bool {
        self.state == SystemdServiceState::Running
    }

    pub fn is_enabled(&self) -> bool {
        matches!(
            self.state,
            SystemdServiceState::Enabled | SystemdServiceState::Running
        )
    }
}

impl Entity for SystemdService {
    type Id = String;

    fn id(&self) -> &Self::Id {
        &self.name
    }
}
