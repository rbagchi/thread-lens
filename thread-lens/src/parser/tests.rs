#![cfg(test)]

use super::ibm::parse_jstack_output_ibm;
use super::openjdk::parse_jstack_output_openjdk;

use super::{detect_jvm_vendor, parse_jstack_output};
use crate::models::{FrameCategory, ThreadCategory};

#[test]
fn test_parse_jstack_output_ibm_sample() {
    let sample_dump = r#"2025-09-21 03:42:28
Full thread dump IBM Semeru Runtime Open Edition 17.0.8.0 (build 17.0.8+7, JRE 17.0.8 Linux amd64-64-Bit ) (J9 0.42 +)

"main" J9VMThread:0x0000000000402000, omrthread:0x00007f8000001000, JavaThread:0x00007f8000001000, state:R, prio=5, OSCPUS=0.00%, tid=0x00007f8000001000, nid=0x1, fnid=0x1, stack:0x00007f8000001000-0x00007f8000002000, core:false, Java callstack:
	at com.example.threadanalyzer.ThreadAnalyzerApplication.main(ThreadAnalyzerApplication.java:100)

"BlockedThread" J9VMThread:0x0000000000403000, omrthread:0x00007f8000003000, JavaThread:0x00007f8000003000, state:B, prio=5, OSCPUS=0.00%, tid=0x00007f8000003000, nid=0x3, fnid=0x3, stack:0x00007f8000003000-0x00007f8000004000, core:false, Java callstack:
	at com.example.threadanalyzer.ThreadAnalyzerApplication.lambda$null$2(ThreadAnalyzerApplication.java:96)
	- waiting on <0x00000000e0000000> (a java.lang.Object)
	at java.lang.Object.wait(java.base@17.0.8/Native Method)

"BlockerThread" J9VMThread:0x0000000000404000, omrthread:0x00007f8000005000, JavaThread:0x00007f8000005000, state:B, prio=5, OSCPUS=0.00%, tid=0x00007f8000005000, nid=0x5, fnid=0x5, stack:0x00007f8000005000-0x00007f8000006000, core:false, Java callstack:
	at com.example.threadanalyzer.ThreadAnalyzerApplication.lambda$main$1(ThreadAnalyzerApplication.java:80)
	- waiting on <0x00000000e0000001> (a java.lang.Object)
	at java.lang.Object.wait(java.base@17.0.8/Native Method)
"#;

    let dump = parse_jstack_output_ibm(sample_dump).expect("Failed to parse IBM dump");

    assert_eq!(dump.threads.len(), 3);
    assert_eq!(dump.jvm_version, "IBM Semeru Runtime Open Edition 17.0.8.0");

    // Test main thread
    let main_thread = &dump.threads[0];
    assert_eq!(main_thread.name, "main");
    assert_eq!(main_thread.state, "RUNNABLE");
    assert_eq!(main_thread.category, ThreadCategory::Application);
    assert_eq!(main_thread.frames.len(), 1);
    assert_eq!(main_thread.frames[0].line, "at com.example.threadanalyzer.ThreadAnalyzerApplication.main(ThreadAnalyzerApplication.java:100)");
    assert_eq!(main_thread.frames[0].category, FrameCategory::Application);

    // Test BlockedThread
    let blocked_thread = &dump.threads[1];
    assert_eq!(blocked_thread.name, "BlockedThread");
    assert_eq!(blocked_thread.state, "BLOCKED");
    assert_eq!(blocked_thread.category, ThreadCategory::Application);
    assert_eq!(blocked_thread.frames.len(), 2);
    assert_eq!(blocked_thread.frames[0].line, "at com.example.threadanalyzer.ThreadAnalyzerApplication.lambda$null$2(ThreadAnalyzerApplication.java:96)");
    assert_eq!(
        blocked_thread.frames[0].category,
        FrameCategory::Application
    );
    assert_eq!(
        blocked_thread.frames[1].line,
        "at java.lang.Object.wait(java.base@17.0.8/Native Method)"
    );
    assert_eq!(blocked_thread.frames[1].category, FrameCategory::Jvm);

    // Test BlockerThread
    let blocker_thread = &dump.threads[2];
    assert_eq!(blocker_thread.name, "BlockerThread");
    assert_eq!(blocker_thread.state, "BLOCKED");
    assert_eq!(blocker_thread.category, ThreadCategory::Application);
    assert_eq!(blocker_thread.frames.len(), 2);
    assert_eq!(blocker_thread.frames[0].line, "at com.example.threadanalyzer.ThreadAnalyzerApplication.lambda$main$1(ThreadAnalyzerApplication.java:80)");
    assert_eq!(
        blocker_thread.frames[0].category,
        FrameCategory::Application
    );
    assert_eq!(
        blocker_thread.frames[1].line,
        "at java.lang.Object.wait(java.base@17.0.8/Native Method)"
    );
    assert_eq!(blocker_thread.frames[1].category, FrameCategory::Jvm);
}

#[test]
fn test_parse_jstack_output_openjdk_sample() {
    let sample_dump = r#"2025-09-21 03:30:13
Full thread dump OpenJDK 64-Bit Server VM (11.0.16+8 mixed mode, sharing):

"main" #1 prio=5 os_prio=0 cpu=0.00ms elapsed=0.00s tid=0x0000e91980193800 nid=0x2d runnable  [0x0000e919527fe000]
   java.lang.Thread.State: RUNNABLE

"Thread-0" #10 prio=5 os_prio=0 cpu=106.01ms elapsed=0.25s tid=0x0000e91980193800 nid=0x2d in Object.wait()  [0x0000e919527fe000]
   java.lang.Thread.State: WAITING (on object monitor)
	at java.lang.Object.wait(java.base@11.0.16/Native Method)
	- waiting on <0x00000000e2e2f648> (a java.lang.Object)
	at com.example.threadanalyzer.ThreadAnalyzerApplication.lambda$null$2(ThreadAnalyzerApplication.java:96)

"BlockedThread" #11 prio=5 os_prio=0 cpu=106.01ms elapsed=0.25s tid=0x0000e91980193800 nid=0x2d in Object.wait()  [0x0000e919527fe000]
   java.lang.Thread.State: BLOCKED (on object monitor)
	at com.example.threadanalyzer.ThreadAnalyzerApplication.lambda$null$2(ThreadAnalyzerApplication.java:96)
"#;

    let dump = parse_jstack_output_openjdk(sample_dump).expect("Failed to parse OpenJDK dump");

    assert_eq!(dump.threads.len(), 3);
    assert_eq!(dump.jvm_version, "OpenJDK 64-Bit Server VM (11.0.16+8)");

    // Test main thread
    let main_thread = &dump.threads[0];
    assert_eq!(main_thread.name, "main");
    assert_eq!(main_thread.state, "RUNNABLE");
    assert_eq!(main_thread.category, ThreadCategory::Jvm);
    assert_eq!(main_thread.frames.len(), 0);

    // Test Thread-0 (WAITING on object monitor -> BLOCKED)
    let thread_0 = &dump.threads[1];
    assert_eq!(thread_0.name, "Thread-0");
    assert_eq!(thread_0.state, "BLOCKED");
    assert_eq!(thread_0.category, ThreadCategory::Application);
    assert_eq!(thread_0.frames.len(), 2);
    assert_eq!(
        thread_0.frames[0].line,
        "at java.lang.Object.wait(java.base@11.0.16/Native Method)"
    );
    assert_eq!(thread_0.frames[0].category, FrameCategory::Jvm);
    assert_eq!(thread_0.frames[1].line, "at com.example.threadanalyzer.ThreadAnalyzerApplication.lambda$null$2(ThreadAnalyzerApplication.java:96)");
    assert_eq!(thread_0.frames[1].category, FrameCategory::Application);

    // Test BlockedThread
    let blocked_thread = &dump.threads[2];
    assert_eq!(blocked_thread.name, "BlockedThread");
    assert_eq!(blocked_thread.state, "BLOCKED");
    assert_eq!(blocked_thread.category, ThreadCategory::Application);
    assert_eq!(blocked_thread.frames.len(), 1);
    assert_eq!(blocked_thread.frames[0].line, "at com.example.threadanalyzer.ThreadAnalyzerApplication.lambda$null$2(ThreadAnalyzerApplication.java:96)");
    assert_eq!(
        blocked_thread.frames[0].category,
        FrameCategory::Application
    );
}
