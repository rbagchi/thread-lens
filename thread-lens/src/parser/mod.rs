use crate::models::ThreadDump;

pub mod jvm_vendor;
pub mod openjdk;
pub mod ibm;
pub mod tests;

pub use jvm_vendor::JvmVendor;
use openjdk::parse_jstack_output_openjdk;
use ibm::parse_jstack_output_ibm;

pub fn detect_jvm_vendor(output: &str) -> JvmVendor {
    if output.contains("OpenJDK") || output.contains("HotSpot") {
        JvmVendor::OpenJDK
    } else if output.contains("IBM J9") || output.contains("Eclipse OpenJ9") || output.contains("OpenJ9") {
        JvmVendor::IBM
    } else {
        JvmVendor::Unknown
    }
}

pub fn parse_jstack_output(output: &str) -> Result<ThreadDump, String> {
    match detect_jvm_vendor(output) {
        JvmVendor::OpenJDK => {
            log::info!("Detected OpenJDK/HotSpot JVM.");
            parse_jstack_output_openjdk(output)
        }
        JvmVendor::IBM => {
            log::info!("Detected IBM J9/Eclipse OpenJ9 JVM.");
            parse_jstack_output_ibm(output)
        }
        JvmVendor::Unknown => {
            log::warn!("Unknown JVM vendor detected. Attempting OpenJDK parsing.");
            parse_jstack_output_openjdk(output)
        }
    }
}