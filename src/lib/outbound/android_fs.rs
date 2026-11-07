//! The app's private files dir on Android (`/data/data/<pkg>/files`),
//! asked of the Android `Context` that `ndk-context` exposes, through JNI.

use std::io;
use std::path::PathBuf;

use jni::objects::{JObject, JString};

/// Local references `files_dir` creates: the `File` and its path string.
/// A hint to the VM, not a cap.
const LOCAL_FRAME_CAPACITY: i32 = 4;

/// The app's private files dir, such as `/data/data/com.scadoshi.count/files`.
///
/// The JNI calls run in their own local reference frame, so the references
/// they create are freed on return even on an already attached thread.
///
/// # Errors
///
/// An [`io::Error`] carrying the JNI failure when the JVM can't be attached
/// or a call throws. A thrown exception is cleared before the error returns.
pub fn files_dir() -> io::Result<PathBuf> {
    let ctx = ndk_context::android_context();
    // SAFETY: ndk-context holds a valid JavaVM for the running app.
    let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }.map_err(io::Error::other)?;
    let mut env = vm.attach_current_thread().map_err(io::Error::other)?;
    // SAFETY: ndk-context holds the app's Context jobject, alive while the
    // app runs.
    let context = unsafe { JObject::from_raw(ctx.context().cast()) };
    let read = env.with_local_frame(LOCAL_FRAME_CAPACITY, |env| -> jni::errors::Result<String> {
        let dir = env
            .call_method(&context, "getFilesDir", "()Ljava/io/File;", &[])?
            .l()?;
        let path = env
            .call_method(&dir, "getAbsolutePath", "()Ljava/lang/String;", &[])?
            .l()?;
        Ok(env.get_string(&JString::from(path))?.into())
    });
    match read {
        Ok(path) => Ok(PathBuf::from(path)),
        Err(e) => {
            // A call that threw leaves its exception pending on this thread,
            // where the next JNI call would abort the app.
            if env.exception_check().unwrap_or(false) {
                let _ = env.exception_describe();
                let _ = env.exception_clear();
            }
            Err(io::Error::other(e))
        }
    }
}
