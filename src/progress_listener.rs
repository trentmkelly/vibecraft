//! Progress callback contract matching Minecraft's `ProgressListener`.

#![allow(dead_code)]

pub trait ProgressListener<C> {
    fn progress_start_no_abort(&mut self, message: C);
    fn progress_start(&mut self, message: C);
    fn progress_stage(&mut self, message: C);
    fn progress_stage_percentage(&mut self, percentage: i32);
    fn stop(&mut self);
}

#[cfg(test)]
mod tests {
    use super::ProgressListener;

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn progress_listener_matches_java_source() {
        const JAVA: &str = vibecraft_java_source!("/net/minecraft/util/ProgressListener.java");
        assert_eq!(JAVA.lines().count(), 15);
        for fragment in [
            "void progressStartNoAbort(Component string)",
            "void progressStart(Component string)",
            "void progressStage(Component string)",
            "void progressStagePercentage(int i)",
            "void stop()",
        ] {
            assert!(JAVA.contains(fragment), "missing ProgressListener source fragment: {fragment}");
        }
    }

    struct RecordingListener {
        events: Vec<String>,
    }

    impl ProgressListener<String> for RecordingListener {
        fn progress_start_no_abort(&mut self, message: String) {
            self.events.push(format!("no_abort:{message}"));
        }

        fn progress_start(&mut self, message: String) {
            self.events.push(format!("start:{message}"));
        }

        fn progress_stage(&mut self, message: String) {
            self.events.push(format!("stage:{message}"));
        }

        fn progress_stage_percentage(&mut self, percentage: i32) {
            self.events.push(format!("percent:{percentage}"));
        }

        fn stop(&mut self) {
            self.events.push("stop".to_string());
        }
    }

    #[test]
    fn progress_listener_preserves_callback_order_and_payloads() {
        let mut listener = RecordingListener { events: Vec::new() };
        listener.progress_start_no_abort("loading".to_string());
        listener.progress_start("world".to_string());
        listener.progress_stage("terrain".to_string());
        listener.progress_stage_percentage(42);
        listener.stop();
        assert_eq!(
            listener.events,
            [
                "no_abort:loading",
                "start:world",
                "stage:terrain",
                "percent:42",
                "stop"
            ]
        );
    }
}
