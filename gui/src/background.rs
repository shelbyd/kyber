use std::sync::mpsc::*;

pub enum BackgroundJob<T> {
    Finished(T),
    Waiting(Receiver<T>),
}

impl<T> BackgroundJob<T>
where
    T: Send + 'static,
{
    pub fn run(job: impl FnOnce() -> T + Send + 'static) -> Self {
        let (tx, rx) = channel();

        std::thread::spawn(move || {
            if let Err(e) = tx.send(job()) {
                log::error!("{}", e);
            }
        });

        BackgroundJob::Waiting(rx)
    }

    pub fn value(&mut self) -> Option<&T> {
        if let BackgroundJob::Waiting(rx) = self {
            if let Ok(t) = rx.try_recv() {
                *self = BackgroundJob::Finished(t);
            }
        }

        match self {
            BackgroundJob::Finished(t) => Some(t),
            BackgroundJob::Waiting(_) => None,
        }
    }
}

pub struct IncrementalLoad<T> {
    rx: Receiver<T>,
    values: Vec<T>,
}

impl<T: Send + 'static> IncrementalLoad<T> {
    pub fn new<I: IntoIterator<Item = T>, F: FnOnce() -> I + Send + 'static>(loader: F) -> Self {
        let (tx, rx) = channel();

        std::thread::spawn(move || {
            for t in loader().into_iter() {
                if let Err(e) = tx.send(t) {
                    log::error!("{}", e);
                }
            }
        });

        Self {
            rx,
            values: Vec::new(),
        }
    }

    pub fn current(&mut self) -> &[T] {
        while let Ok(t) = self.rx.try_recv() {
            self.values.push(t);
        }
        self.values.as_ref()
    }
}
