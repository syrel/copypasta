use crate::common::{ClipboardProvider, Result};
use jni::objects::{JObject, JString, JValue};
use std::ffi::CStr;

pub struct AndroidClipboardContext;

impl AndroidClipboardContext {
    #[inline]
    pub fn new() -> Result<Self> {
        Ok(AndroidClipboardContext)
    }
}

impl ClipboardProvider for AndroidClipboardContext {
    fn get_contents(&mut self) -> Result<String> {
        let ctx = ndk_context::android_context();
        let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }?;
        let mut env = vm.attach_current_thread()?;
        let class_ctx = env.find_class("android/content/Context")?;
        let service_field =
            env.get_static_field(class_ctx, "CLIPBOARD_SERVICE", "Ljava/lang/String;")?;
        let clipboard_manager = env
            .call_method(
                unsafe { JObject::from_raw(ctx.context() as jni::sys::jobject) },
                "getSystemService",
                "(Ljava/lang/String;)Ljava/lang/Object;",
                &[service_field.borrow()],
            )?
            .l()?;

        let clip_data = env
            .call_method(clipboard_manager, "getPrimaryClip", "()Landroid/content/ClipData;", &[])?
            .l()?;

        let item = env
            .call_method(
                clip_data,
                "getItemAt",
                "(I)Landroid/content/ClipData$Item;",
                &[0i32.into()],
            )?
            .l()?;

        let char_seq = env.call_method(item, "getText", "()Ljava/lang/CharSequence;", &[])?.l()?;

        let jstring =
            JString::from(env.call_method(char_seq, "toString", "()Ljava/lang/String;", &[])?.l()?);

        let java_str = env.get_string(&jstring)?;
        let output_string = unsafe { CStr::from_ptr(java_str.as_ptr()).to_owned().into_string()? };

        Ok(output_string)
    }

    fn set_contents(&mut self, text: String) -> Result<()> {
        let ctx = ndk_context::android_context();
        let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }?;
        let mut env = vm.attach_current_thread()?;
        let class_ctx = env.find_class("android/content/Context")?;
        let service_field =
            env.get_static_field(class_ctx, "CLIPBOARD_SERVICE", "Ljava/lang/String;")?;
        let clipboard_manager = env
            .call_method(
                unsafe { JObject::from_raw(ctx.context() as jni::sys::jobject) },
                "getSystemService",
                "(Ljava/lang/String;)Ljava/lang/Object;",
                &[service_field.borrow()],
            )?
            .l()?;

        let class_clip_data = env.find_class("android/content/ClipData")?;

        let clip_type = env.new_string("text").unwrap();
        let clip_content = env.new_string(text).unwrap();

        let clip_data = env.call_static_method(
            class_clip_data,
            "newPlainText",
            "(Ljava/lang/CharSequence;Ljava/lang/CharSequence;)Landroid/content/ClipData;",
            &[JValue::from(&clip_type), JValue::from(&clip_content)],
        )?;

        env.call_method(
            clipboard_manager,
            "setPrimaryClip",
            "(Landroid/content/ClipData;)V",
            &[clip_data.borrow()],
        )?
        .v()?;

        Ok(())
    }
}
