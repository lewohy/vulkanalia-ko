# Render passes

**Code:** [main.rs](https://github.com/KyleMayes/vulkanalia/tree/master/tutorial/src/11_render_passes.rs)

pipeline의 생성을 끝내기 전에, rendering동안 사용될 framebuffer attachment를 Vulkan에 알려줘야합니다. color와 depth buffer의 수, 각 buffer들을 위해 사용할 sample의 수 그리고 rendering 연산동안 그것들의 content가 다루어질 방법을 지정해야합니다. 이 모든 정보는 *render pass* object에 래핑됩니다. 이 객체를 위해 `create_render_pass` 함수를 만들겁니다. 그리고 `App::create`에서 `create_pipeline`이전에 이 함수를 호출합니다.

```rust
impl App {
    unsafe fn create(window: &Window) -> Result<Self> {
        // ...
        create_render_pass(&instance, &device, &mut data)?;
        create_pipeline(&device, &mut data)?;
        // ...
    }
}

unsafe fn create_render_pass(
    instance: &Instance,
    device: &Device,
    data: &mut AppData,
) -> Result<()> {
    Ok(())
}
```

## Attachment description

우리의 경우, 단순히 swapchain으로부터 한 개의 이미지마다 표현된 single color buffer attachment를 가질겁니다. 이것은 [`vk::AttachmentDescription`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.AttachmentDescription.html)로 표현되며 `create_render_pass`에서 만들어질겁니다.

```rust
let color_attachment = vk::AttachmentDescription::builder()
    .format(data.swapchain_format)
    .samples(vk::SampleCountFlags::_1)
    // continued...
```

color attachment의 `format`은 swapchain image의 format과 일치해야합니다. 그리고 아직은 multisampling으로 아무것도 하지 않을겁니다. 그러므로 1 sample을 사용합니다.

```rust
    .load_op(vk::AttachmentLoadOp::CLEAR)
    .store_op(vk::AttachmentStoreOp::STORE)
```

`load_op`와 `store_op`는 rendering전, 후에 attachment의 데이터로 무엇을 할지 결정합니다. `load_op`에 대해 다음과 같은 선택지가 있습니다.

- [`vk::AttachmentLoadOp::LOAD`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.AttachmentLoadOp.html#associatedconstant.LOAD) – attachment의 기존 content를 유지
- [`vk::AttachmentLoadOp::CLEAR`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.AttachmentLoadOp.html#associatedconstant.CLEAR) – 시작할 때 constant로 초기화
- [`vk::AttachmentLoadOp::DONT_CARE`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.AttachmentLoadOp.html#associatedconstant.DONT_CARE) – 기존 content는 undefined이고 신경쓰지 않음

우리의 경우 clear operation을 사용해서 새 frame을 그리기 전에 framebuffer를 black으로 초기화합니다. `store_op`에 대해서는 두 가지 선택지만 있습니다.

- [`vk::AttachmentStoreOp::STORE`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.AttachmentStoreOp.html#associatedconstant.STORE) – 렌더된 content가 메모리에 저장되고 이후에 읽어질 수 있습니다.
- [`vk::AttachmentStoreOp::DONT_CARE`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.AttachmentStoreOp.html#associatedconstant.DONT_CARE) – 렌더링 연산 후에 framebuffer의 content가 undefined됩니다.

화면에 렌더링된 삼각형을 보는것에 관심이 있으므로 store operation으로 진행합니다.

```rust
    .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
    .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
```

`load_op`와 `store_op`는 color 그리고 depth data에 적용됩니다. 그리고 `stencil_load_op`/`stencil_store_op`는 stencil data에 적용됩니다. stencil buffer와 아무것도 하지 않기를 원하므로, loading과 storing의 결과는 관련이 없습니다.

```rust
    .initial_layout(vk::ImageLayout::UNDEFINED)
    .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);
```

Vulkan에서 texture와 framebuffer는 특정 픽셀 포멧과 [`vk::Image`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.Image.html)객체로 표현되지만, 메모리에서 픽셀의 layout은 이미지와 무엇을 하려고 하는지에 따라 바뀝니다.

가장 일반적인 몇 layout은 다음과 같습니다.

- [`vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.ImageLayout.html#associatedconstant.COLOR_ATTACHMENT_OPTIMAL) – color attachment로 사용되는 이미지
- [`vk::ImageLayout::PRESENT_SRC_KHR`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.ImageLayout.html#associatedconstant.PRESENT_SRC_KHR) – swapchain에서 present될 이미지
- [`vk::ImageLayout::TRANSFER_DST_OPTIMAL`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.ImageLayout.html#associatedconstant.TRANSFER_DST_OPTIMAL) – 메모리 copy operation의 destination으로 사용될 이미지

texturing챕터에서 이 주제에 대해 더 깊게 토론하겠지만, 당장 알아야할 중요한 것은 이미지가 이후 수행될 연산에 적합한 특정 layout으로 전환되어야 한다는 것입니다.

`initial_layout`은 render pass가 시작되기 전에 이미지가 갖게 될 layout을 지정합니다. `final_layout`는 render pass가 끝날 때 자동으로 전환될 레이아웃을 지정합니다. `initial_layout`을 위해 [`vk::ImageLayout::UNDEFINED`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.ImageLayout.html#associatedconstant.UNDEFINED)를 사용하는 것은 이미지가 있던 이전의 layout을 신경쓰지 않는다는 것을 의미합니다. 이 special value의 경고는 이미지의 content의 보존이 보증되지 않는것이지만, 어차피 지울거라서 문제가 되지는 않습니다. rendering후에 swapchain을 사용하여 image가 presentation을 위해 준비되길 원합니다. 이것이 왜 [`vk::ImageLayout::PRESENT_SRC_KHR`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.ImageLayout.html#associatedconstant.PRESENT_SRC_KHR)를 `final_layout`로 쓰는지에 대한 이유입니다.

## Subpasses and attachment references

single render pass는 multiple subpasses를 구성할 수 있습니다. subpasses는 이전 passes의 framebuffer의 content에 기반한 후속 rendering operation입니다. 예를 들어, 잇따라서 적용되는 post-processing effect의 시퀀스입니다. 이러한 rendering operation을 하나의 render pass로 그룹화한다면, Vulkan은 operation을 재정렬하고 가능한 더 나은 퍼포먼스를 위해 memory bandwidth를 보존합니다. 그러나 우리의 첫 삼각형을 위해서, single subpass를 고수합니다.

모든 subpass는 이전 섹션에서 구조체를 이용하여 설명한 한 개 이상의 attachment를 참조합니다. 이러한 reference들은 스스로 다음과같은 [`vk::AttachmentReference`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.AttachmentReference.html) 구조체가 됩니다.

```rust
let color_attachment_ref = vk::AttachmentReference::builder()
    .attachment(0)
    .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
```

`attachment` 파라미터는 attachment description array에서 index를 사용하여 어떤 attachment를 참조할지 지정합니다. 우리의 array는 single [`vk::AttachmentDescription`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.AttachmentDescription.html)로 구성되어있으므로, `0`으로 설정합니다. `layout`은 이 참조를 사용하는 subpass동안에 attachment가 가지길 원하는 layout을 지정합니다. subpass가 시작될 때 Vulkan은 자동으로 attachment를 이 layout으로 전환합니다. attachment가 color buffer로써 작동하기를 의도했고, [`vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.ImageLayout.html#associatedconstant.COLOR_ATTACHMENT_OPTIMAL) layout은 이름이 암시하듯, 최적의 퍼포먼스를 줍니다.

subpass는 [`vk::SubpassDescription`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.SubpassDescription.html) 구조체를 사용하여 설명됩니다.

```rust
let color_attachments = &[color_attachment_ref];
let subpass = vk::SubpassDescription::builder()
    .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
    .color_attachments(color_attachments);
```

Vulkan은 미래에 compute subpasses를 지원할수도 있습니다. 그러므로 이것이 graphics subpass라는 것을 명시해야합니다. 그리고 color attachment에 대한 참조를 지정합니다.

배열에서 attachment의 index는 fragment shader에서 `layout(location = 0) out vec4` 디렉티브를 통해 직접적으로 참조됩니다!

다음의 attachment의 타입들은 subpass에 의해 참조될 수 있습니다.

- `input_attachments` – shader로부터 읽은 attachments
- `resolve_attachments` – color attachment를 multisampling하기 위해 사용되는 attachments
- `depth_stencil_attachment` – depth과 stencil data를 위한 attachments
- `preserve_attachments` – subpass에 의해 사용되지 않지만 데이터 보존을 위한 attachments

## Render pass

attachment와 그걸 참조하는 basic subpass가 설명되었으므로, render pass를 생성할 수 있습니다. [`vk::RenderPass`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.RenderPass.html) 객체를 저장하기 위한 새로운 class member variable을 `AppData`안의 `pipeline_layout`필드 바로 위에 추가합니다.

```rust
struct AppData {
    // ...
    render_pass: vk::RenderPass,
    pipeline_layout: vk::PipelineLayout,
}
```

그리고 render pass 객체는 [`vk::RenderPassCreateInfo`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.RenderPassCreateInfo.html) 구조체를 attachment와 subpass의 배열로 채우는것으로 생성할 수 있습니다. [`vk::AttachmentReference`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.AttachmentReference.html) 객체는 배열의 index들을 사용하여 attachment들을 참조합니다.

```rust
let attachments = &[color_attachment];
let subpasses = &[subpass];
let info = vk::RenderPassCreateInfo::builder()
    .attachments(attachments)
    .subpasses(subpasses);

data.render_pass = device.create_render_pass(&info, None)?;
```

pipeline layout과 마찬가지로, render pass는 프로그램동안에 참조됩니다. 그러므로 `App::destroy`에서 마지막에 정리되어야합니다.

```rust
unsafe fn destroy(&mut self) {
    self.device.destroy_pipeline_layout(self.data.pipeline_layout, None);
    self.device.destroy_render_pass(self.data.render_pass, None);
    // ...
}
```

많은 양의 작업이었지만, 다음 챕터에서는 다같이 모여서 마침내 graphics pipeline 객체를 만들겁니다.
