use std::sync::Arc;

use ambient_core::{asset_cache, gpu, main_scene, option_gpu, ui_scene, window::window_physical_size};
use ambient_ecs::{components, query, FrameEvent, System, SystemGroup, World};
use ambient_gizmos::render::GizmoRenderer;
use ambient_gpu::{
    blit::{Blitter, BlitterKey},
    gpu::{Gpu,OptionGpu},
    shader_module::DEPTH_FORMAT,
    texture::{Texture, TextureView},
};
use ambient_native_std::{asset_cache::SyncAssetKeyExt, color::Color};
use ambient_renderer::{renderer_stats, RenderTarget, Renderer, RendererConfig, RendererTarget};
use ambient_ui_native::app_background_color;
use glam::{uvec2, UVec2};
use parking_lot::Mutex;
use tracing::info_span;
use wgpu::FilterMode;
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
};

components!("app-renderers", {
    ui_renderer: Arc<Mutex<UiRenderer>>,
    main_renderer: Arc<Mutex<MainRenderer>>,
});

pub fn systems() -> SystemGroup<Event<'static, ()>> {
    SystemGroup::new(
        "app_renderers",
        vec![
            query(ui_renderer()).to_system(|q, world, qs, event| {
                let gpu = world.resource(gpu()).clone();
                let option_gpu = world.resource_mut(option_gpu()).clone();
                for (_, ui_render) in q.collect_cloned(world, qs) {
                    let mut ui_render = ui_render.lock();
                    match &event {
                        Event::WindowEvent {
                            event: WindowEvent::Resized(size),
                            ..
                        } => ui_render.resize(&gpu, size),
                        Event::WindowEvent {
                            event: WindowEvent::ScaleFactorChanged { new_inner_size, .. },
                            ..
                        } => {
                            ui_render.resize(&gpu, new_inner_size);
                        }
                        _ => {}
                    }
                    let cleared = matches!(event, Event::MainEventsCleared);
                    if cleared {
                        ui_render.render(&gpu,&option_gpu, world);
                    }
                }
            }),
            query(main_renderer()).to_system(|q, world, qs, event| {
                let gpu = world.resource(gpu()).clone();
                let option_gpu = world.resource_mut(option_gpu()).clone();
                for (_, main_renderer) in q.collect_cloned(world, qs) {
                    let mut main_renderer = main_renderer.lock();
                    match event {
                        Event::WindowEvent {
                            event: WindowEvent::Resized(size),
                            ..
                        } => main_renderer.resize(&gpu,&option_gpu, size),
                        Event::MainEventsCleared => {
                            main_renderer.run(world, &FrameEvent);
                        }
                        _ => {}
                    }
                }
            }),
        ],
    )
}

pub struct MainRenderer {
    main: Option<Renderer>,
    ui: Option<Renderer>,
    blit: Arc<Blitter>,
    render_target: RenderTarget,
    size: UVec2,
}

impl MainRenderer {
    pub fn new(gpu: &Gpu, option_gpu:&Mutex<OptionGpu>,world: &mut World, ui: bool, main: bool) -> Self {
        world
            .add_component(world.resource_entity(), renderer_stats(), "".to_string())
            .unwrap();
        let assets = world.resource(asset_cache());
        let wind_size = *world.resource(ambient_core::window::window_physical_size());

        tracing::debug!("Creating render target option_gpu {:?} wind_size{:?}",option_gpu.lock(),wind_size);
        let render_target = RenderTarget::new(gpu, option_gpu,wind_size, None);

        tracing::debug!("Creating self");

        let is_srgb = option_gpu.lock().swapchain_format().is_srgb();
        let gamma_correction = if !is_srgb {
            tracing::debug!(
                "Output format is not in sRGB colorspace. Applying manual gamma correction."
            );
            Some(2.2)
        } else {
            None
        };
        tracing::info!("tracing main {:?}",main);

        Self {
            main: if main {
                tracing::info!("Creating renderer");
                let mut renderer = Renderer::new(
                    gpu,
                    option_gpu,
                    assets,
                    RendererConfig {
                        scene: main_scene(),
                        shadows: true,
                        ..Default::default()
                    },
                );

                tracing::info!("Creating gizmo renderer");
                renderer.post_transparent = Some(Box::new(GizmoRenderer::new(gpu, assets)));
                Some(renderer)
            } else {
                None
            },
            ui: if ui {
                tracing::info!("Creating UI renderer");

                Some(Renderer::new(
                    gpu,
                    option_gpu,
                    assets,
                    RendererConfig {
                        scene: ui_scene(),
                        shadows: false,
                        ..Default::default()
                    },
                ))
            } else {
                None
            },
            blit: BlitterKey {
                format: option_gpu.lock().swapchain_format().into(),
                min_filter: FilterMode::Nearest,
                gamma_correction,
            }
            .get(assets),
            render_target,
            size: wind_size,
        }
    }
    fn resize(&mut self, gpu: &Gpu,option_gpu:&Mutex<OptionGpu>, size: &PhysicalSize<u32>) {
        self.size = uvec2(size.width, size.height);

        if size.width > 0 && size.height > 0 {
            self.render_target = RenderTarget::new(gpu, option_gpu,uvec2(size.width, size.height), None);
        }
    }

    #[cfg(not(target_os = "unknown"))]
    pub(crate) fn dump_to_tmp_file(&self) {
        std::fs::create_dir_all("tmp").unwrap();
        let mut f = std::fs::File::create("tmp/renderer.txt").expect("Unable to create file");
        self.dump(&mut f);
        tracing::info!("Wrote renderer to tmp/renderer.txt");
    }

    #[allow(dead_code)]
    pub fn n_entities(&self) -> usize {
        self.main.as_ref().map(|x| x.n_entities()).unwrap_or(0)
            + self.ui.as_ref().map(|x| x.n_entities()).unwrap_or(0)
    }
    pub fn stats(&self) -> String {
        if let Some(main) = &self.main {
            main.stats()
        } else {
            String::new()
        }
    }

    #[cfg(not(target_os = "unknown"))]
    pub(crate) fn dump(&self, f: &mut dyn std::io::Write) {
        if let Some(main) = &self.main {
            writeln!(f, "## MAIN ##").unwrap();
            main.dump(f);
        }
        if let Some(ui) = &self.ui {
            writeln!(f, "## UI ##").unwrap();
            ui.dump(f);
        }
    }
}

impl std::fmt::Debug for MainRenderer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Renderer").finish()
    }
}

impl System for MainRenderer {
    fn run(&mut self, world: &mut World, _: &FrameEvent) {
        profiling::scope!("Renderers.run");
        let gpu = world.resource(gpu()).clone();
        let option_gpu = world.resource(option_gpu()).clone();
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        let mut post_submit = Vec::new();

        if let Some(main) = &mut self.main {
            profiling::scope!("Main");
            main.render(
                &gpu,
                &option_gpu,
                world,
                &mut encoder,
                &mut post_submit,
                RendererTarget::Target(&self.render_target),
                Some(Color::rgba(0.0, 0., 0.0, 1.)),
            );
        }

        if let Some(ui) = &mut self.ui {
            // tracing::info!("Drawing UI");
            profiling::scope!("UI");
            ui.render(
                &gpu,
                &option_gpu,
                world,
                &mut encoder,
                &mut post_submit,
                RendererTarget::Target(&self.render_target),
                if self.main.is_some() {
                    None
                } else {
                    Some(app_background_color())
                },
            );
        }

        if let Some(surface) = &option_gpu.lock().surface {
            if self.size.x > 0 && self.size.y > 0 {
                let frame = {
                    profiling::scope!("Get swapchain texture");
                    match surface.get_current_texture() {
                        Ok(v) => v,
                        // Reconfigure the surface if lost
                        Err(wgpu::SurfaceError::Lost) => {
                            tracing::warn!("Surface lost");
                            option_gpu.lock().resize(PhysicalSize {
                                width: self.size.x,
                                height: self.size.y,
                                
                            },&gpu.device);
                            return;
                        }
                        // The system is out of memory, we should probably quit
                        Err(wgpu::SurfaceError::OutOfMemory) => panic!("Out of memory"),
                        // All other errors (Outdated, Timeout) should be resolved by the next frame
                        Err(err) => {
                            tracing::warn!("{err:?}");
                            return;
                        }
                    }
                };
                let frame_view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                self.blit.run(
                    &gpu,
                    &mut encoder,
                    &self.render_target.color_buffer_view,
                    &frame_view,
                );

                {
                    profiling::scope!("Submit");
                    gpu.queue.submit(Some(encoder.finish()));
                }
                {
                    profiling::scope!("Present");
                    frame.present();
                }
            } else {
                profiling::scope!("Submit");
                gpu.queue.submit(Some(encoder.finish()));
            }
        } else {
            {
                profiling::scope!("Submit");
                gpu.queue.submit(Some(encoder.finish()));
            }
        }

        for action in post_submit.into_iter() {
            action();
        }

        world
            .set(world.resource_entity(), renderer_stats(), self.stats())
            .unwrap();
    }
}

pub struct UiRenderer {
    ui_renderer: Renderer,
    depth_buffer_view: Arc<TextureView>,
    normals_view: Arc<TextureView>,
}

impl UiRenderer {
    pub fn new(world: &mut World) -> Self {
        let assets = world.resource(asset_cache());
        let gpu = world.resource(gpu()).clone();
        let option_gpu = world.resource(option_gpu()).clone();
        let size = *world.resource(window_physical_size());

        let depth_buffer = Arc::new(Self::create_depth_buffer(
            &gpu,
            &PhysicalSize::new(size.x, size.y),
        ));

        let normals = Arc::new(Texture::new(
            &gpu,
            &wgpu::TextureDescriptor {
                label: Some("RenderTarget.depth_buffer"),
                size: wgpu::Extent3d {
                    width: size.x,
                    height: size.y,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Snorm,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
        ));

        let mut ui_renderer = Renderer::new(
            &gpu,
            &option_gpu,
            assets,
            RendererConfig {
                scene: ui_scene(),
                shadows: false,
                ..Default::default()
            },
        );
        ui_renderer.post_transparent = Some(Box::new(GizmoRenderer::new(&gpu, assets)));
        Self {
            ui_renderer,
            depth_buffer_view: Arc::new(depth_buffer.create_view(&Default::default())),
            normals_view: Arc::new(normals.create_view(&Default::default())),
        }
    }

    fn create_depth_buffer(gpu: &Gpu, size: &PhysicalSize<u32>) -> Texture {
        Texture::new(
            gpu,
            &wgpu::TextureDescriptor {
                label: Some("RenderTarget.depth_buffer"),
                size: wgpu::Extent3d {
                    width: size.width,
                    height: size.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: DEPTH_FORMAT,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            },
        )
    }

    fn resize(&mut self, gpu: &Gpu, size: &PhysicalSize<u32>) {
        let depth_buffer = Arc::new(Self::create_depth_buffer(gpu, size));
        self.depth_buffer_view = Arc::new(depth_buffer.create_view(&Default::default()));
    }

    fn render(&mut self, gpu: &Gpu, option_gpu:&Mutex<OptionGpu>,world: &mut World) {
        let _span = info_span!("UIRender.render").entered();
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("UIRenderer"),
            });
        let frame = {
            profiling::scope!("Get swapchain texture");
            option_gpu.lock().surface
                .as_ref()
                .unwrap()
                .get_current_texture()
                .expect("Failed to acquire next swap chain texture")
        };

        let window_size = world.resource(window_physical_size());
        let frame_view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut post_submit = Vec::new();

        self.ui_renderer.render(
            gpu,
            option_gpu,
            world,
            &mut encoder,
            &mut post_submit,
            RendererTarget::Direct {
                color: &frame_view,
                depth: &self.depth_buffer_view,
                size: wgpu::Extent3d {
                    width: window_size.x,
                    height: window_size.y,
                    depth_or_array_layers: 1,
                },
                normals: &self.normals_view,
            },
            Some(app_background_color()),
        );
        {
            profiling::scope!("Submit");
            gpu.queue.submit(Some(encoder.finish()));
        }
        {
            profiling::scope!("Present");
            frame.present();
        }
        for action in post_submit {
            action();
        }
    }
}
