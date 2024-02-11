#[cfg(target_os = "android")]
use winit::platform::android::activity::AndroidApp;
use winit::event_loop::{EventLoop,EventLoopBuilder};
use winit::platform::android::EventLoopBuilderExtAndroid;

#[allow(dead_code)]
#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: AndroidApp) {
    android_logger::init_once(android_logger::Config::default().with_min_level(log::Level::Info));
   
    let event_loop: EventLoop<()> = EventLoopBuilder::new()
        .with_android_app(app)
        .build();
    log::debug!("start...");
    ambient_app::c::run(event_loop);
}

fn main(){}

