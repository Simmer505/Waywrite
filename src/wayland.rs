use smithay_client_toolkit::reexports::calloop::{EventLoop, LoopHandle}; use smithay_client_toolkit::reexports::calloop_wayland_source::WaylandSource;
use smithay_client_toolkit::{
    activation::{ActivationState, ActivationHandler, RequestData},
    compositor::{CompositorState, CompositorHandler},
    output::{OutputHandler, OutputState},
    registry::{RegistryState, ProvidesRegistryState},
    registry_handlers,
    seat::{SeatState, SeatHandler, Capability},
    shell::{
        xdg::{
            window::{WindowHandler, Window, WindowConfigure, WindowDecorations},
            XdgShell,
        },
        WaylandSurface,
    },
    shm::{
        slot::{SlotPool, Buffer},
        Shm, ShmHandler
    },
    delegate_registry, delegate_compositor, delegate_seat, delegate_output,
    delegate_xdg_shell, delegate_shm, delegate_activation, delegate_xdg_window
};

use wayland_client::{
    Connection, QueueHandle,
    globals::registry_queue_init,
    protocol::{wl_surface, wl_output, wl_seat, wl_shm},
};

use std::time::Duration;

use anyhow::Result;

const WINDOW_HEIGHT: u32 = 256;
const WINDOW_WIDTH: u32 = 512;

fn main() -> Result<()> {
    let conn = Connection::connect_to_env()?;

    let (globals, event_queue) = registry_queue_init(&conn)?;
    let qh = event_queue.handle();
    let mut event_loop: EventLoop::<SimpleWindow> =
        EventLoop::try_new()?;
    let loop_handle = event_loop.handle();
    WaylandSource::new(conn.clone(), event_queue).insert(loop_handle)?;

    let compositor = CompositorState::bind(&globals, &qh)?;
    let xdg_shell = XdgShell::bind(&globals, &qh)?;
    let shm = Shm::bind(&globals, &qh)?;
    let _xdg_activation = ActivationState::bind(&globals, &qh).ok();

    let surface = compositor.create_surface(&qh);
    let window = xdg_shell.create_window(surface, WindowDecorations::RequestServer, &qh);

    window.set_title("A window");
    window.set_app_id("simmer.simplewindow");
    window.set_min_size(Some((WINDOW_WIDTH, WINDOW_HEIGHT)));
    window.set_max_size(Some((WINDOW_WIDTH, WINDOW_HEIGHT)));

    window.commit();

    let pool = SlotPool::new((WINDOW_WIDTH as usize) * (WINDOW_HEIGHT as usize) * 4, &shm)?;

    let mut simple_window = SimpleWindow {
        registry_state: RegistryState::new(&globals),
        seat_state: SeatState::new(&globals, &qh),
        output_state: OutputState::new(&globals, &qh),
        shm,
        buffer: None,
        pool,
        window,
        width: WINDOW_WIDTH,
        height: WINDOW_HEIGHT,
        exit: false,
        first_configure: true,
        _loop_handle: event_loop.handle(),
    };

    loop {
        event_loop.dispatch(Duration::from_millis(16), &mut simple_window)?;

        if simple_window.exit {
            println!("exiting");
            break;
        }
    }


    Ok(())
}

struct SimpleWindow {
    registry_state: RegistryState,
    seat_state: SeatState,
    output_state: OutputState,
    shm: Shm,
    buffer: Option<Buffer>,
    pool: SlotPool,
    window: Window,
    width: u32,
    height: u32,
    exit: bool,
    first_configure: bool,
    _loop_handle: LoopHandle<'static, SimpleWindow>
}

impl CompositorHandler for SimpleWindow {
    fn scale_factor_changed(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface,
        _: i32) {}
    fn transform_changed(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface,
        _: wl_output::Transform) {}
    fn frame(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        _: &wl_surface::WlSurface,
        _: u32
        ) {
        self.draw(conn, qh);
    }
}

impl SeatHandler for SimpleWindow {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }
    fn new_seat(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: wl_seat::WlSeat
        ) {}
    fn new_capability(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: wl_seat::WlSeat,
        _: Capability,
        ) {}
    fn remove_capability(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: wl_seat::WlSeat,
        _: Capability,
        ) {}
    fn remove_seat(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: wl_seat::WlSeat,
        ) {}
}

impl OutputHandler for SimpleWindow {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }
    fn new_output(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: wl_output::WlOutput,
        ) {}
    fn update_output(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: wl_output::WlOutput,
        ) {}
    fn output_destroyed(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: wl_output::WlOutput,
        ) {}
}

impl WindowHandler for SimpleWindow {
    fn request_close(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &Window
        ) {
            self.exit = true;
    }

    fn configure(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        _: &Window,
        _: WindowConfigure,
        _: u32,
        ) {
        if self.first_configure {
            self.first_configure = false;
            self.draw(conn, qh);
        }
    }
}

impl ShmHandler for SimpleWindow {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl ActivationHandler for SimpleWindow {
    type RequestData = RequestData;

    fn new_token(&mut self, _: String, _: &Self::RequestData) {}
}

impl SimpleWindow {
    fn draw(&mut self, _: &Connection, qh: &QueueHandle<Self>) {
        let buffer = self.buffer.get_or_insert_with(|| {
            self.pool
                .create_buffer(self.width as i32, self.height as i32, (self.width as i32) * 4, wl_shm::Format::Argb8888)
                .expect("create buffer")
                .0
        });

        let canvas = match self.pool.canvas(buffer) {
            Some(canvas) => canvas,
            None => {
                let (second_buffer, canvas) = self
                    .pool
                    .create_buffer(
                        self.width as i32,
                        self.height as i32,
                        (self.width as i32) * 4,
                        wl_shm::Format::Argb8888,
                        )
                    .expect("create buffer");
                *buffer = second_buffer;
                canvas
            }
        };

        for pix in canvas.chunks_exact_mut(4) {

            let color: i32 = (255 << 24) + (255 << 16) + (0 << 8) + 0;
            
            let array: &mut [u8; 4] = pix.try_into().unwrap();
            *array = color.to_le_bytes();
        }

        self.window.wl_surface().damage_buffer(0, 0, self.width as i32, self.height as i32);
        self.window.wl_surface().frame(qh, self.window.wl_surface().clone());
        buffer.attach_to(self.window.wl_surface()).expect("buffer attach");
        self.window.commit();

    }
}

delegate_compositor!(SimpleWindow);
delegate_output!(SimpleWindow);
delegate_shm!(SimpleWindow);

delegate_seat!(SimpleWindow);

delegate_xdg_shell!(SimpleWindow);
delegate_xdg_window!(SimpleWindow);
delegate_activation!(SimpleWindow);

delegate_registry!(SimpleWindow);

impl ProvidesRegistryState for SimpleWindow {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState, SeatState,];
}
