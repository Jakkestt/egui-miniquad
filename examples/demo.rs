use std::cell::RefCell;

use mq::{Context, GlContext};

static mut CONTEXT: Option<Dummy> = None;

use {egui_miniquad as egui_mq, miniquad as mq};

struct Dummy {
    ctx: Box<dyn mq::RenderingBackend>,
}

struct Stage {
    egui_mq: egui_mq::EguiMq,
    show_egui_demo_windows: bool,
    egui_demo_windows: egui_demo_lib::DemoWindows,
    color_test: egui_demo_lib::ColorTest,
    pixels_per_point: f32,
}

fn get_context() -> &'static mut dyn mq::RenderingBackend {
    unsafe { &mut *CONTEXT.as_mut().unwrap().ctx }
}

impl Stage {
    fn new() -> Self {
        let mut ctx = mq::window::new_rendering_backend();
        let ret = Self {
            egui_mq: egui_mq::EguiMq::new(ctx.as_mut()),
            show_egui_demo_windows: true,
            egui_demo_windows: Default::default(),
            color_test: Default::default(),
            pixels_per_point: miniquad::window::dpi_scale(),
        };
        unsafe {
            CONTEXT = Some(Dummy { ctx });
        }
        ret
    }
}

impl mq::EventHandler for Stage {
    fn update(&mut self) {}

    fn draw(&mut self) {
        let ctx = get_context();
        ctx.clear(Some((1., 1., 1., 1.)), None, None);
        ctx.begin_default_pass(mq::PassAction::clear_color(0.0, 0.0, 0.0, 1.0));
        ctx.end_render_pass();

        let dpi_scale = miniquad::window::dpi_scale();

        // Run the UI code:
        self.egui_mq.run(ctx, |_mq_ctx, egui_ctx| {
            if self.show_egui_demo_windows {
                self.egui_demo_windows.ui(egui_ctx);
            }

            egui::Window::new("egui ❤ miniquad").show(egui_ctx, |ui| {
                egui::widgets::global_dark_light_mode_buttons(ui);
                ui.checkbox(&mut self.show_egui_demo_windows, "Show egui demo windows");

                ui.group(|ui| {
                    ui.label("Physical pixels per each logical 'point':");
                    ui.label(format!("native: {:.2}", dpi_scale));
                    ui.label(format!("egui:   {:.2}", ui.ctx().pixels_per_point()));
                    ui.add(
                        egui::Slider::new(&mut self.pixels_per_point, 0.75..=3.0).logarithmic(true),
                    )
                    .on_hover_text("Physical pixels per logical point");
                    if ui.button("Reset").clicked() {
                        self.pixels_per_point = dpi_scale;
                    }
                });

                #[cfg(not(target_arch = "wasm32"))]
                {
                    if ui.button("Quit").clicked() {
                        std::process::exit(0);
                    }
                }
            });

            // Don't change scale while dragging the slider
            if !egui_ctx.is_using_pointer() {
                egui_ctx.set_pixels_per_point(self.pixels_per_point);
            }

            egui::Window::new("Color Test").show(egui_ctx, |ui| {
                egui::ScrollArea::both()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        self.color_test.ui(ui);
                    });
            });
        });

        // Draw things behind egui here

        self.egui_mq.draw(ctx);

        // Draw things in front of egui here

        ctx.commit_frame();
    }

    fn mouse_motion_event(&mut self, x: f32, y: f32) {
        self.egui_mq.mouse_motion_event(x, y);
    }

    fn mouse_wheel_event(&mut self, dx: f32, dy: f32) {
        self.egui_mq.mouse_wheel_event(dx, dy);
    }

    fn mouse_button_down_event(&mut self, mb: mq::MouseButton, x: f32, y: f32) {
        let ctx = get_context();
        self.egui_mq.mouse_button_down_event(ctx, mb, x, y);
    }

    fn mouse_button_up_event(&mut self, mb: mq::MouseButton, x: f32, y: f32) {
        let ctx = get_context();
        self.egui_mq.mouse_button_up_event(ctx, mb, x, y);
    }

    fn char_event(&mut self, character: char, _keymods: mq::KeyMods, _repeat: bool) {
        self.egui_mq.char_event(character);
    }

    fn key_down_event(&mut self, keycode: mq::KeyCode, keymods: mq::KeyMods, _repeat: bool) {
        let ctx = get_context();
        self.egui_mq.key_down_event(ctx, keycode, keymods);
    }

    fn key_up_event(&mut self, keycode: mq::KeyCode, keymods: mq::KeyMods) {
        self.egui_mq.key_up_event(keycode, keymods);
    }
}

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        // Log to stdout (if you run with `RUST_LOG=debug`).
        tracing_subscriber::fmt::init();
    }

    let conf = mq::conf::Conf {
        high_dpi: true,
        window_width: 1200,
        window_height: 1024,
        ..Default::default()
    };
    mq::start(conf, || Box::new(Stage::new()));
}
