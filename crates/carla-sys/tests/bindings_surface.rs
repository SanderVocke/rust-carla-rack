use std::ffi::c_char;
use std::mem::size_of;

use carla_sys::*;

#[test]
fn representative_host_api_is_exposed() {
    let _: unsafe extern "C" fn() -> CarlaHostHandle = carla_standalone_host_init;
    let _: unsafe extern "C" fn(CarlaHostHandle) = carla_transport_play;
    let _: unsafe extern "C" fn(CarlaHostHandle, *const c_char) -> bool = carla_load_project;
    let _: unsafe extern "C" fn(CarlaHostHandle, uint, u32) -> *const CarlaParameterInfo =
        carla_get_parameter_info;

    let _: EngineCallbackFunc = None;
    let _: EngineOption = EngineOption_ENGINE_OPTION_PROCESS_MODE;
    let _: EngineProcessMode = EngineProcessMode_ENGINE_PROCESS_MODE_CONTINUOUS_RACK;
    let _: EngineTransportMode = EngineTransportMode_ENGINE_TRANSPORT_MODE_INTERNAL;

    assert_ne!(size_of::<CarlaPluginInfo>(), 0);
    assert_ne!(size_of::<ParameterData>(), 0);
    assert_ne!(size_of::<CarlaTransportInfo>(), 0);
}
