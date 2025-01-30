use std::f32::consts::PI;
use std::sync::Arc;
use std::time::{Duration, Instant};
use vello::{wgpu, AaConfig, RenderParams, Renderer, RendererOptions, Scene};
use vello::kurbo::{Affine, Circle, Point, Rect, Vec2};
use vello::peniko::{Color, Fill};
use vello::util::{DeviceHandle, RenderContext, RenderSurface};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::window::{Window, WindowId};
use crate::compositor::compositor::Compositor;
use crate::compositor::layer::Layer;
use crate::utils::load_image;

const DT_FPS_60_NANO: u128 = 1_000_000_000 / 60;

const AA_CONFIGS: [AaConfig; 3] = [AaConfig::Area, AaConfig::Msaa8, AaConfig::Msaa16];

pub struct App<'s> {
    compositor: Compositor,
    render_ctx: RenderContext,
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    surface: Option<RenderSurface<'s>>,
    frame_count: u64,
    next_redraw_time: Instant,
    last_redraw: Instant,
}

impl<'s> App<'s> {
    pub fn new() -> Self {
        Self {
            compositor: Compositor::new(),
            render_ctx: RenderContext::new(),
            window: None,
            renderer: None,
            surface: None,
            frame_count: 0,
            next_redraw_time: Instant::now(),
            last_redraw: Instant::now(),
        }
    }

    pub fn generate_scenes(&mut self) {
        // Create a red rectangle (Layer 1)
        let mut scene_red = Scene::new();
        scene_red.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            Color::from_rgba8(255, 0, 0, 255),
            None,
            &Rect::new(100.0, 100.0, 200.0, 200.0),
        );
        self.compositor.add_layer(Layer::new(
            1,
            scene_red,
            Affine::IDENTITY,
            // Affine::translate((50.0, 50.0)),
            1.0,
            0,
        ));

        // Create a blue circle (Layer 2)
        let mut scene_blue = Scene::new();
        scene_blue.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            Color::from_rgba8(0, 0, 255, 255),
            None,
            &Circle::new((100.0, 100.0), 50.0),
        );
        self.compositor.add_layer(Layer::new(
            2,
            scene_blue,
            Affine::IDENTITY,
            1.0,
            100,
        ));

        let img = load_image("./gosub-logo.png");
        let mut scene_green = Scene::new();
        scene_green.draw_image(
            &img,
            Affine::IDENTITY,
        );
        self.compositor.add_layer(Layer::new(
            3,
            scene_green,
            Affine::IDENTITY,
            // Affine::translate((50.0, 50.0)),
            0.75,
            50,
        ));
    }
}

impl ApplicationHandler for App<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        // Initialize application and renderer data
        let mut attribs = Window::default_attributes();
        attribs.title = "Pipeline WGPU test".to_string();
        let window = Arc::new(event_loop.create_window(attribs).unwrap());

        let size = window.inner_size();
        let surface_future = self.render_ctx.create_surface(
            window.clone(),
            size.width,
            size.height,
            wgpu::PresentMode::AutoVsync,
        );
        let surface = pollster::block_on(surface_future).unwrap();

        let dev_handle = &self.render_ctx.devices[surface.dev_id];
        let info = self.render_ctx.instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            });
        let info = pollster::block_on(info).unwrap();
        dbg!(&info.get_info());

        let renderer = Renderer::new(
            &dev_handle.device,
            RendererOptions {
                surface_format: Some(surface.format),
                use_cpu: false,
                antialiasing_support: AA_CONFIGS.iter().copied().collect(),
                num_init_threads: None,
            },
        );

        self.window = Some(window);
        self.surface = Some(surface);
        self.renderer = Some(renderer.unwrap());

        self.generate_scenes();
        
        self.frame_count = 0;
        // self.next_redraw_time = Instant::now() + Duration::from_secs_f64(1.0 / 60.0);

        // Request a redraw right away. This will trigger the first frame
        self.window.as_ref().unwrap().request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                self.render_ctx.resize_surface(
                    self.surface.as_mut().unwrap(),
                    size.width,
                    size.height,
                );
            }
            WindowEvent::RedrawRequested => {
                self.frame_count += 1;
                // println!("Frame: {}", self.frame_count);

                let th = self.frame_count as f64 * (PI as f64 / 180.0);
                let op = ((self.frame_count as f64 * 0.01 * 2.0 * PI as f64).sin() + 1.0) / 2.0;
                self.compositor.update_layer(
                    1,
                    Some(Affine::rotate_about(th, Point::new(100.0, 100.0))),
                    None
                );


                let af = Affine::IDENTITY
                    .then_rotate_about(th, Point::new(400.0, 400.0))
                    .then_translate(Vec2::new(op * 100.0, 0.0))
                ;
                self.compositor.update_layer(
                    3,
                    Some(af),
                    Some(op as f32),
                );

                let mut final_scene = Scene::new();
                self.compositor.compose(&mut final_scene);

                let surface = self.surface.as_ref().unwrap();
                let dev_id = surface.dev_id;
                let DeviceHandle { device, queue, .. } = &self.render_ctx.devices[dev_id];

                let width = surface.config.width;
                let height = surface.config.height;

                let surface_texture = surface
                    .surface
                    .get_current_texture()
                    .expect("Failed to get current texture");
                let render_params = RenderParams {
                    base_color: Color::BLACK,
                    width,
                    height,
                    antialiasing_method: AaConfig::Area,
                };

                let _ = self.renderer.as_mut().unwrap().render_to_surface(
                    &device,
                    &queue,
                    &final_scene,
                    &surface_texture,
                    &render_params,
                );
                surface_texture.present();

                // Check if should redraw
                if self.last_redraw.elapsed().as_nanos() > DT_FPS_60_NANO {
                    println!("FPS: {:>6.2}", 1000.0 / self.last_redraw.elapsed().as_millis() as f64);
                    self.last_redraw = Instant::now();
                }

                self.next_redraw_time += Duration::from_secs_f64(5.0);
                // event_loop.set_control_flow(ControlFlow::WaitUntil(self.next_redraw_time));
                event_loop.set_control_flow(ControlFlow::Poll);
                self.window.as_ref().unwrap().request_redraw();
            }
            _ => (),
        }
    }
}