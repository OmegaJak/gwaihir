use super::{
    keyboard_mouse_event_provider::{
        KeyboardMouseEvent, KeyboardMouseEventProvider, KeyboardMouseEventType,
    },
    outputs::{
        keyboard_mouse_activity::{KeyboardMouseActivity, KeyboardMouseActivityData},
        sensor_output::SensorOutput,
    },
    Sensor,
};
use bounded_vec_deque::BoundedVecDeque;
use log::{error, info, warn};
use std::{
    sync::mpsc::{channel, Receiver, Sender},
    thread::{self, JoinHandle},
    time::{Duration, SystemTime},
};

pub const BUCKET_DURATION_SECONDS: u64 = 10;
const BUCKET_DURATION: Duration = Duration::from_secs(BUCKET_DURATION_SECONDS);
const NUM_BUCKETS_TO_KEEP: usize = 30;

pub struct KeyboardMouseSensor {
    rx_from_listener: Receiver<KeyboardMouseEvent>,
    keyboard_quantifier: KeyboardMouseEventHistoricalQuantifier,
    mouse_movement_quantifier: KeyboardMouseEventHistoricalQuantifier,
    mouse_button_quantifier: KeyboardMouseEventHistoricalQuantifier,

    _listener_thread_handle: JoinHandle<()>,
}

pub struct ShutdownMessage {}

impl Sensor for KeyboardMouseSensor {
    fn get_output(&mut self) -> super::outputs::sensor_output::SensorOutput {
        for event in self.rx_from_listener.try_iter() {
            match event.event_type {
                KeyboardMouseEventType::KeyPress => self.keyboard_quantifier.handle_event(event),
                KeyboardMouseEventType::MouseButtonPress(_) => {
                    self.mouse_button_quantifier.handle_event(event)
                }
                KeyboardMouseEventType::MouseMove { x: _, y: _ } => {
                    self.mouse_movement_quantifier.handle_event(event)
                }
                KeyboardMouseEventType::MouseWheel {
                    delta_x: _,
                    delta_y: _,
                } => self.mouse_button_quantifier.handle_event(event),
            }
        }

        let now = SystemTime::now();
        self.mouse_button_quantifier.update_time(now);
        self.mouse_movement_quantifier.update_time(now);
        self.keyboard_quantifier.update_time(now);

        SensorOutput::KeyboardMouseActivity(KeyboardMouseActivity {
            keyboard_usage: self.keyboard_quantifier.sensor_data(),
            mouse_movement: self.mouse_movement_quantifier.sensor_data(),
            mouse_button_usage: self.mouse_button_quantifier.sensor_data(),
        })
    }
}

impl KeyboardMouseSensor {
    pub fn new(
        event_provider: impl KeyboardMouseEventProvider + Send + 'static,
    ) -> (Self, Sender<ShutdownMessage>) {
        let (tx_to_main, rx_from_listener) = channel();
        let (tx_to_listener, rx_from_main) = channel();
        let listener_handle = thread::spawn(move || {
            let mut has_shutdown = false;
            event_provider
                .listen(move |event| {
                    if let Ok(_msg) = rx_from_main.try_recv() {
                        info!("Keyboard/mouse listener received shutdown");
                        has_shutdown = true;
                    }

                    if !has_shutdown {
                        tx_to_main.send(event)
                        .expect(&format!("Failed to send keyboard/mouse event from listener thread, the other end must have hung up"));
                    }
                })
                .unwrap()
        });

        let now = SystemTime::now();
        (
            Self {
                rx_from_listener,
                keyboard_quantifier: KeyboardMouseEventHistoricalQuantifier::new(now.clone()),
                mouse_movement_quantifier: KeyboardMouseEventHistoricalQuantifier::new(now.clone()),
                mouse_button_quantifier: KeyboardMouseEventHistoricalQuantifier::new(now),

                _listener_thread_handle: listener_handle,
            },
            tx_to_listener,
        )
    }
}

struct KeyboardMouseEventHistoricalQuantifier {
    current_bucket_start: SystemTime,
    current_bucket_end: SystemTime,

    last_event: Option<KeyboardMouseEvent>,

    current_bucket_value: f64,
    past_values: BoundedVecDeque<f64>,
}

impl KeyboardMouseEventHistoricalQuantifier {
    pub fn new(now: SystemTime) -> Self {
        Self {
            current_bucket_start: now,
            current_bucket_end: now + BUCKET_DURATION,

            last_event: None,

            current_bucket_value: 0.0,
            past_values: BoundedVecDeque::new(NUM_BUCKETS_TO_KEEP),
        }
    }

    pub fn sensor_data(&mut self) -> KeyboardMouseActivityData {
        return KeyboardMouseActivityData {
            data: self.past_values.iter().rev().cloned().collect(),
        };
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
        self.past_values.push_front(self.current_bucket_value);
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
            ) => f64::sqrt((x - last_x).powi(2) + (y - last_y).powi(2)) / 100.0,
            (EventType::MouseMove { x: _, y: _ }, None) => 0.0,
            (EventType::MouseMove { x: _, y: _ }, _) => {
                panic!("Somehow a mouse move was compared to something else, this shouldn't be possible");
            }
        }
    }
}
