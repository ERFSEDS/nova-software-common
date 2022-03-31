#![allow(clippy::new_without_default)]

use std::time::SystemTime;

use novafc_config_format::{CommandValue, Value};

pub struct Controls {
    pyro1: ControlObject,
    pyro2: ControlObject,
    pyro3: ControlObject,
    beacon: ControlObject,
    data_rate: ControlObject,
}

impl Controls {
    pub fn new() -> Self {
        let pyro1 = ControlObject::Dummy(Dummy::new("Pyro1".to_string()));
        let pyro2 = ControlObject::Dummy(Dummy::new("Pyro2".to_string()));
        let pyro3 = ControlObject::Dummy(Dummy::new("Pyro3".to_string()));

        let beacon = ControlObject::Dummy(Dummy::new("Beacon".to_string()));
        let data_rate = ControlObject::Dummy(Dummy::new("DataRate".to_string()));

        Self {
            pyro1,
            pyro2,
            pyro3,
            beacon,
            data_rate,
        }
    }

    pub fn set(&mut self, object: CommandValue) {
        let state = object.to_value();
        let object = match object {
            CommandValue::Pyro1(_) => &mut self.pyro1,
            CommandValue::Pyro2(_) => &mut self.pyro2,
            CommandValue::Pyro3(_) => &mut self.pyro3,
            CommandValue::Beacon(_) => &mut self.beacon,
            CommandValue::DataRate(_) => &mut self.data_rate,
        };

        object.set(state);
    }
}

enum ControlObject {
    Dummy(Dummy),
}

impl ControlObject {
    pub fn set(&mut self, state: Value) {
        match self {
            ControlObject::Dummy(d) => d.set(state),
        }
    }
}

// This is for debugging purposes only!!!
struct Dummy {
    name: String,
    start: SystemTime,
}

impl Dummy {
    pub fn new(name: String) -> Self {
        Self {
            name,
            start: SystemTime::now(),
        }
    }

    pub fn set(&mut self, state: Value) {
        println!(
            "[{}s] {} was set to value: {:?}",
            self.start.elapsed().unwrap().as_secs_f32(),
            self.name,
            state
        );
    }
}
