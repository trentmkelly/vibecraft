#![allow(dead_code)]

pub struct TickTask<F>
where
    F: FnMut(),
{
    tick: i32,
    runnable: F,
}

impl<F> TickTask<F>
where
    F: FnMut(),
{
    pub fn new(tick: i32, runnable: F) -> Self {
        Self { tick, runnable }
    }

    pub fn get_tick(&self) -> i32 {
        self.tick
    }

    pub fn run(&mut self) {
        (self.runnable)();
    }
}

#[cfg(test)]
mod tests {
    use super::TickTask;
    use std::cell::Cell;

    const TICK_TASK_JAVA: &str = vibecraft_java_source!("/net/minecraft/server/TickTask.java");

    #[test]
    fn tick_task_matches_java_runnable_surface() {
        for sentinel in [
            "public class TickTask implements Runnable",
            "private final int tick;",
            "private final Runnable runnable;",
            "public TickTask(final int tick, final Runnable runnable)",
            "this.tick = tick;",
            "this.runnable = runnable;",
            "public int getTick()",
            "return this.tick;",
            "public void run()",
            "this.runnable.run();",
        ] {
            assert!(
                TICK_TASK_JAVA.contains(sentinel),
                "missing TickTask.java sentinel {sentinel}"
            );
        }

        let calls = Cell::new(0);
        let mut task = TickTask::new(42, || calls.set(calls.get() + 1));

        assert_eq!(task.get_tick(), 42);
        assert_eq!(calls.get(), 0);

        task.run();
        assert_eq!(calls.get(), 1);

        task.run();
        assert_eq!(calls.get(), 2);
        assert_eq!(task.get_tick(), 42);
    }
}
