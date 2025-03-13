// uefi_bootloader/src/macros.rs

/// Convert a string literal to a UTF-16 `CStr16`.
///
/// This is useful for providing fixed strings to UEFI APIs.
#[macro_export]
macro_rules! cstr16 {
    ($str:expr) => {{
        const _STR: &str = $str;
        const LEN: usize = _STR.len();
        const NULL_LEN: usize = LEN + 1;
        
        #[allow(unused_unsafe)]
        const _STR_UTF16: &[u16; NULL_LEN] = &{
            let mut utf16 = [0u16; NULL_LEN];
            let mut i = 0;
            while i < LEN {
                let c = _STR.as_bytes()[i] as u16;
                utf16[i] = c;
                i += 1;
            }
            utf16[LEN] = 0;
            utf16
        };
        
        unsafe {
            uefi::data_types::CStr16::from_u16_with_nul_unchecked(_STR_UTF16)
        }
    }};
}