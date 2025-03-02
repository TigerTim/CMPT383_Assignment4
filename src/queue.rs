use std::sync::mpsc;
use std::thread;

use digest::Output;

pub trait Task {
    type Output: Send;
    fn run(&self) -> Option<Self::Output>;  // create output
    // if "run" gives Some output => mpsc channel in main thread, otherwise (gives None output), it should be ignored
}

pub struct WorkQueue<TaskType: 'static + Task + Send> {
    send_tasks: Option<spmc::Sender<TaskType>>, // Option because it will be set to None to close the queue
    // spmc: distribute tasks to workers via 1 producer

    recv_tasks: spmc::Receiver<TaskType>,
    // drain thread pool when queue is being shut down

    //send_output: mpsc::Sender<TaskType::Output>, // not need in the struct: each worker will have its own clone.
    
    recv_output: mpsc::Receiver<TaskType::Output>,
    // mpsc: receive output from many workers

    workers: Vec<thread::JoinHandle<()>>,   // contain JoinHandles of each of the threads doing processing
}

impl<TaskType: 'static + Task + Send> WorkQueue<TaskType> {
    pub fn new(n_workers: usize) -> WorkQueue<TaskType> {
        // TODO: create the channels; start the worker threads; record their JoinHandles
        let (send_tasks, recv_tasks) = spmc::channel();
        let (send_output, recv_output) = mpsc::channel();

        // Create worker threads
        let mut workers = Vec::with_capacity(n_workers);
        for _ in 0..n_workers {
            let recv_tasks = recv_tasks.clone();
            let send_output = send_output.clone();

            let handle = thread::spawn(move || {
                Self::run(recv_tasks, send_output);
            });

            workers.push(handle);
        }

        WorkQueue { 
            send_tasks: Some(send_tasks), 
            recv_tasks,
            recv_output, 
            workers
        }
    }

    fn run(recv_tasks: spmc::Receiver<TaskType>, send_output: mpsc::Sender<TaskType::Output>) {
        // TODO: the main logic for a worker thread
        loop {
            // receive tasks
            let task_result = recv_tasks.recv();
            // NOTE: task_result will be Err() if the spmc::Sender has been destroyed and no more messages can be received here
            match task_result {
                // channel is closed (sender dropped) => end the thread
                Err(_) => {
                    return;
                }

                // run task
                Ok(task) => {
                    // check task result
                    if let Some(output) = task.run() {
                        
                        // case: cannot send
                        if send_output.send(output).is_err() {
                            return;
                        }
                    }
                    // if the outermost if is false => task result is None => do nothing and continue

                    // PATTERN MATCH APPROACH
                    // match task.run() {
                    //     Some(output) => {
                    //         match send_output.send(output) {
                    //             Err(_) => return,
                    //             Ok(_) => continue
                    //         }
                    //     },
                    //     None => continue
                    //     // task result is None => do nothing and continue
                    // }
                }
            }
        }
    }

    pub fn enqueue(&mut self, t: TaskType) -> Result<(), spmc::SendError<TaskType>> {
        // TODO: send this task to a worker
        match self.send_tasks.as_mut() {
            Some(sender) => sender.send(t),     // send modifies sender => sender must be mut => use as.mut()
            None => panic!()
        }
    }

    // Helper methods that let you receive results in various ways
    pub fn iter(&mut self) -> mpsc::Iter<TaskType::Output> {
        self.recv_output.iter()
    }
    pub fn recv(&mut self) -> TaskType::Output {
        self.recv_output
            .recv()
            .expect("I have been shutdown incorrectly")
    }
    pub fn try_recv(&mut self) -> Result<TaskType::Output, mpsc::TryRecvError> {
        self.recv_output.try_recv()
    }
    pub fn recv_timeout(
        &self,
        timeout: std::time::Duration,
    ) -> Result<TaskType::Output, mpsc::RecvTimeoutError> {
        self.recv_output.recv_timeout(timeout)
    }

    pub fn shutdown(&mut self) {
        // TODO: destroy the spmc::Sender so everybody knows no more tasks are incoming;
        // drain any pending tasks in the queue; wait for each worker thread to finish.
        // HINT: Vec.drain(..)
        self.send_tasks = None;     // destroy spmc::Sender => no more tasks can be sent
        // drain remaining task from the queue
        loop {
            match self.recv_tasks.recv() {    
                Ok(_) => (),
                Err(_) => break     // end of queue 
            }
        }

        for handle in self.workers.drain(..) {
            handle.join().unwrap();
        }
    }
}

impl<TaskType: 'static + Task + Send> Drop for WorkQueue<TaskType> {
    fn drop(&mut self) {
        // "Finalisation in destructors" pattern: https://rust-unofficial.github.io/patterns/idioms/dtor-finally.html
        match self.send_tasks {
            None => {} // already shut down
            Some(_) => self.shutdown(),
        }
    }
}
