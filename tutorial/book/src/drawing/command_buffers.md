# Command buffers

**Code:** [main.rs](https://github.com/KyleMayes/vulkanalia/tree/master/tutorial/src/14_command_buffers.rs)

Vulkan에서 drawing operation과 memory transfer같은 command는 function call을 통해 직접 실행되는게 아닙니다. command buffer 오브젝트에서 실행되기를 원하는 모든 operation들을 기록해야합니다. 이것의 이점은 drawing command들을 세팅하는 등의 고된 작업을 미리 그리고 여러 쓰레드에서 완료될 수 있다는 것입니다. 그 후에 단지 Vulkan에 main loop에서 그 command들을 실행하라고 알려주기만 하면 됩니다.

## Command pools

command buffer들을 만들기 전에 command pool을 먼저 만들어야 합니다. command pool들은 buffer들을 저장하기 위해 사용되는 메모리를 관리하고 command buffer들은 command pool들로부터 할당됩니다. `AppData` 필드를 추가해서 [`vk::CommandPool`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.CommandPool.html)를 저장하도록 합니다.

```rust
struct AppData {
    // ...
    command_pool: vk::CommandPool,
}
```

그리고 새로운 `create_command_pool` 함수를 생성하고 `App::create`에서 framebuffers가 생성된 후에 호출합니다.

```rust
impl App {
    unsafe fn create(window: &Window) -> Result<Self> {
        // ...
        create_framebuffers(&device, &mut data)?;
        create_command_pool(&instance, &device, &mut data)?;
        // ...
    }
}

unsafe fn create_command_pool(
    instance: &Instance,
    device: &Device,
    data: &mut AppData,
) -> Result<()> {
    Ok(())
}
```

command pool 생성은 오직 두개의 파라미터만 받습니다.

```rust
let indices = QueueFamilyIndices::get(instance, data, data.physical_device)?;

let info = vk::CommandPoolCreateInfo::builder()
    .flags(vk::CommandPoolCreateFlags::empty()) // Optional.
    .queue_family_index(indices.graphics);
```

우리가 가져왔던 graphics 그리고 presentation queue처럼 command buffer들은 device queue들중 하나에 제출함으로써 실행됩니다. 각 command pool은 오직 queue의 single type에 제출될 command buffers만 할당할 수 있습니다. drawing을 위한 commands를 기록할겁니다. 이것이 왜 graphics queue family를 선택한 이유입니다.

command pools을 위한 3가지 가능한 flags들이 있습니다.

- [`vk::CommandPoolCreateFlags::TRANSIENT`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.CommandPoolCreateFlags.html#associatedconstant.TRANSIENT) – 매우 자주 새로운 command들이 command buffers에 재기록되는것을 암시합니다.
- [`vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.CommandPoolCreateFlags.html#associatedconstant.RESET_COMMAND_BUFFER) – command buffers가 개별적으로 기록되도록 합니다. 이 플래그가 없다면 모든 command buffer들이 한번에 초기화될겁니다.
- [`vk::CommandPoolCreateFlags::PROTECTED`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.CommandPoolCreateFlags.html#associatedconstant.PROTECTED) – Vulkan이 메모리 접근으로부터 비허가된 연산을 막는 ["protected" memory](https://www.khronos.org/registry/vulkan/specs/1.1-extensions/html/vkspec.html#memory-protected-access-rules)에 저장될 "protected" command buffers를 생성합니다.

프로그램의 시작에서만 command buffers를 기록하고 main loop에서 여러번 실행할겁니다. 그리고 DRM을 이용해서 우리의 삼각형을 보호할 필요는 없기 때문에, 이 중의 어떤 플래그도 사용하지 않을겁니다.

```rust
data.command_pool = device.create_command_pool(&info, None)?;
```

commands는 프로그램 내내 화면에 무언가를 그리기위해 사용되므로 pool은 마지막에만 파괴되어야합니다.

```rust
unsafe fn destroy(&mut self) {
    self.device.destroy_command_pool(self.data.command_pool, None);
    // ...
}
```


## Command buffer allocation

이제 command buffers를 할당하고 drawing commands를 command buffers에 기록할 수 있습니다. drawing command들 중 한가지는 올바른 [`vk::Framebuffer`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.Framebuffer.html)에 바인딩되는것을 포함하기 때문에, 실제로 swapchain에서 모든 이미지에 대해 command buffer를 다시 기록해야합니다. 이를 위해서, [`vk::CommandBuffer`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.CommandBuffer.html) 오브젝트의 리스트를 `AppData`의 필드로써 생성합니다. command buffer들은 command pool이 파괴될 때 자동으로 해제되므로, 명시적으로 청소할 필요는 없습니다.

```rust
struct AppData {
    // ...
    command_buffers: Vec<vk::CommandBuffer>,
}
```

이제 `create_command_buffers` 함수에서 작업을 시작합니다. 이 함수는 각 swapchain image에 대한 commands를 할당하고 기록합니다.

```rust
impl App {
    unsafe fn create(window: &Window) -> Result<Self> {
        // ...
        create_command_pool(&instance, &device, &mut data)?;
        create_command_buffers(&device, &mut data)?;
        // ...
    }
}

unsafe fn create_command_buffers(device: &Device, data: &mut AppData) -> Result<()> {
    Ok(())
}
```

command buffers는 [`allocate_command_buffers`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/trait.DeviceV1_0.html#method.allocate_command_buffers) 함수를 통해 할당됩니다. 이 함수는 [`vk::CommandBufferAllocateInfo`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.CommandBufferAllocateInfo.html) 구조체를 파라미터로 받습니다. 이 구조체는 command pool과 할당할 buffer들의 수를 지정합니다.

```rust
let allocate_info = vk::CommandBufferAllocateInfo::builder()
    .command_pool(data.command_pool)
    .level(vk::CommandBufferLevel::PRIMARY)
    .command_buffer_count(data.framebuffers.len() as u32);

data.command_buffers = device.allocate_command_buffers(&allocate_info)?;
```

`level` 파라미터는 할당된 command buffer가 primary/secondary command buffers인지 지정합니다.

- [`vk::CommandBufferLevel::PRIMARY`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.CommandBufferLevel.html#associatedconstant.PRIMARY) – 실행을 위해 queue로 전송될 수 있지만, 다른 command buffers로부터 호출될 수 없습니다.
- [`vk::CommandBufferLevel::SECONDARY`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.CommandBufferLevel.html#associatedconstant.SECONDARY) – 직접적으로 전송될 수 없지만, primary command buffers로부터 호출될 수 있습니다.

여기서는 secondary command buffer 기능을 사용하지 않을것이지만, primary command buffers에서의 공통 연산을 재사용하는데 도움이 되는것을 상상할 수 있습니다.

## Starting command buffer recording

작은 [`vk::CommandBufferBeginInfo`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.CommandBufferBeginInfo.html) 구조체를 매개변수로 사용하여 [`begin_command_buffer`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/trait.DeviceV1_0.html#method.begin_command_buffer)를 호출함으로써 command buffer기록을 시작합니다. 구조체는 특정 command buffer의 사용에 대한 몇가지 정보를 지정합니다.

```rust
for (i, command_buffer) in data.command_buffers.iter().enumerate() {
    let inheritance = vk::CommandBufferInheritanceInfo::builder();

    let info = vk::CommandBufferBeginInfo::builder()
        .flags(vk::CommandBufferUsageFlags::empty()) // Optional.
        .inheritance_info(&inheritance);             // Optional.

    device.begin_command_buffer(*command_buffer, &info)?;
}
```

`flags` 파라미터는 어떻게 command buffer를 사용할 지 지정합니다. 다음과 같은 값들이 가능합니다.

- [`vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.CommandBufferUsageFlags.html#associatedconstant.ONE_TIME_SUBMIT) – command buffer가 실행되자마자 저장됩니다.
- [`vk::CommandBufferUsageFlags::RENDER_PASS_CONTINUE`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.CommandBufferUsageFlags.html#associatedconstant.RENDER_PASS_CONTINUE) – single render pass에 완전피 포함되는 secondary command buffer입니다.
- [`vk::CommandBufferUsageFlags::SIMULTANEOUS_USE`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.CommandBufferUsageFlags.html#associatedconstant.SIMULTANEOUS_USE) – command buffer가 pending execution중에도 재제출이 가능합니다.

당장은 어떤 플래그도 적용되지 않습니다.

## Starting a render pass

render pass를 시작하기 전에, 몇가지 파리미터를 빌드해야합니다.

```rust
let render_area = vk::Rect2D::builder()
    .offset(vk::Offset2D::default())
    .extent(data.swapchain_extent);
```

여기에 render area의 크기를 정의합니다. render area는 render pass의 실행동안 어디에서 shader 로드 및 저장이 발생할지 정의합니다. 이 region의 바깥 픽셀은 undefined value를 갖게될겁니다. render area는 최적의 퍼포먼스를 위해 attachment들의 크기와 맞아야합니다.

```rust
let color_clear_value = vk::ClearValue {
    color: vk::ClearColorValue {
        float32: [0.0, 0.0, 0.0, 1.0],
    },
};
```

다음으로 render pass의 시작에서 framebuffer를 clear하기 위해 사용될 clear value를 정의해야합니다(왜냐하면 render pass를 생성할 때 [`vk::AttachmentLoadOp::CLEAR`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.AttachmentLoadOp.html#associatedconstant.CLEAR)를 사용했기 때문). [`vk::ClearValue`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/union.ClearValue.html)은 color attachments를 위하거나 depth/stencil attachments를 위해 사용되는 clear values인 union입니다. 여기서 `color` 필드를 100% opacity의 검정색을 정의하는 4개 `f32`을 사용하여 [`vk::ClearColorValue`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/union.ClearColorValue.html) union으로 설정합니다.

drawing은 [`cmd_begin_render_pass`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/trait.DeviceV1_0.html#method.cmd_begin_render_pass)를 사용하야 render pass를 시작함으로써 시작합니다. render pass는 [`vk::RenderPassBeginInfo`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.RenderPassBeginInfo.html) 구조체 안에서 몇가지 파라미터를 사용하여 구성됩니다.

```rust
let clear_values = &[color_clear_value];
let info = vk::RenderPassBeginInfo::builder()
    .render_pass(data.render_pass)
    .framebuffer(data.framebuffers[i])
    .render_area(render_area)
    .clear_values(clear_values);
```

첫 번쨰 파라미터는 render pass 자체이고 attachment가 바인딩됩니다. 각 swapchain image에 대해 framebuffer를 생성했고 framebuffer는 swapchain image를 color attachment로 지정합니다. 그리고 이전에 생성한 render area과 clear value를 제공합니다.

```rust
device.cmd_begin_render_pass(
    *command_buffer, &info, vk::SubpassContents::INLINE);
```

이제 render pass를 시작할 수 있습니다. commands를 기록하는 모든 함수들은 `cmd_prefix`로 알 수 있습니다. 그 함수들은 `()`를 리턴하므로 recording를 끝내기까지 error handling이 필요하지 않습니다.

모든 command를 위한 첫 번째 파리미터는 command를 기록할 command buffer입니다. 두 번째 차라미터는 이전에 제공한 render pass의 디테일을 지정합니다. 마지막 파라미터는 drawing command가 render pass에서 어떻게 제공될 지 컨트롤합니다. 이 파라미터는 두 값중 한개를 가질 수 있습니다.

- [`vk::SubpassContents::INLINE`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.SubpassContents.html#associatedconstant.INLINE) – render pass commands가 primary command buffer에 그 자체로 임베드됩니다. 그리고 secondary command buffer는 실행되지 않습니다.
- [`vk::SubpassContents::SECONDARY_COMMAND_BUFFERS`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.SubpassContents.html#associatedconstant.SECONDARY_COMMAND_BUFFERS) – render pass commands가 secondary command buffers에서 실행됩니다.

secondary command buffers를 사용하지 않을것이므로, 첫번째 옵션으로 갑니다.

## Basic drawing commands

이제 graphics pipeline을 바인딩합니다.

```rust
device.cmd_bind_pipeline(
    *command_buffer, vk::PipelineBindPoint::GRAPHICS, data.pipeline);
```

두 번쨰 파라미터는 pipeline 오브젝트가 graphics/compute pipeline인지를 지정합니다. 이제 Vulkan에 graphics pipeline에서 어떤 command가 실행될지와 fragment shader에서 어떤 attachment가 사용될지를 알려주었으므로, 남은것은 삼각형을 그리라고 알려주는것입니다.

```rust
device.cmd_draw(*command_buffer, 3, 1, 0, 0);
```

실제 drawing function은 약간 anticlimactic이지만, 미리 지정한 모든 정보들 덕분에 꽤 간단합니다. drawing function은 command buffer외에 다음과 같은 파라미터가 따라옵니다.

- `vertex_count` – vertex buffer를 갖지 않더라도 기술적으로는 여전히 그리기위한 3개의 vertex들을 갖습니다.
- `instance_count` – instanced rendering을 위해 사용됩니다. 이것을 하지 않으면 `1`을 씁니다.
- `first_vertex` – vertex buffer로의 offset으로 사용됩니다. `gl_VertexIndex`의 가장 낮은값을 정의합니다.
- `first_instance` – instanced rendering을 위한 offset으로 사용됩니다. `gl_InstanceVertex`의 가장 낮은 값을 정의합니다.

## Finishing up

이제 render pass를 끝낼 수 있습니다.

```rust
device.cmd_end_render_pass(*command_buffer);
```

그리고 command buffer를 기록하는 것을 끝냈습니다.

```rust
device.end_command_buffer(*command_buffer)?;
```

다음 챕터에서는 main loop를 위한 코드를 작성할겁니다. 이 코드는 swapchain으로부터 이미지를 얻고, 적절한 command buffer를 실행할겁니다 그리고 완료된 이미지를 swapchain에 반환할겁니다.
