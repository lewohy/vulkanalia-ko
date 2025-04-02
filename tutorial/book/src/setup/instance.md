# Instance

**Code:** [main.rs](https://github.com/KyleMayes/vulkanalia/tree/master/tutorial/src/01_instance_creation.rs)


가장 먼저 할 일은 *instance*를 만들어서 Vulkan 라이브러리를 초기화하는 것입니다. 생성한 instance는 애플리케이션과 Vulkan 라이브러리간의 커넥션입니다. instance를 생성하는 것은 애플리케이션에 드라이버에 대한 몇가지 디테일을 포함시키는것입니다. 시작하기위해, 다음을 임포트합니다.

```rust
use anyhow::{anyhow, Result};
use log::*;
use vulkanalia::loader::{LibloadingLoader, LIBRARY};
use vulkanalia::window as vk_window;
use vulkanalia::prelude::v1_0::*;
```

여기서 먼저 `anyhow`에서 [`anyhow!`](https://docs.rs/anyhow/latest/anyhow/macro.anyhow.html)매크로를 임포트합니다. 이 매크로는 `anyhow`오류 인스턴스를 쉽게 생성하는데 사용됩니다. 그리고 `log::*`를 임포트해서 `log`크레이트의 로깅 매크로를 사용합니다. 다음으로, `LibloadingLoader`를 임포트합니다. 이것은 `vulkanalia`의 `libloading` integration의 역할을 합니다. 이건 Vulkan shared library에서 초기 Vulkan 커맨드를 로드하기위해 사용합니다.
운영체제에 맞는 표준 Vulkan shared library 이름(예를들어 Windows에서는 `vulkan-1.dll`)이 `LIBRARY`로 임포트됩니다.

다음으로 `vulkanalia`의 window integration을 `vk_window`로 임포트합니다. 이 챕터에서 우리는 window에 렌더링하기 위해 필요한 전역 Vulkan extensions을 열거하기 위해 사용합니다. 이후 챕터에서는 `vk_window`또한 사용하여 Vulkan instance를 `winit` window와 링크시킬겁니다.

마지막으로 `vulkanalia`에서 Vulkan 1.0의 프렐루드를 임포트합니다. 이것은 이번 챕터와 이후 챕터에서 필요한 Vulkan과 관련된 다른 임포트를 제공합니다.

이제, 인스턴스를 생성하기 위해 애플리케이션의 몇몇 정보를 구조체에 채워야합니다. 이 데이터들은 기술적으로는 optional이지만, 몇몇 애플리케이션을 최적화하는데 유용한 정보를 드라이버에 제공할 수 있습니다(왜냐하면, 인스턴스는 특정 행동과 함께 잘 알려진 그래픽 엔진을 사용하기 때문에).  이 구조체는 [`vk::ApplicationInfo`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.ApplicationInfo.html)라고 불립니다. 그리고 이 구조체를 `create_instance`라는 새로운 함수로 만들것입니다. 이 함수는 window와 Vulkan entry point(나중에 생성합니다)를 파라미터로 갖고 Vulkan 인스턴스를 반환합니다.

```rust
unsafe fn create_instance(window: &Window, entry: &Entry) -> Result<Instance> {
    let application_info = vk::ApplicationInfo::builder()
        .application_name(b"Vulkan Tutorial\0")
        .application_version(vk::make_version(1, 0, 0))
        .engine_name(b"No Engine\0")
        .engine_version(vk::make_version(1, 0, 0))
        .api_version(vk::make_version(1, 0, 0));
}
```

함수의 파라미터 대신에 Vulkan의 많은 양의 정보가 구조체로 들어갑니다. 그리고 인스턴스를 생성하기 위해 충분한 정보를 제공할 구조체를 하나 더 채워야합니다. 다음 구조체는 optional이 아닙니다. 그리고 이 구조체는 Vulkan 드라이버에 global extension과 validation layer를 알려줍니다. 여기서 global이란 특정 장치가 아닌 전체적인 프로그램에 적용되는 것을 의미합니다. global은 몇 챕터 뒤에서 명확해집니다. 먼저, 필요한 global extension과 이 extensions들을 열거하고 null-terminated C 문자열(`char * const c_char`)로 변환하기 위해 `vulkanlia`의 window integration을 사용해야 합니다.

```rust
let extensions = vk_window::get_required_instance_extensions(window)
    .iter()
    .map(|e| e.as_ptr())
    .collect::<Vec<_>>();
```

필요한 global extension리스트를 가지고 함수로 넘어온 Vulkan entry point를 사용하면 Vulkan 인스턴스를 생성하고 반환할 수 있습니다.

```rust
let info = vk::InstanceCreateInfo::builder()
    .application_info(&application_info)
    .enabled_extension_names(&extensions);

Ok(entry.create_instance(&info, None)?)
```

Vulkan에서 일반적인 객체 생성 함수 파라미터의 패턴은 다음처럼 보입니다.

- 생성 정보를 위한 구조체의 reference
- optional인 커스텀 allocator callback의 reference, 튜토리얼에서는 항상 `None`를 씁니다.

지금은, entry point로부터 Vulkan instance를 생성하는 함수를 만들었습니다. 다음으로 Vulkan entry point를 만들어야 합니다. entry point는 instance support를 쿼리하고 인스턴스를 생성하는데 사용될 Vulkan 커맨드를 로딩합니다. 그러나 이걸 하기 전에, `App` 구조체에 몇 가지 필드를 추가해서 Vulkan entry point와 생성할 instance를 저장할 수 있게 합니다.

```rust
struct App {
    entry: Entry,
    instance: Instance,
}
```

이 필드들을 조작하기 위해 `App::create`메소드를 다음과 같이 업데이트합니다.

```rust
unsafe fn create(window: &Window) -> Result<Self> {
    let loader = LibloadingLoader::new(LIBRARY)?;
    let entry = Entry::new(loader).map_err(|b| anyhow!("{}", b))?;
    let instance = create_instance(window, &entry)?;
    Ok(Self { entry, instance })
}
```

여기서 Vulkan function loader를 처음으로 생성합니다. 이 로더는 초기  Vulkan 커맨드를 Vulkan shared library에서 로딩하는데 사용됩니다. 다음으로 Vulkan entry point를 만들었던 function loader를 사용해서 생성합니다. 로더는 Vulkan 인스턴스를 관리하기 위해 필요한 모든 커맨드를 로딩합니다. 마지막으로, 이제  만들었던 `create_instance`함수를 Vulkan entry point를 사용해서 호출할 수 있습니다.

## Cleaning up

[`Instance`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/struct.Instance.html)는 프로그램이 종료되자마자 파괴되어야 합니다. 인스턴스는 `App::destory`에서 [`destory_instance`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/trait.InstanceV1_0.html#method.destroy_instance)를 사용하여 파괴할 수 있습니다.

```rust
unsafe fn destroy(&mut self) {
    self.instance.destroy_instance(None);
}
```

객체를 생성하는데 사용된 Vulkan 커맨드처럼, 파괴하는데 사용되는 커맨드도 custom allocator callback에 대한 Optional reference를 가질 수 있습니다. 이전처럼, 디폴트 allocation behavior를 지시하기 위해 `None`를 넘겨줍니다.

## Non-conformant Vulkan implementations

모든 플랫폼이 Vulkan specification을 완전히 따르는 Vulkan API를 구현할정도로 운이 좋지는 않습니다.  어떤 플랫폼에서는 불가능하거나, 일관성 없는 구현/Vulkan 스펙에서 말하는 동작과 다른 구현을 사용하여 Vulkan 애플리케이션의 실제 동작이 상당히 다른 Vulkan feature가 있을 수 있습니다.

Vulkan SDK의 버전 1.3.216부터는 비-일관적인 Vulkan 구현을 사용하는 애플리케이션은 몇가지 추가 Vulkan extension을 활성화해야합니다. 이런 호환성 확장은 개발자들에게 그들의 애플리케이션이 비-일관적인 Vulkan의 구현을 사용하고 있다는 것과 개발자들이 모든 것이 Vulkan의 스펙에서 말한대로 작동하지 않을 수도 있다는 것을 강제로 알게 하는 것에 주 목적이 있습니다.

이 튜토리얼은 이러한 호환 Vulkan 확장을 활용해서 애플리케이션이 Vulkan 구현을 충분이 준수하지 않는 플랫폼에서도 실행되도록 합니다.

그러나, 이렇게 물을 수도 있습니다. "왜 그런 호환 확장을 활성화 하나요? 왜 입문자들을 위한 Vulkan 튜토리얼에서 그런 비주류 플랫폼을 걱정해야 합니까?" 밝혀진 바와 같이, 그렇게까지 비주류가 아닌 macOS도 그러한 Vulkan 구현을 충분히 따르지 못하는 플랫폼입니다.

introduction에서 언급했듯이., Apple은 그들만의 low-level graphic API인 [Metal](https://en.wikipedia.org/wiki/Metal_(API))을 갖고 있습니다. macOS를 위한 Vulkan SDK의 일부로 제공되는 Vulkan 구현체([MoltenVK](https://moltengl.com/))은 애플리케이션과 Metal사이에서 애플리케이션이 만들어내는 Vulkan API call을 Metal call로 번역합니다. [MoltenVK는 Vulkan specification을 완전히 준수하지 않기 때문에](https://www.lunarg.com/wp-content/uploads/2022/05/The-State-of-Vulkan-on-Apple-15APR2022.pdf) macOS를 지원하려면 앞서 언급한 호환 Vulkan extension을 활성화해야 합니다.

코멘트로, MoltenVK가 full-conformant하지 않지만, macOS에서 튜토리얼을 따라하는 동안 Vulkan specification과의 차이로 발생하는 문제를 마주하지는 않을겁니다.

## Enabling compatibility extensions

> **NOTE:** macOS에서 튜토리얼을 진행하고 있지 않더라도, 이번 섹션에서 추가된 코드 몇개는 튜토리얼의 나머지 부분에서 참조되므로 스킵하면 안됩니다.

사용하고 있는 Vulkan의 버전이 compatibility extension requirement를 소개하는 Vulkan버전보다 같거나 큰지 확인하고 싶을겁니다. 이 목표를 마음에 담고, 첫번째 임포트를 추가합니다.

```rust
use vulkanalia::Version;
```

임포트 한 수에, 최소 버전에 대한 상수를 정의합니다.

```rust
const PORTABILITY_MACOS_VERSION: Version = Version::new(1, 3, 216);
```

extension 열거와 인스턴스 생성 코드를 다음으로 바꿉니다.

```rust
let mut extensions = vk_window::get_required_instance_extensions(window)
    .iter()
    .map(|e| e.as_ptr())
    .collect::<Vec<_>>();

// Required by Vulkan SDK on macOS since 1.3.216.
let flags = if 
    cfg!(target_os = "macos") && 
    entry.version()? >= PORTABILITY_MACOS_VERSION
{
    info!("Enabling extensions for macOS portability.");
    extensions.push(vk::KHR_GET_PHYSICAL_DEVICE_PROPERTIES2_EXTENSION.name.as_ptr());
    extensions.push(vk::KHR_PORTABILITY_ENUMERATION_EXTENSION.name.as_ptr());
    vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR
} else {
    vk::InstanceCreateFlags::empty()
};

let info = vk::InstanceCreateInfo::builder()
    .application_info(&application_info)
    .enabled_extension_names(&extensions)
    .flags(flags);

```

이 코드는 애플리케이션이 Vulkan 구현을 준수하지 못하는 플랫폼에서 컴파일되고 Vulkan 버전이 우리가 정의한 최소 버전을 만족하거나 넘는다면, `KHR_PORTABILITY_ENUMERATION_EXTENSION`을 활성화합니다(여기서는 단순히 macOS인지 체크합니다).

또한 이 코드는 `KHR_GET_PHYSICAL_DEVICE_PROPERTIES2_EXTENSION`을 같은 조건에서 활성화합니다. 이 확장은 `KHR_PORTABILITY_SUBSET_EXTENSION` device 확장을 활성화하기 위해 필요합니다(logical device를 set up하는 이후 튜토리얼에서 추가됩니다).

## [`Instance`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/struct.Instance.html) vs  [`vk::Instance`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.Instance.html)

`create_instance`함수를 호출할 때, 반환되는것은 Vulkan 커맨드인 [`vkCreateInstance`](https://www.khronos.org/registry/vulkan/specs/1.4-extensions/man/html/vkCreateInstance.html)가 리턴하는 raw Vulkan 인스턴스([`vk::Instance`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.Instance.html))가 아닙니다. 대신 얻게되는 것은 `vulkanalia`에서 정의한 custom type입니다. 이 타입은 raw Vulkan 인스턴스와 특정 인스턴스에 로드된 커맨드들의 조합입니다.

우리가 사용한것이 [`Instance`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/struct.Instance.html)이며 (`vulkanalia` 프렐루드에서 임포트됨) raw Vulkan 인스턴스인 [`vk::Instance`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.Instance.html)와 혼동해서는 안됩니다. 이후 챕터에서는 [`Device`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/struct.Device.html)타입 또한 사용할것입니다. 이 타입은 [`Instance`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/struct.Instance.html)처럼, raw Vulkan 장치([`vk::Device`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.Device.html))와 특정 장치를 위해 로드된 커맨드 짝지어집니다. 운좋게도, 이 튜토리얼에서는 [`vk::Instance`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.Instance.html)나 [`vk::Device`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.Device.html)를 직접적으로 사용하지는 않습니다. 이것들을 혼동할까봐 걱정하지 않아도 됩니다.

하나의 [`Instance`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/struct.Instance.html)는 Vulkan 인스턴스 그리고 연관된 커맨드들을 포함하기 때문에, [`Instance`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/struct.Instance.html)를 위해 구현된 커맨드 wrapper는 Vulkan커맨드에서 필요한 경우에  Vulkan 인스턴스를 제공할 수 있습니다.

[`vkDestroyInstance`](https://www.khronos.org/registry/vulkan/specs/1.4-extensions/man/html/vkDestroyInstance.html)커맨드의 문서를 보면, 두개의 파라미터를 취하는 것을 볼 수 있습니다. 하나는 파괴할 인스턴스이고, 하나는 optional custom allocator callback입니다. 그러나 [`destroy_instance`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/trait.InstanceV1_0.html#method.destroy_instance)문서를 보면, 오직 한개의 optional custom allocator callback를 받는 것을 볼 수 있습니다. 왜냐하면 위에서 설명했듯이 raw Vulkan 핸들을 첫번째 파라미터로 제공해줄 수 있기 때문입니다.

인스턴스 생성 이후 더 복잡한 단계로 가기 전에, validation layer를 체크해서 디버깅 옵션을 평가합니다.

