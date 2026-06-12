use egui_thematic::ThemeConfig;

/// Fluent UI 3 浅色主题预设
///
/// 配色参考 Microsoft Fluent Design System：
/// - 品牌主色: #0078D4 (brand[80])
/// - 中性灰阶: grey[14] ~ grey[98]
/// - 圆角: 6px (Fluent3 标准)
pub fn fluent3_light_preset() -> ThemeConfig {
    ThemeConfig {
        name: "Fluent3 Light".to_string(),
        dark_mode: false,

        // ── 文本颜色 ──
        // grey[14] — 主文本
        override_text_color: Some([36, 36, 36, 255]),
        // grey[44] — 弱文本
        override_weak_text_color: Some([97, 97, 97, 255]),
        // brand[80] — 超链接
        override_hyperlink_color: Some([0, 120, 212, 255]),

        // ── 背景色 ──
        // grey[96] — 淡背景
        override_faint_bg_color: Some([245, 245, 245, 255]),
        // white — 极端背景
        override_extreme_bg_color: Some([255, 255, 255, 255]),
        // grey[94] — 代码背景
        override_code_bg_color: Some([240, 240, 240, 255]),

        // ── 语义色 ──
        // #FFB900 — 警告 (Fluent Yellow)
        override_warn_fg_color: Some([255, 185, 0, 255]),
        // #D13438 — 错误 (Fluent Red)
        override_error_fg_color: Some([209, 52, 56, 255]),

        // ── 窗口 ──
        // white — 窗口填充
        override_window_fill: Some([255, 255, 255, 255]),
        // grey[82] — 窗口描边
        override_window_stroke_color: Some([214, 214, 214, 255]),
        override_window_stroke_width: Some(1.0),
        // Fluent3 圆角 6px
        override_window_corner_radius: Some(6),
        override_window_shadow_size: Some(8),

        // ── 面板 ──
        // grey[98] — 面板填充
        override_panel_fill: Some([250, 250, 250, 255]),

        // ── 弹窗阴影 ──
        override_popup_shadow_size: Some(12),

        // ── 选区 ──
        // brand[80] — 选区背景
        override_selection_bg: Some([0, 120, 212, 255]),
        // brand[60] — 选区描边
        override_selection_stroke_color: Some([0, 90, 158, 255]),
        override_selection_stroke_width: Some(1.0),

        // ── 非交互控件 (标签、分隔线等) ──
        // grey[98]
        override_widget_noninteractive_bg_fill: Some([250, 250, 250, 255]),
        override_widget_noninteractive_weak_bg_fill: Some([245, 245, 245, 255]),
        override_widget_noninteractive_bg_stroke_color: Some([214, 214, 214, 255]),
        override_widget_noninteractive_bg_stroke_width: Some(1.0),
        override_widget_noninteractive_corner_radius: Some(6),
        override_widget_noninteractive_fg_stroke_color: Some([36, 36, 36, 255]),
        override_widget_noninteractive_fg_stroke_width: Some(1.0),
        override_widget_noninteractive_expansion: Some(0.0),

        // ── 非活动控件 (默认按钮等) ──
        // grey[96]
        override_widget_inactive_bg_fill: Some([245, 245, 245, 255]),
        override_widget_inactive_weak_bg_fill: Some([250, 250, 250, 255]),
        override_widget_inactive_bg_stroke_color: Some([214, 214, 214, 255]),
        override_widget_inactive_bg_stroke_width: Some(1.0),
        override_widget_inactive_corner_radius: Some(6),
        override_widget_inactive_fg_stroke_color: Some([36, 36, 36, 255]),
        override_widget_inactive_fg_stroke_width: Some(1.0),
        override_widget_inactive_expansion: Some(0.0),

        // ── 悬停控件 ──
        // brand 浅色 tint — 悬停背景
        override_widget_hovered_bg_fill: Some([225, 239, 250, 255]),
        override_widget_hovered_weak_bg_fill: Some([245, 248, 252, 255]),
        override_widget_hovered_bg_stroke_color: Some([0, 120, 212, 255]),
        override_widget_hovered_bg_stroke_width: Some(1.0),
        override_widget_hovered_corner_radius: Some(6),
        override_widget_hovered_fg_stroke_color: Some([36, 36, 36, 255]),
        override_widget_hovered_fg_stroke_width: Some(1.0),
        override_widget_hovered_expansion: Some(0.0),

        // ── 活动/按下控件 ──
        // brand[80] — 按下背景
        override_widget_active_bg_fill: Some([0, 90, 158, 255]),
        override_widget_active_weak_bg_fill: Some([0, 120, 212, 255]),
        override_widget_active_bg_stroke_color: Some([0, 59, 106, 255]),
        override_widget_active_bg_stroke_width: Some(1.0),
        override_widget_active_corner_radius: Some(6),
        override_widget_active_fg_stroke_color: Some([255, 255, 255, 255]),
        override_widget_active_fg_stroke_width: Some(1.0),
        override_widget_active_expansion: Some(0.0),

        // ── 展开/打开控件 ──
        // brand 浅色
        override_widget_open_bg_fill: Some([225, 239, 250, 255]),
        override_widget_open_weak_bg_fill: Some([245, 248, 252, 255]),
        override_widget_open_bg_stroke_color: Some([0, 120, 212, 255]),
        override_widget_open_bg_stroke_width: Some(1.0),
        override_widget_open_corner_radius: Some(6),
        override_widget_open_fg_stroke_color: Some([36, 36, 36, 255]),
        override_widget_open_fg_stroke_width: Some(1.0),
        override_widget_open_expansion: Some(0.0),

        // ── 其他 ──
        override_resize_corner_size: Some(6.0),
        override_text_cursor_width: Some(2.0),
        override_clip_rect_margin: Some(3.0),
        override_button_frame: Some(true),
        override_collapsing_header_frame: Some(false),
        override_indent_has_left_vline: Some(false),
        override_striped: Some(false),
        override_slider_trailing_fill: Some(false),
    }
}
