# Window surface

**Code:** [main.rs](https://github.com/KyleMayes/vulkanalia/tree/master/tutorial/src/05_window_surface.rs)

Vulkan은 platform agnostic API이기 때문에, 스스로 window system과 직접적으로 interface할 수 없습니다. Vulkan과 화면에 결과를 보여줄 window system사이의 연결을 생성하기 위해, WSI(Window System Integration) extension을 사용해야합니다. 이번 챕터에서 그 첫번째인 [`VK_KHR_surface`](https://www.khronos.org/registry/vulkan/specs/1.4-extensions/man/html/VK_KHR_surface.html)에 대해 논의할겁니다. 이 extension은 [`vk::SurfaceKHR`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.SurfaceKHR.html) 오브젝트를 노출시킵니다. 이 오브젝트는 렌더링된 이미지를 surface로 표시하기 위해 surface의 추상 타입을 나타냅니다. 프로그램의 surface는 이미 열어뒀던 `winit`에 의한 window를 기반으로 동작합니다.

[`VK_KHR_surface`](https://www.khronos.org/registry/vulkan/specs/1.4-extensions/man/html/VK_KHR_surface.html) extension는 instance level의 extension입니다. 그리고 [`vk_window::get_required_instance_extensions`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/window/fn.get_required_instance_extensions.html)에 의해 반환된 리스트에 포함되어있기 때문에, 실제로 이미 활성화했습니다. 그 리스트는 몇몇 다른 WSI extension을 포함하고 다음 몇 챕터 후에 사용할겁니다.

window surface는 physical device 선택에 실제로 영향을 줄 수 있기 때문에, instance의 생성 바로 직후에 생성되어야합니다. 이 과정을 미뤘던 이유는 window surface가 render target과 presentation의 큰 주제의 일부인데, 그것들에 대한 설명은 basic setup을 어렵게 했을겁니다. 만약 off-screen rendering이 필요하다면, Vulkan에서 window surface는 전반적으로 optional인 component이라는것에도 유의합니다. Vulkan은 보이지않는 window를 생성하는것 같은 hack(OpenGL에서는 필수적)없이 그런것을 가능하게 해줍니다.

구조체 [`vk::SurfaceKHR`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.SurfaceKHR.html)같은 extension을 위한 import는 자유롭게 하는 반면, extension에 의해 추가된 Vulkan command를 호출하기 전에 [`VK_KHR_surface`](https://www.khronos.org/registry/vulkan/specs/1.4-extensions/man/html/VK_KHR_surface.html)를 위한 `vulkanalia` extension을 import해야 합니다. [`vk::KhrSurfaceExtension`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/trait.KhrSurfaceExtension.html) import를 추가합니다.

```rust
use vulkanalia::vk::KhrSurfaceExtension;
```

## Window surface creation

`AppData`의 다른 field위에 `surface`필드를 추가함으로써 시작합니다.

```rust
struct AppData {
    surface: vk::SurfaceKHR,
    // ...
}
```

[`vk::SurfaceKHR`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.SurfaceKHR.html) 오브젝트와 이 오브젝트의 사용이 플랫폼 불가지론적일지라도, 이 오브젝트의 생성은 window system detail에 의존하기때문에 불가지론적이지 않습니다. 예를들어, Windows에서 이 오브젝트의 생성은 `HWND`와 `HMODULE`를 필요로합니다. 그러므로 extension에 platform-specific addition이 있는데, Windows에서는 [`VK_KHR_win32_surface`](https://www.khronos.org/registry/vulkan/specs/1.4-extensions/man/html/VK_KHR_win32_surface.html)라고 불립니다. 그리고 이것은 [`vk_window::get_required_instance_extensions`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/window/fn.get_required_instance_extensions.html)의 리스트에 자동으로 포함되어 있습니다.

어떻게 platform specific extension이 Windows에서 surface를 생성하기위해 사용되는지 보여드릴겁니다. 그러나 이 튜토리얼에서 실제로 사용하지는 않을겁니다. `vulkanalia`는 [`vk_window::create_surface`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/window/fn.create_surface.html)를 갖고있는데, 이것은 우리를 위한 플랫폼과의 차이를 핸들링합니다. 여전히, 이것에 의존하기 전에, 이것이 scene의 뒤에서 뭘하는지 보는것이 좋습니다.

window surface는 Vulkan 오브젝트이므로, [`vk::Win32SurfaceCreateInfoKHR`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.Win32SurfaceCreateInfoKHR.html) 구조체가 딸려옵니다. 이 구조체도 채워줘야합니다. 두 가지 중요한 파라미터를 갖습니다: `hisntance`과 `hwnd`입니다. 이것들은 process와 window를 핸들링합니다.

```rust
use winit::platform::windows::WindowExtWindows;

let info = vk::Win32SurfaceCreateInfoKHR::builder()
    .hinstance(window.hinstance())
    .hwnd(window.hwnd());
```

`WindowExtWindows` 트레잇은 `winit` `Window`구조체에서 platform-specific 메소드에 접근하게 해주기 때문에 이 트레잇은 `winit`에서 import됩니다. 이번 케이스에서, 이 트레잇은 `winit`에 의해 생성된 window를 위한 process와 window핸들을 얻게 해 줍니다.

이 작업 후에, surface생성 디테일과 custom allocator에 대한 파라미터를 포함하는 [`create_win32_surface_khr`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/trait.KhrWin32SurfaceExtension.html#method.create_win32_surface_khr)로 생성된 surface를 생성할 수 있습니다. 기술적으로, 이것은 WSI extension 함수이지만, 이 함수는 너무 공통적으로 사용돼서 표준 Vulkan loader가 이 함수를 포함합니다. 그래서 다른 extension과 다르게 명시적으로 로드할 필요가 없습니다. 그러나 [`VK_KHR_win32_surface`](https://www.khronos.org/registry/vulkan/specs/1.4-extensions/man/html/VK_KHR_win32_surface.html) ([`vk::KhrWin32SurfaceExtension`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/trait.KhrWin32SurfaceExtension.html))를 위한 `vulkanalia` extension 트레잇을 import해야 합니다,

```rust
use vk::KhrWin32SurfaceExtension;

let surface = instance.create_win32_surface_khr(&info, None).unwrap();
```

이 과정은 Linux같은 [`create_xcb_surface_khr`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/trait.KhrXcbSurfaceExtension.html#method.create_xcb_surface_khr)가 XCB connection과 window를 X11에서 creation detail로 취급하는 다른 플랫폼에서도 비슷합니다.

[`vk_window::create_surface`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/window/fn.create_surface.html) 함수는 각 플랫폼을 위한 다른 구현을 통해 정확한 operation을 수행합니다. 이제 이것을 우리의 프로그램과 통합할겁니다. `App::create`에서 physical device를 선택하기 바로 직전에 함수 호출을 추가합니다.

```rust
unsafe fn create(window: &Window) -> Result<Self> {
    // ...
    let instance = create_instance(window, &entry, &mut data)?;
    data.surface = vk_window::create_surface(&instance, &window, &window)?;
    pick_physical_device(&instance, &mut data)?;
    // ...
}
```

파라미터는 Vulkan instance와 `winit` window입니다. 일단 surface를 갖게되면, Vulkan API를 통해 `App::destroy`에서 파괴되어야 합니다.

```rust
unsafe fn destroy(&mut self) {
    // ...
    self.instance.destroy_surface_khr(self.data.surface, None);
    self.instance.destroy_instance(None);
}
```

instance전에 surface를 파괴하는것을 확인하세요.

## Querying for presentation support

Vulkan 구현이 window system integration을 지원하더라도, 이것이 시스템에서 모든 디바이스를 지원하는것을 의미하지 않습니다. 그러므로 우리의 physical device 선택 코드를 선택된 디바이스가 만들었던 surface에 이미지를 보여줄수 있는지 확신할 수 있도록 확장해야 합니다. presentation은 queue-specific feature이기 때문에, 문제는 실제로 생성한 surface에 표시하는것을 지원하는 queue family를 찾아내는 것입니다.

drawing command를 지원하는 queue family와 presentation을 지원하는 queue family가 겹치지 않을 수도 있습니다. 그러므로 `QueueFamilyIndices`구조체를 수정하므로써 구분된 presentation queue가 있을수도 있음을 고려해야 합니다.

```rust
struct QueueFamilyIndices {
    graphics: u32,
    present: u32,
}
```

다음으로 `QueueFamilyIndices::get` 메소드를 수정해서 우리의 window surface로 표시하는게 가능한 queue family를 찾도록 합니다. 이것을 위한 함수는 [`get_physical_device_surface_support_khr`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/trait.KhrSurfaceExtension.html#method.get_physical_device_surface_support_khr)입니다. 이 함수는 파라미터로 physical device, queue family index 그리고 surface를 취하고 그런 physical device, queue family 그리고 surface의 조합이 presentation을 지원하는지 여부를 반환합니다.

`QueueFamilyIndices::get`을 수정해서 graphics queue family index가 찾아진 경우 밑에서 presentation queue family index를 찾도록 합니다.

```rust
let mut present = None;
for (index, properties) in properties.iter().enumerate() {
    if instance.get_physical_device_surface_support_khr(
        physical_device,
        index as u32,
        data.surface,
    )? {
        present = Some(index as u32);
        break;
    }
}
```

또한 present를 마지막 표현식에 추가합니다.

```rust
if let (Some(graphics), Some(present)) = (graphics, present) {
    Ok(Self { graphics, present })
} else {
    Err(anyhow!(SuitabilityError("Missing required queue families.")))
}
```

결국에는 이것들이 같은 queue family가 될 가능성이 높음을 숙지합니다. 그러나 프로그램동안 일관적인 접근법을 위해 그것들을 별도의 queue처럼 취급할겁니다. 그럼에도 불구하고, 향상된 퍼포먼스를 위해 같은 queue에서 drawing과 presentation을 지원하는 physical device를 명시적으로 지정하는 로직을 추가할 수도 있습니다.

## Creating the presentation queue

남은것은 logical device 생성 절차를 수정해서 presentation queue를 생성하고 [`vk::Queue`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.Queue.html) handle을 가져오는 것입니다. `AppData`에 그 queue 핸들을 위한 field를 추가합니다.

```rust
struct AppData {
    // ...
    present_queue: vk::Queue,
}
```

다음으로, 두 family로부터 queue를 생성하기 위해 여러개의 [`vk::DeviceQueueCreateInfo`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.DeviceQueueCreateInfo.html) 구조체가 필요합니다. 이걸 하는 쉬운 방법은 요구되는 queue를 위한 필수적인 모든 unique queue의 세트를 생성하는 겁니다. 이것을 `create_logical_device` 함수 안에서 할겁니다.

```rust
let indices = QueueFamilyIndices::get(instance, data, data.physical_device)?;

let mut unique_indices = HashSet::new();
unique_indices.insert(indices.graphics);
unique_indices.insert(indices.present);

let queue_priorities = &[1.0];
let queue_infos = unique_indices
    .iter()
    .map(|i| {
        vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(*i)
            .queue_priorities(queue_priorities)
    })
    .collect::<Vec<_>>();
```

그리고 이전의 `queue_infos` slice를 지우고 [`vk::DeviceCreateInfo`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.DeviceCreateInfo.html)를 위해 `queue_infos`리스트에 대한 참조를 가져옵니다.

```rust
let info = vk::DeviceCreateInfo::builder()        
    .queue_create_infos(&queue_infos)
    .enabled_layer_names(&layers)
    .enabled_extension_names(&extensions)
    .enabled_features(&features);
```

만약 queue family들이 같다면, index를 한번만 넘겨주면 됩니다. 마지막으로 queue handle을 가져오기위한 call을 추가합니다.

```rust
data.present_queue = device.get_device_queue(indices.present, 0);
```

queue family들이 같은 경우에, 지금은 두 handle이 같은 값을 가질겁니다. 다음 챕터에서는 swapchain을 찾고 어떻게 그게 surface로 image를 표시하는 기능을 주는지 알아볼겁니다.
