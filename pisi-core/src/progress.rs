use std::sync::Arc;

pub trait ProgressReporter: Send + Sync {
    fn on_progress(&self, percent: f32, speed: Option<String>, eta: Option<String>);
    fn on_message(&self, msg: &str);
    fn on_finish(&self, msg: &str);
}

impl<F1, F2, F3> ProgressReporter for (F1, F2, F3)
where
    F1: Fn(f32, Option<String>, Option<String>) + Send + Sync,
    F2: Fn(&str) + Send + Sync,
    F3: Fn(&str) + Send + Sync,
{
    fn on_progress(&self, percent: f32, speed: Option<String>, eta: Option<String>) {
        (self.0)(percent, speed, eta);
    }
    fn on_message(&self, msg: &str) {
        (self.1)(msg);
    }
    fn on_finish(&self, msg: &str) {
        (self.2)(msg);
    }
}

pub struct NullReporter;
impl ProgressReporter for NullReporter {
    fn on_progress(&self, _percent: f32, _speed: Option<String>, _eta: Option<String>) {}
    fn on_message(&self, _msg: &str) {}
    fn on_finish(&self, _msg: &str) {}
}

pub struct TeeReporter {
    reporters: Arc<Vec<Box<dyn ProgressReporter>>>,
}

impl TeeReporter {
    pub fn new(reporters: Vec<Box<dyn ProgressReporter>>) -> Self {
        TeeReporter {
            reporters: Arc::new(reporters),
        }
    }
}

impl ProgressReporter for TeeReporter {
    fn on_progress(&self, percent: f32, speed: Option<String>, eta: Option<String>) {
        for r in self.reporters.iter() {
            r.on_progress(percent, speed.clone(), eta.clone());
        }
    }
    fn on_message(&self, msg: &str) {
        for r in self.reporters.iter() {
            r.on_message(msg);
        }
    }
    fn on_finish(&self, msg: &str) {
        for r in self.reporters.iter() {
            r.on_finish(msg);
        }
    }
}
