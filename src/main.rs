use device_query::{DeviceQuery, DeviceState, Keycode};
use iced::widget::{Canvas, Column, Container, Row, button, canvas, container, text};
use iced::{
    Background, Border, Color, Element, Length, Point, Rectangle, Renderer, Size, Subscription, Task, Theme, mouse,
    window,
};
use palette::{Hsl, Hsv, IntoColor, Oklch, Srgb};
use serde::{Deserialize, Serialize};
use std::time::Instant;
use xcap::Monitor;

const PREVIEW_SIZE: u32 = 21;
const MAX_COLOR_HISTORY: usize = 10;
const PREVIEW_CANVAS_SIZE: f32 = 168.0;

fn main() -> iced::Result {
    let settings = Settings::load();
    iced::application("Pixel Peeker", App::update, App::view)
        .subscription(App::subscription)
        .theme(|_| Theme::Dark)
        .window(create_window_settings(&settings))
        .run_with(move || (App::new(settings), Task::none()))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Settings {
    window_width: f32,
    window_height: f32,
    window_x: Option<i32>,
    window_y: Option<i32>,
    color_history: Vec<SerializableColor>,
    zoom_factor: f32,
    always_on_top: bool,

    #[serde(skip)]
    path: Option<std::path::PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SerializableColor {
    r: f32,
    g: f32,
    b: f32,
}

impl From<Color> for SerializableColor {
    fn from(color: Color) -> Self {
        Self { r: color.r, g: color.g, b: color.b }
    }
}

impl From<SerializableColor> for Color {
    fn from(color: SerializableColor) -> Self {
        Color::from_rgb(color.r, color.g, color.b)
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            window_width: 600.0,
            window_height: 500.0,
            window_x: None,
            window_y: None,
            color_history: Vec::new(),
            zoom_factor: 1.0,
            always_on_top: true,
            path: None,
        }
    }
}

impl Settings {
    fn load() -> Self {
        if let Some(settings_path) = Self::get_settings_path() {
            if let Ok(contents) = std::fs::read_to_string(&settings_path) {
                if let Ok(mut settings) = serde_json::from_str::<Settings>(&contents) {
                    settings.path = Some(settings_path);
                    return settings;
                }
            }
        }
        Self::default()
    }

    fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let settings_path = Self::get_settings_path().ok_or("Could not determine settings directory")?;

        if let Some(parent) = settings_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create settings directory: {}", e))?;
        }

        let contents = serde_json::to_string_pretty(self)?;
        std::fs::write(&settings_path, contents).map_err(|e| format!("Failed to write settings file: {}", e))?;

        Ok(())
    }

    fn get_settings_path() -> Option<std::path::PathBuf> {
        if let Some(project_dir) = directories::ProjectDirs::from("com", "kdheepak", "pixel-peeker") {
            return Some(project_dir.config_dir().join("pixel-peeker.json"));
        }

        if let Some(base_dir) = directories::BaseDirs::new() {
            return Some(base_dir.config_dir().join("pixel-peeker").join("pixel-peeker.json"));
        }

        if let Some(user_dir) = directories::UserDirs::new() {
            return Some(user_dir.home_dir().join(".config").join("pixel-peeker").join("pixel-peeker.json"));
        }

        if let Ok(config_dir) = std::env::var("XDG_CONFIG_HOME") {
            return Some(std::path::PathBuf::from(config_dir).join("pixel-peeker").join("pixel-peeker.json"));
        }

        Some(std::path::PathBuf::from("pixel-peeker.json"))
    }
}

fn create_window_settings(settings: &Settings) -> window::Settings {
    let position = if let (Some(x), Some(y)) = (settings.window_x, settings.window_y) {
        window::Position::Specific(iced::Point::new(x as f32, y as f32))
    } else {
        window::Position::default()
    };

    window::Settings {
        size: Size::new(settings.window_width, settings.window_height),
        position,
        min_size: Some(Size::new(400.0, 300.0)),
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
    ZoomFactor(f32),
    WindowResized(Size),
    WindowMoved(iced::Point),
    ToggleAlwaysOnTop,
    ClearHistory,
    SaveSettings,
    WindowEvent(window::Event),
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
struct InputState {
    space_pressed_last_frame: bool,
    device_state: DeviceState,
}

struct App {
    current_color: Option<ColorInfo>,
    frozen_color: Option<ColorInfo>,
    input_state: InputState,
    color_history: Vec<Color>,
    zoom_factor: f32,
    settings: Settings,
    settings_dirty: bool,
    last_save_time: Instant,
}

impl App {
    fn new(settings: Settings) -> Self {
        let color_history: Vec<Color> = settings.color_history.iter().map(|c| Color::from(c.clone())).collect();

        Self {
            current_color: None,
            frozen_color: None,
            input_state: InputState::default(),
            color_history,
            zoom_factor: settings.zoom_factor,
            settings,
            settings_dirty: false,
            last_save_time: Instant::now(),
        }
    }

    fn update_settings(&mut self) {
        self.settings.color_history = self.color_history.iter().map(|c| SerializableColor::from(*c)).collect();
        self.settings.zoom_factor = self.zoom_factor;
        self.settings_dirty = true;
    }

    fn save_settings_if_dirty(&mut self) {
        if self.settings_dirty {
            if let Err(e) = self.settings.save() {
                eprintln!("Failed to save settings: {}", e);
            }
            self.settings_dirty = false;
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ZoomFactor(zoom_factor) => {
                self.zoom_factor = zoom_factor;
                self.update_settings();
                Task::none()
            },
            Message::WindowResized(size) => {
                self.settings.window_width = size.width;
                self.settings.window_height = size.height;
                self.settings_dirty = true;
                Task::none()
            },
            Message::WindowMoved(position) => {
                self.settings.window_x = Some(position.x as i32);
                self.settings.window_y = Some(position.y as i32);
                self.settings_dirty = true;
                Task::none()
            },
            Message::WindowEvent(event) => {
                match event {
                    window::Event::Resized(size) => {
                        return self.update(Message::WindowResized(size));
                    },
                    window::Event::Moved(position) => {
                        return self.update(Message::WindowMoved(position));
                    },
                    window::Event::CloseRequested => {
                        self.save_settings_if_dirty();
                        if let Err(e) = self.settings.save() {
                            eprintln!("Final save failed: {}", e);
                        }
                    },
                    _ => {},
                }
                Task::none()
            },
            Message::ToggleAlwaysOnTop => {
                self.settings.always_on_top = !self.settings.always_on_top;
                self.settings_dirty = true;
                Task::none()
            },
            Message::ClearHistory => {
                self.color_history.clear();
                self.update_settings();
                self.save_settings_if_dirty();
                Task::none()
            },
            Message::SaveSettings => {
                self.save_settings_if_dirty();
                Task::none()
            },
            Message::Tick(now) => {
                self.update_color_picking();
                if self.settings_dirty && now.duration_since(self.last_save_time).as_secs() >= 5 {
                    self.save_settings_if_dirty();
                }
                Task::none()
            },
            Message::CopyColor(format) => {
                if let Some(color_info) = self.get_active_color() {
                    let text = format_color(&color_info.color, &format);
                    iced::clipboard::write(text)
                } else {
                    Task::none()
                }
            },
            Message::HistoryColorClicked(color) => {
                self.frozen_color = Some(ColorInfo { color, position: (0, 0), preview: None });
                Task::none()
            },
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
                    background: Some(Background::Color(Color::from_rgb(0.05, 0.05, 0.05))),
                    ..Default::default()
                }
            } else {
                |_: &Theme| container::Style {
                    background: Some(Background::Color(Color::from_rgb(0.1, 0.1, 0.2))),
                    ..Default::default()
                }
            })
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([iced::time::every(std::time::Duration::from_millis(33)).map(Message::Tick)])
    }

    fn update_color_picking(&mut self) {
        let input_event = self.process_input();
        let mouse_pos = self.get_mouse_position();

        match input_event {
            InputEvent::Freeze => {
                self.handle_freeze(mouse_pos);
                return;
            },
            InputEvent::Unfreeze => {
                self.frozen_color = None;
                return;
            },
            InputEvent::None => {},
        }

        if self.is_frozen() {
            return;
        }

        self.capture_at_position(mouse_pos);
    }

    fn get_active_color(&self) -> Option<&ColorInfo> {
        self.frozen_color.as_ref().or(self.current_color.as_ref())
    }

    fn get_display_position(&self) -> (i32, i32) {
        self.get_active_color().map(|info| info.position).unwrap_or_else(|| self.get_mouse_position())
    }

    fn is_frozen(&self) -> bool {
        self.frozen_color.is_some()
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
            self.frozen_color = None;
            self.capture_at_position(position);
        }

        if let Some(current) = &self.current_color {
            self.frozen_color = Some(current.clone());
            self.add_to_history(current.color);
            self.save_settings_if_dirty();
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
                        monitor.capture_region(region.x as u32, region.y as u32, region.width, region.height)
                    {
                        let center_x = PREVIEW_SIZE / 2 - region.offset_x;
                        let center_y = PREVIEW_SIZE / 2 - region.offset_y;

                        if let Some(color) = extract_color_at(&image, center_x, center_y) {
                            let preview = create_preview(&image, center_x, center_y);
                            self.current_color = Some(ColorInfo { color, position, preview });
                        }
                        return;
                    }
                }
            }
        }
    }

    fn calculate_capture_region(&self, monitor: &Monitor, x: i32, y: i32) -> Option<CaptureRegion> {
        let bounds = MonitorBounds::from_monitor(monitor)?;

        let half_size = (PREVIEW_SIZE / 2) as i32;

        let region_x = x - half_size;
        let region_y = y - half_size;

        let clamped_x = region_x.max(bounds.x).min(bounds.x + bounds.width as i32 - PREVIEW_SIZE as i32);
        let clamped_y = region_y.max(bounds.y).min(bounds.y + bounds.height as i32 - PREVIEW_SIZE as i32);

        let offset_x = (clamped_x - region_x).max(0) as u32;
        let offset_y = (clamped_y - region_y).max(0) as u32;

        Some(CaptureRegion {
            x: clamped_x,
            y: clamped_y,
            width: PREVIEW_SIZE,
            height: PREVIEW_SIZE,
            offset_x,
            offset_y,
        })
    }

    fn create_title(&self) -> Element<'_, Message> {
        text("Pixel Peeker").size(20).color(Color::from_rgb(1.0, 1.0, 0.8)).into()
    }

    fn create_preview_row(&self, color_info: &ColorInfo) -> Element<'_, Message> {
        let preview_canvas: Element<'_, Message> = if let Some(preview) = &color_info.preview {
            Canvas::new(PreviewRenderer {
                rgb_data: preview.rgb_data.clone(),
                width: preview.width,
                height: preview.height,
                zoom_factor: self.zoom_factor,
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

        let preview_with_shadow: Element<'_, Message> = Container::new(preview_canvas)
            .style(|_theme: &Theme| container::Style {
                shadow: iced::Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.5),
                    offset: iced::Vector::new(4.0, 4.0),
                    blur_radius: 8.0,
                },
                border: Border { color: Color::from_rgb(0.3, 0.3, 0.3), width: 1.0, radius: 6.0.into() },
                background: Some(Background::Color(Color::from_rgb(0.1, 0.1, 0.1))),
                ..Default::default()
            })
            .padding(4)
            .into();

        let zoom_slider = self.create_zoom_slider();

        let info_column = self.create_color_info_column(color_info);

        Row::new().spacing(20).push(Column::new().push(preview_with_shadow).push(zoom_slider)).push(info_column).into()
    }

    fn create_color_info_column(&self, color_info: &ColorInfo) -> Element<'_, Message> {
        let mut column = Column::new()
            .spacing(5)
            .push(text("Mouse Position:").color(Color::from_rgb(1.0, 1.0, 0.8)))
            .push(text(format!("({}, {})", color_info.position.0, color_info.position.1)).size(14))
            .push(text("Picked Color:").color(Color::from_rgb(1.0, 1.0, 0.8)))
            .push(self.create_color_swatch(color_info.color));

        for format in [ColorFormat::Rgb, ColorFormat::Hex, ColorFormat::Hsv, ColorFormat::Hsl, ColorFormat::Oklch] {
            column = column.push(self.create_color_row(&color_info.color, format));
        }

        column.into()
    }

    fn create_color_swatch(&self, color: Color) -> Element<'_, Message> {
        container(text("   "))
            .style(move |_theme: &Theme| container::Style {
                background: Some(Background::Color(color)),
                border: Border { color: Color::from_rgb(0.5, 0.5, 0.5), width: 1.0, radius: 4.0.into() },
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

    fn create_zoom_slider(&self) -> Element<'_, Message> {
        let zoom_ui = Column::new()
            .spacing(10)
            .push(iced::widget::Text::new(format!("Zoom: {:.1}×", self.zoom_factor)))
            .push(iced::widget::slider(1.0..=5.0, self.zoom_factor, Message::ZoomFactor).step(0.1));
        zoom_ui.into()
    }

    fn create_status_text(&self) -> Element<'_, Message> {
        let (status_text, status_color) = if self.is_frozen() {
            ("Frozen (press ESC to unfreeze)", Color::from_rgb(0.4, 0.7, 1.0))
        } else {
            ("Live (press SPACE to freeze)", Color::from_rgb(0.4, 1.0, 0.6))
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
                    border: Border { color: Color::from_rgb(0.5, 0.5, 0.5), width: 1.0, radius: 3.0.into() },
                    shadow: Default::default(),
                    text_color: Color::BLACK,
                })
                .width(Length::Fixed(24.0))
                .height(Length::Fixed(18.0));
            history_row = history_row.push(color_button);
        }

        Column::new().push(text("Color History:").color(Color::from_rgb(1.0, 1.0, 0.8))).push(history_row).into()
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
}

#[derive(Debug)]
struct CaptureRegion {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    offset_x: u32,
    offset_y: u32,
}

fn extract_color_at(image: &xcap::image::RgbaImage, x: u32, y: u32) -> Option<Color> {
    if x < image.width() && y < image.height() {
        let pixel = image.get_pixel(x, y);
        Some(Color::from_rgb(pixel[0] as f32 / 255.0, pixel[1] as f32 / 255.0, pixel[2] as f32 / 255.0))
    } else {
        None
    }
}

fn create_preview(image: &xcap::image::RgbaImage, center_x: u32, center_y: u32) -> Option<PreviewData> {
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
                [0, 0, 0]
            };

            rgb_data.extend_from_slice(&pixel_data);
        }
    }

    Some(PreviewData { rgb_data, width: PREVIEW_SIZE, height: PREVIEW_SIZE })
}

fn format_color(color: &Color, format: &ColorFormat) -> String {
    let r = (color.r * 255.0).round() as u8;
    let g = (color.g * 255.0).round() as u8;
    let b = (color.b * 255.0).round() as u8;

    match format {
        ColorFormat::Rgb => format!("rgb({}, {}, {})", r, g, b),
        ColorFormat::Hex => format!("#{:02X}{:02X}{:02X}", r, g, b),
        ColorFormat::Hsv => {
            let hsv: Hsv = Srgb::new(color.r, color.g, color.b).into_color();
            format!(
                "hsv({:.0}deg, {:.0}%, {:.0}%)",
                hsv.hue.into_positive_degrees(),
                hsv.saturation * 100.0,
                hsv.value * 100.0
            )
        },
        ColorFormat::Hsl => {
            let hsl: Hsl = Srgb::new(color.r, color.g, color.b).into_color();
            format!(
                "hsl({:.0}deg, {:.0}%, {:.0}%)",
                hsl.hue.into_positive_degrees(),
                hsl.saturation * 100.0,
                hsl.lightness * 100.0
            )
        },
        ColorFormat::Oklch => {
            let oklch: Oklch = Srgb::new(color.r, color.g, color.b).into_color();
            format!("oklch({:.2} {:.2} {:.1}deg)", oklch.l, oklch.chroma, oklch.hue.into_positive_degrees())
        },
    }
}

struct PreviewRenderer {
    rgb_data: Vec<u8>,
    width: u32,
    height: u32,
    zoom_factor: f32,
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

        let base_cell_size = bounds.width / self.width as f32;
        let zoomed_cell_size = base_cell_size * self.zoom_factor;

        let total_grid_width = self.width as f32 * zoomed_cell_size;
        let total_grid_height = self.height as f32 * zoomed_cell_size;

        let offset_x = (bounds.width - total_grid_width) / 2.0;
        let offset_y = (bounds.height - total_grid_height) / 2.0;

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
                        Point::new(offset_x + x as f32 * zoomed_cell_size, offset_y + y as f32 * zoomed_cell_size),
                        Size::new(zoomed_cell_size, zoomed_cell_size),
                    );

                    frame.fill_rectangle(cell_rect.position(), cell_rect.size(), color);

                    if x == self.width / 2 && y == self.height / 2 {
                        self.draw_crosshair(&mut frame, cell_rect, zoomed_cell_size);
                    }
                }
            }
        }

        vec![frame.into_geometry()]
    }
}

impl PreviewRenderer {
    fn draw_crosshair(&self, frame: &mut iced::widget::canvas::Frame, cell_rect: Rectangle, cell_size: f32) {
        let center = cell_rect.center();
        let half = cell_size / 2.0;

        let bg_stroke = iced::widget::canvas::Stroke::default().with_color(Color::WHITE).with_width(4.0);

        let fg_stroke = iced::widget::canvas::Stroke::default().with_color(Color::BLACK).with_width(2.0);

        frame.stroke(
            &iced::widget::canvas::Path::line(
                Point::new(center.x, center.y - half),
                Point::new(center.x, center.y + half),
            ),
            bg_stroke,
        );
        frame.stroke(
            &iced::widget::canvas::Path::line(
                Point::new(center.x - half, center.y),
                Point::new(center.x + half, center.y),
            ),
            bg_stroke,
        );

        frame.stroke(
            &iced::widget::canvas::Path::line(
                Point::new(center.x, center.y - half),
                Point::new(center.x, center.y + half),
            ),
            fg_stroke,
        );
        frame.stroke(
            &iced::widget::canvas::Path::line(
                Point::new(center.x - half, center.y),
                Point::new(center.x + half, center.y),
            ),
            fg_stroke,
        );

        let dot_radius = 2.0;
        frame.fill(&iced::widget::canvas::Path::circle(center, dot_radius), Color::WHITE);
        frame.fill(&iced::widget::canvas::Path::circle(center, dot_radius - 0.5), Color::BLACK);
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
