use crate::analyzer::is_application_frame;
use std::fs;
use crate::parser::parse_jstack_output;

#[test]
fn test_is_application_frame_with_ibm_dump_data() {
    // Frame from IBM dump that should be identified as application frame
    let app_frame_with_app_prefix = "at app//com.example.threadanalyzer.ThreadAnalyzerApplication.lambda$main$2(ThreadAnalyzerApplication.java:95)";
    let app_frame_with_app_prefix_2 = "at app//com.example.threadanalyzer.ThreadAnalyzerApplication.lambda$main$1(ThreadAnalyzerApplication.java:79)";
    let app_frame_with_app_prefix_3 = "at app//com.example.threadanalyzer.ThreadAnalyzerApplication.lambda$main$1(ThreadAnalyzerApplication.java:83)";

    // Frames from IBM dump that should NOT be identified as application frames
    let spark_frame = "at app//spark.Service.lambda$init$2(Service.java:644)";
    let jetty_frame = "at app//org.eclipse.jetty.server.Server.join(Server.java:551)";
    let jetty_util_frame = "at app//org.eclipse.jetty.util.thread.QueuedThreadPool.join(QueuedThreadPool.java:567)";

    let app_package_prefix = Some("com.example.threadanalyzer");

    // Assertions for application frames
    assert!(is_application_frame(app_frame_with_app_prefix, app_package_prefix));
    assert!(is_application_frame(app_frame_with_app_prefix_2, app_package_prefix));
    assert!(is_application_frame(app_frame_with_app_prefix_3, app_package_prefix));

    // Assertions for non-application frames
    assert!(!is_application_frame(spark_frame, app_package_prefix));
    assert!(!is_application_frame(jetty_frame, app_package_prefix));
    assert!(!is_application_frame(jetty_util_frame, app_package_prefix));

    // Test with None app_package_prefix
    assert!(!is_application_frame(app_frame_with_app_prefix, None));
    assert!(!is_application_frame(spark_frame, None));
}

#[test]
fn test_is_application_frame_with_actual_ibm_dump() {
    let dump_bytes = fs::read("src/test_data/ibm_thread_dump.jstack").expect("Unable to read ibm_thread_dump.jstack as bytes");
    let dump_content = String::from_utf8_lossy(&dump_bytes).to_string();
    let threads = parse_jstack_output(&dump_content);

    let app_package_prefix = Some("com.example.threadanalyzer");

    let mut found_app_frame = false;
    let mut found_spark_frame = false;
    let mut found_jetty_frame = false;

    for thread in threads {
        for frame in thread.stack_trace {
            if is_application_frame(&frame, app_package_prefix) {
                println!("Found app frame: {}", frame);
                found_app_frame = true;
            } else if frame.contains("spark.") {
                println!("Found spark frame: {}", frame);
                found_spark_frame = true;
            } else if frame.contains("jetty.") {
                println!("Found jetty frame: {}", frame);
                found_jetty_frame = true;
            }
        }
    }

    assert!(found_app_frame, "Should have found at least one application frame");
    assert!(found_spark_frame, "Should have found at least one spark frame");
    assert!(found_jetty_frame, "Should have found at least one jetty frame");
}

#[test]
fn test_is_application_frame_with_minimal_ibm_dump() {
    let dump_content = fs::read_to_string("src/test_data/minimal_ibm_app_thread.jstack").expect("Unable to read minimal_ibm_app_thread.jstack");
    let threads = parse_jstack_output(&dump_content);

    let app_package_prefix = Some("com.example.threadanalyzer");

    let mut found_app_frame = false;

    for thread in threads {
        for frame in thread.stack_trace {
            if is_application_frame(&frame, app_package_prefix) {
                println!("Found app frame in minimal dump: {}", frame);
                found_app_frame = true;
            }
        }
    }

    assert!(found_app_frame, "Should have found at least one application frame in minimal dump");
}
