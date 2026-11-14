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

void base_probe(void) {
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
    SZ(cui_menu_opts); SZ(cui_menu_item_opts); SZ(cui_split_opts); SZ(cui_avatar_opts);
    SZ(cui_user_menu_opts); SZ(cui_list_item_opts); SZ(cui_timeline_opts); SZ(cui_text_fx_opts);
    SZ(cui_chat_opts); SZ(cui_reasoning_opts); SZ(cui_tool_call_opts); SZ(cui_plan_step);
    SZ(cui_agent_plan_opts); SZ(cui_agent_status_opts); SZ(cui_source);
    SZ(cui_response_actions_opts); SZ(cui_prompt_suggestion); SZ(cui_prompt_suggestions_opts);
    SZ(cui_model); SZ(cui_model_picker_opts); SZ(cui_composer_opts); SZ(cui_viz_item);
    SZ(cui_context_opts); SZ(cui_dot_series); SZ(cui_dot_chart_opts); SZ(cui_dot_chart_result);
    SZ(cui_event_track_opts); SZ(cui_event_track_result); SZ(cui_readout_opts);
    SZ(cui_dot_display_opts); SZ(cui_command); SZ(cui_corners); SZ(cui_canvas_result);
    SZ(cui_shadow); SZ(cui_theme); SZ(cui_file_dialog_opts); SZ(cui_file_entry);

    puts("[offset]");
    OFF(cui_box, dir); OFF(cui_box, gap); OFF(cui_box, bg);
    OFF(cui_button_opts, tooltip); OFF(cui_button_opts, color);
    OFF(cui_input_opts, size); OFF(cui_input_opts, placeholder);
    OFF(cui_app_desc, renderer);
    OFF(cui_series, values); OFF(cui_chart_opts, labels);
    OFF(cui_menu_opts, placement); OFF(cui_composer_opts, model);
    OFF(cui_dot_chart_result, x); OFF(cui_theme, font_scale);
    OFF(cui_text_fx_opts, speed); OFF(cui_chat_opts, avatar);

    puts("[enum]");
    EN(CUI_GRID); EN(CUI_ALIGN_STRETCH); EN(CUI_JUSTIFY_BETWEEN);
    EN(CUI_ELEVATION_FLOATING); EN(CUI_DIRECTION_AUTO); EN(CUI_THEME_DARK);
    EN(CUI_ACCENT_ROSE); EN(CUI_BUTTON_LINK); EN(CUI_SIZE_LG);
    EN(CUI_TEXT_END); EN(CUI_TONE_ACCENT); EN(CUI_SIDE_TOP);
    EN(CUI_MOUSE_MIDDLE); EN(CUI_RENDERER_SOFTWARE);
    EN(CUI_KEY_9); EN(CUI_KEY_F12);
    EN(CUI_AXIS_HOURS); EN(CUI_TASK_DONE);
}
