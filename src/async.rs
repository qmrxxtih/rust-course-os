
use alloc::{
    collections::BTreeMap, sync::Arc
};

use core::{
    sync::atomic::{
        AtomicU64, Ordering
    }, 
    task::{
        Context, Poll, Waker
    }
};

use crossbeam_queue::ArrayQueue;


/// Basic structure representing task ID.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct TaskId(u64);


/// Structure representing running task.
pub struct Task {
    id: TaskId,
    future: core::pin::Pin<alloc::boxed::Box<dyn Future<Output = ()>>>
}


/// Structure representing task executor.
pub struct Executor {
    tasks: BTreeMap<TaskId, Task>,
    queue: Arc<ArrayQueue<TaskId>>,
    wakers: BTreeMap<TaskId, Waker>,
}


/// Structure representing task waker - main signal point of task changing state.
struct TaskWaker {
    id: TaskId,
    queue: Arc<ArrayQueue<TaskId>>,
}


#[allow(unused)]
impl Task {
    pub fn new(future: impl Future<Output = ()> + 'static) -> Task {
        Self {
            id: TaskId::new(),
            future: alloc::boxed::Box::pin(future),
        }
    }

    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}


#[allow(unused)]
impl TaskId {
    // Returns task with serial ID (incremented for each task, starting at 1)
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        TaskId (NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}


#[allow(unused)]
impl Executor {
    /// Creates new executor structure.
    pub fn new() -> Self {
        Self {
            tasks: BTreeMap::new(),
            queue: Arc::new(ArrayQueue::new(100)),
            wakers: BTreeMap::new(),
        }
    }

    /// Spawns new asynchronous task.
    pub fn spawn(&mut self, task: Task) {
        let id = task.id;
        if self.tasks.insert(task.id, task).is_some() {
            panic!("task with same ID!");
        }
        self.queue.push(id).expect("task queue is full!");
    }

    /// Runs the executor's execution loop, executing ready tasks.
    pub fn run(&mut self) -> ! {
        loop {
            // run tasks in queue
            self.run_tasks();
            // if the queue is emptied, idle
            self.idle_sleep();
        }
    }

    /// Calls HALT instruction if task queue is empty, freeing up load from the processor.
    fn idle_sleep(&self) {
        // interrupts cannot happen!
        let mut x = false;
        x86_64::instructions::interrupts::without_interrupts(|| {
            if self.queue.is_empty() {
                x = true
            }
        });
        if x {
            x86_64::instructions::hlt();
        }
    }

    /// Execute all function that are ready to be executed.
    fn run_tasks(&mut self) {
        while let Some(id) = self.queue.pop() {
            // get task by its ID
            let task = match self.tasks.get_mut(&id) {
                Some(t) => t,
                None => continue,
            };
            // get (or create) task waker for given task
            let waker = self.wakers.entry(id).or_insert_with(|| TaskWaker::new(id, self.queue.clone()));
            // create context of the waker
            let mut context = Context::from_waker(waker);
            // poll the task
            match task.poll(&mut context) {
                // if the task is ready with result, remove it from queue
                Poll::Ready(()) => {
                    self.tasks.remove(&id);
                    self.wakers.remove(&id);
                },
                // otherwise do nothing
                Poll::Pending => {}
            }
        }
    }
}


impl TaskWaker {
    fn new(id: TaskId, queue: Arc<ArrayQueue<TaskId>>) -> Waker {
        Waker::from(Arc::new(TaskWaker {id, queue}))
    }

    fn wake_task(&self) {
        self.queue.push(self.id).expect("the task queue is full!");
    }
}

impl alloc::task::Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_task();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_task();
    }
}
