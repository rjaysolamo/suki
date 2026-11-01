//! Platform calls stay outside UI and business logic.
//! This feasibility probe reads a non-sensitive Android system value through JNI.
#[cfg(target_os = "android")]
pub fn runtime_info() -> Result<String, String> {
    let context = ndk_context::android_context();
    // ndk-context owns this VM for the Android activity lifetime; we never destroy it.
    let vm = unsafe { jni::JavaVM::from_raw(context.vm().cast()) }
        .map_err(|_| "Android VM is unavailable. Retry after reopening the app.".to_string())?;
    let mut env = vm
        .attach_current_thread()
        .map_err(|_| "Could not attach to Android. Please retry.".to_string())?;
    let sdk = env
        .get_static_field("android/os/Build$VERSION", "SDK_INT", "I")
        .and_then(|value| value.i())
        .map_err(|_| "Could not read Android runtime information.".to_string())?;
    Ok(format!("Connected to Android API {sdk}."))
}

#[cfg(not(target_os = "android"))]
pub fn runtime_info() -> Result<String, String> {
    Err("This check requires the installed Android application.".to_string())
}
