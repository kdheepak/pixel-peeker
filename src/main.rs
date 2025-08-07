use device_query::{DeviceQuery, DeviceState, Keycode};
use iced::widget::{Canvas, Column, Container, Row, button, canvas, container, text};
use iced::{
    Background, Border, Color, Element, Length, Point, Rectangle, Renderer, Size, Subscription,
    Task, Theme, mouse, window,
};
use palette::{Hsl, Hsv, IntoColor, Oklch, Srgb};
use std::time::Instant;
use xcap::Monitor;

const PREVIEW_SIZE: u32 = 21;
const CAPTURE_THROTTLE_MS: u128 = 50;
const MAX_COLOR_HISTORY: usize = 10;
const PREVIEW_CANVAS_SIZE: f32 = 168.0;

fn main() -> iced::Result {
    iced::application("Pixel Picker", App::update, App::view)
        .subscription(App::subscription)
        .theme(|_| Theme::Dark)
        .window(create_window_settings())
        .run()
}

fn create_window_settings() -> window::Settings {
    window::Settings {
        size: Size::new(500.0, 500.0),
        position: window::Position::default(),
        min_size: None,
        max_size: None,
        visible: true,
        resizable: true,
        decorations: true,
        transparent: false,
        level: window::Level::AlwaysOnTop,
        icon: None,
        platform_specific: Default::default(),
        exit_on_close_request: true,
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Tick(Instant),
    CopyColor(ColorFormat),
    HistoryColorClicked(Color),
}

#[derive(Debug, Clone)]
pub enum ColorFormat {
    Rgb,
    Hex,
    Hsv,
    Hsl,
    Oklch,
}

#[derive(Debug, Clone)]
struct ColorInfo {
    color: Color,
    position: (i32, i32),
    preview: Option<PreviewData>,
}

#[derive(Debug, Clone)]
struct PreviewData {
    rgb_data: Vec<u8>,
    width: u32,
    height: u32,
}

#[derive(Default)]
struct App {
    current_color: Option<ColorInfo>,
    frozen_color: Option<ColorInfo>,
    input_state: InputState,
    color_history: Vec<Color>,
    last_capture_time: Option<Instant>,
}

#[derive(Default)]
struct InputState {
    space_pressed_last_frame: bool,
    device_state: DeviceState,
}

impl App {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick(_) => {
                self.update_color_picking();
                Task::none()
            }
            Message::CopyColor(format) => {
                if let Some(color_info) = self.get_active_color() {
                    let text = format_color(&color_info.color, &format);
                    iced::clipboard::write(text)
                } else {
                    Task::none()
                }
            }
            Message::HistoryColorClicked(color) => {
                self.frozen_color = Some(ColorInfo {
                    color,
                    position: (0, 0),
                    preview: None,
                });
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let mut content = Column::new().spacing(10).push(self.create_title());

        let (display_x, display_y) = self.get_display_position();
        content = content.push(text(format!("Mouse: ({}, {})", display_x, display_y)));

        if let Some(color_info) = self.get_active_color() {
            let preview_row = self.create_preview_row(color_info);
            content = content.push(preview_row);
        } else {
            content = content.push(text("No preview available - checking monitors..."));
        }

        content = content.push(self.create_status_text());

        if !self.color_history.is_empty() {
            content = content.push(self.create_history_section());
        }

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .style(if self.is_frozen() {
                |_: &Theme| container::Style {
                    background: Some(Background::Color(Color::from_rgb(0.1, 0.1, 0.2))),
                    ..Default::default()
                }
            } else {
                |_: &Theme| container::Style {
                    background: Some(Background::Color(Color::from_rgb(0.05, 0.05, 0.05))),
                    ..Default::default()
                }
            })
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        iced::time::every(std::time::Duration::from_millis(33)).map(Message::Tick)
    }

    fn update_color_picking(&mut self) {
        let input_event = self.process_input();
        let mouse_pos = self.get_mouse_position();

        match input_event {
            InputEvent::Freeze => {
                self.handle_freeze(mouse_pos);
                return;
            }
            InputEvent::Unfreeze => {
                self.frozen_color = None;
                return;
            }
            InputEvent::None => {}
        }

        if self.is_frozen() || !self.should_capture() {
            return;
        }

        self.capture_at_position(mouse_pos);
    }

    fn get_active_color(&self) -> Option<&ColorInfo> {
        self.frozen_color.as_ref().or(self.current_color.as_ref())
    }

    fn get_display_position(&self) -> (i32, i32) {
        self.get_active_color()
            .map(|info| info.position)
            .unwrap_or_else(|| self.get_mouse_position())
    }

    fn is_frozen(&self) -> bool {
        self.frozen_color.is_some()
    }

    fn should_capture(&mut self) -> bool {
        let now = Instant::now();
        if let Some(last_capture) = self.last_capture_time {
            if now.duration_since(last_capture).as_millis() < CAPTURE_THROTTLE_MS {
                return false;
            }
        }
        self.last_capture_time = Some(now);
        true
    }

    fn get_mouse_position(&self) -> (i32, i32) {
        let mouse = self.input_state.device_state.get_mouse();
        (mouse.coords.0, mouse.coords.1)
    }

    fn process_input(&mut self) -> InputEvent {
        let keys = self.input_state.device_state.get_keys();
        let space_pressed = keys.contains(&Keycode::Space);
        let esc_pressed = keys.contains(&Keycode::Escape);

        let just_pressed = space_pressed && !self.input_state.space_pressed_last_frame;
        self.input_state.space_pressed_last_frame = space_pressed;

        if just_pressed {
            InputEvent::Freeze
        } else if esc_pressed {
            InputEvent::Unfreeze
        } else {
            InputEvent::None
        }
    }

    fn handle_freeze(&mut self, position: (i32, i32)) {
        if self.is_frozen() {
            // Re-capture at current position before freezing again
            self.frozen_color = None;
            self.capture_at_position(position);
        }

        if let Some(current) = &self.current_color {
            self.frozen_color = Some(current.clone());
            self.add_to_history(current.color);
        }
    }

    fn add_to_history(&mut self, color: Color) {
        if self.color_history.last().copied() != Some(color) {
            self.color_history.push(color);
            if self.color_history.len() > MAX_COLOR_HISTORY {
                self.color_history.remove(0);
            }
        }
    }

    fn capture_at_position(&mut self, position: (i32, i32)) {
        let (x, y) = position;

        if let Ok(monitors) = Monitor::all() {
            for monitor in monitors {
                if let Some(region) = self.calculate_capture_region(&monitor, x, y) {
                    if let Ok(image) =
                        monitor.capture_region(region.x, region.y, region.width, region.height)
                    {
                        let pixel_pos = region.get_pixel_position(x, y);
                        if let Some(color) = extract_color_at(&image, pixel_pos.0, pixel_pos.1) {
                            let preview = create_preview(&image, pixel_pos.0, pixel_pos.1);
                            self.current_color = Some(ColorInfo {
                                color,
                                position,
                                preview,
                            });
                        }
                        return;
                    }
                }
            }
        }
    }

    fn calculate_capture_region(&self, monitor: &Monitor, x: i32, y: i32) -> Option<CaptureRegion> {
        let bounds = MonitorBounds::from_monitor(monitor)?;

        if !bounds.contains_point(x, y) {
            return None;
        }

        let relative_pos = bounds.to_relative(x, y);
        let half_size = (PREVIEW_SIZE / 2) as i32;

        let region_x = (relative_pos.0 - half_size).max(0) as u32;
        let region_y = (relative_pos.1 - half_size).max(0) as u32;
        let region_width = PREVIEW_SIZE.min(bounds.width - region_x).max(1);
        let region_height = PREVIEW_SIZE.min(bounds.height - region_y).max(1);

        Some(CaptureRegion {
            x: region_x,
            y: region_y,
            width: region_width,
            height: region_height,
            monitor_bounds: bounds,
        })
    }

    // UI creation methods
    fn create_title(&self) -> Element<'_, Message> {
        text("Pixel Picker")
            .size(20)
            .color(Color::from_rgb(1.0, 1.0, 0.8))
            .into()
    }

    fn create_preview_row(&self, color_info: &ColorInfo) -> Element<'_, Message> {
        let preview_canvas: Element<'_, Message> = if let Some(preview) = &color_info.preview {
            Canvas::new(PreviewRenderer {
                rgb_data: preview.rgb_data.clone(),
                width: preview.width,
                height: preview.height,
            })
            .width(Length::Fixed(PREVIEW_CANVAS_SIZE))
            .height(Length::Fixed(PREVIEW_CANVAS_SIZE))
            .into()
        } else {
            Canvas::new(EmptyRenderer)
                .width(Length::Fixed(PREVIEW_CANVAS_SIZE))
                .height(Length::Fixed(PREVIEW_CANVAS_SIZE))
                .into()
        };

        let info_column = self.create_color_info_column(color_info);

        Row::new()
            .spacing(20)
            .push(preview_canvas)
            .push(info_column)
            .into()
    }

    fn create_color_info_column(&self, color_info: &ColorInfo) -> Element<'_, Message> {
        let mut column = Column::new()
            .spacing(5)
            .push(text("Mouse Position:").color(Color::from_rgb(1.0, 1.0, 0.8)))
            .push(
                text(format!(
                    "({}, {})",
                    color_info.position.0, color_info.position.1
                ))
                .size(14),
            )
            .push(text("Picked Color:").color(Color::from_rgb(1.0, 1.0, 0.8)))
            .push(self.create_color_swatch(color_info.color));

        // Add color format rows
        for format in [
            ColorFormat::Rgb,
            ColorFormat::Hex,
            ColorFormat::Hsv,
            ColorFormat::Hsl,
            ColorFormat::Oklch,
        ] {
            column = column.push(self.create_color_row(&color_info.color, format));
        }

        column.into()
    }

    fn create_color_swatch(&self, color: Color) -> Element<'_, Message> {
        container(text("   "))
            .style(move |_theme: &Theme| container::Style {
                background: Some(Background::Color(color)),
                border: Border {
                    color: Color::from_rgb(0.5, 0.5, 0.5),
                    width: 1.0,
                    radius: 4.0.into(),
                },
                shadow: Default::default(),
                text_color: None,
            })
            .width(Length::Fixed(60.0))
            .height(Length::Fixed(30.0))
            .into()
    }

    fn create_color_row(&self, color: &Color, format: ColorFormat) -> Element<'_, Message> {
        let label = format_color(color, &format);

        Row::new()
            .spacing(10)
            .push(text(label).width(Length::Fill))
            .push(button("Copy").on_press(Message::CopyColor(format)))
            .into()
    }

    fn create_status_text(&self) -> Element<'_, Message> {
        let (status_text, status_color) = if self.is_frozen() {
            (
                "Frozen (press ESC to unfreeze)",
                Color::from_rgb(0.4, 0.7, 1.0),
            )
        } else {
            (
                "Live (press SPACE to freeze)",
                Color::from_rgb(0.4, 1.0, 0.6),
            )
        };

        text(status_text).color(status_color).into()
    }

    fn create_history_section(&self) -> Element<'_, Message> {
        let mut history_row = Row::new().spacing(5);

        for &color in &self.color_history {
            let color_button = button(text("   "))
                .on_press(Message::HistoryColorClicked(color))
                .style(move |_theme: &Theme, _status| button::Style {
                    background: Some(Background::Color(color)),
                    border: Border {
                        color: Color::from_rgb(0.5, 0.5, 0.5),
                        width: 1.0,
                        radius: 3.0.into(),
                    },
                    shadow: Default::default(),
                    text_color: Color::BLACK,
                })
                .width(Length::Fixed(24.0))
                .height(Length::Fixed(18.0));
            history_row = history_row.push(color_button);
        }

        Column::new()
            .push(text("Color History:").color(Color::from_rgb(1.0, 1.0, 0.8)))
            .push(history_row)
            .into()
    }
}

#[derive(Debug)]
enum InputEvent {
    Freeze,
    Unfreeze,
    None,
}

#[derive(Debug, Clone)]
struct MonitorBounds {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
}

impl MonitorBounds {
    fn from_monitor(monitor: &Monitor) -> Option<Self> {
        Some(Self {
            x: monitor.x().ok()?,
            y: monitor.y().ok()?,
            width: monitor.width().ok()?,
            height: monitor.height().ok()?,
        })
    }

    fn contains_point(&self, x: i32, y: i32) -> bool {
        x >= self.x
            && x < self.x + self.width as i32
            && y >= self.y
            && y < self.y + self.height as i32
    }

    fn to_relative(&self, x: i32, y: i32) -> (i32, i32) {
        (x - self.x, y - self.y)
    }
}

#[derive(Debug)]
struct CaptureRegion {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    monitor_bounds: MonitorBounds,
}

impl CaptureRegion {
    fn get_pixel_position(&self, screen_x: i32, screen_y: i32) -> (u32, u32) {
        let relative = self.monitor_bounds.to_relative(screen_x, screen_y);
        let x = (relative.0 - self.x as i32)
            .max(0)
            .min(self.width as i32 - 1) as u32;
        let y = (relative.1 - self.y as i32)
            .max(0)
            .min(self.height as i32 - 1) as u32;
        (x, y)
    }
}

fn extract_color_at(image: &xcap::image::RgbaImage, x: u32, y: u32) -> Option<Color> {
    if x < image.width() && y < image.height() {
        let pixel = image.get_pixel(x, y);
        Some(Color::from_rgb(
            pixel[0] as f32 / 255.0,
            pixel[1] as f32 / 255.0,
            pixel[2] as f32 / 255.0,
        ))
    } else {
        None
    }
}

fn create_preview(
    image: &xcap::image::RgbaImage,
    center_x: u32,
    center_y: u32,
) -> Option<PreviewData> {
    let half_size = (PREVIEW_SIZE / 2) as i32;
    let mut rgb_data = Vec::with_capacity((PREVIEW_SIZE * PREVIEW_SIZE * 3) as usize);

    for dy in -half_size..=half_size {
        for dx in -half_size..=half_size {
            let sample_x = center_x as i32 + dx;
            let sample_y = center_y as i32 + dy;

            let pixel_data = if sample_x >= 0
                && sample_y >= 0
                && sample_x < image.width() as i32
                && sample_y < image.height() as i32
            {
                let pixel = image.get_pixel(sample_x as u32, sample_y as u32);
                [pixel[0], pixel[1], pixel[2]]
            } else {
                [0, 0, 0] // Black for areas outside the captured region
            };

            rgb_data.extend_from_slice(&pixel_data);
        }
    }

    Some(PreviewData {
        rgb_data,
        width: PREVIEW_SIZE,
        height: PREVIEW_SIZE,
    })
}

fn format_color(color: &Color, format: &ColorFormat) -> String {
    let srgb = Srgb::new(color.r, color.g, color.b);

    match format {
        ColorFormat::Rgb => {
            let r = (color.r * 255.0) as u8;
            let g = (color.g * 255.0) as u8;
            let b = (color.b * 255.0) as u8;
            format!("RGB: ({}, {}, {})", r, g, b)
        }
        ColorFormat::Hex => {
            let r = (color.r * 255.0) as u8;
            let g = (color.g * 255.0) as u8;
            let b = (color.b * 255.0) as u8;
            format!("HEX: #{:02X}{:02X}{:02X}", r, g, b)
        }
        ColorFormat::Hsv => {
            let hsv: Hsv = srgb.into_color();
            format!(
                "HSV: ({:.0}°, {:.0}%, {:.0}%)",
                hsv.hue.into_positive_degrees(),
                hsv.saturation * 100.0,
                hsv.value * 100.0
            )
        }
        ColorFormat::Hsl => {
            let hsl: Hsl = srgb.into_color();
            format!(
                "HSL: ({:.0}°, {:.0}%, {:.0}%)",
                hsl.hue.into_positive_degrees(),
                hsl.saturation * 100.0,
                hsl.lightness * 100.0
            )
        }
        ColorFormat::Oklch => {
            let oklch: Oklch = srgb.into_color();
            format!(
                "OKLCH: (L: {:.2}, C: {:.2}, h: {:.1}°)",
                oklch.l,
                oklch.chroma,
                oklch.hue.into_positive_degrees()
            )
        }
    }
}

struct PreviewRenderer {
    rgb_data: Vec<u8>,
    width: u32,
    height: u32,
}

impl<Message> canvas::Program<Message> for PreviewRenderer {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<iced::widget::canvas::Geometry> {
        let mut frame = iced::widget::canvas::Frame::new(renderer, bounds.size());
        let cell_size = bounds.width / self.width as f32;

        for y in 0..self.height {
            for x in 0..self.width {
                let idx = (y * self.width + x) as usize * 3;
                if idx + 2 < self.rgb_data.len() {
                    let color = Color::from_rgb(
                        self.rgb_data[idx] as f32 / 255.0,
                        self.rgb_data[idx + 1] as f32 / 255.0,
                        self.rgb_data[idx + 2] as f32 / 255.0,
                    );

                    let cell_rect = Rectangle::new(
                        Point::new(x as f32 * cell_size, y as f32 * cell_size),
                        Size::new(cell_size, cell_size),
                    );

                    frame.fill_rectangle(cell_rect.position(), cell_rect.size(), color);

                    // Draw crosshair at center
                    if x == self.width / 2 && y == self.height / 2 {
                        self.draw_crosshair(&mut frame, cell_rect, cell_size);
                    }
                }
            }
        }

        vec![frame.into_geometry()]
    }
}

impl PreviewRenderer {
    fn draw_crosshair(
        &self,
        frame: &mut iced::widget::canvas::Frame,
        cell_rect: Rectangle,
        cell_size: f32,
    ) {
        let center = cell_rect.center();
        let half = cell_size / 2.0;
        let stroke = iced::widget::canvas::Stroke::default()
            .with_color(Color::WHITE)
            .with_width(2.0);

        // Vertical line
        frame.stroke(
            &iced::widget::canvas::Path::line(
                Point::new(center.x, center.y - half),
                Point::new(center.x, center.y + half),
            ),
            stroke,
        );

        // Horizontal line
        frame.stroke(
            &iced::widget::canvas::Path::line(
                Point::new(center.x - half, center.y),
                Point::new(center.x + half, center.y),
            ),
            stroke,
        );
    }
}

struct EmptyRenderer;

impl<Message> canvas::Program<Message> for EmptyRenderer {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<iced::widget::canvas::Geometry> {
        let frame = iced::widget::canvas::Frame::new(renderer, bounds.size());
        vec![frame.into_geometry()]
    }
}
