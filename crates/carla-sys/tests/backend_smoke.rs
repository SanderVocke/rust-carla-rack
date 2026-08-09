use std::ffi::CStr;
use std::ptr;

use carla_sys::*;

#[test]
fn starts_and_stops_dummy_engine() {
    unsafe {
        let host = carla_standalone_host_init();
        assert!(!host.is_null(), "Carla did not create a standalone host");

        carla_set_engine_option(
            host,
            EngineOption_ENGINE_OPTION_PROCESS_MODE,
            EngineProcessMode_ENGINE_PROCESS_MODE_CONTINUOUS_RACK as i32,
            ptr::null(),
        );
        carla_set_engine_option(
            host,
            EngineOption_ENGINE_OPTION_TRANSPORT_MODE,
            EngineTransportMode_ENGINE_TRANSPORT_MODE_INTERNAL as i32,
            ptr::null(),
        );

        let initialized =
            carla_engine_init(host, c"Dummy".as_ptr(), c"rust-carla-rack-smoke".as_ptr());
        if !initialized {
            let error = carla_get_last_error(host);
            let message = if error.is_null() {
                "unknown Carla error".into()
            } else {
                CStr::from_ptr(error).to_string_lossy()
            };
            panic!("failed to initialize Carla's Dummy engine: {message}");
        }

        let running = carla_is_engine_running(host);
        let closed = carla_engine_close(host);

        assert!(running, "Carla's Dummy engine did not enter running state");
        assert!(closed, "Carla reported an error while closing the engine");
    }
}
