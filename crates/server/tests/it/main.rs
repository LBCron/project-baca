//! Functional integration tests linked into one binary: each file under
//! `tests/` becomes its own ~90 MB executable, so these share a single link.
//! Timing-sensitive suites (performance, load_stress) stay separate binaries
//! so parallel functional tests cannot skew their latency assertions.

#[path = "../common/mod.rs"]
mod common;

mod admin_test;
mod api_boundary_test;
mod auth_test;
mod catalog_test;
mod database_test;
mod integration_test;
mod reliability_test;
mod security_owasp_test;
mod semantic_ai_test;
mod smoke_test;
