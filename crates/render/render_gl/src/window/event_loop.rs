use winit::event_loop::EventLoop;

static mut EVENT_LOOP_CONTAINER: Option<EventLoop<()>> = None;

pub(crate) struct EventLoopContainer;

impl EventLoopContainer {

    pub(crate) fn take_or_init() -> EventLoop<()> {
        unsafe {
            if EVENT_LOOP_CONTAINER.is_some() {
                return Self::take();
            } else {
                return EventLoop::new();
            }
        }
    }

    fn take() -> EventLoop<()> {
        unsafe {
            if EVENT_LOOP_CONTAINER.is_none() {
                panic!("Event loop not set");
            }
            EVENT_LOOP_CONTAINER.take().unwrap()
        }
    }

    #[allow(unused)]
    pub(crate) fn set(event_loop: EventLoop<()>) {
        unsafe {
            if EVENT_LOOP_CONTAINER.is_some() {
                panic!("Event loop already set");
            }
            EVENT_LOOP_CONTAINER = Some(event_loop);
        }
    }
}