use device_query::{DeviceQuery, DeviceState, Keycode};
use iced::widget::{button, canvas, container, text, Canvas, Column, Container, Row};
use iced::{
    mouse, Background, Border, Color, Element, Length,
    Point, Rectangle, Size, Subscription, Task, Theme, Renderer,
};
use std::time::Instant;
use xcap::Monitor;

fn main() -> iced::Result {
    iced::application("PixelPicker", PixelPickerApp::update, PixelPickerApp::view)
        .subscription(PixelPickerApp::subscription)
        .theme(|_| Theme::Dark)
        .window_size(Size::new(500.0, 400.0))
        .run()
}

#[derive(Debug, Clone)]
pub enum Message {
    Tick(Instant),
    CopyRgb,
    CopyHex,
    HistoryColorClicked(Color),
}

#[derive(Default)]
struct PixelPickerApp {
    selected_color: Option<Color>,
    preview_image: Option<(Vec<u8>, u32, u32)>,
    frozen_position: Option<(i32, i32)>,
    frozen_color: Option<Color>,
    frozen_preview: Option<(Vec<u8>, u32, u32)>,
    space_pressed_last_frame: bool,
    device_state: DeviceState,
    color_history: Vec<Color>,
}

impl PixelPickerApp {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick(_) => {
                self.update_color_picking();
                Task::none()
            }
            Message::CopyRgb => {
                if let Some(color) = self.selected_color {
                    let rgb_string = format!(
                        "{}, {}, {}",
                        (color.r * 255.0) as u8,
                        (color.g * 255.0) as u8,
                        (color.b * 255.0) as u8
                    );
                    iced::clipboard::write(rgb_string)
                } else {
                    Task::none()
                }
            }
            Message::CopyHex => {
                if let Some(color) = self.selected_color {
                    let hex_string = format!(
                        "#{:02X}{:02X}{:02X}",
                        (color.r * 255.0) as u8,
                        (color.g * 255.0) as u8,
                        (color.b * 255.0) as u8
                    );
                    iced::clipboard::write(hex_string)
                } else {
                    Task::none()
                }
            }
            Message::HistoryColorClicked(color) => {
                let hex_string = format!(
                    "#{:02X}{:02X}{:02X}",
                    (color.r * 255.0) as u8,
                    (color.g * 255.0) as u8,
                    (color.b * 255.0) as u8
                );
                iced::clipboard::write(hex_string)
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let title = text("PixelPicker")
            .size(20)
            .color(Color::from_rgb(1.0, 1.0, 0.8));

        let mut content = Column::new().spacing(10).push(title);

        // Always show debug info about current state
        let (display_x, display_y) = self.frozen_position.unwrap_or_else(|| {
            let mouse = self.device_state.get_mouse();
            (mouse.coords.0, mouse.coords.1)
        });
        
        content = content.push(text(format!("Mouse: ({}, {})", display_x, display_y)));

        if let Some((rgb_data, width, height)) = &self.preview_image {
            let preview_canvas = Canvas::new(PreviewRenderer {
                rgb_data: rgb_data.clone(),
                width: *width,
                height: *height,
            })
            .width(Length::Fixed(168.0))
            .height(Length::Fixed(168.0));

            let mut info_column = Column::new().spacing(5);

            if let Some(color) = self.selected_color {
                info_column = info_column
                    .push(text("Mouse Position:").color(Color::from_rgb(1.0, 1.0, 0.8)))
                    .push(text(format!("({}, {})", display_x, display_y)).size(14))
                    .push(text("Picked Color:").color(Color::from_rgb(1.0, 1.0, 0.8)))
                    .push(
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
                            .height(Length::Fixed(30.0)),
                    )
                    .push(text(format!(
                        "RGB: ({}, {}, {})",
                        (color.r * 255.0) as u8,
                        (color.g * 255.0) as u8,
                        (color.b * 255.0) as u8
                    )))
                    .push(text(format!(
                        "HEX: #{:02X}{:02X}{:02X}",
                        (color.r * 255.0) as u8,
                        (color.g * 255.0) as u8,
                        (color.b * 255.0) as u8
                    )));

                let (h, s, v) = rgb_to_hsv(
                    (color.r * 255.0) as u8,
                    (color.g * 255.0) as u8,
                    (color.b * 255.0) as u8,
                );
                info_column = info_column.push(text(format!(
                    "HSV: ({:.0}°, {:.0}%, {:.0}%)",
                    h,
                    s * 100.0,
                    v * 100.0
                )));

                let button_row = Row::new()
                    .spacing(10)
                    .push(button("Copy RGB").on_press(Message::CopyRgb))
                    .push(button("Copy HEX").on_press(Message::CopyHex));

                info_column = info_column.push(button_row);
            } else {
                info_column = info_column.push(text("No color detected"));
            }

            let preview_row = Row::new()
                .spacing(20)
                .push(preview_canvas)
                .push(info_column);

            content = content.push(preview_row);
        } else {
            content = content.push(text("No preview available - checking monitors..."));
        }

        let is_frozen = self.frozen_position.is_some();
        let status_text = if is_frozen {
            "Frozen (press ESC to unfreeze)"
        } else {
            "Live (press SPACE to freeze)"
        };
        let status_color = if is_frozen {
            Color::from_rgb(0.4, 0.7, 1.0)
        } else {
            Color::from_rgb(0.4, 1.0, 0.6)
        };

        content = content.push(text(status_text).color(status_color));

        if !self.color_history.is_empty() {
            content = content.push(text("Color History:").color(Color::from_rgb(1.0, 1.0, 0.8)));

            let mut history_row = Row::new().spacing(5);
            for color in &self.color_history {
                let color_button = button(text("   "))
                    .on_press(Message::HistoryColorClicked(*color))
                    .style(move |_theme: &Theme, _status| button::Style {
                        background: Some(Background::Color(*color)),
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
            content = content.push(history_row);
        }

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        iced::time::every(std::time::Duration::from_millis(33))
            .map(Message::Tick)
    }

    fn update_color_picking(&mut self) {
        let keys = self.device_state.get_keys();
        let space_pressed = keys.contains(&Keycode::Space);
        let esc_pressed = keys.contains(&Keycode::Escape);

        let mouse = self.device_state.get_mouse();
        let (x, y) = (mouse.coords.0, mouse.coords.1);

        let just_pressed = space_pressed && !self.space_pressed_last_frame;
        self.space_pressed_last_frame = space_pressed;

        // Handle freeze/unfreeze logic first
        if just_pressed {
            self.frozen_position = Some((x, y));
            // Don't immediately capture when freezing, use the last known values
            if let (Some(color), Some(preview)) = (self.selected_color, self.preview_image.clone()) {
                self.frozen_color = Some(color);
                self.frozen_preview = Some(preview);
                if self.color_history.last().copied() != Some(color) {
                    self.color_history.push(color);
                    if self.color_history.len() > 10 {
                        self.color_history.remove(0);
                    }
                }
            }
            return;
        } else if esc_pressed {
            self.frozen_position = None;
            self.frozen_color = None;
            self.frozen_preview = None;
            return;
        }

        // Only update colors when not frozen
        if self.frozen_position.is_some() {
            return;
        }

        // Try to capture screen - but limit frequency to avoid performance issues
        if let Ok(monitors) = Monitor::all() {
            for monitor in monitors {
                let mon_x = match monitor.x().ok() {
                    Some(v) => v,
                    None => continue,
                };
                let mon_y = match monitor.y().ok() {
                    Some(v) => v,
                    None => continue,
                };
                let mon_width = match monitor.width().ok() {
                    Some(v) => v,
                    None => continue,
                };
                let mon_height = match monitor.height().ok() {
                    Some(v) => v,
                    None => continue,
                };
                
                if x >= mon_x
                    && x < mon_x + mon_width as i32
                    && y >= mon_y
                    && y < mon_y + mon_height as i32
                {
                    // Try to capture, but don't block if it fails
                    if let Ok(image) = monitor.capture_image() {
                        self.selected_color = self.get_color_from_image(&monitor, &image, x, y);
                        self.preview_image = self.get_preview_from_image(&monitor, &image, x, y, 21);
                    }
                    break;
                }
            }
        }
    }

    fn get_color_from_image(
        &self,
        monitor: &Monitor,
        image: &xcap::image::RgbaImage,
        screen_x: i32,
        screen_y: i32,
    ) -> Option<Color> {
        let mon_x = monitor.x().ok()?;
        let mon_y = monitor.y().ok()?;
        let mon_width = monitor.width().ok()?;
        let mon_height = monitor.height().ok()?;
        
        let relative_x = screen_x - mon_x;
        let relative_y = screen_y - mon_y;
        let scale_x = image.width() as f64 / mon_width as f64;
        let scale_y = image.height() as f64 / mon_height as f64;
        let image_x = (relative_x as f64 * scale_x).round() as u32;
        let image_y = (relative_y as f64 * scale_y).round() as u32;
        
        if image_x < image.width() && image_y < image.height() {
            let pixel = image.get_pixel(image_x, image_y);
            Some(Color::from_rgb(
                pixel[0] as f32 / 255.0,
                pixel[1] as f32 / 255.0,
                pixel[2] as f32 / 255.0,
            ))
        } else {
            None
        }
    }

    fn get_preview_from_image(
        &self,
        monitor: &Monitor,
        image: &xcap::image::RgbaImage,
        screen_x: i32,
        screen_y: i32,
        size: u32,
    ) -> Option<(Vec<u8>, u32, u32)> {
        let mon_x = monitor.x().ok()?;
        let mon_y = monitor.y().ok()?;
        let mon_width = monitor.width().ok()?;
        let mon_height = monitor.height().ok()?;
        
        let half_size = (size / 2) as i32;
        let mut rgb_data = Vec::new();
        let scale_x = image.width() as f64 / mon_width as f64;
        let scale_y = image.height() as f64 / mon_height as f64;
        
        for dy in -half_size..=half_size {
            for dx in -half_size..=half_size {
                let sample_x = screen_x + dx;
                let sample_y = screen_y + dy;
                if sample_x >= mon_x
                    && sample_x < mon_x + mon_width as i32
                    && sample_y >= mon_y
                    && sample_y < mon_y + mon_height as i32
                {
                    let relative_x = sample_x - mon_x;
                    let relative_y = sample_y - mon_y;
                    let image_x = (relative_x as f64 * scale_x).round() as u32;
                    let image_y = (relative_y as f64 * scale_y).round() as u32;
                    if image_x < image.width() && image_y < image.height() {
                        let pixel = image.get_pixel(image_x, image_y);
                        rgb_data.extend_from_slice(&[pixel[0], pixel[1], pixel[2]]);
                    } else {
                        rgb_data.extend_from_slice(&[0, 0, 0]);
                    }
                } else {
                    rgb_data.extend_from_slice(&[0, 0, 0]);
                }
            }
        }
        Some((rgb_data, size, size))
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
                    let r = self.rgb_data[idx];
                    let g = self.rgb_data[idx + 1];
                    let b = self.rgb_data[idx + 2];
                    let color = Color::from_rgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0);

                    let cell_rect = Rectangle::new(
                        Point::new(x as f32 * cell_size, y as f32 * cell_size),
                        Size::new(cell_size, cell_size),
                    );

                    frame.fill_rectangle(cell_rect.position(), cell_rect.size(), color);

                    if x == self.width / 2 && y == self.height / 2 {
                        let center = cell_rect.center();
                        let half = cell_size / 2.0;
                        frame.stroke(
                            &iced::widget::canvas::Path::line(
                                Point::new(center.x, center.y - half),
                                Point::new(center.x, center.y + half),
                            ),
                            iced::widget::canvas::Stroke::default()
                                .with_color(Color::WHITE)
                                .with_width(2.0),
                        );
                        frame.stroke(
                            &iced::widget::canvas::Path::line(
                                Point::new(center.x - half, center.y),
                                Point::new(center.x + half, center.y),
                            ),
                            iced::widget::canvas::Stroke::default()
                                .with_color(Color::WHITE)
                                .with_width(2.0),
                        );
                    }
                }
            }
        }

        vec![frame.into_geometry()]
    }
}

fn rgb_to_hsv(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let r = r as f32 / 255.0;
    let g = g as f32 / 255.0;
    let b = b as f32 / 255.0;

    let max = r.max(g.max(b));
    let min = r.min(g.min(b));
    let delta = max - min;

    let h = if delta == 0.0 {
        0.0
    } else if max == r {
        60.0 * (((g - b) / delta) % 6.0)
    } else if max == g {
        60.0 * ((b - r) / delta + 2.0)
    } else {
        60.0 * ((r - g) / delta + 4.0)
    };

    let h = if h < 0.0 { h + 360.0 } else { h };
    let s = if max == 0.0 { 0.0 } else { delta / max };
    let v = max;

    (h, s, v)
}
