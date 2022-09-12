use std::{ffi::c_void, thread::ThreadId, time::Duration};

use tokio::{
    runtime::Builder,
    sync::{mpsc, oneshot},
    task::LocalSet,
};

use crate::flutter_bindings::{FlutterEngine, FlutterEngineRunTask, FlutterTask};

use super::{FlutterApplication, SendFlutterTask};

pub(super) struct Task {
    task: SendFlutterTask,
    target_time_nanos: u64,
}

#[derive(Clone, Copy)]
struct SendFlutterEngine(FlutterEngine);

unsafe impl Send for SendFlutterEngine {}

pub(super) struct TaskRunner {
    new_sender: Option<oneshot::Sender<SendFlutterEngine>>,
    sender: mpsc::UnboundedSender<Task>,
    thread_id: ThreadId,
    thread_name: String,
}

impl TaskRunner {
    pub(super) fn new(name: String) -> Self {
        let (new_sender, new_receiver) = oneshot::channel::<SendFlutterEngine>();
        let (sender, mut receiver) = mpsc::unbounded_channel::<Task>();
        let join_handle = std::thread::Builder::new()
            .name(name.clone())
            .spawn(move || {
                let engine = new_receiver.blocking_recv().unwrap();
                let rt = Builder::new_current_thread().enable_time().build().unwrap();
                let local = LocalSet::new();
                local.block_on(&rt, async move {
                    log::debug!("Waiting for tasks on {:?}", std::thread::current().name());
                    while let Some(Task {
                        task,
                        target_time_nanos,
                    }) = receiver.recv().await
                    {
                        let now = FlutterApplication::current_time();
                        if now >= target_time_nanos {
                            FlutterApplication::unwrap_result(unsafe {
                                FlutterEngineRunTask(engine.0, &task.0)
                            });
                        } else {
                            tokio::task::spawn_local(async move {
                                tokio::time::sleep(Duration::from_nanos(target_time_nanos - now))
                                    .await;
                                FlutterApplication::unwrap_result(unsafe {
                                    FlutterEngineRunTask(engine.0, &task.0)
                                });
                            });
                        }
                    }
                    log::debug!(
                        "Done receiving tasks on {:?}",
                        std::thread::current().name()
                    );
                });
            })
            .unwrap();

        Self {
            new_sender: Some(new_sender),
            sender,
            thread_id: join_handle.thread().id(),
            thread_name: name,
        }
    }

    pub(super) fn run(&mut self, engine: FlutterEngine) {
        let engine = SendFlutterEngine(engine);
        if let Some(sender) = self.new_sender.take() {
            sender.send(engine).ok().unwrap();
        }
    }

    pub(super) extern "C" fn runs_task_on_current_thread_callback(user_data: *mut c_void) -> bool {
        let this = unsafe { &*(user_data as *const Self) as &Self };
        this.thread_id == std::thread::current().id()
    }

    pub(super) extern "C" fn post_task_callback(
        task: FlutterTask,
        target_time_nanos: u64,
        user_data: *mut c_void,
    ) {
        let task = SendFlutterTask(task);
        let this = unsafe { &*(user_data as *const Self) as &Self };
        this.sender
            .send(Task {
                task,
                target_time_nanos,
            })
            .ok()
            .unwrap();
    }
}
