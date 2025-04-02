# Framebuffers

**Code:** [main.rs](https://github.com/KyleMayes/vulkanalia/tree/master/tutorial/src/13_framebuffers.rs)

이전 몇 챕터에서 framebuffer에 대해 많은 것을 이야기했고 render pass를 설정해서 single framebuffer가 swapchain image와 같은 포맷을 갖는 것을 기대했지만, 실제로 아무것도 생성하지 않았습니다.

render pass creation동안에 지정된 attachments들은 [`vk::Framebuffer`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.Framebuffer.html) 객체로 래핑되어 바인딩됩니다. framebuffer 객체는 attachment들을 나타내는 모든 [`vk::ImageView`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.ImageView.html) 객체를 참조합니다. 한개의 single attachment만 쓸 우리의 경우에는 color attachment입니다. 그러나, attachment로 사용할 이미지는 presentation을 위해 이미지를 찾아올 때 swapchain이 반환하는 이미지에 따라 달라집니다. 이것은 swapchain의 모든 이미지에 대해 framebuffer를 생성하고 drawing time에 찾아온 이미지에 대응하는 framebuffer를 써야한다는 것을 의미합니다.

목적을 달성하기 위해서, `AppData`에 framebuffer들을 저장하기 위해서 또다른 `Vec`필드를 만듭니다.

```rust
struct AppData {
    // ...
    framebuffers: Vec<vk::Framebuffer>,
}
```

`create_framebuffers`라는 새로운 함수에서 이 배열을 위한 오브젝트를 만들겁니다. 이 함수는 `App::create`에서 graphics pipeline을 만든 직후 바로 호출됩니다.

```rust
impl App {
    unsafe fn create(window: &Window) -> Result<Self> {
        // ...
        create_pipeline(&device, &mut data)?;
        create_framebuffers(&device, &mut data)?;
        // ...
    }
}

unsafe fn create_framebuffers(device: &Device, data: &mut AppData) -> Result<()> {
    Ok(())
}
```

swapchain image view들을 매핑하면서 시작합니다.

```rust
unsafe fn create_framebuffers(device: &Device, data: &mut AppData) -> Result<()> {
    data.framebuffers = data
        .swapchain_image_views
        .iter()
        .map(|i| {
        
        })
        .collect::<Result<Vec<_>, _>>()?;
        
    Ok(())
}
```

그러면 각 image view에 대한 framebuffer를 만들겁니다.

```rust
let attachments = &[*i];
let create_info = vk::FramebufferCreateInfo::builder()
    .render_pass(data.render_pass)
    .attachments(attachments)
    .width(data.swapchain_extent.width)
    .height(data.swapchain_extent.height)
    .layers(1);

device.create_framebuffer(&create_info, None)
```

보이듯이, framebuffer를 만드는것은 꽤나 직관적입니다. 먼저 framebuffer가 어떤 `render_pass`화 호환될지 지정해야합니다. framebuffer는 호환되는 render pass들이랑만 사용할 수 있습니다. 그리고 그것은 대략 동일한 수와 타입의 attachment를 사용해야한다는 것을 의미합니다.

`attachments` 필드는 render pass `attachment` 배열에서 각 attachment에 바인딩되어야할 [`vk::ImageView`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.ImageView.html) 오브젝트를 지정합니다.

`width`와 `height` 파라미터는 자명합니다. `layer`는 image 배열들의 layer의 수를 가리킵니다. 우리의 swapchain 이미지는 single image이므로, layer의 수는 `1`입니다.

framebuffer가 의존하고있는 image view와 render pass전에 framebuffer를 삭제해야하지만, 렌더링을 끝낸 후에만 해야합니다.

```rust
unsafe fn destroy(&mut self) {
    self.data.framebuffers
        .iter()
        .for_each(|f| self.device.destroy_framebuffer(*f, None));
    // ...
}
```

렌더링을 위해 필요한 오브젝트를 가져아하는 milestone에 도달했습니다. 다음챕터에서는 첫번째 실제 drawing command를 작성할겁니다.
