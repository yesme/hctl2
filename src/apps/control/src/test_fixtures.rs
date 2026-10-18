//! Test-only fixture delivery for the control's unit tests.
//!
//! A fixture is executed from a link to the build's script, never from a file
//! the test wrote: a file just opened for writing can answer execve with
//! ETXTBSY on Linux while another test thread's forked child still holds the
//! inherited write descriptor. The built scripts live in `src/fixtures/scripts`
//! (see `src/BUCK`'s `script_fixtures`) and reach a test through `HCTL2_FIXTURES`.
use std::path::Path;

pub(crate) fn link_fixture(name: &str, path: &Path) {
    let root =
        std::env::var_os("HCTL2_FIXTURES").expect("HCTL2_FIXTURES must be set to run this test");
    let source = Path::new(&root).join(name);
    // The environment carries a project-relative path; a symlink resolves its
    // target relative to the link's own directory, so it must be absolute.
    let source = std::fs::canonicalize(&source)
        .unwrap_or_else(|error| panic!("fixture {}: {error}", source.display()));
    std::os::unix::fs::symlink(&source, path)
        .unwrap_or_else(|error| panic!("link {} -> {}: {error}", path.display(), source.display()));
}
