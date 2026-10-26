use eframe::egui::{
    self, pos2, vec2, Align2, Color32, CornerRadius, FontFamily, FontId, Frame, Margin, Rect,
    Response, RichText, Sense, Shape, Stroke, TextStyle, Ui, Vec2,
};

#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub background: Color32,
    pub header: Color32,
    pub card: Color32,
    pub card_alt: Color32,
    pub border: Color32,
    pub text: Color32,
    pub subtext: Color32,
    pub accent: Color32,
    pub accent_hover: Color32,
    pub accent_text: Color32,
    pub hover: Color32,
    pub selected: Color32,
    pub badge: Color32,
    pub danger: Color32,
    pub success: Color32,
    pub warning: Color32,
    pub dark: bool,
}

impl Palette {
    pub fn light() -> Self {
        Self {
            background: Color32::from_rgb(250, 250, 250),
            header: Color32::from_rgb(246, 246, 246),
            card: Color32::WHITE,
            card_alt: Color32::from_rgb(247, 247, 247),
            border: Color32::from_rgb(222, 222, 222),
            text: Color32::from_rgb(32, 32, 34),
            subtext: Color32::from_rgb(110, 110, 115),
            accent: Color32::from_rgb(53, 132, 228),
            accent_hover: Color32::from_rgb(41, 112, 200),
            accent_text: Color32::WHITE,
            hover: Color32::from_rgb(236, 241, 248),
            selected: Color32::from_rgb(226, 226, 228),
            badge: Color32::from_rgb(232, 232, 234),
            danger: Color32::from_rgb(196, 43, 48),
            success: Color32::from_rgb(38, 145, 89),
            warning: Color32::from_rgb(176, 127, 25),
            dark: false,
        }
    }

    pub fn dark() -> Self {
        Self {
            background: Color32::from_rgb(32, 32, 34),
            header: Color32::from_rgb(42, 42, 45),
            card: Color32::from_rgb(51, 51, 54),
            card_alt: Color32::from_rgb(44, 44, 47),
            border: Color32::from_rgb(70, 70, 74),
            text: Color32::from_rgb(240, 240, 242),
            subtext: Color32::from_rgb(164, 164, 170),
            accent: Color32::from_rgb(53, 132, 228),
            accent_hover: Color32::from_rgb(78, 151, 236),
            accent_text: Color32::WHITE,
            hover: Color32::from_rgb(61, 61, 66),
            selected: Color32::from_rgb(78, 78, 84),
            badge: Color32::from_rgb(72, 72, 78),
            danger: Color32::from_rgb(224, 80, 84),
            success: Color32::from_rgb(73, 190, 125),
            warning: Color32::from_rgb(222, 174, 63),
            dark: true,
        }
    }
}

pub fn apply(ctx: &egui::Context, dark: bool, font_scale: f64) -> Palette {
    let palette = if dark {
        Palette::dark()
    } else {
        Palette::light()
    };
    let mut visuals = if dark {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };
    visuals.window_fill = palette.background;
    visuals.panel_fill = palette.background;
    visuals.extreme_bg_color = palette.card_alt;
    visuals.faint_bg_color = palette.card_alt;
    visuals.hyperlink_color = palette.accent;
    visuals.selection.bg_fill = Color32::from_rgba_unmultiplied(53, 132, 228, 70);
    visuals.selection.stroke = Stroke::new(1.0_f32, palette.accent);
    visuals.widgets.noninteractive.bg_fill = palette.card;
    visuals.widgets.noninteractive.weak_bg_fill = palette.card;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, palette.border);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, palette.text);
    visuals.widgets.noninteractive.corner_radius = CornerRadius::same(8);
    visuals.widgets.inactive.bg_fill = palette.card;
    visuals.widgets.inactive.weak_bg_fill = palette.card;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, palette.border);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, palette.text);
    visuals.widgets.inactive.corner_radius = CornerRadius::same(8);
    visuals.widgets.hovered.bg_fill = palette.hover;
    visuals.widgets.hovered.weak_bg_fill = palette.hover;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, palette.border);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, palette.text);
    visuals.widgets.hovered.corner_radius = CornerRadius::same(8);
    visuals.widgets.active.bg_fill = palette.selected;
    visuals.widgets.active.weak_bg_fill = palette.selected;
    visuals.widgets.active.bg_stroke = Stroke::new(1.0_f32, palette.accent);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, palette.text);
    visuals.widgets.active.corner_radius = CornerRadius::same(8);
    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.text_styles.insert(
        TextStyle::Heading,
        FontId::new(22.0, FontFamily::Proportional),
    );
    style
        .text_styles
        .insert(TextStyle::Body, FontId::new(13.5, FontFamily::Proportional));
    style.text_styles.insert(
        TextStyle::Button,
        FontId::new(13.5, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Small,
        FontId::new(11.5, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Monospace,
        FontId::new(12.5, FontFamily::Monospace),
    );
    style.spacing.item_spacing = vec2(8.0, 8.0);
    style.spacing.button_padding = vec2(12.0, 7.0);
    style.spacing.window_margin = Margin::ZERO;
    ctx.set_style(style);
    ctx.set_pixels_per_point((font_scale as f32).clamp(0.8, 2.0));
    palette
}

pub fn card_frame(palette: &Palette, radius: u8) -> Frame {
    Frame::new()
        .fill(palette.card)
        .stroke(Stroke::new(1.0_f32, palette.border))
        .corner_radius(CornerRadius::same(radius))
        .inner_margin(Margin::same(12))
}

pub fn card<R>(ui: &mut Ui, palette: &Palette, add_contents: impl FnOnce(&mut Ui) -> R) -> R {
    card_frame(palette, 12).show(ui, add_contents).inner
}

pub fn section_title(ui: &mut Ui, palette: &Palette, title: &str) {
    ui.add_space(6.0);
    ui.label(
        RichText::new(title)
            .strong()
            .size(13.0)
            .color(palette.subtext),
    );
    ui.add_space(2.0);
}

pub fn badge(ui: &mut Ui, palette: &Palette, text: &str) {
    Frame::new()
        .fill(palette.badge)
        .corner_radius(CornerRadius::same(20))
        .inner_margin(Margin::symmetric(9, 3))
        .show(ui, |ui| {
            ui.label(RichText::new(text).size(11.5).color(palette.subtext));
        });
}

pub fn grade_badge(ui: &mut Ui, palette: &Palette, grade: &str) {
    let (fill, text) = match grade {
        "Platinum" => (
            Color32::from_gray(if palette.dark { 150 } else { 205 }),
            palette.text,
        ),
        "Gold" => (
            Color32::from_rgb(
                if palette.dark { 141 } else { 242 },
                if palette.dark { 118 } else { 237 },
                if palette.dark { 55 } else { 222 },
            ),
            palette.text,
        ),
        "Silver" => (palette.badge, palette.text),
        "Bronze" => (
            Color32::from_rgb(219, 191, 163),
            Color32::from_rgb(70, 46, 20),
        ),
        _ => (palette.badge, palette.text),
    };
    Frame::new()
        .fill(fill)
        .corner_radius(CornerRadius::same(20))
        .inner_margin(Margin::symmetric(9, 3))
        .show(ui, |ui| {
            ui.label(RichText::new(grade).size(11.5).color(text))
        });
}

pub fn pill_button(ui: &mut Ui, palette: &Palette, label: &str, min_width: f32) -> Response {
    ui.add(
        egui::Button::new(RichText::new(label).color(palette.accent_text))
            .fill(palette.accent)
            .corner_radius(CornerRadius::same(20))
            .min_size(Vec2::new(min_width, 34.0)),
    )
}

pub fn secondary_button(ui: &mut Ui, palette: &Palette, label: &str) -> Response {
    ui.add(
        egui::Button::new(RichText::new(label).color(palette.text))
            .fill(palette.card)
            .stroke(Stroke::new(1.0_f32, palette.border))
            .corner_radius(CornerRadius::same(8)),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    Plus,
    Close,
    Heart,
    Search,
    Menu,
    Back,
    Forward,
    Gear,
    More,
    Play,
    Power,
    Download,
    Refresh,
    Check,
    Info,
    Terminal,
}

pub fn icon_button(ui: &mut Ui, palette: &Palette, icon: Icon, tooltip: &str) -> Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::new(30.0, 30.0), Sense::click());
    let painter = ui.painter();
    if response.hovered() {
        painter.rect_filled(rect, CornerRadius::same(8), palette.hover);
    }
    draw_icon(painter, rect, icon, palette.text);
    response.on_hover_text(tooltip)
}

fn draw_icon(painter: &egui::Painter, rect: Rect, icon: Icon, color: Color32) {
    let center = rect.center();
    let stroke = Stroke::new(1.6_f32, color);
    match icon {
        Icon::Plus => {
            painter.line_segment(
                [
                    pos2(center.x - 6.0, center.y),
                    pos2(center.x + 6.0, center.y),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    pos2(center.x, center.y - 6.0),
                    pos2(center.x, center.y + 6.0),
                ],
                stroke,
            );
        }
        Icon::Close => {
            painter.line_segment(
                [
                    pos2(center.x - 5.0, center.y - 5.0),
                    pos2(center.x + 5.0, center.y + 5.0),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    pos2(center.x + 5.0, center.y - 5.0),
                    pos2(center.x - 5.0, center.y + 5.0),
                ],
                stroke,
            );
        }
        Icon::Heart => {
            let left = pos2(center.x - 3.5, center.y - 1.5);
            let right = pos2(center.x + 3.5, center.y - 1.5);
            let fill = Color32::from_rgb(214, 72, 128);
            painter.circle_filled(left, 4.0, fill);
            painter.circle_filled(right, 4.0, fill);
            painter.add(Shape::convex_polygon(
                vec![
                    pos2(center.x - 7.0, center.y),
                    pos2(center.x + 7.0, center.y),
                    pos2(center.x, center.y + 8.0),
                ],
                fill,
                Stroke::NONE,
            ));
        }
        Icon::Search => {
            painter.circle_stroke(pos2(center.x - 2.0, center.y - 2.0), 5.0, stroke);
            painter.line_segment(
                [
                    pos2(center.x + 2.0, center.y + 2.0),
                    pos2(center.x + 7.0, center.y + 7.0),
                ],
                stroke,
            );
        }
        Icon::Menu => {
            for offset in [-5.0, 0.0, 5.0] {
                painter.line_segment(
                    [
                        pos2(center.x - 7.0, center.y + offset),
                        pos2(center.x + 7.0, center.y + offset),
                    ],
                    stroke,
                );
            }
        }
        Icon::Back => {
            painter.line_segment(
                [
                    pos2(center.x + 2.0, center.y - 6.0),
                    pos2(center.x - 4.0, center.y),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    pos2(center.x - 4.0, center.y),
                    pos2(center.x + 2.0, center.y + 6.0),
                ],
                stroke,
            );
        }
        Icon::Forward => {
            painter.line_segment(
                [
                    pos2(center.x - 2.0, center.y - 6.0),
                    pos2(center.x + 4.0, center.y),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    pos2(center.x + 4.0, center.y),
                    pos2(center.x - 2.0, center.y + 6.0),
                ],
                stroke,
            );
        }
        Icon::Gear => {
            painter.circle_stroke(center, 4.0, stroke);
            for index in 0..8 {
                let angle = index as f32 * std::f32::consts::TAU / 8.0;
                let direction = vec2(angle.cos(), angle.sin());
                painter.line_segment([center + direction * 6.0, center + direction * 8.5], stroke);
            }
        }
        Icon::More => {
            for offset in [-5.0, 0.0, 5.0] {
                painter.circle_filled(pos2(center.x, center.y + offset), 1.5, color);
            }
        }
        Icon::Play => {
            painter.add(Shape::convex_polygon(
                vec![
                    pos2(center.x - 4.0, center.y - 6.0),
                    pos2(center.x + 6.0, center.y),
                    pos2(center.x - 4.0, center.y + 6.0),
                ],
                color,
                Stroke::NONE,
            ));
        }
        Icon::Power => {
            painter.circle_stroke(pos2(center.x, center.y + 1.0), 6.0, stroke);
            painter.line_segment(
                [
                    pos2(center.x, center.y - 7.0),
                    pos2(center.x, center.y + 1.0),
                ],
                stroke,
            );
        }
        Icon::Download => {
            painter.line_segment(
                [
                    pos2(center.x, center.y - 6.0),
                    pos2(center.x, center.y + 3.0),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    pos2(center.x - 4.0, center.y - 1.0),
                    pos2(center.x, center.y + 3.0),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    pos2(center.x + 4.0, center.y - 1.0),
                    pos2(center.x, center.y + 3.0),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    pos2(center.x - 5.0, center.y + 6.0),
                    pos2(center.x + 5.0, center.y + 6.0),
                ],
                stroke,
            );
        }
        Icon::Refresh => {
            painter.circle_stroke(center, 6.0, stroke);
            painter.line_segment(
                [
                    pos2(center.x + 4.0, center.y - 5.0),
                    pos2(center.x + 7.0, center.y - 2.0),
                ],
                stroke,
            );
        }
        Icon::Check => {
            painter.line_segment(
                [
                    pos2(center.x - 6.0, center.y),
                    pos2(center.x - 2.0, center.y + 4.0),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    pos2(center.x - 2.0, center.y + 4.0),
                    pos2(center.x + 6.0, center.y - 5.0),
                ],
                stroke,
            );
        }
        Icon::Info => {
            painter.circle_stroke(center, 7.0, stroke);
            painter.text(
                center,
                Align2::CENTER_CENTER,
                "i",
                FontId::proportional(11.0),
                color,
            );
        }
        Icon::Terminal => {
            let inner = Rect::from_min_max(rect.min + vec2(6.0, 6.0), rect.max - vec2(6.0, 6.0));
            painter.rect_filled(inner, CornerRadius::same(3), Color32::TRANSPARENT);
            painter.line_segment(
                [
                    pos2(center.x - 4.0, center.y - 2.0),
                    pos2(center.x - 1.0, center.y),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    pos2(center.x - 1.0, center.y),
                    pos2(center.x - 4.0, center.y + 2.0),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    pos2(center.x + 1.0, center.y + 3.0),
                    pos2(center.x + 5.0, center.y + 3.0),
                ],
                stroke,
            );
        }
    }
}

pub fn switch(ui: &mut Ui, palette: &Palette, value: &mut bool) -> bool {
    let (rect, response) = ui.allocate_exact_size(Vec2::new(40.0, 22.0), Sense::click());
    let painter = ui.painter();
    let track = if *value {
        palette.accent
    } else {
        palette.badge
    };
    painter.rect_filled(rect, CornerRadius::same(11), track);
    let knob_x = if *value {
        rect.right() - 12.0
    } else {
        rect.left() + 12.0
    };
    painter.circle_filled(pos2(knob_x, rect.center().y), 8.0, Color32::WHITE);
    if response.clicked() {
        *value = !*value;
        true
    } else {
        false
    }
}

pub fn clickable_row(
    ui: &mut Ui,
    palette: &Palette,
    title: &str,
    subtitle: &str,
    accessory: &str,
) -> Response {
    let height = if subtitle.is_empty() { 44.0 } else { 58.0 };
    let (rect, response) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), height), Sense::click());
    let painter = ui.painter();
    let fill = if response.hovered() {
        palette.hover
    } else {
        palette.card
    };
    painter.rect_filled(rect, CornerRadius::same(8), palette.border);
    let inner = Rect::from_min_max(rect.min + vec2(1.0, 1.0), rect.max - vec2(1.0, 1.0));
    painter.rect_filled(inner, CornerRadius::same(7), fill);
    let title_y = if subtitle.is_empty() {
        rect.center().y
    } else {
        rect.center().y - 8.0
    };
    painter.text(
        pos2(rect.left() + 14.0, title_y),
        Align2::LEFT_CENTER,
        title,
        FontId::proportional(13.5),
        palette.text,
    );
    if !subtitle.is_empty() {
        painter.text(
            pos2(rect.left() + 14.0, rect.center().y + 8.0),
            Align2::LEFT_CENTER,
            subtitle,
            FontId::proportional(11.5),
            palette.subtext,
        );
    }
    if !accessory.is_empty() {
        painter.text(
            pos2(rect.right() - 14.0, rect.center().y),
            Align2::RIGHT_CENTER,
            accessory,
            FontId::proportional(14.0),
            palette.subtext,
        );
    }
    response
}

pub fn centered_title(ui: &mut Ui, title: &str) {
    ui.vertical_centered(|ui| {
        ui.label(RichText::new(title).strong().size(15.0));
    });
}
