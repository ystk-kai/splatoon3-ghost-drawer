use super::value_objects::*;
use crate::domain::shared::Entity;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProController {
    pub id: String,
    pub controller_type: ControllerType,
    pub current_state: HidReport,
    pub is_connected: bool,
    pub device_path: Option<String>,
}

impl ProController {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            controller_type: ControllerType::ProController,
            current_state: HidReport::new(),
            is_connected: false,
            device_path: None,
        }
    }

    pub fn with_type(mut self, controller_type: ControllerType) -> Self {
        self.controller_type = controller_type;
        self
    }

    pub fn connect(&mut self, device_path: impl Into<String>) {
        self.device_path = Some(device_path.into());
        self.is_connected = true;
    }

    pub fn disconnect(&mut self) {
        self.device_path = None;
        self.is_connected = false;
        self.current_state = HidReport::new();
    }

    pub fn press_button(&mut self, button: Button) {
        self.current_state.buttons.press(button);
    }

    pub fn release_button(&mut self, button: Button) {
        self.current_state.buttons.release(button);
    }

    pub fn set_dpad(&mut self, dpad: DPad) {
        self.current_state.dpad = dpad;
    }

    pub fn move_left_stick(&mut self, position: StickPosition) {
        self.current_state.left_stick = position;
    }

    pub fn move_right_stick(&mut self, position: StickPosition) {
        self.current_state.right_stick = position;
    }

    pub fn reset_state(&mut self) {
        self.current_state = HidReport::new();
    }

    pub fn apply_action(&mut self, action: &ControllerAction) {
        match &action.action_type {
            ActionType::PressButton(button) => self.press_button(*button),
            ActionType::ReleaseButton(button) => self.release_button(*button),
            ActionType::SetDPad(dpad) => self.set_dpad(*dpad),
            ActionType::MoveLeftStick(pos) => self.move_left_stick(*pos),
            ActionType::MoveRightStick(pos) => self.move_right_stick(*pos),
            ActionType::SetReport(report) => self.current_state = *report,
            ActionType::Wait => {} // No state change
        }
    }

    pub fn get_report_bytes(&self) -> [u8; 8] {
        self.current_state.to_bytes()
    }
}

impl Entity for ProController {
    type Id = String;

    fn id(&self) -> &Self::Id {
        &self.id
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ControllerSession {
    pub id: String,
    pub controller_id: String,
    pub command_queue: VecDeque<ControllerCommand>,
    pub current_command: Option<ControllerCommand>,
    pub current_action_index: usize,
    pub started_at: Option<u64>,
    pub is_active: bool,
}

impl ControllerSession {
    pub fn new(id: impl Into<String>, controller_id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            controller_id: controller_id.into(),
            command_queue: VecDeque::new(),
            current_command: None,
            current_action_index: 0,
            started_at: None,
            is_active: false,
        }
    }

    pub fn queue_command(&mut self, command: ControllerCommand) {
        self.command_queue.push_back(command);
    }

    pub fn start(&mut self) {
        self.is_active = true;
        self.started_at = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        );
        self.next_command();
    }

    pub fn stop(&mut self) {
        self.is_active = false;
        self.current_command = None;
        self.current_action_index = 0;
    }

    pub fn pause(&mut self) {
        self.is_active = false;
    }

    pub fn resume(&mut self) {
        self.is_active = true;
    }

    pub fn next_command(&mut self) -> bool {
        if let Some(command) = self.command_queue.pop_front() {
            self.current_command = Some(command);
            self.current_action_index = 0;
            true
        } else {
            self.current_command = None;
            false
        }
    }

    pub fn current_action(&self) -> Option<&ControllerAction> {
        self.current_command
            .as_ref()?
            .sequence
            .get(self.current_action_index)
    }

    pub fn advance_action(&mut self) -> bool {
        if let Some(command) = &self.current_command {
            self.current_action_index += 1;
            if self.current_action_index >= command.sequence.len() {
                self.next_command()
            } else {
                true
            }
        } else {
            false
        }
    }

    pub fn is_completed(&self) -> bool {
        self.current_command.is_none() && self.command_queue.is_empty()
    }

    pub fn remaining_commands(&self) -> usize {
        self.command_queue.len() + if self.current_command.is_some() { 1 } else { 0 }
    }
}

impl Entity for ControllerSession {
    type Id = String;

    fn id(&self) -> &Self::Id {
        &self.id
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ControllerMapping {
    pub id: String,
    pub name: String,
    pub artwork_id: String,
    pub commands: Vec<ControllerCommand>,
    pub created_at: u64,
    pub updated_at: u64,
}

impl ControllerMapping {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        artwork_id: impl Into<String>,
    ) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        Self {
            id: id.into(),
            name: name.into(),
            artwork_id: artwork_id.into(),
            commands: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn add_command(&mut self, command: ControllerCommand) {
        self.commands.push(command);
        self.update_timestamp();
    }

    pub fn remove_command(&mut self, index: usize) -> Option<ControllerCommand> {
        if index < self.commands.len() {
            self.update_timestamp();
            Some(self.commands.remove(index))
        } else {
            None
        }
    }

    pub fn update_timestamp(&mut self) {
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
    }

    pub fn total_duration_ms(&self) -> u32 {
        self.commands.iter().map(|c| c.total_duration_ms()).sum()
    }
}

impl Entity for ControllerMapping {
    type Id = String;

    fn id(&self) -> &Self::Id {
        &self.id
    }
}
