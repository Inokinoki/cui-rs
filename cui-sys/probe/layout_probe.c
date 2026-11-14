/* Layout probe: prints the C ABI facts (struct sizes, field offsets, enum
 * values) that cui-sys's FFI declarations must match, as a TOML table. Run
 * by vendor.sh against the sources in cui-sys/c/; the table lands in
 * cui-sys/layout/<version>.toml and the layout tests check the Rust structs
 * against it. Extend the lists below alongside the FFI in src/lib.rs. */
#include <stdio.h>
#include <stddef.h>
#include "cui.h"

#define SZ(T)     printf("%s = %zu\n", #T, sizeof(T))
#define OFF(T, F) printf("\"%s.%s\" = %zu\n", #T, #F, offsetof(T, F))
#define EN(V)     printf("%s = %d\n", #V, (int)(V))

int main(void) {
    printf("version = \"%s\"\n\n", CUI_VERSION_STRING);

    puts("[size]");
    SZ(cui_vec2); SZ(cui_rect); SZ(cui_color); SZ(cui_size);
    SZ(cui_box); SZ(cui_card_opts); SZ(cui_scroll_opts); SZ(cui_text_opts);
    SZ(cui_button_opts); SZ(cui_segmented_opts); SZ(cui_check_opts);
    SZ(cui_slider_opts); SZ(cui_chip_opts); SZ(cui_input_opts);
    SZ(cui_number_opts); SZ(cui_select_opts); SZ(cui_tabs_opts);
    SZ(cui_nav_opts); SZ(cui_dialog_opts); SZ(cui_sheet_opts);
    SZ(cui_toast_opts); SZ(cui_badge_opts); SZ(cui_empty_opts);
    SZ(cui_metric_opts); SZ(cui_progress_opts); SZ(cui_table_column);
    SZ(cui_table_sort); SZ(cui_table_opts); SZ(cui_series);
    SZ(cui_chart_annotation); SZ(cui_chart_opts); SZ(cui_app_desc);

    puts("[offset]");
    OFF(cui_box, dir); OFF(cui_box, gap); OFF(cui_box, bg);
    OFF(cui_button_opts, tooltip); OFF(cui_button_opts, color);
    OFF(cui_input_opts, size); OFF(cui_input_opts, placeholder);
    OFF(cui_app_desc, renderer);
    OFF(cui_series, values); OFF(cui_chart_opts, labels);

    puts("[enum]");
    EN(CUI_GRID); EN(CUI_ALIGN_STRETCH); EN(CUI_JUSTIFY_BETWEEN);
    EN(CUI_ELEVATION_FLOATING); EN(CUI_DIRECTION_AUTO); EN(CUI_THEME_DARK);
    EN(CUI_ACCENT_ROSE); EN(CUI_BUTTON_LINK); EN(CUI_SIZE_LG);
    EN(CUI_TEXT_END); EN(CUI_TONE_ACCENT); EN(CUI_SIDE_TOP);
    EN(CUI_MOUSE_MIDDLE); EN(CUI_RENDERER_SOFTWARE);
    EN(CUI_KEY_9); EN(CUI_KEY_F12);
    return 0;
}
