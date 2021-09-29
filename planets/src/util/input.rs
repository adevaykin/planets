use std::rc::Rc;
use std::cell::RefCell;

use bitflags::bitflags;

use winit::event::VirtualKeyCode;

pub type InputControllerMutRef = Rc<RefCell<InputController>>;

bitflags! {
    pub struct Key: u32 {
        const NONE =  0b00000000;
        const UP =    0b00000001;
        const DOWN =  0b00000010;
        const LEFT =  0b00000100;
        const RIGHT = 0b00010000;
        const SPACE = 0b00100000;
        const ENTER = 0b01000000;
    }
}

impl Key {
    pub fn from_vitrual_key_code(code: VirtualKeyCode) -> Key {
        match code {
            VirtualKeyCode::Up => Key::UP,
            VirtualKeyCode::Down => Key::DOWN,
            VirtualKeyCode::Left => Key::LEFT,
            VirtualKeyCode::Right => Key::RIGHT,
            VirtualKeyCode::Space => Key::SPACE,
            VirtualKeyCode::Return => Key::ENTER,
            _ => Key::NONE
        }
    }
}

pub struct InputController {
    pressed: Key,
}

impl InputController {
    pub fn new() -> InputController {
        InputController { pressed: Key::NONE }
    }

    pub fn is_pressed(&self, key: Key) -> bool {
        (self.pressed & key) != Key::NONE
    }

    pub fn set_pressed(&mut self, key: Key) {
        self.pressed |= key;
    }

    pub fn set_released(&mut self, key: Key) {
        self.pressed = self.pressed & !(key);
    }
}

#[cfg(test)]
mod tests {
    use super::Key;
    use super::InputController;

    #[test]
    fn set_pressed() {
        let mut controller = InputController::new();

        controller.set_pressed(Key::UP);
        assert_eq!(controller.is_pressed(Key::UP), true);

        controller.set_pressed(Key::LEFT);
        assert_eq!(controller.is_pressed(Key::UP), true);
        assert_eq!(controller.is_pressed(Key::LEFT), true);
    }

    #[test]
    fn set_released() {
        let mut controller = InputController::new();

        controller.set_pressed(Key::UP);
        controller.set_pressed(Key::LEFT);

        controller.set_released(Key::UP);
        assert_eq!(controller.is_pressed(Key::UP), false);
        assert_eq!(controller.is_pressed(Key::LEFT), true);

        controller.set_released(Key::LEFT);
        assert_eq!(controller.is_pressed(Key::LEFT), false);
    }
}