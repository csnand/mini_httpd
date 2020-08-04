use std::thread;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;

type Job = Box<dyn FnOnce() + Send + 'static>;
enum Message {
    NewJob(Job),
    Terminate,
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);
        for i in 0..size {
            workers.push(Worker::new(i,Arc::clone(&receiver)));
        }
        ThreadPool { workers, sender }
    }

    pub fn execute<F> (&self, f: F)
    where
        F: FnOnce() + Send + 'static, {
        let job = Box::new(f);
        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("Sending terminate message to all workers.");
        for _ in &mut self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }
        println!("Shutting down all workers");
        for worker in &mut self.workers {
            println!("shutting down worker {}", worker.id);
            //take() -> take out Some() replace with None
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}



struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        Worker {id: id, thread: Some(
            thread::spawn(move|| loop {
                let message = receiver.lock().unwrap().recv().unwrap();
                match message {
                    Message::NewJob(job) => {
                        println!("Worker {} got a job, executing.", id);
                        job();
                    }
                    Message::Terminate => {
                        println!("Worker {} was told to terminate", id);
                        break;
                    }
                }
            })
        )}
    }
}
