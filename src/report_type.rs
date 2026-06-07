#![allow(dead_code)]

use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReportTypeModel {
    pub header: &'static str,
    pub nuggets: &'static [&'static str],
}

pub const REPORT_TYPE_CRASH: ReportTypeModel = ReportTypeModel {
    header: "Minecraft Crash Report",
    nuggets: &[
        "Who set us up the TNT?",
        "Everything's going to plan. No, really, that was supposed to happen.",
        "Uh... Did I do that?",
        "Oops.",
        "Why did you do that?",
        "I feel sad now :(",
        "My bad.",
        "I'm sorry, Dave.",
        "I let you down. Sorry :(",
        "On the bright side, I bought you a teddy bear!",
        "Daisy, daisy...",
        "Oh - I know what I did wrong!",
        "Hey, that tickles! Hehehe!",
        "I blame Dinnerbone.",
        "You should try our sister game, Minceraft!",
        "Don't be sad. I'll do better next time, I promise!",
        "Don't be sad, have a hug! <3",
        "I just don't know what went wrong :(",
        "Shall we play a game?",
        "Quite honestly, I wouldn't worry myself about that.",
        "I bet Cylons wouldn't have this problem.",
        "Sorry :(",
        "Surprise! Haha. Well, this is awkward.",
        "Would you like a cupcake?",
        "Hi. I'm Minecraft, and I'm a crashaholic.",
        "Ooh. Shiny.",
        "This doesn't make any sense!",
        "Why is it breaking :(",
        "Don't do that.",
        "Ouch. That hurt :(",
        "You're mean.",
        "This is a token for 1 free hug. Redeem at your nearest Mojangsta: [~~HUG~~]",
        "There are four lights!",
        "But it works on my machine.",
    ],
};

pub const REPORT_TYPE_PROFILE: ReportTypeModel = ReportTypeModel {
    header: "Minecraft Profiler Results",
    nuggets: &[
        "I'd Rather Be Surfing",
        "Shiny numbers!",
        "Am I not running fast enough? :(",
        "I'm working as hard as I can!",
        "Will I ever be good enough for you? :(",
        "Speedy. Zoooooom!",
        "Hello world",
        "40% better than a crash report.",
        "Now with extra numbers",
        "Now with less numbers",
        "Now with the same numbers",
        "You should add flames to things, it makes them go faster!",
        "Do you feel the need for... optimization?",
        "*cracks redstone whip*",
        "Maybe if you treated it better then it'll have more motivation to work faster! Poor server.",
    ],
};

pub const REPORT_TYPE_TEST: ReportTypeModel = ReportTypeModel {
    header: "Minecraft Test Report",
    nuggets: &[
        "Don't mind me",
        "One day I will be a real crash!",
        "Booo! Haha, did I scare you?",
        "Help, I'm trapped in a report factory!",
        "Have I answered your question?",
        "No hugs here, sorry",
        "I Can't Believe It's Not A Crash Report!",
        "Where's the kaboom!?",
    ],
};

pub const REPORT_TYPE_NETWORK_PROTOCOL_ERROR: ReportTypeModel = ReportTypeModel {
    header: "Minecraft Network Protocol Error Report",
    nuggets: &[
        "0xBADF00D",
        "+'${`%&NO CARRIER",
        "Please insert The Internet CD #4",
        "Sabotage!",
        "Are you sure you are not moving wrongly?",
        "This time is not my fault, I promise!",
        "All lines are down!",
        "Maybe a shark bit some cable",
        "404",
        "I'm sorry, I don't speak that language",
        "What we've got here is failure to communicate",
        "It's the tubes, they're clogged!",
        "Abort, Retry, Ignore?",
        "Could be worse, I guess",
        "Wait, was the last bit one or zero?",
        "Too many suspicious packets",
        "Don't worry, I'll be fine",
        "Maybe this time it will work!",
        "I heard pigeons are more reliable",
    ],
};

pub const REPORT_TYPE_CHUNK_IO_ERROR: ReportTypeModel = ReportTypeModel {
    header: "Minecraft Chunk IO Error Report",
    nuggets: &[
        "I have failed you!",
        "Let's not do it again...",
        "Worst magic trick ever!",
        "Remember to backup your worlds regularly",
        "Pirates stole your chunk!",
        "Ker-chunk!",
        "Ideally, this shouldn't be here",
        "Let's hope it wasn't anything important",
        "Computers were a mistake",
        "Welp",
        "Not my proudest moment",
        "Who needs blocks in a block game, right?",
        "This chunk is no more...it has ceased to be...this is an EX-chunk",
        "loss.mca",
    ],
};

pub const REPORT_TYPES: [ReportTypeModel; 5] = [
    REPORT_TYPE_CRASH,
    REPORT_TYPE_PROFILE,
    REPORT_TYPE_TEST,
    REPORT_TYPE_NETWORK_PROTOCOL_ERROR,
    REPORT_TYPE_CHUNK_IO_ERROR,
];

pub const WITTY_COMMENT_UNAVAILABLE: &str = "Witty comment unavailable :(";

impl ReportTypeModel {
    pub fn get_error_comment(self) -> &'static str {
        self.get_error_comment_at_nanos(current_nanos())
    }

    pub fn get_error_comment_at_nanos(self, nanos: u128) -> &'static str {
        if self.nuggets.is_empty() {
            return WITTY_COMMENT_UNAVAILABLE;
        }

        let index = (nanos % self.nuggets.len() as u128) as usize;
        self.nuggets
            .get(index)
            .copied()
            .unwrap_or(WITTY_COMMENT_UNAVAILABLE)
    }

    pub fn append_header_at_nanos(
        self,
        builder: &mut String,
        extra_comments: &[&str],
        nanos: u128,
    ) {
        builder.push_str("---- ");
        builder.push_str(self.header);
        builder.push_str(" ----\n");
        builder.push_str("// ");
        builder.push_str(self.get_error_comment_at_nanos(nanos));
        builder.push('\n');

        for extra_comment in extra_comments {
            builder.push_str("// ");
            builder.push_str(extra_comment);
            builder.push('\n');
        }

        builder.push('\n');
    }

    pub fn header_at_nanos(self, extra_comments: &[&str], nanos: u128) -> String {
        let mut builder = String::new();
        self.append_header_at_nanos(&mut builder, extra_comments, nanos);
        builder
    }
}

fn current_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::{
        ReportTypeModel, REPORT_TYPES, REPORT_TYPE_CHUNK_IO_ERROR, REPORT_TYPE_CRASH,
        REPORT_TYPE_NETWORK_PROTOCOL_ERROR, REPORT_TYPE_PROFILE, REPORT_TYPE_TEST,
        WITTY_COMMENT_UNAVAILABLE,
    };

    #[test]
    fn report_type_static_records_match_java_headers_and_sizes() {
        assert_eq!(REPORT_TYPES.len(), 5);
        assert_eq!(REPORT_TYPE_CRASH.header, "Minecraft Crash Report");
        assert_eq!(REPORT_TYPE_CRASH.nuggets.len(), 34);
        assert_eq!(REPORT_TYPE_PROFILE.header, "Minecraft Profiler Results");
        assert_eq!(REPORT_TYPE_PROFILE.nuggets.len(), 15);
        assert_eq!(REPORT_TYPE_TEST.header, "Minecraft Test Report");
        assert_eq!(REPORT_TYPE_TEST.nuggets.len(), 8);
        assert_eq!(
            REPORT_TYPE_NETWORK_PROTOCOL_ERROR.header,
            "Minecraft Network Protocol Error Report"
        );
        assert_eq!(REPORT_TYPE_NETWORK_PROTOCOL_ERROR.nuggets.len(), 19);
        assert_eq!(
            REPORT_TYPE_CHUNK_IO_ERROR.header,
            "Minecraft Chunk IO Error Report"
        );
        assert_eq!(REPORT_TYPE_CHUNK_IO_ERROR.nuggets.len(), 14);
    }

    #[test]
    fn report_type_error_comment_matches_java_nanos_modulo() {
        assert_eq!(
            REPORT_TYPE_CRASH.get_error_comment_at_nanos(0),
            "Who set us up the TNT?"
        );
        assert_eq!(
            REPORT_TYPE_CRASH.get_error_comment_at_nanos(34),
            "Who set us up the TNT?"
        );
        assert_eq!(
            REPORT_TYPE_TEST.get_error_comment_at_nanos(7),
            "Where's the kaboom!?"
        );
        assert_eq!(
            REPORT_TYPE_CHUNK_IO_ERROR.get_error_comment_at_nanos(13),
            "loss.mca"
        );
    }

    #[test]
    fn report_type_error_comment_fallback_matches_java_catch_block() {
        let empty = ReportTypeModel {
            header: "Empty",
            nuggets: &[],
        };

        assert_eq!(
            empty.get_error_comment_at_nanos(0),
            WITTY_COMMENT_UNAVAILABLE
        );
    }

    #[test]
    fn report_type_append_header_matches_java_format() {
        let mut builder = String::from("prefix\n");
        REPORT_TYPE_TEST.append_header_at_nanos(&mut builder, &["extra one", "extra two"], 0);

        assert_eq!(
            builder,
            "prefix\n---- Minecraft Test Report ----\n// Don't mind me\n// extra one\n// extra two\n\n"
        );
    }
}
