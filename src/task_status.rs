pub enum TaskStatus<T> {
    Done(Box<T>),
    Failed,
    Waiting(std::time::Instant),
}