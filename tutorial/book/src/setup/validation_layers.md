# Validation layers

**Code:** [main.rs](https://github.com/KyleMayes/vulkanalia/tree/master/tutorial/src/02_validation_layers.rs)

Vulkan API는 최소한의 드라이버 오버헤드를 중심으로 설계되었고 이러한 목표의 징후 중 하나는 기본적으로 API에서 오류검사가 제한적입니다. 열거를 잘못된 값으로 세팅하는 등의 작은 실수가 일반적으로 핸들링되지 않고 단순히 크래시나 undefined behavior를 발생시킵니다. Vulkan은 하고있는 일에 매우 명시적으로 할것을 요구하며 이것은 새로운 GPU feature를 사용하는거나 이것을 logical device creation time에 요청해야하는 것을 잊는 등의 쉬운 실수를 하게 합니다.

그러나 그것은 API에 그러한 검증을 넣는것이 불가능한 것을 의미하지는 않습니다. Vulkan은 validation layers로 알려진 우아한 시스템을 도입합니다. Validation layer는 optional component입니다. 이 컴포넌트들은 Vulkan function call을 후킹해서 추자적인 연산을 적용합니다. 일반적인 validation layer의 연산은 다음과 같습니다.

- misuse를 감지하기 위해 specification에 대한 파라미터 값 검사
- 리소스 leak을 찾기 위해 오브젝트의 생성과 파괴를 추적
- 호출이 발생하는 쓰레드를 추적하여 쓰레드 안정성 검사
- 모든 호출과 파라미터를 표준 출력에 로깅
- profiling과 replaying을 위해 Vulkan 호출을 추적

diagnostics validation layer의 함수 구현이 어떻게 구현되는지 예시는 이렇습니다. (C언어로)

```c
VkResult vkCreateInstance(
    const VkInstanceCreateInfo* pCreateInfo,
    const VkAllocationCallbacks* pAllocator,
    VkInstance* instance
) {
    if (pCreateInfo == nullptr || instance == nullptr) {
        log("Null pointer passed to required parameter!");
        return VK_ERROR_INITIALIZATION_FAILED;
    }

    return real_vkCreateInstance(pCreateInfo, pAllocator, instance);
}
```

이런 validation layer는 관심있는 모든 디버깅 기능들에 포함하기 위해 쌓아둘 수 있습니다. 단순히 디버그 빌드를 위해 validation layer를 활성화 할 수 있고, 릴리즈 빌드를 위해 완전히 끌 수 있습니다. 이런 방식은 두가지 모두의 장점을 얻을 수 있습니다.

Vulkan은 built-in validation layer가 딸려있지 않습니다. 그러니 LunarG Vulkan SDK는 일반적인 오류를 체크하기 위한 괜찮은 layer세트를 제공합니다. 그리고 그런 layer 세트는 완전히 [오픈소스](https://github.com/KhronosGroup/Vulkan-ValidationLayers)라서 이게 검증하는 실수가 어떤 종류인지 확인해 볼 수 있고 기여할 수 있습니다. validation layer를 사용하는 것은 undefined behavior에 실수로 의존하는 다른 드라이버에서 애플리케이션이 중단되는것을 막기 위한 가장 좋은 방법입니다.

validation layer는 시스템에 설치된 경우에만 사용할 수 있습니다. 예를 들어, LunarG validation layer는 Vulkan SDK가 설치된 PC에서만 이용가능합니다.

Vulkan에는 이전에 인스턴스별, 장치별로 두 가지 다른 방식의 validation layer가 있었습니다. 이 아이디어는 instance layer가 인스턴스같은 전역 Vulkan 객체와 관련된 호출만 체크하고, device specific layer는 특정 GPU와 관련된 호출만 체크하는 것입니다. device specific layer는 이제 deprecated되었고, 이것은 instance validation layer가 모든 Vulkan call에 적용된다는 것을 의미합니다. specification 문서는 여전히 몇가지 구현에 필요한 호환성을 위해 device 수준의 validation layer를 활성화하는것을 권장합니다. 나중에 보겠지만, logical device레벨과 같은 layer를 지정할겁니다.

시작하기 전에, 이번 챕터를 위해 새로운 임포트가 필요합니다.

```rust
use std::collections::HashSet;
use std::ffi::CStr;
use std::os::raw::c_void;

use vulkanalia::vk::ExtDebugUtilsExtension;
```

`HashSet`는 지원되는 layer를 저장하거나 querying하는데 사용되고 나머지 임포트는 [`vk::ExtDebugUtilsExtension`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/trait.ExtDebugUtilsExtension.html)의 exception을 사용하여 validation layer로부터 로그 메세지를 작성하는 함수에서 사용될겁니다. 그리고 그 함수는 디버깅 기능을 관리하기 위한 커맨드 wrapper를 제공합니다.

## Using validation layers

이 섹션에서는 Vulkan SDK에서 제공된 standard diagnostics layer를 활성화할겁니다. extension처럼, validation layer도 레이어들의 이름을 지정함으로써 활성화되어야 합니다. 모든 유용한 standard validation은 `VK_LAYER_KHRONOS_validation`으로 알려진 SDK에 포함된 layer에 번들되어 있습니다.

활성화할 레이어와 그 레이어들을 활성화/비활성화를 지정하기 위해 프로그램에 두 개의 configuration 변수를 추가합니다. 활성화에 대한 값을 프로그램이 디버그에서 컴파일되는지 아닌지에 따라 결정되도록 합니다.

```rust
const VALIDATION_ENABLED: bool =
    cfg!(debug_assertions);

const VALIDATION_LAYER: vk::ExtensionName =
    vk::ExtensionName::from_bytes(b"VK_LAYER_KHRONOS_validation");
```

`create_instance`함수에 새로운 코드를 추가합니다. 이 함수는 지원되는 instance layer를 `HashSet`에 모으고, validation layer가 이용가능한지 체크하고, validation layer를 포함하는 레이어 이름 리스트를 만듭니다. 이 코드는 [`vk::ApplicationInfo`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.ApplicationInfo.html)의 구조체 생성 바로 아래로 갑니다.

```rust
let available_layers = entry
    .enumerate_instance_layer_properties()?
    .iter()
    .map(|l| l.layer_name)
    .collect::<HashSet<_>>();

if VALIDATION_ENABLED && !available_layers.contains(&VALIDATION_LAYER) {
    return Err(anyhow!("Validation layer requested but not supported."));
}

let layers = if VALIDATION_ENABLED {
    vec![VALIDATION_LAYER.as_ptr()]
} else {
    Vec::new()
};
```

그러면 `enabled_layer_names` 빌더 메소드에 call을 추가함으로써 [`vk::InstanceCreateInfo`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.InstanceCreateInfo.html)에서 요청된 layer들을 지정해야 합니다.

```rust
let info = vk::InstanceCreateInfo::builder()
    .application_info(&application_info)
    .enabled_layer_names(&layers)
    .enabled_extension_names(&extensions)
    .flags(flags);
```

이제 프로그램을 디버그 모드로 실행하고 `Validation layer requested but not supported.`메세지가 뜨지 않는지 확인합니다. 만약 뜬다면, FAQ를 살펴봅니다.  이 검사를 통과하면, [`create_instance`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/trait.EntryV1_0.html#method.create_instance)는 [`vk::ErrorCode::LAYER_NOT_PRESENT`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.ErrorCode.html#associatedconstant.LAYER_NOT_PRESENT)를 반환하지 않지만, 프로그램을 실행해서 확인해야 합니다.

## Message callback

validation layer는 기본적으로 디버그 메세지를 stdout에 출력할 것입니다. 그러나 우리는 프로그램에 명시적인 콜팩을 제공함으로써 출력을 핸들링할 수 있습니다. 모든 메세지가 필수적인(fatal)에러가 아니기 때문에, 보고싶어하는 메세지의 종류를 결정하도록 해줍니다. 이런 핸들링을 하기 싫다면 당장은 이 챕터를 스킵해도 됩니다.

프로그램에서 메세지와 관련 정보를 핸들링하는 callback을 set up하기 위해, [`VK_EXT_debug_utils`](https://www.khronos.org/registry/vulkan/specs/1.4-extensions/man/html/VK_EXT_debug_utils.html) extension을 사용하는 callback을 이용해서 디버그 메신저를 set up해야 합니다.

`create_instance`에 코드를 좀 더 추가합니다. 이번에는 `extension`리스트를 수정 가능하도록 만들어서 validation layer가 활성화되어 있을 때 디버그 유틸리리 extension을 추가합니다.

```rust
let mut extensions = vk_window::get_required_instance_extensions(window)
    .iter()
    .map(|e| e.as_ptr())
    .collect::<Vec<_>>();

if VALIDATION_ENABLED {
    extensions.push(vk::EXT_DEBUG_UTILS_EXTENSION.name.as_ptr());
}
```

`vulkanalia`는 각 Vulkan extension에 메타데이터 컬렉션을 제공합니다. 이번 케이스에서는 로딩할 extension의 이름이 필요하므로 [`vk::EXT_DEBUG_UTILS_EXTENSION`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/constant.EXT_DEBUG_UTILS_EXTENSION.html)구조체의 `name`필드를 desired extension names에 추가합니다.

프로그램을 실행하고 [`vk::ErrorCode::EXTENSION_NOT_PRESENT`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.ErrorCode.html#associatedconstant.EXTENSION_NOT_PRESENT)오류가 없는지 확인합니다. 진짜로 이 extension의 존재를 확인하지 않아도 됩니다. 이 확장은 validation layer의 이용가능성에 의해 암시될것이기 때문입니다.

이제 디버그 콜백 함수가 어떤지 살펴봅시다. `debug_callback`라는 새로운 `extern "system"` 함수를 추가합니다. 이 함수는 [`vk::PFN_vkDebugUtilsMessengerCallbackEXT`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/type.PFN_vkDebugUtilsMessengerCallbackEXT.html) 프로토타입과 일치합니다. `extern "system"`은 Vulkan이 Rust함수를 호출하도록 허용하기 위해 필요합니다.

```rust
extern "system" fn debug_callback(
    severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    type_: vk::DebugUtilsMessageTypeFlagsEXT,
    data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _: *mut c_void,
) -> vk::Bool32 {
    let data = unsafe { *data };
    let message = unsafe { CStr::from_ptr(data.message) }.to_string_lossy();

    if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::ERROR {
        error!("({:?}) {}", type_, message);
    } else if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::WARNING {
        warn!("({:?}) {}", type_, message);
    } else if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::INFO {
        debug!("({:?}) {}", type_, message);
    } else {
        trace!("({:?}) {}", type_, message);
    }

    vk::FALSE
}
```

첫 번째 피라미터는 메세지의 심각도를 지정합니다. 이것은 다음의 플래그 중 하나입니다.

- [`vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.DebugUtilsMessageSeverityFlagsEXT.html#associatedconstant.VERBOSE) – Diagnostic 메세지
- [`vk::DebugUtilsMessageSeverityFlagsEXT::INFO`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.DebugUtilsMessageSeverityFlagsEXT.html#associatedconstant.INFO) – 리소스 생성과 같은 정보적인 메세지
- [`vk::DebugUtilsMessageSeverityFlagsEXT::WARNING`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.DebugUtilsMessageSeverityFlagsEXT.html#associatedconstant.WARNING) – 오류까지는 아니지만 애플리케이션에서 매우 버그에 가까운 행동에 대한 메세지
- [`vk::DebugUtilsMessageSeverityFlagsEXT::ERROR`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.DebugUtilsMessageSeverityFlagsEXT.html#associatedconstant.ERROR) – 잘못되거나 크래시를 일으키는 행동에 대한 메세지

이 열거형의 값들은 메세지가 몇개의 심각도 수준과 비교하여 같거나 나쁜지 확인하기 위해 비교연산자를 사용하는 방식으로 set up됩니다. 메세지의 심각도는 메세지를 로깅할 때 어떤 `log`매크로를 쓸 지 경정하기 위해 쓰입니다.

`type_`파라미터는 다음과 같은 값들을 가질 수 있습니다.

- [`vk::DebugUtilsMessageTypeFlagsEXT::GENERAL`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.DebugUtilsMessageTypeFlagsEXT.html#associatedconstant.GENERAL) – specification이나 performance와 관련없는 어떤 이벤트가 발생.
- [`vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.DebugUtilsMessageTypeFlagsEXT.html#associatedconstant.VALIDATION) – specification을 위반하거나 실수할 가능성을 가리키는 뭔가가 발생.
- [`vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.DebugUtilsMessageTypeFlagsEXT.html#associatedconstant.PERFORMANCE) – 잠재적인 비-최적화된 Vulkan의 사용

`data`파라미터는 그 메세지의 디테일을 포함하는 [`vk::DebugUtilsMessengerCallbackDatEXT`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.DebugUtilsMessengerCallbackDataEXT.html)를 가리키며 가장 중요한 정보는 다음입니다.

- `message` – null로 끝나는 스트링(`*const c_char`)으로 표현된 디버그 메세지
- `objects` – 메세지와 관련된 Vulkan 오브젝트의 배열
- `object_count` – 배열 내의 오브젝트들의 수

마지막으로, 마지막 파라미터는(여기서는 `_`로 무시됨) 포인터를 포함합니다. 포인터는 callback의 setup과정에서 설정되었고 이를 통해 사용자 자신의 데이터를 넘길 수 있습니다.

callback은 (Vulkan) boolean을 반환합니다. 이 boolean은 validation layer 메시지를 트리거한 Vulkan이 abort되어야하는지를 가리킵니다. callback이 true라면, 그 call은 [`vk::ErrorCode::VALIDATION_FAILED_EXT`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.ErrorCode.html#associatedconstant.VALIDATION_FAILED_EXT)오류 코드와 함께 abort됩니다. 이런 상황은 보통 validation layer를 테스트하는 경우에만 사용됩니다. 따라서 항상 [`vk::FALSE`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/constant.FALSE.html)를 반환시키면 됩니다.

이제 남은것은 Vulkan에 callback함수를 알려주는 것입니다. 아마 조금 놀랍게도, Vulkan에서 debug callback도 명시적으로 만들어지고 파괴되는 핸들에 의해 관리되어야 합니다. 그러한 callback은 debug messenger의 일부이고 원하는 만큼 많이 가질 수 있습니다. `AppData`구조체에 필드를 추가합니다.

```rust
struct AppData {
    messenger: vk::DebugUtilsMessengerEXT,
}
```

이제 `create_instance`함수와 시그니처를 다음과 같이 수정합니다.

```rust
unsafe fn create_instance(
    window: &Window,
    entry: &Entry,
    data: &mut AppData
) -> Result<Instance> {
    // ...

    let instance = entry.create_instance(&info, None)?;

    if VALIDATION_ENABLED {
        let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(vk::DebugUtilsMessageSeverityFlagsEXT::all())
            .message_type(
                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
            )
            .user_callback(Some(debug_callback));

        data.messenger = instance.create_debug_utils_messenger_ext(&debug_info, None)?;
    }

    Ok(instance)
}

```

> **Note:** Vulkan flags의 set에서 `all` 이라는 static 메소드(예를들어, `vk::DebugUtilsMessageSeverityFlagsExt::all()`)를 호출하는 것은, 이름에서 알 수 있듯이, `vulkanalia`에 의해 알려진 타입의 모든 flag를 포함하는 flag 세트를 반환합니다. flag의 완전한 세트는 특정 extension들만 활성화되거나 사용/타게팅하는 Vulkan버전보다 최신것에 의해 추가된 플래그가 추가되었을때만 유효한 플래그를 포함합니다.
>
> 위 코드에서 flag세트가 [특정 extension이 활성화된 경우에만 유효한](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VK_EXT_device_address_binding_report.html) flag([`vk::DebugUtilsMessageTypeFlagsEXT::DEVICE_ADDRESS_BINDING`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.DebugUtilsMessageTypeFlagsEXT.html#associatedconstant.DEVICE_ADDRESS_BINDING))를 포함하므로 원하는 [`vk::DebugUtilsMessageTypeFlagsEXT`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.DebugUtilsMessageTypeFlagsEXT.html) flags를 명시적으로 리스팅했습니다.
>
> 대부분의 경우에서 unsupported flag를 사용하는 것은 오류나 애플리케이션의 동작을 발생시키지 않습니다. 그러나 그런 플래그를 사용하는것은 validation layer가 활성화된 경우 분명히 validation error를 초래합니다(이 챕터에서 주목하는 부분입니다.).

먼저 Vulkan instance를 return expression에서 추출했고 debug callback에 추가하기위해 사용할 수 있습니다.

다음으로 [`vk::DebugUtilsMessengerCreateInfoEXT`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.DebugUtilsMessengerCreateInfoEXT.html) 구조체를 생성합니다. 이 구조제는 debug callback과 어떻게 호출되어야 할 지에 대한 정보를 제공합니다.

`message_severity`필드는 모든 심각도 타입을 지정할 수 있도록 해 주며, 콜백이 그러한 심각도에 대해 호출되기를 원합니다. 저는 모든 심각도의 메세지가 포함되기를 요청했습니다. 이것은 많은 양의 verbose general debug info를 만들어내지만, 관심이 없는 경우에 log level를 사용하여 필터링할 수 있습니다.

마찬가지로, `message_type`필드는 콜백이 알림을 받을 메세지의 타입을 필터링하게 해줍니다. 저는 여기서 모든 타입을 활성화했습니다. 유용하지 않다면, 언제나 몇개를 비활성화 할 수 있습니다.

Finally, `user_callback`필드는 콜백 함수를 지정합니다. 마지막 파라미터를 통해 콜백함수로 전달될 `user_data`필드에 대한 mutable reference를 선택적으로 전달할 수 있습니다. 예를들어, `AppData`구조체에 대한 포인터를 넘기기 위해 사용할 수 있습니다.

Lastly, Vulkan instance를 이용해서 debug callback를 등록하기 위해  [`create_debug_utils_messenger_ext`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/trait.ExtDebugUtilsExtension.html#method.create_debug_utils_messenger_ext)를 호출합니다.

지금은 `create_instance`함수가 `AppData`를 참조하므로, `App`과  `App::create`를 수정할 필요가 있습니다.

> **Note:** `AppData::default()`는 `AppData`구조체에서 `#[derive(Default)]`에 의해 생성된 [`Default` trait](https://doc.rust-lang.org/std/default/trait.Default.html)의 구현을 사용할겁니다. 이것은 `Vec`같은 빈 리스트로 초기화되는 컨테이너와 [`vk::DebugUtilsMessengerEXT`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.DebugUtilsMessengerEXT.html)같은 null handle로 초기화된 Vulkan handle를 만들어낼겁니다. 만약 Vulkan handle이 사용되기 전에 적절히 초기화되지 않는다면, 이 챕터에서 활성화한 validation layer는 정확히 무엇을 놓친건지 알려줄겁니다.

```rust
struct App {
    entry: Entry,
    instance: Instance,
    data: AppData,
}

impl App {
    unsafe fn create(window: &Window) -> Result<Self> {
        // ...
        let mut data = AppData::default();
        let instance = create_instance(window, &entry, &mut data)?;
        Ok(Self { entry, instance, data })
    }
}
```

생성한 [`vk::DebugUtilsMessengerEXT`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.DebugUtilsMessengerEXT.html)오브젝트는 app이 종료되기 전에 청소되어야합니다. 이 작업을 instance를 파괴하기 전에 `App::destroy`에서 할 겁니다.

```rust
unsafe fn destroy(&mut self) {
    if VALIDATION_ENABLED {
        self.instance.destroy_debug_utils_messenger_ext(self.data.messenger, None);
    }

    self.instance.destroy_instance(None);
}
```

## Debugging instance creation and destruction

비록 프로그램에 validation layer를 사용해서 디버깅을 추가했지만, 아직 모든것을 커버하지 않습니다. [`create_debug_utils_messenger_ext`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/trait.ExtDebugUtilsExtension.html#method.create_debug_utils_messenger_ext) call은 생성되기 위해 유효한 instance를 요구하고 [`destroy_debug_utils_messenger_ext`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/trait.ExtDebugUtilsExtension.html#method.destroy_debug_utils_messenger_ext)는 instance가 파괴되기 전에 호출되어야 합니다. 이것은 [`create_instance`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/trait.EntryV1_0.html#method.create_instance)과 [`destroy_instance`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/trait.InstanceV1_0.html#method.destroy_instance) call에서 발생하는 어떠한 issue도 디버깅하지 못하도록 합니다.

그러나 [extension documentation](https://github.com/KhronosGroup/Vulkan-Docs/blob/77d9f42e075e6a483a37351c14c5e9e3122f9113/appendices/VK_EXT_debug_utils.txt#L84-L91)문서를 꼼꼼히 읽는다면, 두 function call을 위해 특별히 별도의 debug utils messenger를 만들어낸 방법이 있다는 것을 볼겁니다. 이 debug utils messenger는 단순히  [`vk::InstanceCreateInfo`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.InstanceCreateInfo.html)의 `next` extension필드에 [`vk::DebugUtilsMessengerCreateInfoEXT`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.DebugUtilsMessengerCreateInfoEXT.html) 구조체에 대한 포인터를 넘겨줄 것을 요구합니다. 이것을 하기 전에, Vulkan에서 어떻게 구조체를 확장하는지 논의해봅시다.

많은 Vulkan 구조체에서 보이는 `s_type`필드는 Overview챕터의 [Builders section](https://kylemayes.github.io/vulkanalia/overview.html#builders)에서 짧게 언급되었습니다. 이 필드는 구조체의 타입을 나타내는 [`vk::StructureType`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.StructureType.html)의 variant로 설정되어야 합니다(예를들어, [`vk::ApplicationInfo`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.ApplicationInfo.html) 구조체라면 [`vk::StructureType::APPLICATION_INFO`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.StructureType.html#associatedconstant.APPLICATION_INFO)).

이 필드의 목적이 궁금할 수도 있습니다: "Vulkan command로 구조체를 넘길 때 Vulkan은 이미 타입을 알고 있지 않나요?". 이 필드의 목적은 Vulkan 구조체에서 `s_type`과 항상 같이다니는 `next`필드를 wrap up하는 것입니다: Vulkan 구조체를 다른 Vulkan구조체로의 확장가능성

Vulkan에서 `next`필드는 아마도 [structure pointer chain](https://www.khronos.org/registry/vulkan/specs/1.2-extensions/html/vkspec.html#fundamentals-validusage-pNext)를 지정하기 위해 사용될겁니다. `next`는 null이거나 Vulkan에 의해 구조체로 확장이 허가된 Vulkan 구조체에 대한 포인터일 수 있습니다. 이 구조체의 체인에서 각 구조체는 root 구조체가 넘겨질 Vulkan command에 추가적인 정보를 제공하기 위해 사용됩니다. 이 Vulkan의 기능은 backwards compabilitity를 깨지 않고 Vulkan command의 기능을 확장할 수 있게 해줍니다.

Vulkan command에 이러한 구조체 체인을 넘기면, Vulkan command는 구조체로부터 모든 정보를 수집하기 위해 순회합니다. 이 때문에, Vulkan은 체인의 각 구조체의 타입을 알지 못합니다. 따라서 `s_type`필드가 필요합니다.

`vulkanalia`에 의해 제공된 builder는 type-safe한 방식으로 이런 포인터 체인을 쉽게 만들 수 있도록 해줍니다. 예를들어, [`vk::InstanceCreateInfoBuilder`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.InstanceCreateInfoBuilder.html) builder에서, 특히 `push_next`메소드를 봅니다. 이 메소드는 [`vk::ExtendsInstanceCreateInfo`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/trait.ExtendsInstanceCreateInfo.html) 트레잇이 구현된 어느 Vulkan 구조체든 [`vk::InstanceCreateInfo`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.InstanceCreateInfo.html)에 대한 포인터 체인에 추가할 수 있게 해 줍니다.

한가지 그런 구조체는 [`vk::DebugUtilsMessengerCreateInfoEXT`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.DebugUtilsMessengerCreateInfoEXT.html)입니다. 이를 사용하여 [`vk::InstanceCreateInfo`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.InstanceCreateInfo.html) 구조체를 확장하여 debug callback을 설정합니다. 이것을 하기 위해 `create_instance`함수를 수정합니다. 이번에는 `info`구조체를 mutable로 만들고 mutable `debug_info`구조체를 `info`밑으로 이동시켜서 `info`의 포인터 체인에 추가합니다.

```rust
let mut info = vk::InstanceCreateInfo::builder()
    .application_info(&application_info)
    .enabled_layer_names(&layers)
    .enabled_extension_names(&extensions)
    .flags(flags);

let mut debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
    .message_severity(vk::DebugUtilsMessageSeverityFlagsEXT::all())
    .message_type(
        vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
            | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
            | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
    )
    .user_callback(Some(debug_callback));

if VALIDATION_ENABLED {
    info = info.push_next(&mut debug_info);
}
```

> **Note:** 같은 심각도, 타입 그리고 callback을 가지고 같은 debug info를 사용하여 [`create_debug_utils_messenger_ext`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/trait.ExtDebugUtilsExtension.html#method.create_debug_utils_messenger_ext)를 호출하고 [`vk::InstanceCreateInfo`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.InstanceCreateInfo.html) instance의 extension으로 추가하는것이 중복처럼 보일 수도 있습니다. 그러나 이 두가지 사용은 다른 목적을 제공합니다. 여기서의 사용([`vk::InstanceCreateInfo`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.InstanceCreateInfo.html)에 debug info를 추가하는것)은 instance의 생성과 파과과정에 디버깅을 설정는 것입니다. [`create_debug_utils_messenger_ext`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/trait.ExtDebugUtilsExtension.html#method.create_debug_utils_messenger_ext)를 호출하는 것은 다른 모든 것들에 대한 영구적인 디버깅을 세팅하는것입니다. [Vulkan specification의 관련 문서의](https://registry.khronos.org/vulkan/specs/1.3-extensions/html/chap4.html#VkInstanceCreateInfo) "To capture events that occur while creating or destroying an instance" 로 시작하는 문단을 보십시오.

`debug_info`는 [`create_instance`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/trait.EntryV1_0.html#method.create_instance)의 호출 이 완료되기 전에 살아있어야 하기 때문에 조건문 밖에서 정의되어야 합니다. 운이 좋게도, Rust 컴파일러에 의존해서 충분히 오래 살지 못하는 구조체를 포인터 체인에 넣는것을 방지할 수 있습니다. 왜냐하면 `vulkanalia` 빌더를 위한 lifetime이 정의되어있기 때문입니다.

이제 프로그램을 실행하고 debug callback로부터 로그를 볼 수 있지만, 먼저 `RUST_LOG`환경변수를 설정해서 `pretty_env_logger`가 관심있는 레벨의 로그를 활성화 하도록 합니다. 최초로 로그 레벨을 `debug`로 설정해서 작동하는지 확인합니다. 여기에 Windows(PowerShell)에서의 예시가 있습니다.

![log](https://kylemayes.github.io/vulkanalia/images/validation_layer_test.png)

모든것들이 작동한다면, 경고나 오류메세지를 보지 않을겁니다. 오류를 디버깅하지 않는다면, 앞으로가서 `RUST_LOG`를 사용해서 로그레벨을 `info`로 올리고 로그의 verbosity를 줄이고 싶을겁니다.

## Configuration

[`vk::DebugUtilsMessengerCreateInfoEXT`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.DebugUtilsMessengerCreateInfoEXT.html) 구조체의 플래그에 명시된것보다 더 많은 validation layer의 동작이 있습니다. Vulkan SDK를 살펴보고 `Config`디렉터리로 가세요. 거기에서 `vk_layer_settings.txt`파일을 찾을 수 있을겁니다. 그 파일이 layer를 어떻게 설정하는지 설명해 줄 겁니다.

애플리케이션을 위한 layer setting을 configure하기 위해서, 프로젝트의 executable 디렉터리로 파일을 복사합니다. 그리고 요구된 동작을 세팅하기 위해 지시를 따릅니다. 그러나, 이 튜토리얼의 나머지 부분에서 기본 세팅을 쓸겁니다.

이 튜토리얼에서 몇가지 의도된 실수를 만들어서 어떻게 validation layer가 그것들을 캐치하는지 보여주고 정확히 Vulkan에서 뭘하는지 아는것이 얼마나 중요한지 알려줄겁니다. 이제 시스템에서 Vulkan device를 찾아봅니다.
