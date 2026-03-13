use glutin::config::{ConfigTemplateBuilder, GlConfig};
use glutin::context::{ContextApi, ContextAttributesBuilder, NotCurrentGlContext, PossiblyCurrentContext};
use glutin::display::{GetGlDisplay, GlDisplay};
use glutin::surface::{GlSurface, Surface, SurfaceAttributesBuilder, WindowSurface};
use glutin_winit::{DisplayBuilder, GlWindow};
use raw_window_handle::HasRawWindowHandle;
use std::num::NonZeroU32;
use winit::dpi::PhysicalSize;
use winit::event_loop::EventLoop;
use winit::window::Window as WinitWindow;

/// Window wrapper that manages OpenGL context and surface
pub struct Window {
    pub winit_window: WinitWindow,
    gl_context: PossiblyCurrentContext,
    gl_surface: Surface<WindowSurface>,
    width: u32,
    height: u32,
}

impl Window {
    /// Create a new window with OpenGL context
    pub fn new(event_loop: &EventLoop<()>, title: &str, width: u32, height: u32) -> Self {
        // Build window with glutin
        use winit::window::WindowBuilder;

        let window_builder = WindowBuilder::new()
            .with_title(title)
            .with_inner_size(PhysicalSize::new(width, height))
            .with_resizable(true);

        let template = ConfigTemplateBuilder::new()
            .with_alpha_size(8)
            .with_depth_size(24)
            .with_stencil_size(8);

        let display_builder = DisplayBuilder::new().with_window_builder(Some(window_builder));

        let (window, gl_config) = display_builder
            .build(event_loop, template, |mut configs| {
                configs.next().unwrap()
            })
            .expect("Failed to create window and GL config");

        let window = window.expect("Failed to create window");

        // Create GL context
        let raw_window_handle = window.raw_window_handle();
        let gl_display = gl_config.display();

        let context_attrs = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::OpenGl(Some(glutin::context::Version::new(3, 3))))
            .build(Some(raw_window_handle));

        let gl_context = unsafe {
            gl_display
                .create_context(&gl_config, &context_attrs)
                .expect("Failed to create GL context")
        };

        // Create GL surface
        let (width, height) = {
            let size = window.inner_size();
            (size.width, size.height)
        };

        let attrs = SurfaceAttributesBuilder::<WindowSurface>::new().build(
            raw_window_handle,
            NonZeroU32::new(width).unwrap(),
            NonZeroU32::new(height).unwrap(),
        );

        let gl_surface = unsafe {
            gl_display
                .create_window_surface(&gl_config, &attrs)
                .expect("Failed to create GL surface")
        };

        // Make context current
        let gl_context = gl_context
            .make_current(&gl_surface)
            .expect("Failed to make GL context current");

        // Load OpenGL function pointers
        gl::load_with(|symbol| {
            let symbol = std::ffi::CString::new(symbol).unwrap();
            gl_display.get_proc_address(symbol.as_c_str())
        });

        // Enable vsync
        gl_surface
            .set_swap_interval(&gl_context, glutin::surface::SwapInterval::Wait(NonZeroU32::new(1).unwrap()))
            .expect("Failed to set vsync");

        // Initialize OpenGL state
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::Enable(gl::CULL_FACE);
            gl::CullFace(gl::BACK);
            gl::FrontFace(gl::CCW);
            gl::ClearColor(0.53, 0.81, 0.92, 1.0); // Sky blue
        }

        println!("OpenGL initialized:");
        unsafe {
            let version = std::ffi::CStr::from_ptr(gl::GetString(gl::VERSION) as *const i8);
            let renderer = std::ffi::CStr::from_ptr(gl::GetString(gl::RENDERER) as *const i8);
            println!("  Version: {:?}", version);
            println!("  Renderer: {:?}", renderer);
        }

        Self {
            winit_window: window,
            gl_context,
            gl_surface,
            width,
            height,
        }
    }

    /// Swap buffers and present the frame
    pub fn swap_buffers(&self) {
        self.gl_surface
            .swap_buffers(&self.gl_context)
            .expect("Failed to swap buffers");
    }

    /// Handle window resize
    pub fn resize(&mut self, new_width: u32, new_height: u32) {
        if new_width == 0 || new_height == 0 {
            return;
        }

        self.width = new_width;
        self.height = new_height;

        self.gl_surface.resize(
            &self.gl_context,
            NonZeroU32::new(new_width).unwrap(),
            NonZeroU32::new(new_height).unwrap(),
        );

        unsafe {
            gl::Viewport(0, 0, new_width as i32, new_height as i32);
        }
    }

    /// Get current window dimensions
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Get aspect ratio
    pub fn aspect_ratio(&self) -> f32 {
        self.width as f32 / self.height as f32
    }

    /// Request window redraw
    pub fn request_redraw(&self) {
        self.winit_window.request_redraw();
    }

    /// Set cursor visibility
    pub fn set_cursor_visible(&self, visible: bool) {
        self.winit_window.set_cursor_visible(visible);
    }

    /// Set cursor grab (for mouse look)
    pub fn set_cursor_grab(&self, grabbed: bool) {
        use winit::window::CursorGrabMode;

        if grabbed {
            // Try confined mode first, fall back to locked
            if self.winit_window
                .set_cursor_grab(CursorGrabMode::Confined)
                .is_err()
            {
                let _ = self.winit_window
                    .set_cursor_grab(CursorGrabMode::Locked);
            }
            self.set_cursor_visible(false);
        } else {
            let _ = self.winit_window
                .set_cursor_grab(CursorGrabMode::None);
            self.set_cursor_visible(true);
        }
    }

    /// Center the cursor
    pub fn center_cursor(&self) {
        use winit::dpi::PhysicalPosition;
        let center = PhysicalPosition::new(self.width as f64 / 2.0, self.height as f64 / 2.0);
        let _ = self.winit_window.set_cursor_position(center);
    }
}
