use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    sync::mpsc::{sync_channel, Receiver, SyncSender},
    task::{Context, Poll, Waker},
    thread,
    time::Duration
};

use futures::{
    future::{BoxFuture, FutureExt},
    task::{waker_ref, ArcWake}
};
use std::result::Result::Ok;
use std::option::Option::Some;


pub struct TimerFuture {
    shared_state: Arc<Mutex<SharedState>>
}

struct SharedState {
    completed: bool,
    waker: Option<Waker>
}
// 为 TimerFuture 实现Future
impl Future for TimerFuture {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut shared_state = self.shared_state.lock().unwrap();
        if shared_state.completed {
            Poll::Ready(())
        } else {
            shared_state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

impl TimerFuture {
    pub fn new(duration: Duration) -> Self {
        let shared_state = Arc::new(Mutex::new(SharedState {
            completed: false,
            waker: None
        }));

        let thread_shared_state = shared_state.clone();

        thread::spawn(move || {
            thread::sleep(duration);
            println!("done");
            let mut shared_state = thread_shared_state.lock().unwrap();
            shared_state.completed = true;
            if let Some(waker) = shared_state.waker.take() {
                waker.wake();
            }
        });

        TimerFuture { shared_state }
    }
}

// 自己实现executor

pub struct Executor {
    // receiver 队列
    ready_queue: Receiver<Arc<Task>>
}

pub struct Spawner {
    // sender 只有一个
    task_sender: SyncSender<Arc<Task>>
}

struct Task {
    feature: Mutex<Option<BoxFuture<'static, ()>>>,
    task_sender: SyncSender<Arc<Task>>
}

pub fn new_executor_and_spawner() -> (Executor, Spawner) {
    const MAX_QUEUE_TASKS:usize = 10000;
    let (task_sender, ready_queue) = sync_channel(MAX_QUEUE_TASKS);

    (Executor{ ready_queue }, Spawner{ task_sender })
}

impl Spawner {
    pub fn spawn(&self, future: impl Future<Output = ()> + 'static + Send) {
        let future_boxed = future.boxed();
        let task = Arc::new(Task {
            feature: Mutex::new(Some(future_boxed)),
            task_sender: self.task_sender.clone()
        });
        self.task_sender.send(task).expect("任务队列已满");
    }
}

impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        let cloned = arc_self.clone();
        arc_self
            .task_sender
            .send(cloned)
            .expect("任务队列已满");
    }
}

impl Executor {
    pub fn run(&self) {
        while let Ok(task) = self.ready_queue.recv(){
            let mut feature_slot = task.feature.lock().unwrap();
            if let Some(mut future) = feature_slot.take(){
                let waker = waker_ref(&task);
                let context = &mut Context::from_waker(&*waker);
                if future.as_mut().poll(context).is_pending() {
                    *feature_slot = Some(future);
                }
            }
        }
    }
}



