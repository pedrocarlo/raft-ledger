pub trait Executor {
    async fn tick(&mut self);
}
