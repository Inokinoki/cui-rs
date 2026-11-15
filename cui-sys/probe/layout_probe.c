/* Layout probe: prints the C ABI facts that cui-sys's FFI declarations must
 * match, as a TOML table. Run by vendor.sh against the sources in cui-sys/c/;
 * the table lands in cui-sys/layout/<version>.toml and the layout tests
 * check the Rust structs against it.
 *
 * base_probe emits the version line; the [enum] section (every enum member
 * in the header) and the [size] / [offset] sections (every Rust FFI struct,
 * every field) are emitted by gen_probe.c, which scripts/gen_ffi.py
 * regenerates from the header — no hand-maintained lists to keep in sync. */
#include <stdio.h>
#include <stddef.h>
#include "cui.h"

#define SZ(T)     printf("%s = %zu\n", #T, sizeof(T))
#define OFF(T, F) printf("\"%s.%s\" = %zu\n", #T, #F, offsetof(T, F))
#define EN(V)     printf("%s = %d\n", #V, (int)(V))

void base_probe(void) {
    printf("version = \"%s\"\n\n", CUI_VERSION_STRING);
}
