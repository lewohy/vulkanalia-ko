// SPDX-License-Identifier: Apache-2.0

// DO NOT EDIT.
//
// This file has been generated by the Kotlin project in the `generator`
// directory from a Vulkan API registry.

#![allow(
    non_camel_case_types,
    non_snake_case,
    clippy::bad_bit_mask,
    clippy::let_unit_value,
    clippy::missing_safety_doc,
    clippy::missing_transmute_annotations,
    clippy::too_many_arguments,
    clippy::type_complexity,
    clippy::unnecessary_cast,
    clippy::upper_case_acronyms,
    clippy::useless_transmute
)]

use core::ffi::{c_char, c_void};

use crate::*;

/// <https://www.khronos.org/registry/vulkan/specs/latest/man/html/PFN_vkAllocationFunction.html>
pub type PFN_vkAllocationFunction = Option<
    unsafe extern "system" fn(*mut c_void, usize, usize, SystemAllocationScope) -> *mut c_void,
>;

/// <https://www.khronos.org/registry/vulkan/specs/latest/man/html/PFN_vkDebugReportCallbackEXT.html>
pub type PFN_vkDebugReportCallbackEXT = Option<
    unsafe extern "system" fn(
        DebugReportFlagsEXT,
        DebugReportObjectTypeEXT,
        u64,
        usize,
        i32,
        *const c_char,
        *const c_char,
        *mut c_void,
    ) -> Bool32,
>;

/// <https://www.khronos.org/registry/vulkan/specs/latest/man/html/PFN_vkDebugUtilsMessengerCallbackEXT.html>
pub type PFN_vkDebugUtilsMessengerCallbackEXT = Option<
    unsafe extern "system" fn(
        DebugUtilsMessageSeverityFlagsEXT,
        DebugUtilsMessageTypeFlagsEXT,
        *const DebugUtilsMessengerCallbackDataEXT,
        *mut c_void,
    ) -> Bool32,
>;

/// <https://www.khronos.org/registry/vulkan/specs/latest/man/html/PFN_vkDeviceMemoryReportCallbackEXT.html>
pub type PFN_vkDeviceMemoryReportCallbackEXT =
    Option<unsafe extern "system" fn(*const DeviceMemoryReportCallbackDataEXT, *mut c_void)>;

/// <https://www.khronos.org/registry/vulkan/specs/latest/man/html/PFN_vkFreeFunction.html>
pub type PFN_vkFreeFunction = Option<unsafe extern "system" fn(*mut c_void, *mut c_void)>;

/// <https://www.khronos.org/registry/vulkan/specs/latest/man/html/PFN_vkGetInstanceProcAddrLUNARG.html>
pub type PFN_vkGetInstanceProcAddrLUNARG =
    Option<unsafe extern "system" fn(Instance, *const c_char) -> PFN_vkVoidFunction>;

/// <https://www.khronos.org/registry/vulkan/specs/latest/man/html/PFN_vkInternalAllocationNotification.html>
pub type PFN_vkInternalAllocationNotification = Option<
    unsafe extern "system" fn(*mut c_void, usize, InternalAllocationType, SystemAllocationScope),
>;

/// <https://www.khronos.org/registry/vulkan/specs/latest/man/html/PFN_vkInternalFreeNotification.html>
pub type PFN_vkInternalFreeNotification = Option<
    unsafe extern "system" fn(*mut c_void, usize, InternalAllocationType, SystemAllocationScope),
>;

/// <https://www.khronos.org/registry/vulkan/specs/latest/man/html/PFN_vkReallocationFunction.html>
pub type PFN_vkReallocationFunction = Option<
    unsafe extern "system" fn(
        *mut c_void,
        *mut c_void,
        usize,
        usize,
        SystemAllocationScope,
    ) -> *mut c_void,
>;

/// <https://www.khronos.org/registry/vulkan/specs/latest/man/html/PFN_vkVoidFunction.html>
pub type PFN_vkVoidFunction = Option<unsafe extern "system" fn()>;
