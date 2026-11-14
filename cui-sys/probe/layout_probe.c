/* Layout probe: prints the C ABI facts that cui-sys's FFI declarations must
 * match, as a TOML table. Run by vendor.sh against the sources in cui-sys/c/;
 * the table lands in cui-sys/layout/<version>.toml and the layout tests
 * check the Rust structs against it.
 *
 * base_probe emits the version line and the [enum] spot checks below; the
 * [size] and [offset] sections (every Rust FFI struct, every field) are
 * emitted by gen_probe.c, which scripts/gen_ffi.py regenerates from the
 * header — no hand-maintained lists to keep in sync. */
#include <stdio.h>
#include <stddef.h>
#include "cui.h"

#define SZ(T)     printf("%s = %zu\n", #T, sizeof(T))
#define OFF(T, F) printf("\"%s.%s\" = %zu\n", #T, #F, offsetof(T, F))
#define EN(V)     printf("%s = %d\n", #V, (int)(V))

void base_probe(void) {
    printf("version = \"%s\"\n\n", CUI_VERSION_STRING);

    puts("[enum]");
    EN(CUI_GRID); EN(CUI_ALIGN_STRETCH); EN(CUI_JUSTIFY_BETWEEN);
    EN(CUI_ELEVATION_FLOATING); EN(CUI_DIRECTION_AUTO); EN(CUI_THEME_DARK);
    EN(CUI_ACCENT_ROSE); EN(CUI_BUTTON_LINK); EN(CUI_SIZE_LG);
    EN(CUI_TEXT_END); EN(CUI_TONE_ACCENT); EN(CUI_SIDE_TOP);
    EN(CUI_MOUSE_MIDDLE); EN(CUI_RENDERER_SOFTWARE);
    EN(CUI_KEY_9); EN(CUI_KEY_F12);
    EN(CUI_AXIS_HOURS); EN(CUI_TASK_DONE);
}
