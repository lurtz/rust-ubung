use std::sync::{Arc, Mutex};

use tokio::sync::watch as Channel_type;

pub struct LeSharedState {
    counter: usize,
    x: usize,
    y: usize,
    sender: Channel_type::Sender<String>,
}

impl Default for LeSharedState {
    fn default() -> Self {
        let sender = Channel_type::Sender::<String>::new("".to_string());
        Self {
            counter: Default::default(),
            x: Default::default(),
            y: Default::default(),
            sender,
        }
    }
}

fn exchange(current: &mut usize, new: &usize) -> usize {
    let old_current = *current;
    *current = *new;
    old_current
}

impl LeSharedState {
    pub fn inc_counter(&mut self) -> usize {
        self.counter += 1;
        self.counter
    }

    pub fn set_x(&mut self, x: usize) -> usize {
        exchange(&mut self.x, &x)
    }

    pub fn set_y(&mut self, y: usize) -> usize {
        exchange(&mut self.y, &y)
    }

    pub fn get_z(&self) -> usize {
        self.x + self.y
    }

    pub fn send_event(&self, event: &str) -> Result<(), Channel_type::error::SendError<String>> {
        self.sender.send(event.to_string())
    }

    pub fn get_event_update_receiver(&self) -> Channel_type::Receiver<String> {
        self.sender.subscribe()
    }
}

#[derive(Clone, Default)]
pub struct State {
    state: Arc<Mutex<LeSharedState>>,
}

impl State {
    pub fn inc_counter(&mut self) -> usize {
        l(&self.state).inc_counter()
    }

    pub fn set_x(&mut self, x: usize) -> usize {
        l(&self.state).set_x(x)
    }

    pub fn set_y(&mut self, y: usize) -> usize {
        l(&self.state).set_y(y)
    }

    pub fn get_z(&self) -> usize {
        l(&self.state).get_z()
    }

    pub fn send_event(&self, event: &str) -> Result<(), Channel_type::error::SendError<String>> {
        l(&self.state).send_event(event)
    }

    pub fn get_event_update_receiver(&self) -> Channel_type::Receiver<String> {
        l(&self.state).get_event_update_receiver()
    }
}

fn l(state: &Arc<Mutex<LeSharedState>>) -> std::sync::MutexGuard<'_, LeSharedState> {
    state.lock().unwrap()
}
