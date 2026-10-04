#[inline]
pub unsafe fn CloseHandle(hobject: HANDLE) -> windows_core::BOOL {
    windows_core::link!("kernel32.dll" "system" fn CloseHandle(hobject : HANDLE) -> windows_core::BOOL);
    unsafe { CloseHandle(hobject) }
}
#[inline]
pub unsafe fn ConvertSidToStringSidW(
    sid: PSID,
    stringsid: *mut windows_core::PWSTR,
) -> windows_core::BOOL {
    windows_core::link!("advapi32.dll" "system" fn ConvertSidToStringSidW(sid : PSID, stringsid : *mut windows_core::PWSTR) -> windows_core::BOOL);
    unsafe { ConvertSidToStringSidW(sid, stringsid as _) }
}
#[inline]
pub unsafe fn ConvertStringSecurityDescriptorToSecurityDescriptorW<P0>(
    stringsecuritydescriptor: P0,
    stringsdrevision: u32,
    securitydescriptor: *mut PSECURITY_DESCRIPTOR,
    securitydescriptorsize: Option<*mut u32>,
) -> windows_core::BOOL
where
    P0: windows_core::Param<windows_core::PCWSTR>,
{
    windows_core::link!("advapi32.dll" "system" fn ConvertStringSecurityDescriptorToSecurityDescriptorW(stringsecuritydescriptor : windows_core::PCWSTR, stringsdrevision : u32, securitydescriptor : *mut PSECURITY_DESCRIPTOR, securitydescriptorsize : *mut u32) -> windows_core::BOOL);
    unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            stringsecuritydescriptor.param().abi(),
            stringsdrevision,
            securitydescriptor as _,
            securitydescriptorsize.unwrap_or(core::mem::zeroed()) as _,
        )
    }
}
#[inline]
pub unsafe fn GetCurrentProcess() -> HANDLE {
    windows_core::link!("kernel32.dll" "system" fn GetCurrentProcess() -> HANDLE);
    unsafe { GetCurrentProcess() }
}
#[inline]
pub unsafe fn GetNamedPipeClientProcessId(
    pipe: HANDLE,
    clientprocessid: *mut u32,
) -> windows_core::BOOL {
    windows_core::link!("kernel32.dll" "system" fn GetNamedPipeClientProcessId(pipe : HANDLE, clientprocessid : *mut u32) -> windows_core::BOOL);
    unsafe { GetNamedPipeClientProcessId(pipe, clientprocessid as _) }
}
#[inline]
pub unsafe fn GetProcessTimes(
    hprocess: HANDLE,
    lpcreationtime: *mut FILETIME,
    lpexittime: *mut FILETIME,
    lpkerneltime: *mut FILETIME,
    lpusertime: *mut FILETIME,
) -> windows_core::BOOL {
    windows_core::link!("kernel32.dll" "system" fn GetProcessTimes(hprocess : HANDLE, lpcreationtime : *mut FILETIME, lpexittime : *mut FILETIME, lpkerneltime : *mut FILETIME, lpusertime : *mut FILETIME) -> windows_core::BOOL);
    unsafe {
        GetProcessTimes(
            hprocess,
            lpcreationtime as _,
            lpexittime as _,
            lpkerneltime as _,
            lpusertime as _,
        )
    }
}
#[inline]
pub unsafe fn GetSecurityDescriptorDacl(
    psecuritydescriptor: PSECURITY_DESCRIPTOR,
    lpbdaclpresent: *mut windows_core::BOOL,
    pdacl: *mut PACL,
    lpbdacldefaulted: *mut windows_core::BOOL,
) -> windows_core::BOOL {
    windows_core::link!("advapi32.dll" "system" fn GetSecurityDescriptorDacl(psecuritydescriptor : PSECURITY_DESCRIPTOR, lpbdaclpresent : *mut windows_core::BOOL, pdacl : *mut PACL, lpbdacldefaulted : *mut windows_core::BOOL) -> windows_core::BOOL);
    unsafe {
        GetSecurityDescriptorDacl(
            psecuritydescriptor,
            lpbdaclpresent as _,
            pdacl as _,
            lpbdacldefaulted as _,
        )
    }
}
#[inline]
pub unsafe fn GetSecurityDescriptorSacl(
    psecuritydescriptor: PSECURITY_DESCRIPTOR,
    lpbsaclpresent: *mut windows_core::BOOL,
    psacl: *mut PACL,
    lpbsacldefaulted: *mut windows_core::BOOL,
) -> windows_core::BOOL {
    windows_core::link!("advapi32.dll" "system" fn GetSecurityDescriptorSacl(psecuritydescriptor : PSECURITY_DESCRIPTOR, lpbsaclpresent : *mut windows_core::BOOL, psacl : *mut PACL, lpbsacldefaulted : *mut windows_core::BOOL) -> windows_core::BOOL);
    unsafe {
        GetSecurityDescriptorSacl(
            psecuritydescriptor,
            lpbsaclpresent as _,
            psacl as _,
            lpbsacldefaulted as _,
        )
    }
}
#[inline]
pub unsafe fn GetTokenInformation(
    tokenhandle: HANDLE,
    tokeninformationclass: TOKEN_INFORMATION_CLASS,
    tokeninformation: Option<*mut core::ffi::c_void>,
    tokeninformationlength: u32,
    returnlength: *mut u32,
) -> windows_core::BOOL {
    windows_core::link!("advapi32.dll" "system" fn GetTokenInformation(tokenhandle : HANDLE, tokeninformationclass : TOKEN_INFORMATION_CLASS, tokeninformation : *mut core::ffi::c_void, tokeninformationlength : u32, returnlength : *mut u32) -> windows_core::BOOL);
    unsafe {
        GetTokenInformation(
            tokenhandle,
            tokeninformationclass,
            tokeninformation.unwrap_or(core::mem::zeroed()) as _,
            tokeninformationlength,
            returnlength as _,
        )
    }
}
#[inline]
pub unsafe fn LocalFree(hmem: HLOCAL) -> HLOCAL {
    windows_core::link!("kernel32.dll" "system" fn LocalFree(hmem : HLOCAL) -> HLOCAL);
    unsafe { LocalFree(hmem) }
}
#[inline]
pub unsafe fn OpenProcess(dwdesiredaccess: u32, binherithandle: bool, dwprocessid: u32) -> HANDLE {
    windows_core::link!("kernel32.dll" "system" fn OpenProcess(dwdesiredaccess : u32, binherithandle : windows_core::BOOL, dwprocessid : u32) -> HANDLE);
    unsafe { OpenProcess(dwdesiredaccess, binherithandle.into(), dwprocessid) }
}
#[inline]
pub unsafe fn OpenProcessToken(
    processhandle: HANDLE,
    desiredaccess: u32,
    tokenhandle: *mut HANDLE,
) -> windows_core::BOOL {
    windows_core::link!("advapi32.dll" "system" fn OpenProcessToken(processhandle : HANDLE, desiredaccess : u32, tokenhandle : *mut HANDLE) -> windows_core::BOOL);
    unsafe { OpenProcessToken(processhandle, desiredaccess, tokenhandle as _) }
}
#[inline]
pub unsafe fn QueryFullProcessImageNameW(
    hprocess: HANDLE,
    dwflags: u32,
    lpexename: windows_core::PWSTR,
    lpdwsize: *mut u32,
) -> windows_core::BOOL {
    windows_core::link!("kernel32.dll" "system" fn QueryFullProcessImageNameW(hprocess : HANDLE, dwflags : u32, lpexename : windows_core::PWSTR, lpdwsize : *mut u32) -> windows_core::BOOL);
    unsafe { QueryFullProcessImageNameW(hprocess, dwflags, lpexename, lpdwsize as _) }
}
#[inline]
pub unsafe fn SetSecurityInfo(
    handle: HANDLE,
    objecttype: SE_OBJECT_TYPE,
    securityinfo: SECURITY_INFORMATION,
    psidowner: Option<PSID>,
    psidgroup: Option<PSID>,
    pdacl: Option<*const ACL>,
    psacl: Option<*const ACL>,
) -> u32 {
    windows_core::link!("advapi32.dll" "system" fn SetSecurityInfo(handle : HANDLE, objecttype : SE_OBJECT_TYPE, securityinfo : SECURITY_INFORMATION, psidowner : PSID, psidgroup : PSID, pdacl : *const ACL, psacl : *const ACL) -> u32);
    unsafe {
        SetSecurityInfo(
            handle,
            objecttype,
            securityinfo,
            psidowner.unwrap_or(core::mem::zeroed()) as _,
            psidgroup.unwrap_or(core::mem::zeroed()) as _,
            pdacl.unwrap_or(core::mem::zeroed()) as _,
            psacl.unwrap_or(core::mem::zeroed()) as _,
        )
    }
}
#[inline]
pub unsafe fn bind(s: SOCKET, name: *const SOCKADDR, namelen: i32) -> i32 {
    windows_core::link!("ws2_32.dll" "system" fn bind(s : SOCKET, name : *const SOCKADDR, namelen : i32) -> i32);
    unsafe { bind(s, name, namelen) }
}
#[inline]
pub unsafe fn listen(s: SOCKET, backlog: i32) -> i32 {
    windows_core::link!("ws2_32.dll" "system" fn listen(s : SOCKET, backlog : i32) -> i32);
    unsafe { listen(s, backlog) }
}
#[inline]
pub unsafe fn setsockopt(
    s: SOCKET,
    level: i32,
    optname: i32,
    optval: Option<*const i8>,
    optlen: i32,
) -> i32 {
    windows_core::link!("ws2_32.dll" "system" fn setsockopt(s : SOCKET, level : i32, optname : i32, optval : *const i8, optlen : i32) -> i32);
    unsafe {
        setsockopt(
            s,
            level,
            optname,
            optval.unwrap_or(core::mem::zeroed()) as _,
            optlen,
        )
    }
}
#[inline]
pub unsafe fn shutdown(s: SOCKET, how: i32) -> i32 {
    windows_core::link!("ws2_32.dll" "system" fn shutdown(s : SOCKET, how : i32) -> i32);
    unsafe { shutdown(s, how) }
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ACL {
    pub AclRevision: u8,
    pub Sbz1: u8,
    pub AclSize: u16,
    pub AceCount: u16,
    pub Sbz2: u16,
}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct ADDRESS_FAMILY(pub u16);
pub const AF_HYPERV: i32 = 34;
pub const DACL_SECURITY_INFORMATION: i32 = 4;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FILETIME {
    pub dwLowDateTime: u32,
    pub dwHighDateTime: u32,
}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct HANDLE(pub *mut core::ffi::c_void);
pub type HLOCAL = HANDLE;
pub const LABEL_SECURITY_INFORMATION: i32 = 16;
pub type PACL = *mut ACL;
pub const PROCESS_QUERY_LIMITED_INFORMATION: i32 = 4096;
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct PSECURITY_DESCRIPTOR(pub *mut core::ffi::c_void);
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct PSID(pub *mut core::ffi::c_void);
pub const SDDL_REVISION_1: i32 = 1;
pub const SD_SEND: i32 = 1;
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct SECURITY_INFORMATION(pub u32);
pub const SE_KERNEL_OBJECT: SE_OBJECT_TYPE = 6;
pub type SE_OBJECT_TYPE = i32;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SID_AND_ATTRIBUTES {
    pub Sid: PSID,
    pub Attributes: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SOCKADDR {
    pub sa_family: ADDRESS_FAMILY,
    pub sa_data: [i8; 14],
}
impl Default for SOCKADDR {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct SOCKET(pub usize);
pub const SOCKET_ERROR: i32 = -1;
pub const SOL_SOCKET: i32 = 65535;
pub const SOMAXCONN: i32 = 2147483647;
pub const SO_UPDATE_ACCEPT_CONTEXT: i32 = 28683;
pub type TOKEN_INFORMATION_CLASS = i32;
pub const TOKEN_QUERY: i32 = 8;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TOKEN_USER {
    pub User: SID_AND_ATTRIBUTES,
}
pub const TokenUser: TOKEN_INFORMATION_CLASS = 1;
