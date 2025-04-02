# Image views

**Code:** [main.rs](https://github.com/KyleMayes/vulkanalia/tree/master/tutorial/src/07_image_views.rs)

swapchain에 있는 어떤 [`vk::Image`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.Image.html)든 사용하기 위해, render pipeline에서 [`vk::ImageView`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.ImageView.html) 오브젝트를 만들어야 합니다. image view는 거의 문자 그대로 image에 대한 view입니다. image view는 어떻게 image에 접근하는 방법과 이미지의 어느 부분에 접근해야 하는지, 예를들어 만약 이미지가 2D texture로 처리될지 mipmapping level이 없는 depth texture로 처리될지를 설명합니다.

이번 챕터에서는 `create_swapchain_image_views` 함수를 작성할겁니다. 이 함수는 swapchain의 모든 이미지를 위한 기본적인 image view를 생성하므로 나중에 이것을 color target으로 사용할 수 있습니다.

먼저 `AppData`에 image view들을 저장할 필드를 추가합니다.

```rust
struct AppData {
    // ...
    swapchain_image_views: Vec<vk::ImageView>,
}
```

`create_swapchain_image_views` 함수를 생성하고 `App::create`안에서 swapchain 생성 바로 직후 호출합니다.

```rust
impl App {
    unsafe fn create(window: &Window) -> Result<Self> {
        // ...
        create_swapchain(window, &instance, &device, &mut data)?;
        create_swapchain_image_views(&device, &mut data)?;
        // ...
    }
}

unsafe fn create_swapchain_image_views(
    device: &Device,
    data: &mut AppData,
) -> Result<()> {
    Ok(())
}
```

다음으로 해야하는 것은 swapchain image들에 대해 각각 image view를 만들기 위해 순회합니다.

```rust
unsafe fn create_swapchain_image_views(
    device: &Device,
    data: &mut AppData,
) -> Result<()> {
    data.swapchain_image_views = data
        .swapchain_images
        .iter()
        .map(|i| {

        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(())
}
```

생성한 각 image view에 대해 먼저 그 image view를 위한 color component mapping를 정의해야 합니다. 이를 통해 color channel을 휘둘러볼 수 있습니다. 예를 들어, monochrome texture를 위해 모든 채널을 red channel로 매핑할 수 있습니다. 채널에 `0`이나 `1`같은 상수값을 매핑할 수도 있습니다. 우리의 경우에는 default mapping을 고수합니다.

```rust
let components = vk::ComponentMapping::builder()
    .r(vk::ComponentSwizzle::IDENTITY)
    .g(vk::ComponentSwizzle::IDENTITY)
    .b(vk::ComponentSwizzle::IDENTITY)
    .a(vk::ComponentSwizzle::IDENTITY);
```

다음으로 image view에 대한 subresource range를 정의할겁니다. subresource range는 image의 목적과 image의 어느 부분이 접근될 지를 설명합니다. 우리의 이미지는 어떠한 mipmapping level이나 multiple layer없이 color target로 쓰일겁니다.

```rust
let subresource_range = vk::ImageSubresourceRange::builder()
    .aspect_mask(vk::ImageAspectFlags::COLOR)
    .base_mip_level(0)
    .level_count(1)
    .base_array_layer(0)
    .layer_count(1);
```

만약 stereographic 3D 애플리케이션에서 작업중이라면, multiple layer와 함께 swapchain을 만들었을 겁니다. 그러면 각 image에 대해 multiple image view를 만들어서 서로 다른 레이어에 접근하여 왼쪽 눈과 오른쪽 눈에 대한 view를 나타낼 수 있을겁니다.

이제 [`vk::ImageViewCreateInfo`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.ImageViewCreateInfo.html) 구조체를 만듭니다. 이 구조체는 image view 생성에 관한 파라미터를 제공합니다.

```rust
let info = vk::ImageViewCreateInfo::builder()
    .image(*i)
    .view_type(vk::ImageViewType::_2D)
    .format(data.swapchain_format)
    .components(components)
    .subresource_range(subresource_range);
```

`view_type`과 `format` 필드는 이미지가 어떻게 해석될 지 지정합니다. `view_type` 필드는 이미지를 1D textures, 2D textures, 3D texture 그리고 cube map으로 취급할 수 있도록 해줍니다.

image view를 생성하는것은 이제 [`create_image_view`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/trait.DeviceV1_0.html#method.create_image_view)호출의 문제입니다.

```rust
device.create_image_view(&info, None)
```

이미지와 다르게, image view는 우리에 의해 명시적으로 생성됐고, 그래서 그것을 파괴하는 비슷한 루프를 `App::destroy`d에 다시 추가해야합니다.

```rust
unsafe fn destroy(&mut self) {
    self.data.swapchain_image_views
        .iter()
        .for_each(|v| self.device.destroy_image_view(*v, None));
    // ...
}
```

한개의 image view는 image를 texture로 사용하여 시작하기에 충분합니다. 그러나 아직 render target로 사용되기에 준비가 된것은 아닙니다. framebuffer로 알려져있는 한가지 추가적인 간접적인 단계가 필요합니다. 그러나 먼저 graphics pipeline을 설정하는게 먼저입니다.
