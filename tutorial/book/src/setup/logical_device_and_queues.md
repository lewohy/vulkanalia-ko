# Logical device and queues

**Code:** [main.rs](https://github.com/KyleMayes/vulkanalia/tree/master/tutorial/src/04_logical_device.rs)

사용할 physical device를 선택한 후에 장치와 인터페이스를 형성하기 위해 logical device를 설정해야 합니다. logical device생성 과정은 instance생성 과정과 비슷하며 사용하기를 원하는 기능들을 설명합니다. 또한 어떤 queue family가 이용가능한지 쿼리했으므로, 어떤 queue를 생성할지 지정해야 합니다. 다양한 요구사항을 갖고있다면, 한개의 physical device로부터 여러개의 logical device를 생성하는것도 가능합니다.

logical device가 저장될 새로운 `App` field를 추가하면서 시작합니다.

```rust
struct App {
    // ...
    device: Device,
}
```

다음으로, `create_logical_device`함수를 추가합니다. 이 함수는 `App::create`에서 호출되고 생성된 logical device를 `App`의 초기화자에 추가해줍니다.

```rust
impl App {
    unsafe fn create(window: &Window) -> Result<Self> {
        // ...
        let device = create_logical_device(&entry, &instance, &mut data)?;
        Ok(Self { entry, instance, data, device })
    }
}

unsafe fn create_logical_device(
    entry: &Entry,
    instance: &Instance,
    data: &mut AppData,
) -> Result<Device> {
}

```

## Specifying the queues to be created

logical device를 생성하는 것은 struct의 많은 디테일을 또 지정하는 것을 포함합니다. 그 중 첫번째는 [`vk::DeviceQueueCreateInfo`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.DeviceQueueCreateInfo.html)입니다. 이 구조체는 single queue family를 위한 필요한 queue의 수를 설명합니다. 당장은 graphics capabilities를 사용하는 큐만 관심이 있습니다.

```rust
let indices = QueueFamilyIndices::get(instance, data, data.physical_device)?;

let queue_priorities = &[1.0];
let queue_info = vk::DeviceQueueCreateInfo::builder()
    .queue_family_index(indices.graphics)
    .queue_priorities(queue_priorities);
```

현재 이용가능한 드라이버는 각 queue family를 위한 작은 수의 큐만 생성하는 것을 허용하고 실제로 한개보다 많이 필요하지 않을겁니다. 왜냐하면, multiple thread에서 모든 command buffer를 생성할 수 있고 그 버퍼들을 한번에 메인쓰레드로 single-low-overhead call로 보낼수 있기 때문입니다.

Vulkan은 `0.0`에서 `1.0`사이의 floating point number를 사용하여 command buffer execution의 스케쥴링에 영향을 줄 수 있도록 큐에 프로퍼티를 할당하게 해줍니다. 이런 작업은 single queue를 생성할때도 필요합니다.

## Specifying the layers to enable

제공해줘야 할 다음 정보는 [`vk::InstanceCreateInfo`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.InstanceCreateInfo.html)구조체와 닯았습니다. 일단은 다시 활성화하기 원하는 어떤 layer나 extension들을 지정해야합니다. 그러나 이번에는 global이라기보다는 device specific한 extension을 지정합니다.

device specific extension의 예로, [`VK_KHR_swapchain`](https://www.khronos.org/registry/vulkan/specs/1.4-extensions/man/html/VK_KHR_swapchain.html)가 있는데, 이것은 device에서 window로 렌더링된 이미지를 보내게 해줍니다. 이런 기능이 없는 시스템의 Vulkan device가 존재할 수 있습니다. 예를 들어, 오직 compute operation만 지원하는 경우가 그렇습니다. swapchain 챕터에서 다시 그런 extension을 봅니다.

Vulkan의 이전 구현에서는 instance와 device specific validation layer를 구분했습니다. 그러나 더이상 그러진 않습니다. 이것은 `enabled_layer_names`로 넘길 layer의 이름이 이후 최신업데이트에서는 무시될것이라는것을 의미합니다. 그러나 아직 옜날 구현에 호환성을 맞추기 위해 이름을 지정하는것은 좋은 생각입니다.

아직은 어떤 device extension을 활성화하지 않을것이므로, validation이 활성화된 경우 validation layer를 포함하는 layer 이름 리스트를 생성합니다.

```rust
let layers = if VALIDATION_ENABLED {
    vec![VALIDATION_LAYER.as_ptr()]
} else {
    vec![]
};
```

## Specifying the extensions to enable

[`Instance`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/struct.Instance.html)챕터에서 설명했듯이, 특정 Vulkan extension은 Vulkan specification을 완전히 준수하지 않는 Vulkan 구현을 사용하는 애플리케이션을 위해 활성화되어야 합니다. 그 챕터에서는, 비-준수 구현과의 호환성을 위해 instance extension을 활성화했습니다. 여기서는, 같은 목적으로 필요한 device extension을 활성화할겁니다.

```rust
let mut extensions = vec![];

// Required by Vulkan SDK on macOS since 1.3.216.
if cfg!(target_os = "macos") && entry.version()? >= PORTABILITY_MACOS_VERSION {
    extensions.push(vk::KHR_PORTABILITY_SUBSET_EXTENSION.name.as_ptr());
}
```

## Specifying used device features

지정해야할 다음 정보는 사용하게 될 device feature의 집합입니다. 이 기능들은 이전챕터에서 [`get_physical_device_features`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/trait.InstanceV1_0.html#method.get_physical_device_features)를 사용하여 지원하는 기능을 쿼리했습니다(geometry shader같은 기능들). 당장은 특별한게 필요하지 않으므로, 단순히 우리는 정의만 하고 전부 default value(`false`)로 남겨둡니다. Vulkan과 더 재밌는 일을 시작할때 다시 돌아옵니다.

```rust
let features = vk::PhysicalDeviceFeatures::builder();
```

## Creating the logical device

이전의 두 구조체에서, validation layer(활성화 된 경우)과 device extension이 준비되면 [`vk::DeviceCreateInfo`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.DeviceCreateInfo.html)구조체를 채울 수 있습니다.

```rust
let queue_infos = &[queue_info];
let info = vk::DeviceCreateInfo::builder()
    .queue_create_infos(queue_infos)
    .enabled_layer_names(&layers)
    .enabled_extension_names(&extensions)
    .enabled_features(&features);
```

끝입니다. 이제 logical device를 적절히 네이밍된 [`create_device`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/trait.InstanceV1_0.html#method.create_device) 메소드를 통해서 인스턴스화할 수 있습니다.

```rust
let device = instance.create_device(data.physical_device, &info, None)?;
```

파라미터들은 소통하기위한 physical device입니다. 지정했던 queue와 usage info 그리고 optional allocation callback입니다. instance 생성 함수와 유사하게, 이 call은 존재하지 않는 extension을 활성화하거나, 지원되지 않는 feature들을 사용하는 요구사항을 지정하는경우 오류를 반환합니다.

이 device는 `App:destroy`에서 파괴되어야합니다.

```rust
unsafe fn destroy(&mut self) {
    self.device.destroy_device(None);
    // ...
}
```

logical device는 instance와 직접적으로 상호작용하지 않습니다. 왜 파라미터로 제공되지 않았는지에 대한 이유입니다.

## Retrieving queue handles

queue들은 logical device따라 자동으로 생성됩니다. 그러나 아직 그 queue들을 interface하기위한 핸들을 가지고있지 않습니다. 먼저, graphics queue의 핸들을 저장하기 위한 `AppData` field를 추가합니다.

```rust
struct AppData {
    // ...
    graphics_queue: vk::Queue,
}
```

device queue는 device가 파괴될 때 암시적으로 청소됩니다. 따라서 `App::destory`에서 아무것도 안해도 됩니다.

각 queue family를 위한 queue 핸들을 가져오기 위해 [`get_device_queue`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/trait.DeviceV1_0.html#method.get_device_queue) 함수를 사용할 수 있습니다. 파라미터는 logical device, queue family 그리고 queue index입니다. 이 family를 위한 single queue를 생성할거기 때문에, 단순히 index 0을 사용합니다.

```rust
data.graphics_queue = device.get_device_queue(indices.graphics, 0);
```

마지막으로 `create_logical_device`에서 생성된 logical device를 리턴합니다.

```rust
Ok(device)
```

logical device와 queue핸들을 가지고 이제 뭔가를 하기 위해 그래픽카드를 쓸 수 있습니다. 다음 몇 챕터에서 window system에 결과를 보여주기 위해 resource를 설정할겁니다.
