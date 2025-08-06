use device_query::{DeviceQuery, DeviceState, Keycode};
use eframe::egui::{self, Color32, Pos2, Rect, Stroke, StrokeKind, Vec2};
use xcap::Monitor;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_always_on_top()
            .with_resizable(true)
            .with_inner_size([400.0, 250.0]),
        ..Default::default()
    };
    eframe::run_native(
        "PixelPicker",
        options,
        Box::new(|_cc| Ok(Box::new(PixelPickerApp::default()))),
    )
}

#[derive(Default)]
struct PixelPickerApp {
    selected_color: Option<Color32>,
    preview_image: Option<(Vec<u8>, u32, u32)>,
    frozen_position: Option<(i32, i32)>,
    frozen_color: Option<Color32>,
    frozen_preview: Option<(Vec<u8>, u32, u32)>,
    space_pressed_last_frame: bool,
}

impl eframe::App for PixelPickerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label(
                egui::RichText::new("PixelPicker")
                    .heading()
                    .color(Color32::LIGHT_YELLOW),
            );

            let device_state = DeviceState::new();
            let mouse = device_state.get_mouse();
            let keys = device_state.get_keys();
            let space_pressed = keys.contains(&Keycode::Space);
            let esc_pressed = keys.contains(&Keycode::Escape);

            // Get current mouse position
            let (x, y) = (mouse.coords.0, mouse.coords.1);

            // Detect spacebar press edge (pressed now but not in last frame)
            let just_pressed = space_pressed && !self.space_pressed_last_frame;
            self.space_pressed_last_frame = space_pressed;

            // Handle freeze/unfreeze logic
            if just_pressed {
                // Freeze data at current mouse position
                self.frozen_position = Some((x, y));
                self.frozen_color = get_color_at(x, y);
                self.frozen_preview = get_preview_at(x, y, 21);
            } else if esc_pressed {
                // Clear frozen state
                self.frozen_position = None;
                self.frozen_color = None;
                self.frozen_preview = None;
            }

            // Use frozen data if available
            let (display_x, display_y) = self.frozen_position.unwrap_or((x, y));
            self.selected_color = self.frozen_color.clone().or_else(|| get_color_at(x, y));
            self.preview_image = self
                .frozen_preview
                .clone()
                .or_else(|| get_preview_at(x, y, 21));

            ui.add_space(4.0);

            if let Some((rgb_data, width, height)) = &self.preview_image {
                ui.separator();
                ui.horizontal(|ui| {
                    let cell_size = 8.0;
                    let grid_size =
                        Vec2::new(*width as f32 * cell_size, *height as f32 * cell_size);

                    let (response, painter) = ui.allocate_painter(grid_size, egui::Sense::hover());
                    let rect = response.rect;

                    for y in 0..*height {
                        for x in 0..*width {
                            let idx = (y * width + x) as usize * 3;
                            if idx + 2 < rgb_data.len() {
                                let r = rgb_data[idx];
                                let g = rgb_data[idx + 1];
                                let b = rgb_data[idx + 2];
                                let color = Color32::from_rgb(r, g, b);

                                let cell_rect = Rect::from_min_size(
                                    Pos2::new(
                                        rect.left() + x as f32 * cell_size,
                                        rect.top() + y as f32 * cell_size,
                                    ),
                                    Vec2::new(cell_size, cell_size),
                                );

                                painter.rect_filled(cell_rect, 0.0, color);

                                if x == width / 2 && y == height / 2 {
                                    painter.rect_stroke(
                                        cell_rect,
                                        0.0,
                                        Stroke::new(2.0, Color32::WHITE),
                                        StrokeKind::Outside,
                                    );
                                    painter.rect_stroke(
                                        cell_rect.shrink(1.0),
                                        0.0,
                                        Stroke::new(1.0, Color32::BLACK),
                                        StrokeKind::Outside,
                                    );
                                }
                            }
                        }
                    }

                    if let Some(color) = self.selected_color {
                        ui.vertical(|ui| {
                            ui.label(
                                egui::RichText::new("Mouse Position:").color(Color32::LIGHT_YELLOW),
                            );

                            ui.monospace(format!("({}, {})", display_x, display_y));

                            ui.add_space(8.0);
                            ui.label(
                                egui::RichText::new("Picked Color:").color(Color32::LIGHT_YELLOW),
                            );

                            let swatch_size = Vec2::new(60.0, 30.0);
                            let (response, painter) =
                                ui.allocate_painter(swatch_size, egui::Sense::hover());
                            painter.rect_filled(response.rect, 4.0, color);
                            painter.rect_stroke(
                                response.rect,
                                4.0,
                                Stroke::new(1.0, Color32::GRAY),
                                StrokeKind::Outside,
                            );

                            ui.monospace(format!(
                                "RGB: ({}, {}, {})",
                                color.r(),
                                color.g(),
                                color.b()
                            ));
                            ui.monospace(format!(
                                "HEX: #{:02X}{:02X}{:02X}",
                                color.r(),
                                color.g(),
                                color.b()
                            ));

                            let (h, s, v) = rgb_to_hsv(color.r(), color.g(), color.b());
                            ui.monospace(format!(
                                "HSV: ({:.0}°, {:.0}%, {:.0}%)",
                                h,
                                s * 100.0,
                                v * 100.0
                            ));

                            ui.add_space(8.0);

                            ui.horizontal(|ui| {
                                if ui.button("Copy RGB").clicked() {
                                    ctx.copy_text(format!(
                                        "{}, {}, {}",
                                        color.r(),
                                        color.g(),
                                        color.b()
                                    ));
                                }
                                if ui.button("Copy HEX").clicked() {
                                    ctx.copy_text(format!(
                                        "#{:02X}{:02X}{:02X}",
                                        color.r(),
                                        color.g(),
                                        color.b()
                                    ));
                                }
                            });
                        });
                    } else {
                        ui.label("No color detected");
                    }
                });
            } else {
                ui.label("No preview available");
            }
            let is_frozen = self.frozen_position.is_some();

            // Status label
            ui.horizontal(|ui| {
                let status_text = if is_frozen {
                    "🧊 Frozen (press ESC to unfreeze)"
                } else {
                    "🎯 Live (press SPACE to freeze)"
                };
                let status_color = if is_frozen {
                    Color32::from_rgb(100, 180, 255)
                } else {
                    Color32::from_rgb(100, 255, 150)
                };

                ui.colored_label(status_color, status_text);
            });
        });
        ctx.request_repaint();
    }
}

fn get_color_at(screen_x: i32, screen_y: i32) -> Option<Color32> {
    let monitors = Monitor::all().ok()?;

    for monitor in monitors {
        // Get monitor bounds - these are logical coordinates
        let mon_x = monitor.x().ok()?;
        let mon_y = monitor.y().ok()?;
        let mon_width = monitor.width().ok()?;
        let mon_height = monitor.height().ok()?;

        // Check if mouse is within this monitor's bounds
        if screen_x >= mon_x
            && screen_x < mon_x + mon_width as i32
            && screen_y >= mon_y
            && screen_y < mon_y + mon_height as i32
        {
            let image = monitor.capture_image().ok()?;

            // Calculate position relative to monitor
            let relative_x = screen_x - mon_x;
            let relative_y = screen_y - mon_y;

            // Calculate scaling factors
            let scale_x = image.width() as f64 / mon_width as f64;
            let scale_y = image.height() as f64 / mon_height as f64;

            // Scale to image coordinates
            let image_x = (relative_x as f64 * scale_x).round() as u32;
            let image_y = (relative_y as f64 * scale_y).round() as u32;

            // Check bounds and get pixel
            if image_x < image.width() && image_y < image.height() {
                let pixel = image.get_pixel(image_x, image_y);
                return Some(Color32::from_rgb(pixel[0], pixel[1], pixel[2]));
            }
        }
    }

    None
}

fn get_preview_at(screen_x: i32, screen_y: i32, size: u32) -> Option<(Vec<u8>, u32, u32)> {
    let monitors = Monitor::all().ok()?;
    let half_size = (size / 2) as i32;

    for monitor in monitors {
        // Get monitor bounds - these are logical coordinates
        let mon_x = monitor.x().ok()?;
        let mon_y = monitor.y().ok()?;
        let mon_width = monitor.width().ok()?;
        let mon_height = monitor.height().ok()?;

        // Check if mouse is within this monitor's bounds
        if screen_x >= mon_x
            && screen_x < mon_x + mon_width as i32
            && screen_y >= mon_y
            && screen_y < mon_y + mon_height as i32
        {
            let image = monitor.capture_image().ok()?;
            let mut rgb_data = Vec::new();

            // Calculate scaling factors
            let scale_x = image.width() as f64 / mon_width as f64;
            let scale_y = image.height() as f64 / mon_height as f64;

            // Sample pixels around the mouse position
            for dy in -half_size..=half_size {
                for dx in -half_size..=half_size {
                    let sample_x = screen_x + dx;
                    let sample_y = screen_y + dy;

                    // Check if sample point is within monitor bounds
                    if sample_x >= mon_x
                        && sample_x < mon_x + mon_width as i32
                        && sample_y >= mon_y
                        && sample_y < mon_y + mon_height as i32
                    {
                        // Calculate position relative to monitor
                        let relative_x = sample_x - mon_x;
                        let relative_y = sample_y - mon_y;

                        // Scale to image coordinates
                        let image_x = (relative_x as f64 * scale_x).round() as u32;
                        let image_y = (relative_y as f64 * scale_y).round() as u32;

                        // Check bounds and get pixel
                        if image_x < image.width() && image_y < image.height() {
                            let pixel = image.get_pixel(image_x, image_y);
                            rgb_data.extend_from_slice(&[pixel[0], pixel[1], pixel[2]]);
                        } else {
                            // Out of image bounds, use black
                            rgb_data.extend_from_slice(&[0, 0, 0]);
                        }
                    } else {
                        // Out of monitor bounds, use black
                        rgb_data.extend_from_slice(&[0, 0, 0]);
                    }
                }
            }

            return Some((rgb_data, size, size));
        }
    }

    None
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
