use std::fs;
use std::sync::OnceLock;

use ab_glyph::{Font, FontRef, PxScale, ScaleFont as _};
use chrono::{DateTime, Datelike, Local, Timelike};
use image::{DynamicImage, Rgba, RgbaImage};

pub const DATE_KEY_INDEX: u8 = 3;
pub const TIME_KEY_INDEX: u8 = 4;

const KEY_IMAGE_WIDTH: u32 = 72;
const KEY_IMAGE_HEIGHT: u32 = 72;
const KEY_CONTENT_PADDING_PIXELS: u32 = 2;
const LINE_SPACING_FRACTION: f32 = 0.12;
const BACKGROUND_COLOR: Rgba<u8> = Rgba([8, 10, 18, 255]);
const FOREGROUND_COLOR: Rgba<u8> = Rgba([230, 240, 255, 255]);
const FONT_CANDIDATE_PATHS: &[&str] = &[
    "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
    "/usr/share/fonts/truetype/liberation/LiberationSans-Bold.ttf",
    "/usr/share/fonts/truetype/dejavu/DejaVuSansMono-Bold.ttf",
];

static LOADED_FONT_BYTES: OnceLock<Vec<u8>> = OnceLock::new();

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClockSnapshot {
    pub date_year_line: String,
    pub date_month_line: String,
    pub date_day_line: String,
    pub hour_minute_line: String,
    pub second_line: String,
    pub timezone_line: String,
}

impl ClockSnapshot {
    pub fn from_local_now() -> Self {
        let now = Local::now();
        Self::from_datetime(now)
    }

    pub fn from_datetime(now: DateTime<Local>) -> Self {
        Self {
            date_year_line: format!("{:04}", now.year()),
            date_month_line: format!("{:02}", now.month()),
            date_day_line: format!("{:02}", now.day()),
            hour_minute_line: format!("{:02}:{:02}", now.hour(), now.minute()),
            second_line: format!("{:02}", now.second()),
            timezone_line: format_timezone_label(now),
        }
    }

    pub fn date_lines(&self) -> [&str; 3] {
        [
            self.date_year_line.as_str(),
            self.date_month_line.as_str(),
            self.date_day_line.as_str(),
        ]
    }

    pub fn time_lines(&self) -> [&str; 3] {
        [
            self.hour_minute_line.as_str(),
            self.second_line.as_str(),
            self.timezone_line.as_str(),
        ]
    }
}

pub fn render_date_key_image(snapshot: &ClockSnapshot) -> Result<DynamicImage, String> {
    render_centered_multiline_key(snapshot.date_lines().as_slice())
}

pub fn render_time_key_image(snapshot: &ClockSnapshot) -> Result<DynamicImage, String> {
    render_centered_multiline_key(snapshot.time_lines().as_slice())
}

fn format_timezone_label(now: DateTime<Local>) -> String {
    let abbreviated = now.format("%Z").to_string();
    if !abbreviated.is_empty() && abbreviated != now.format("%z").to_string() {
        return abbreviated;
    }
    now.format("%:z").to_string()
}

fn render_centered_multiline_key(lines: &[&str]) -> Result<DynamicImage, String> {
    let font = load_font()?;
    let available_width = KEY_IMAGE_WIDTH.saturating_sub(KEY_CONTENT_PADDING_PIXELS * 2) as f32;
    let available_height = KEY_IMAGE_HEIGHT.saturating_sub(KEY_CONTENT_PADDING_PIXELS * 2) as f32;
    let font_size = largest_font_size_that_fits(&font, lines, available_width, available_height);
    let scale = PxScale::from(font_size);
    let scaled_font = font.as_scaled(scale);
    let line_height = scaled_font.height();
    let line_gap = line_height * LINE_SPACING_FRACTION;
    let total_text_height =
        line_height * lines.len() as f32 + line_gap * (lines.len().saturating_sub(1) as f32);

    let mut canvas = RgbaImage::from_pixel(KEY_IMAGE_WIDTH, KEY_IMAGE_HEIGHT, BACKGROUND_COLOR);
    let mut cursor_y =
        (KEY_IMAGE_HEIGHT as f32 - total_text_height) / 2.0 + scaled_font.ascent();

    for line in lines {
        let line_width = measure_line_width(&scaled_font, line);
        let cursor_x = (KEY_IMAGE_WIDTH as f32 - line_width) / 2.0;
        draw_text_line(&mut canvas, &font, scale, cursor_x, cursor_y, line);
        cursor_y += line_height + line_gap;
    }

    Ok(DynamicImage::ImageRgba8(canvas))
}

fn largest_font_size_that_fits(
    font: &FontRef<'_>,
    lines: &[&str],
    available_width: f32,
    available_height: f32,
) -> f32 {
    let mut low = 6.0_f32;
    let mut high = available_height;
    let mut best = low;

    for _ in 0..24 {
        let mid = (low + high) / 2.0;
        if text_block_fits(font, lines, mid, available_width, available_height) {
            best = mid;
            low = mid;
        } else {
            high = mid;
        }
    }

    best.floor().max(6.0)
}

fn text_block_fits(
    font: &FontRef<'_>,
    lines: &[&str],
    font_size: f32,
    available_width: f32,
    available_height: f32,
) -> bool {
    let scale = PxScale::from(font_size);
    let scaled_font = font.as_scaled(scale);
    let line_height = scaled_font.height();
    let line_gap = line_height * LINE_SPACING_FRACTION;
    let total_height =
        line_height * lines.len() as f32 + line_gap * (lines.len().saturating_sub(1) as f32);
    if total_height > available_height {
        return false;
    }

    lines
        .iter()
        .all(|line| measure_line_width(&scaled_font, line) <= available_width)
}

fn measure_line_width(scaled_font: &ab_glyph::PxScaleFont<&FontRef<'_>>, line: &str) -> f32 {
    let mut width = 0.0_f32;
    let mut previous_glyph_id = None;
    for character in line.chars() {
        let glyph_id = scaled_font.font().glyph_id(character);
        if let Some(previous) = previous_glyph_id {
            width += scaled_font.kern(previous, glyph_id);
        }
        width += scaled_font.h_advance(glyph_id);
        previous_glyph_id = Some(glyph_id);
    }
    width
}

fn draw_text_line(
    canvas: &mut RgbaImage,
    font: &FontRef<'_>,
    scale: PxScale,
    origin_x: f32,
    baseline_y: f32,
    line: &str,
) {
    let scaled_font = font.as_scaled(scale);
    let mut cursor_x = origin_x;
    let mut previous_glyph_id = None;

    for character in line.chars() {
        let glyph_id = font.glyph_id(character);
        if let Some(previous) = previous_glyph_id {
            cursor_x += scaled_font.kern(previous, glyph_id);
        }

        let glyph = glyph_id.with_scale_and_position(scale, ab_glyph::point(cursor_x, baseline_y));
        if let Some(outlined) = font.outline_glyph(glyph) {
            let bounds = outlined.px_bounds();
            outlined.draw(|x, y, coverage| {
                if coverage <= 0.0 {
                    return;
                }
                let pixel_x = bounds.min.x as i32 + x as i32;
                let pixel_y = bounds.min.y as i32 + y as i32;
                if pixel_x < 0 || pixel_y < 0 {
                    return;
                }
                let pixel_x = pixel_x as u32;
                let pixel_y = pixel_y as u32;
                if pixel_x >= canvas.width() || pixel_y >= canvas.height() {
                    return;
                }
                let existing = *canvas.get_pixel(pixel_x, pixel_y);
                *canvas.get_pixel_mut(pixel_x, pixel_y) =
                    blend_coverage(existing, FOREGROUND_COLOR, coverage);
            });
        }

        cursor_x += scaled_font.h_advance(glyph_id);
        previous_glyph_id = Some(glyph_id);
    }
}

fn blend_coverage(background: Rgba<u8>, foreground: Rgba<u8>, coverage: f32) -> Rgba<u8> {
    let alpha = coverage.clamp(0.0, 1.0);
    let inverse = 1.0 - alpha;
    Rgba([
        (background[0] as f32 * inverse + foreground[0] as f32 * alpha).round() as u8,
        (background[1] as f32 * inverse + foreground[1] as f32 * alpha).round() as u8,
        (background[2] as f32 * inverse + foreground[2] as f32 * alpha).round() as u8,
        255,
    ])
}

fn load_font() -> Result<FontRef<'static>, String> {
    let font_bytes = LOADED_FONT_BYTES.get_or_init(|| {
        for candidate_path in FONT_CANDIDATE_PATHS {
            if let Ok(bytes) = fs::read(candidate_path) {
                return bytes;
            }
        }
        Vec::new()
    });

    if font_bytes.is_empty() {
        return Err(format!(
            "no usable font found; tried {}",
            FONT_CANDIDATE_PATHS.join(", ")
        ));
    }

    FontRef::try_from_slice(font_bytes.as_slice())
        .map_err(|error| format!("failed to parse font: {error}"))
}

