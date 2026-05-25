/**
 * musl_shim.c — Compatibility stubs for symbols missing in OHOS musl.
 *
 * These symbols are referenced by Rust's std library but are not
 * provided by the OpenHarmony musl libc implementation.
 */

#include <errno.h>
#include <string.h>

// `posix_spawn_file_actions_addchdir_np` is a glibc extension for
// changing directory in a spawned process. We don't spawn processes
// in the agent core, but the linker needs a symbol to resolve.
int posix_spawn_file_actions_addchdir_np(void* file_actions, const char* path) {
    (void)file_actions;
    (void)path;
    // Not supported — POSIX chdir semantics don't apply here.
    return -1;
}

// `__xpg_strerror_r` is the XSI-compliant version of strerror_r.
// OHOS musl may only provide the GNU variant. We wrap the standard
// strerror_r (which on musl is the POSIX-conformant one) directly.
// For musl, strerror_r _is_ the XPG-compliant version, but Rust
// expects the __xpg alias for conditional compilation compatibility.
int __xpg_strerror_r(int errnum, char* buf, size_t buflen) {
    // musl's strerror_r is already POSIX-conformant (returns int).
    return strerror_r(errnum, buf, buflen);
}
