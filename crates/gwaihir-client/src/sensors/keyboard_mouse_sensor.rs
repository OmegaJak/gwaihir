use super::{
    keyboard_mouse_event_provider::{
        KeyboardMouseEvent, KeyboardMouseEventProvider, KeyboardMouseEventType,
    },
    outputs::{keyboard_mouse_activity::KeyboardMouseActivity, sensor_output::SensorOutput},
    Sensor,
};
use log::{error, warn};
use simple_moving_average::{NoSumSMA, SMA};
use std::{
    sync::mpsc::{channel, Receiver},
    thread::{self, JoinHandle},
    time::{Duration, SystemTime},
};

const BUCKET_DURATION: Duration = Duration::from_secs(1);

pub struct KeyboardMouseSensor {
    event_rx: Receiver<KeyboardMouseEvent>,
    keyboard_average: CalculatorGuy,
    mouse_movement_average: CalculatorGuy,
    mouse_button_average: CalculatorGuy,

    _listener_thread_handle: JoinHandle<()>,
}

impl Sensor for KeyboardMouseSensor {
    fn get_output(&mut self) -> super::outputs::sensor_output::SensorOutput {
        for event in self.event_rx.try_iter() {
            match event.event_type {
                KeyboardMouseEventType::KeyPress => self.keyboard_average.handle_event(event),
                KeyboardMouseEventType::MouseButtonPress(_) => {
                    self.mouse_button_average.handle_event(event)
                }
                KeyboardMouseEventType::MouseMove { x: _, y: _ } => {
                    self.mouse_movement_average.handle_event(event)
                }
                KeyboardMouseEventType::MouseWheel {
                    delta_x: _,
                    delta_y: _,
                } => self.mouse_button_average.handle_event(event),
            }
        }

        let now = SystemTime::now();
        self.mouse_button_average.update_time(now);
        self.mouse_movement_average.update_time(now);
        self.keyboard_average.update_time(now);

        SensorOutput::KeyboardMouseActivity(KeyboardMouseActivity {
            keyboard_usage: self.keyboard_average.average(),
            mouse_movement: self.mouse_movement_average.average(),
            mouse_button_usage: self.mouse_button_average.average(),
        })
    }
}

impl KeyboardMouseSensor {
    pub fn new(event_provider: impl KeyboardMouseEventProvider + Send + 'static) -> Self {
        let (tx, rx) = channel();
        let listener_handle = thread::spawn(move || {
            event_provider
                .listen(move |event| {
                    tx.send(event)
                        .unwrap_or_else(|e| error!("Failed to send keyboard/mouse event: {:?}", e))
                })
                .unwrap()
        });

        let now = SystemTime::now();
        Self {
            event_rx: rx,
            keyboard_average: CalculatorGuy::new(now.clone()),
            mouse_movement_average: CalculatorGuy::new(now.clone()),
            mouse_button_average: CalculatorGuy::new(now),

            _listener_thread_handle: listener_handle,
        }
    }
}

struct CalculatorGuy {
    current_bucket_start: SystemTime,
    current_bucket_end: SystemTime,

    last_event: Option<KeyboardMouseEvent>,

    current_bucket_value: f64,
    moving_average: NoSumSMA<f64, f64, 10>,
}

impl CalculatorGuy {
    pub fn new(now: SystemTime) -> Self {
        Self {
            current_bucket_start: now,
            current_bucket_end: now + BUCKET_DURATION,

            last_event: None,

            current_bucket_value: 0.0,
            moving_average: NoSumSMA::new(),
        }
    }

    pub fn average(&self) -> f64 {
        self.moving_average.get_average()
    }

    pub fn update_time(&mut self, now: SystemTime) {
        while now > self.current_bucket_end {
            self.end_bucket();
        }
    }

    pub fn handle_event(&mut self, event: KeyboardMouseEvent) {
        if event.time < self.current_bucket_start {
            warn!("Discarding event that was before the start of the current bucket");
            return;
        }

        while event.time >= self.current_bucket_end {
            self.end_bucket();
        }

        self.current_bucket_value += event
            .event_type
            .quantify(self.last_event.take().map(|e| e.event_type));
        self.last_event = Some(event);
    }

    fn end_bucket(&mut self) {
        self.moving_average.add_sample(self.current_bucket_value);
        self.current_bucket_value = 0.0;
        (self.current_bucket_start, self.current_bucket_end) = (
            self.current_bucket_end,
            self.current_bucket_end + BUCKET_DURATION,
        )
    }
}

trait Quantifiable {
    fn quantify(&self, last_value: Option<Self>) -> f64
    where
        Self: Sized;
}

impl Quantifiable for KeyboardMouseEventType {
    fn quantify(&self, last_value: Option<Self>) -> f64
    where
        Self: Sized,
    {
        use KeyboardMouseEventType as EventType;

        match (self, last_value) {
            (EventType::KeyPress, _) => 1.0,
            (EventType::MouseButtonPress(_), _) => 1.0,
            (EventType::MouseWheel { delta_x, delta_y }, _) => {
                (delta_x.abs() + delta_y.abs()) as f64
            }
            (
                EventType::MouseMove { x, y },
                Some(EventType::MouseMove {
                    x: last_x,
                    y: last_y,
                }),
            ) => f64::sqrt((x - last_x).powi(2) + (y - last_y).powi(2)),
            (EventType::MouseMove { x: _, y: _ }, None) => 0.0,
            (EventType::MouseMove { x: _, y: _ }, _) => {
                panic!("Somehow a mouse move was compared to something else, this shouldn't be possible");
            }
        }
    }
}
