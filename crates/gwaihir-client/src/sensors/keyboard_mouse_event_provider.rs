use std::time::SystemTime;

pub use rdev::Button as MouseButton;
pub use rdev::ListenError;

pub struct KeyboardMouseEvent {
    pub time: SystemTime,
    pub event_type: KeyboardMouseEventType,
}

#[derive(Debug)]
pub enum KeyboardMouseEventType {
    KeyPress,
    MouseButtonPress(MouseButton),
    MouseMove { x: f64, y: f64 },
    MouseWheel { delta_x: i64, delta_y: i64 },
}

pub trait KeyboardMouseEventProvider {
    fn listen(&self, callback: impl FnMut(KeyboardMouseEvent) + 'static)
        -> Result<(), ListenError>;
}

pub struct RdevKeyboardMouseEventProvider {}

impl KeyboardMouseEventProvider for RdevKeyboardMouseEventProvider {
    fn listen(
        &self,
        callback: impl FnMut(KeyboardMouseEvent) + 'static,
    ) -> Result<(), ListenError> {
        listen(callback)
    }
}

impl RdevKeyboardMouseEventProvider {
    pub fn new() -> Self {
        Self {}
    }
}

fn listen(mut callback: impl FnMut(KeyboardMouseEvent) + 'static) -> Result<(), ListenError> {
    rdev::listen(move |event| {
        if let Some(event) = map_rdev_event(event) {
            callback(event)
        }
    })
}

fn map_rdev_event(event: rdev::Event) -> Option<KeyboardMouseEvent> {
    Some(KeyboardMouseEvent {
        time: event.time,
        event_type: map_rdev_event_type(event.event_type)?,
    })
}

fn map_rdev_event_type(event_type: rdev::EventType) -> Option<KeyboardMouseEventType> {
    match event_type {
        rdev::EventType::KeyPress(_) => Some(KeyboardMouseEventType::KeyPress),
        rdev::EventType::ButtonPress(b) => Some(KeyboardMouseEventType::MouseButtonPress(b)),
        rdev::EventType::MouseMove { x, y } => Some(KeyboardMouseEventType::MouseMove { x, y }),
        rdev::EventType::Wheel { delta_x, delta_y } => {
            Some(KeyboardMouseEventType::MouseWheel { delta_x, delta_y })
        }
        rdev::EventType::KeyRelease(_) | rdev::EventType::ButtonRelease(_) => None,
    }
}
