pub enum TaskStatus<T> {
    Done(Box<T>),
    Failed,
    Waiting,
}

pub trait GetTask<T> {
    fn get_task_status(&mut self) -> &TaskStatus<T>;
}

impl<T> GetTask<T> for TaskStatus<T> {
    fn get_task_status(&mut self) -> &TaskStatus<T> {
        self
    }
}
