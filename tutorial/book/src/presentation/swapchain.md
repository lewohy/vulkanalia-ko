# Swapchain

**Code:** [main.rs](https://github.com/KyleMayes/vulkanalia/tree/master/tutorial/src/06_swapchain_creation.rs)

Vulkan은 "default framebuffer"의 개념을 갖고 있지 않습니다. 그러므로 소유하고 있는 buffer를 스크린에 시각화하기 전에 그 렌더링할 버퍼를 소유하는 infrastructure을 요구합니다. 이 infrastructure는 *swapchain*으로 알려져있고, Vulkan에서는 명시적으로 생성되어야 합니다. 이 swapchain은 근본적으로 스크린에 표시되기를 기다리는 이미지의 queue입니다. 애플리케이션은 그려내기 위한 이미지를 얻어냅니다. 그리고 그 이미지를 queue에 반환합니다. 정확히 어떻게 그 queue가 작동하는지와 그 queue로부터 이미지를 표시하는 조건들은 swapchain이 어떻게 설정되었는지에 의존합니다. 그러나 swapchain의 일반적인 목적은 스크린의 refresh rate와 이미지의 presentation을 동기화하는 것입니다.

## Checking for swapchain support

다양한 이유로 모든 그래픽카드가 스크린에 직접 이미지를 제공할 수 있는게 아닙니다, 예를들어 server를 위해 디자인되었고 어떤 디스플레이 output을 갖고 있지 않은 경우가 있습니다. 두번째로, 이미지 프레젠테이션은 window system과 window와 연관된 surface에 심하게 묶여있기 때문에, 실제 Vulkan core의 역할이 아닙니다. [`VK_KHR_swapchain`](https://www.khronos.org/registry/vulkan/specs/1.4-extensions/man/html/VK_KHR_swapchain.html) device extension의 support를 query한 후 활성화해야만 합니다. 이전과 같이, [`VK_KHR_swapchain`](https://www.khronos.org/registry/vulkan/specs/1.4-extensions/man/html/VK_KHR_swapchain.html)를 위한 `vulkanlia` extension trait를 import해야합니다.

```rust
use vulkanalia::vk::KhrSwapchainExtension;
```

그리고 첫번째로 `check_physical_device` 함수를 확장해서 이 extension이 지원되는지 체크할겁니다. 어떻게 [`vk::PhysicalDevice`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.PhysicalDevice.html)에 의해 지원되는 확장을 리스팅하는지 이전에 봤고, 꽤 간단하게 합니다.

첫번쨰로 요구되는 device extension의 리스트를 선언합니다. 활성화할 validation layer의 리스트와 비슷합니다.

다음으로, `check_physical_device_extensions` 라는 새로운 함수를 추가합니다. 이 함수는 `check_physical_device`에서 추가적인 체크로써 호출됩니다.

함수의 바디를 수정해서 extension을 열거하고 요구된 extension들이 거기에 있는지 확인합니다.

```rust
unsafe fn check_physical_device_extensions(
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
) -> Result<()> {
    let extensions = instance
        .enumerate_device_extension_properties(physical_device, None)?
        .iter()
        .map(|e| e.extension_name)
        .collect::<HashSet<_>>();
    if DEVICE_EXTENSIONS.iter().all(|e| extensions.contains(e)) {
        Ok(())
    } else {
        Err(anyhow!(SuitabilityError("Missing required device extensions.")))
    }
}
```

이제 코드를 실행하고 그래픽카드가 정말로 swapchain생성을 지원하는지 확인합니다. 이것은 presentation queue의 이용가능성으로 노트될겁니다. 이전 챕터에서 체크했듯이, presentation queue는 swapchain extension이 지원되는지 암시합니다. 그러나 여전히 이것들에 명시적인것이 좋은 생각이고 extension은 명시적으로 활성화되어야 합니다.

## Enabling device extensions

swapchain의 사용은 [`VK_KHR_swapchain`](https://www.khronos.org/registry/vulkan/specs/1.4-extensions/man/html/VK_KHR_swapchain.html) extension를 먼저 활성화하는것을 요구합니다. extension을 활성화하는 것은 `create_logical_device` 함수에서 device extension의 리스트에 작은 변화만 주는 것을 요구합니다. device extensions을 `DEVICE_EXTENSIONS`에서 null-terminated string을 생성하여 초기화합니다.

```rust
let mut extensions = DEVICE_EXTENSIONS
    .iter()
    .map(|n| n.as_ptr())
    .collect::<Vec<_>>();
```

## Querying details of swapchain support

swapchain이 이용가능한지 체크하는것만으로는 충분하지 않습니다. 왜냐하면 swapchain이 우리의 window surface와 실제로 호환이 안될수도 있기 때문입니다. swapchain을 생성하는 것은 instance보다 더 많은 세팅을 포함합니다. 그러므로 더 진행하기 전에 더 많은 디테일을 query해야 합니다.

세 가지 체크해야할 기본적인 프로퍼티 종류가 있습니다.

- Basic surface capabilities(최소/최대 swapchain의 이미지 수, 최소/최대 image의 width와 height)
- Surface formats(pixel format, color space)
- Available presentation modes

`QueueFamilyIndices`와 비슷하게, 프로퍼티들이 query된 후 이 디테일들을 여러군데에 넘기기 위해 구조체를 사용할겁니다. 앞서 진술한 세가지 프로퍼티 타입은 다음 구조체와 구조체의 리스토로 들어옵니다.

```rust
#[derive(Clone, Debug)]
struct SwapchainSupport {
    capabilities: vk::SurfaceCapabilitiesKHR,
    formats: Vec<vk::SurfaceFormatKHR>,
    present_modes: Vec<vk::PresentModeKHR>,
}
```

이제 새로운 `SwapchainSupport::get` 메소드를 만듭니다. 이 메소드는 구조체를 필요한 구조체로 초기화합니다.

이 구조체들의 의미와 그 구조체들이 포함하는 데이터가 정확히 무엇인지에 대한 것은 다음 섹션에서 설명합니다.

모든 디테일은 이제 구조체안에 있습니다. 그러므로 `check_physical_device`를 한번 더 확장해서 그 swapchain의 support가 충분한지 확인하도록 이용합니다. swapchain support는 최소한 한 개의 image format 지원과 우리가 가진 window surface에서 한개의 presentation mode만 지원하면 이 튜토리얼에는 충분합니다.

```rust
unsafe fn check_physical_device(
    instance: &Instance,
    data: &AppData,
    physical_device: vk::PhysicalDevice,
) -> Result<()> {
    // ...

    let support = SwapchainSupport::get(instance, data, physical_device)?;
    if support.formats.is_empty() || support.present_modes.is_empty() {
        return Err(anyhow!(SuitabilityError("Insufficient swapchain support.")));
    }

    Ok(())
}
```

swapchain extension이 이용가능한지 확인한 후에만 swapchain support를 위한 쿼리를 하는 것이 중요합니다.

## Choosing the right settings for the swapchain

추가한 조건이 만족한다면, 그 support는 분명히 충분합니다. 그러나 여전히 다양한 optimality의 많이 다른 모드가 있습니다. 함수 몇개를 작성해서 가장 좋은 swapchain을 위한 적절한 세팅을 찾도록 할겁니다. 결정하기 위한 세 가지 타입이 있습니다.

- Surface format (color depth)
- Presentation mode (conditions for "swapping" images to the screen)
- Swap extent (resolution of images in swapchain)

이러한 모든 세팅에 대해 각각 이용가능하다면 사용할 이상적인 값을 염두해 두고, 그렇지 않으면 몇가지 로직을 만들어서 최선의 것을 찾도록 할겁니다.

## Surface format

이 세팅을 위한 함수는 다음과 같이 시작합니다. 나중에 `SwapchainSupport`의 `formats` field를 argument로써 넘겨줄겁니다.

```rust
fn get_swapchain_surface_format(
    formats: &[vk::SurfaceFormatKHR],
) -> vk::SurfaceFormatKHR {
}
```

각 [`vk::SurfaceFormatKHR`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.SurfaceFormatKHR.html) entry는 `format`과 `color_space` 멤버를 포함합니다. `format` 멤버는 color channel과 type들을 명시합니다. 예를 들어, [`vk::Format::B8G8R8A8_SRGB`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.Format.html#associatedconstant.B8G8R8A8_SRGB) 는 B, G, R과 alpha 채널을 이 순서로, 8 bit unsigned integer로 총 32 bit를 한 픽셀에 저장한다는 것을 의미합니다. `color_space` 멤버는 sRGB color space가 지원되는지 또는 [`vk::ColorSpaceKHR::SRGB_NONLINEAR`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.ColorSpaceKHR.html#associatedconstant.SRGB_NONLINEAR) flag를 사용하지 않는지 가리킵니다.

color space에 대해서는 sRGB가 이용가능하다면 이것을 쓸겁니다. 왜냐하면 sRGB가 [더 정확히 인지된 색을 만들어내기 때문입니다.](http://stackoverflow.com/questions/12524623/) 또한 이것은 나중에 텍스쳐같은것에서 사용할 이미지를 위한 꽤 많이 표준화된 color space입니다. 이 때문에 sRGB color format를 사용할것이고, 그것중 가장 일반적인것은 [`vk::Format::B8G8R8A8_SRGB`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.Format.html#associatedconstant.B8G8R8A8_SRGB) 입니다.

리스트를 따라가서 선호하는 조합이 가능한지 봅니다.

```rust
fn get_swapchain_surface_format(
    formats: &[vk::SurfaceFormatKHR],
) -> vk::SurfaceFormatKHR {
    formats
        .iter()
        .cloned()
        .find(|f| {
            f.format == vk::Format::B8G8R8A8_SRGB
                && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
        })
        .unwrap_or_else(|| formats[0])
}
```

그것도 실패하면, 이용가능한 포맷들에 얼마나 그들이 "좋은지"에 기반하여 랭크를 메길수 있지만, 웬만한 경우에 단지 지정된 첫번째 포맷에 정착해도 됩니다,. 그러므로 `.unwrap_or_else(|| formats[0])`를 씁니다.

## Presentation mode

presentation mode는 틀림없이 swapchain을 위한 가장 중요한 세팅입니다. 왜냐하면 presentation mode는 스크린에 보일 이미지에 대한 실제 조건을 나타내기 때문입니다. Vulkan에는 4가지 가능한 모드가 있습니다.

- [`vk::PresentModeKHR::IMMEDIATE`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.PresentModeKHR.html#associatedconstant.IMMEDIATE) – 애플리케이션에서 전송되어 제공된 이미지가 스크린으로 바로갑니다. tearing이 발생할 수 있습니다.
- [`vk::PresentModeKHR::FIFO`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.PresentModeKHR.html#associatedconstant.FIFO) – swapchain은 queue이고 디스플레이는 refresh될 때 큐의 맨앞에서 이미지를 가져갑니다. 프로그램은 렌더링된 이미지를 queue의 맨 뒤에 삽입합니다. queue가 꽉 찬다면 프로그램은 기다려야합니다. 이 모드는 최근 게임에서 찾을 수 있는 vertical sync와 거의 비슷합니다. 디스플레이가 refresh되는 시점은 "vertical blank"로 알려져 있습니다.
- [`vk::PresentModeKHR::FIFO_RELAXED`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.PresentModeKHR.html#associatedconstant.FIFO_RELAXED) – 이 모드가 FIFO방식과 다른 점은 마지막 vertical blank에서 애플리케이션이 늦고 queue가 비어있을때만 다릅니다. 다음 vertical blank를 기다리는것 대신, 이미지가 마지막으로 도착할 때 바로 전송됩니다. 이 모드는 visible tearing이 발생할 수 있습니다.
- [`vk::PresentModeKHR::MAILBOX`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.PresentModeKHR.html#associatedconstant.MAILBOX) – 두 번째 모드의 또다른 바리에이션입니다. queue가 꽉 찼을 때 애플리케이션을 블로킹하는것 대신, 이미 queue에 들어간 이미지가 새로운것으로 교체됩니다. 이 모드는 tearing를 피하면서 가능한한 프레임을 빠르게 렌더링하는데 사용됩니다. 이 모드는 표준 vertical sync보다 latency 이슈가 더 적습니다. 버퍼가 3개 존재한다는 것이 framrate가 언락됨을 의미하지는 않더라도,,이 모드는 일반적으로 "triple buffering"로 알려져있습니다.

오직 [`vk::PresentModeKHR::FIFO`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.PresentModeKHR.html#associatedconstant.FIFO)모드가 이용가능하다고 보장됩니다. 그러므로 다시, 이용가능한 가장 좋은 모드를 찾기 위한 함수를 작성해야합니다.

```rust
fn get_swapchain_present_mode(
    present_modes: &[vk::PresentModeKHR],
) -> vk::PresentModeKHR {
}
```

개인적으로  에너지 사용이 고려대상이 아니라면, [`vk::PresentModeKHR::MAILBOX`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.PresentModeKHR.html#associatedconstant.MAILBOX)를 가장 좋은 trade-off로 생각합니다. vertical blank직전까지 가능한 up-to-date이미지에 의해 렌더링된 꽤 적은 low latency를 유지하게 해주면서도 tearing을 피하도록 해줍니다. 에너지 사용량이 더 중요한 모바일 디바이스에서는, 아마 [`vk::PresentModeKHR::FIFO`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.PresentModeKHR.html#associatedconstant.FIFO) 를 대신 사용하기를 원할겁니다. 이제, 리스트를 훑어서 [`vk::PresentModeKHR::MAILBOX`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.PresentModeKHR.html#associatedconstant.MAILBOX)가 이용 가능한지 봅니다.

```rust
fn get_swapchain_present_mode(
    present_modes: &[vk::PresentModeKHR],
) -> vk::PresentModeKHR {
    present_modes
        .iter()
        .cloned()
        .find(|m| *m == vk::PresentModeKHR::MAILBOX)
        .unwrap_or(vk::PresentModeKHR::FIFO)
}
```

## Swap extent

하나의 major 프로퍼티만 남았습니다. 이를 위한 마지막 함수를 추가할겁니다.

```rust
fn get_swapchain_extent(
    window: &Window,
    capabilities: vk::SurfaceCapabilitiesKHR,
) -> vk::Extent2D {
}
```

swap extent는 swapchain image의 해상도입니다. 그리고 이것은 거의 항상 그리려고 하는 window의 해상도와 일치합니다. 가능한 해성도의 범위는 [`vk::SurfaceCapabilitiesKHR`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.SurfaceCapabilitiesKHR.html) 구조체에 정의되어 있습니다. Vulkan은 `current_extent` 멤버의 width와 height을 설정함으로써 window의 해상도와 맞추라고 알려줍니다. 그러나 몇 window manager는 다르게하는것을 허용하고 그것은 `current_extent`의 width와 height를 special value(`u32`의 최대값)로 설정함으로서 지시할 수 있습니다. 이번 경우에는 `min_image_extent`와 `max_image_extent` bound를 통해 window와 가장 매치되는 해상도를 선택할겁니다.

```rust
fn get_swapchain_extent(
    window: &Window,
    capabilities: vk::SurfaceCapabilitiesKHR,
) -> vk::Extent2D {
    if capabilities.current_extent.width != u32::MAX {
        capabilities.current_extent
    } else {
        vk::Extent2D::builder()
            .width(window.inner_size().width.clamp(
                capabilities.min_image_extent.width,
                capabilities.max_image_extent.width,
            ))
            .height(window.inner_size().height.clamp(
                capabilities.min_image_extent.height,
                capabilities.max_image_extent.height,
            ))
            .build()
    }
}
```

[`clamp` 함수](https://doc.rust-lang.org/std/cmp/trait.Ord.html#method.clamp)를 사용해서 실제 window크기를 Vulkan device에 의해 지원되는 범위안의 크기로 제한합니다.

## Creating the swapchain

런타임에 결정해야할 선택에 도움을 주는 헬퍼 함수들을 전부 만들었기때문에, 마침내 작동하는 swapchain을 만들기 위해 필요한 정보를 가졌습니다.

그 함수콜들의 결과와 같이 시작하는 `create_swapchain`함수를 만들고, `App::create` logical device생성 이후에 이 함수를 호출하도록 합니다.

```rust
impl App {
    unsafe fn create(window: &Window) -> Result<Self> {
        // ...
        let device = create_logical_device(&instance, &mut data)?;
        create_swapchain(window, &instance, &device, &mut data)?;
        // ...
    }
}

unsafe fn create_swapchain(
    window: &Window,
    instance: &Instance,
    device: &Device,
    data: &mut AppData,
) -> Result<()> {
    let indices = QueueFamilyIndices::get(instance, data, data.physical_device)?;
    let support = SwapchainSupport::get(instance, data, data.physical_device)?;

    let surface_format = get_swapchain_surface_format(&support.formats);
    let present_mode = get_swapchain_present_mode(&support.present_modes);
    let extent = get_swapchain_extent(window, support.capabilities);

    Ok(())
}
```

이 프로퍼티 외에도 swapchain에서 갖고싶어하는 이미지가 얼마나 많아야하는지 결정해야합니다. 구현은 작동하기위한 최소 갯수를 지정합니다.

```rust
let image_count = support.capabilities.min_image_count;
```

그러나, 단순히 이 최소값에 고수하는것은 때때로 렌더링할 다른 이미지를 얻을 수 있게 되기전에 드라이버가 내부 연산을 완료할 때 까지 기다려야함을 의미합니다. 따라서 최소값보다 한개 더 많은 수를 요청하는것이 권장됩니다.

```rust
let image_count = support.capabilities.min_image_count + 1;
```

또한 maximum이 존재하지 않는 `0`인 경우에, 이걸 하는 동안 최대 갯수를 넘지 않도록 해야합니다.

```rust
let mut image_count = support.capabilities.min_image_count + 1;
if support.capabilities.max_image_count != 0
    && image_count > support.capabilities.max_image_count
{
    image_count = support.capabilities.max_image_count;
}
```

다음으로, 여러 queue family에서서 사용될 swapchain image들을 어떻게 핸들링할지 지정해야합니다. 애플리케이션에서 이런 경우는 graphics queue family가 presentation queue와 다를때 입니다. graphics queue에서 온 swapchain의 이미지에 렌더링 작업을 한 후, 그것들을 presentation queue에 제출할겁니다. 여러 queue에서 접근된 이미지를 핸들링하는 두 가지 방법이 있습니다.

- [`vk::SharingMode::EXCLUSIVE`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.SharingMode.html#associatedconstant.EXCLUSIVE) – 이미지는 특정 시간에 한개의 queue family에 의해 소유됩니다. 그리고 다른 queue family에서 사용되기 전에 소유권이 명시적으로 이동되어야 합니다. 이 옵션은 최고의 퍼포먼스를 제공합니다.
- [`vk::SharingMode::CONCURRENT`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.SharingMode.html#associatedconstant.CONCURRENT) – 이미지는 여러 queue family사이에 소유권 이동 없이 사용될 수 있습니다.

만약 queue family들이 다르다면, 이 튜토리얼에서는 소유권 챕터를 피하기 위해 concurrent mode를 사용할겁니다. 왜냐하면 소유권 챕터는 나중에 더 잘 설명될 몇가지 개념을 포함하기 때문입니다. concurrent mode는 먼저 `queue_family_indices` 빌더 메소드를 사용하여 어떤 queue family 사이에서 소유권이 이동될 지 명시해야합니다. 만약 graphics queue family와 presentation queue family가 같다면, 그리고 웬만한 하드웨어가 그런 케이스일거고, exclusive mode를 고수해야합니다. 왜냐하면 concurrent mode는 적어도 두개의 구분된 queue family를 명시하기를 요구하기 때문입니다.

```rust
let mut queue_family_indices = vec![];
let image_sharing_mode = if indices.graphics != indices.present {
    queue_family_indices.push(indices.graphics);
    queue_family_indices.push(indices.present);
    vk::SharingMode::CONCURRENT
} else {
    vk::SharingMode::EXCLUSIVE
};
```

Vulkan object의 전통처럼, swapchain 오브젝트를 만드는 것은 큰 구조체를 채우는것을 요구합니다. 그 과정은 매우 친숙하게 시작합니다.

```rust
let info = vk::SwapchainCreateInfoKHR::builder()
    .surface(data.surface)
    // continued...
```

swapchain이 묶일 surface를 지정한 후에, swapchain 이미지의 디테일이 지정됩니다.

```rust
    .min_image_count(image_count)
    .image_format(surface_format.format)
    .image_color_space(surface_format.color_space)
    .image_extent(extent)
    .image_array_layers(1)
    .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
```

`image_array_layers`는 각 이미지가 구성하는 레이어의 양을 지정합니다. 이 값은 stereoscopic 3D 애플리케이션을 개발하는게 아니라면, 항상 `1`입니다. `image_usage` 비트마스크는 swapchain의 이미지들이 어떤 작업에 사용될 지 지정합니다. 이 튜토리얼에서는 그 이미지들에 직접 렌더링할것이고, 이는 이미지들이 color attachment로 사용될 것을 의미합니다. post-processing같은 작업을 수행하기 위해 이미지를 별도의 이미지로 렌더링하는것도 가능합니다. 그런 경우에 [`vk::ImageUsageFlags::TRANSFER_DST`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.ImageUsageFlags.html#associatedconstant.TRANSFER_DST)같은 값을 사용할겁니다. 그리고 렌더링된 이미지를 swapchain 이미지로 전송하기 위해 memory operation을 사용할겁니다.

```rust
    .image_sharing_mode(image_sharing_mode)
    .queue_family_indices(&queue_family_indices)
```

다음으로 image sharing mode와 swapchain의 이미지를 공유할수 있게 된 queue family들의 index들을 제공해야합니다.

```rust
    .pre_transform(support.capabilities.current_transform)
```

만약 특정 transform이 지원된다면(`capabilities`에서 `supported_transform`) 그 transform이 swapchain에서 이미지에 적용되어야 함을 지정할 수 있습니다. 90도 시계방향 회전 또는 수직 뒤집기 등이 그렇습니다. 이것들을 지정하기 위해 어떠한 transformation도 원해서는 안됩니다. 단순히 현재 transformation을 지정해야 합니다.

```rust
    .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
```

`composite_alpha` 메소드는 alpha 채널이 window system에서 다른 window와 blending을 위해 사용되어야 하는지를 지정합니다. 거의 alpha 채널을 무시하기를 원할겁니다. 그러므로 [`vk::CompositeAlphaFlagsKHR::OPAQUE`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.CompositeAlphaFlagsKHR.html#associatedconstant.OPAQUE)를 사용합니다.

```rust
    .present_mode(present_mode)
    .clipped(true)
```

`present_mode` 멤버는 그 자체를 설명합니다. 만약 `clipped` 멤버가 `true`로 설정되어 있다면, obscure한 픽셀의 색상에 관해서 신경쓰지 않는다는 것을 의미합니다. 예를들어, 다른 윈도우가 그 픽셀 앞에 있는 경우에 그렇습니다. 이러한 픽셀을 다시 읽을수 있고 예측 가능한 결과를 얻고싶어하는게 아니라면, clipping을 활성화하여 최고의 퍼포먼스를 얻을 수 있을 것입니다.

```rust
    .old_swapchain(vk::SwapchainKHR::null());
```

마지막 한개의 메소드 `old_swapchain`이 남았습니다. Vulkan에서 애플리케이션이 실행중일 때 swapchain이 invalid거나 unoptimized되는게 가능합니다. 예를들어 윈도우가 resize된 경우 그렇습니다. 이런 경우에 swapchain은 실제로 처음부터 다시 생성되어야하고, 이전의 swapchain에 대한 참조는 이 메소드에서 지정되어야 합니다. 이 과정은 이후 챕터에서 배울 복잡한 주제입니다. 지금은, 오직 한개의 swapchain만 만든다고 가정합니다. 이 메소드는 기본값이 null handle이므로 생략해도 되지만, 완전성을 위해 남겨줍니다.

[`vk::SwapchainKHR`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.SwapchainKHR.html) 오브젝트를 저장할 `AppData` 필드를 추가합니다.

```rust
struct AppData {
    // ...
    swapchain: vk::SwapchainKHR,
}
```

이제 swapchain을 생성하는 것은 [`create_swapchain_khr`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/trait.KhrSwapchainExtension.html#method.create_swapchain_khr)를 호출하는것만큼 간단합니다.

```rust
data.swapchain = device.create_swapchain_khr(&info, None)?;
```

파라미터는 swapchain 생성 정보와 optional custom allocator입니다. 새로운거는 없습니다. swapchain도 `App::destroy`에서 device전에 청소되어야 합니다.

```rust
unsafe fn destroy(&mut self) {
    self.device.destroy_swapchain_khr(self.data.swapchain, None);
    // ...
}
```

이제 애플리케이션을 실행하고 swapchain이 성공적으로 생성되는지 확인합니다. 만약 이 시점에 [`vkCreateSwapchainKHR`](https://www.khronos.org/registry/vulkan/specs/1.4-extensions/man/html/vkCreateSwapchainKHR.html)안에서 access violation error를 얻는다거나 `Failed to find 'vkGetInstanceProcAddress' in layer SteamOverlayVulkanLayer.dll` 같은 메세지를 본다면, [FAQ entry](https://kylemayes.github.io/vulkanalia/faq.html)의 Steam overlay layer에 관해 보세요

[`vk::SwapchainCreateInfoKHR`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.SwapchainCreateInfoKHR.html)구조체를 를 만드는 곳에서 validation layer가 활성화 된 채로 `.image_extent(extent)`라인을 지워보세요. validation layer가 즉시 실수를 캐치하고 `image_extent`로 제공된 잘못된 값을 호출한다는 도움되는 몇가지 메세지를 출력하는것을 볼겁니다.

![log](https://kylemayes.github.io/vulkanalia/images/swapchain_validation_layer.png)

## Retrieving the swapchain images

이제 swapchain이 생성되었습니다. 남아있는 것은 swapchain안의 [`vk::Image`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.Image.html)들의 핸들을 가져오는 겁니다. 이후 챕터에서 렌더링 연산동안 이 이미지들을 참조할겁니다. `AppData`에 핸들을 저장할 필드를 추가합니다.

```rust
struct AppData {
    // ...
    swapchain_images: Vec<vk::Image>,
}
```

swapchain을 위한 구현에 의해 이미지가 생성됩니다. 그리고 이 이미지들은 swapchain이 파괴될 때 자동으로 청소됩니다. 그러므로 어떠한 cleanup코드를 추가할 필요는 없습니다.

저는 `create_swapchain`함수의 끝, [`create_swapchain_khr`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/trait.KhrSwapchainExtension.html#method.create_swapchain_khr) call 바로 다음에 핸들을 가져오기 위한 코드를 추가했습니다.

```rust
data.swapchain_images = device.get_swapchain_images_khr(data.swapchain)?;
```

마지막으로 한가지, swapchain image를 위해 선택한 format과 extent를 `AppData` 필드에 저장하세요. 이후 챕터에서 필요할겁니다.

```rust
struct AppData {
    // ...
    swapchain_format: vk::Format,
    swapchain_extent: vk::Extent2D,
    swapchain: vk::SwapchainKHR,
    swapchain_images: Vec<vk::Image>,
}
```

그리고 `create_swapchain`에서는

```rust
data.swapchain_format = surface_format.format;
data.swapchain_extent = extent;
```

window로 그려지고 표시될 수 있는 이미지의 세트를 갖게 되었습니다. 다음 챕터는 어떻게 이미지를 render target으로 설정하는지 설명합니다. 그리고 실제 graphic pipeline과 drawing command를 살펴볼겁니다.
