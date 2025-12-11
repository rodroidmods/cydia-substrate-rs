#[macro_export]
macro_rules! substrate_hook {
    (
        $(#[$attr:meta])*
        $vis:vis fn $name:ident($($arg:ident: $ty:ty),* $(,)?) $(-> $ret:ty)? $body:block
    ) => {
        paste::paste! {
            static mut [<ORIGINAL_ $name:upper>]: Option<unsafe extern "C" fn($($ty),*) $(-> $ret)?> = None;

            $(#[$attr])*
            $vis unsafe extern "C" fn [<hooked_ $name>]($($arg: $ty),*) $(-> $ret)? $body

            $vis fn [<install_ $name _hook>](target: *mut std::ffi::c_void) -> Result<(), &'static str> {
                if target.is_null() {
                    return Err("Target symbol is null");
                }

                let mut original: *mut std::ffi::c_void = std::ptr::null_mut();
                unsafe {
                    $crate::MSHookFunction(
                        target,
                        [<hooked_ $name>] as *mut std::ffi::c_void,
                        &mut original as *mut *mut std::ffi::c_void,
                    );
                    [<ORIGINAL_ $name:upper>] = Some(std::mem::transmute(original));
                }
                Ok(())
            }

            #[allow(dead_code)]
            $vis unsafe fn [<call_original_ $name>]($($arg: $ty),*) $(-> $ret)? {
                if let Some(original) = [<ORIGINAL_ $name:upper>] {
                    original($($arg),*)
                } else {
                    panic!("Original function not available");
                }
            }
        }
    };
}

#[macro_export]
macro_rules! ms_hook_symbol {
    ($image:expr, $symbol:expr, $hook_installer:expr) => {{
        let sym = $crate::ms_find_symbol($image, $symbol);
        if sym.is_null() {
            Err("Symbol not found")
        } else {
            $hook_installer(sym)
        }
    }};
}

#[macro_export]
macro_rules! define_hook {
    (
        fn $name:ident($($arg:ident: $ty:ty),* $(,)?) $(-> $ret:ty)?
    ) => {
        paste::paste! {
            static mut [<$name:upper _ORIGINAL>]: Option<unsafe extern "C" fn($($ty),*) $(-> $ret)?> = None;

            #[allow(dead_code)]
            pub unsafe fn [<$name _original>]($($arg: $ty),*) $(-> $ret)? {
                if let Some(f) = [<$name:upper _ORIGINAL>] {
                    f($($arg),*)
                } else {
                    panic!("Original function {} not hooked", stringify!($name));
                }
            }

            #[allow(dead_code)]
            pub unsafe fn [<install_ $name>](
                symbol: *mut std::ffi::c_void,
                replacement: unsafe extern "C" fn($($ty),*) $(-> $ret)?
            ) -> Result<(), &'static str> {
                if symbol.is_null() {
                    return Err("Symbol is null");
                }

                let mut orig: *mut std::ffi::c_void = std::ptr::null_mut();
                $crate::MSHookFunction(
                    symbol,
                    replacement as *mut std::ffi::c_void,
                    &mut orig as *mut *mut std::ffi::c_void,
                );
                [<$name:upper _ORIGINAL>] = Some(std::mem::transmute(orig));
                Ok(())
            }
        }
    };
}

#[macro_export]
macro_rules! hook_function {
    ($target:expr => $replacement:expr) => {{
        let mut original: *mut std::ffi::c_void = std::ptr::null_mut();
        unsafe {
            $crate::MSHookFunction(
                $target as *mut std::ffi::c_void,
                $replacement as *mut std::ffi::c_void,
                &mut original as *mut *mut std::ffi::c_void,
            );
        }
        original
    }};
}

#[macro_export]
macro_rules! find_and_hook {
    ($lib:expr, $symbol:expr, $replacement:expr) => {{
        let image = $crate::Image::by_name($lib)
            .ok_or("Failed to get image")?;
        let sym = $crate::ms_find_symbol(image.handle(), $symbol);
        if sym.is_null() {
            Err("Symbol not found")
        } else {
            $crate::ms_hook_function(sym, $replacement as *mut std::ffi::c_void)
        }
    }};
}
