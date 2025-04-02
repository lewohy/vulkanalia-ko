# Physical devices and queue families

**Code:** [main.rs](https://github.com/KyleMayes/vulkanalia/tree/master/tutorial/src/03_physical_device_selection.rs)

[`Instance`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/struct.Instance.html)를 통해 Vulkan library를 초기화한 후에, 원하는 기능을 지원하는 시스템에서 그래픽카드를 찾아서 선택해야합니다. 사실은 몇개의 그래픽카드든 선택하고 동시에 사용할수도 있습니다. 그러나 이번 튜토리얼에서는 우리의 요구에 맞는 첫번째 그래픽카드를 사용하는 것을 고수할겁니다.

이 작업과 physical device와 관련 정보를 `AppData` instance에 작성하는 `pick_physical_device`함수를 추가할겁니다. 이 함수와 여기 함수에서 호출하는 함수들은 커스텀 error type(`SuitabilityError`)을 사용해서 physical device가 애플리케이션의 요구사항을 만족하지 안흐면 signal을 보내도록 합니다. 이 error type은 `thiserror` 크레이트를 사용해서 error type에 대한 필수적인 보일러플레이트 코드를 구현합니다.

```rust
use thiserror::Error;

impl App {
    unsafe fn create(window: &Window) -> Result<Self> {
        // ...
        pick_physical_device(&instance, &mut data)?;
        Ok(Self { entry, instance, data })
    }
}

#[derive(Debug, Error)]
#[error("Missing {0}.")]
pub struct SuitabilityError(pub &'static str);

unsafe fn pick_physical_device(instance: &Instance, data: &mut AppData) -> Result<()> {
    Ok(())
}
```

선택된 그래픽카드는 [`vk::PhysicalDevice`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.PhysicalDevice.html)핸들에 저장됩니다. 이 핸들은 `AppData`구조체의 새로운 필드로써 추가됩니다. 이 객체는 [`Instance`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/struct.Instance.html)가 파괴되면 암시적으로 파괴됩니다. 그러므로 `App::destroy`메소드에서는 아무것도 안해도 됩니다.

## Device suitability

어떤 디바이스가 요구사항에 맞는지 판단할 방법이 필요합니다. 요구하는 모든것들을 만족하지 않는 physical device가 제공되면 `SuitabilityError`를 반환하는 함수를 만들어서 시작할겁니다.

```rust
unsafe fn check_physical_device(
    instance: &Instance,
    data: &AppData,
    physical_device: vk::PhysicalDevice,
) -> Result<()> {
    Ok(())
}
```

physical device가 요구사항과 맞는지 평가하기 위해 몇가지 디테일을 쿼리하는것으로 시작할 수 있습니다. name, type, 지원되는 Vulkan version같은 basic device properties는 [`get_physical_device_properties`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/trait.InstanceV1_0.html#method.get_physical_device_properties)를 사용하여 쿼리될 수 있습니다.

```rust
let properties = instance
    .get_physical_device_properties(physical_device);
```

texture compression, 64 bit floats, multi-viewport rendering(VR에 유용)같은 optional feature에 대한 지원은 [`get_physical_device_features`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/trait.InstanceV1_0.html#method.get_physical_device_features)를 사용해서 쿼리될 수 있습니다.

```rust
let features = instance
    .get_physical_device_features(physical_device);
```

나중에 논의할 device memory와 queue families를 고려하는 등의 디바이스로부터 더 많은 디테일들이 있습니다.(다음 섹션에서 봅니다.)

예시로, 애플리케이션이 오직 geometry shader를 지원하는 전용 그래픽카드에서만 이용가능하다고 해봅시다. 그러면 `check_physical_device`함수가 다음처럼 됩니다.

```rust
unsafe fn check_physical_device(
    instance: &Instance,
    data: &AppData,
    physical_device: vk::PhysicalDevice,
) -> Result<()> {
    let properties = instance.get_physical_device_properties(physical_device);
    if properties.device_type != vk::PhysicalDeviceType::DISCRETE_GPU {
        return Err(anyhow!(SuitabilityError("Only discrete GPUs are supported.")));
    }

    let features = instance.get_physical_device_features(physical_device);
    if features.geometry_shader != vk::TRUE {
        return Err(anyhow!(SuitabilityError("Missing geometry shader support.")));
    }

    Ok(())
}
```

장치가 적합한지를 판단하고 첫번째 장치로 이동하는 대신, 각 디바이스에 점수를 부여하고 가장 높은것을 선택할 수 있습니다. 이러한 방법은 전용 그래픽카드에 높은 점수를 부여함으로써 선호할 수 있지만, 만약 integrated GPU만 이용가능하다면, integrated GPU로 fall back됩니다. 단순히 선택지의 이름을 보여주고, 유저가 선택하도록 할 수도 있습니다.

다음으로 실제로 필요한 기능을 논의합니다.

## Queue families

이전에 간단히 언급했듯이, Vulkan의 모든 연산, 즉 그리는것에서 텍스쳐에 업로드하는것까지, queue에 제출될 command들을 요구합니다. 다른 queue families에서 나온 여러 유형의 큐가 있고 큐의 각 family는 command의 subset만 허용합니다. 예를들어, compute command의 처리만 허용하거나 memory transfer와 관련된 command만 허용하는 queue family가 있습니다.

디바이스에 의해 지원되는 queue families가 무엇인지, 이중 어떤것이 우리가 쓰고싶어하는 커맨드를 지원하는지 체크해야합니다. 이를 위해서 새로운 `QueueFamilyIndices`구조체를 만들어서 필요한 queue families의 indices를 저장하도록 합니다.

당장은 graphics command를 지원하는 큐를 찾으러 갑니다. 따라서 구조체와 구현은 다음과 같습니다.

```rust
#[derive(Copy, Clone, Debug)]
struct QueueFamilyIndices {
    graphics: u32,
}

impl QueueFamilyIndices {
    unsafe fn get(
        instance: &Instance,
        data: &AppData,
        physical_device: vk::PhysicalDevice,
    ) -> Result<Self> {
        let properties = instance
            .get_physical_device_queue_family_properties(physical_device);

        let graphics = properties
            .iter()
            .position(|p| p.queue_flags.contains(vk::QueueFlags::GRAPHICS))
            .map(|i| i as u32);

        if let Some(graphics) = graphics {
            Ok(Self { graphics })
        } else {
            Err(anyhow!(SuitabilityError("Missing required queue families.")))
        }
    }
}
```

[`get_physical_device_queue_family_properties`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/trait.InstanceV1_0.html#method.get_physical_device_queue_family_properties)가 반환한 queue properties는 physical device가 지원하는 queue families에 대한 다양한 정보를 가지고 있습니다. 지원되는 연산의 타입, queue family에 기반한 생성가능한 queue의 수를 포함합니다. 여기서 [`vk::QueueFlags::GRAPHICS`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.QueueFlags.html#associatedconstant.GRAPHICS)가 표시한 graphic operation을 지원하는 첫 번째 queue family를 찾습니다.

원하던 queue family를 찾는 메소드를 찾았기 때문에, 우리가 원하는 command들을 디바이스가 처리 가능한지 확인하기 위해 `check_physical_device`에서 사용하여 확인할 수 있습니다.

```rust
unsafe fn check_physical_device(
    instance: &Instance,
    data: &AppData,
    physical_device: vk::PhysicalDevice,
) -> Result<()> {
    QueueFamilyIndices::get(instance, data, physical_device)?;
    Ok(())
}
```

마지막으로 physical device들을 순회하고 `check_physical_device`에 의해 지시된 요구사항을 만족하는 첫 번째 장치를 선택할 수 있습니다.

```rust
unsafe fn pick_physical_device(instance: &Instance, data: &mut AppData) -> Result<()> {
    for physical_device in instance.enumerate_physical_devices()? {
        let properties = instance.get_physical_device_properties(physical_device);

        if let Err(error) = check_physical_device(instance, data, physical_device) {
            warn!("Skipping physical device (`{}`): {}", properties.device_name, error);
        } else {
            info!("Selected physical device (`{}`).", properties.device_name);
            data.physical_device = physical_device;
            return Ok(());
        }
    }

    Err(anyhow!("Failed to find suitable physical device."))
}
```

적절한 physical device를 찾기 위한 모든 일을 했습니다. 다음 스텝은 logical device를 만들어서 physical device와의 인터페이스를 형성합니다.
