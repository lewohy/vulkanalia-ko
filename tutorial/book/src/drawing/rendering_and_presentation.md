# Rendering and presentation

**Code:** [main.rs](https://github.com/KyleMayes/vulkanalia/tree/master/tutorial/src/15_hello_triangle.rs)

이 챕터는 모든것이 모이는 챕터입니다. 화면에 삼각형을 놓기 위해 main loop에서 호출될 `App::render` 함수를 구현합니다.

## Synchronization

`App::render` 함수는 다음 연산들을 수행합니다.

- swapchain으로부터 이미지를 얻습니다.
- framebuffer에서 이미지를 attachment로 이용하여 command buffer를 실행합니다.
- presentation을 위해 이미지를 swapchain으로 반환합니다.

각 이벤트들은 single function call을 이용하여 실행되지만, 비동기적으로 실행됩니다. function call들은 연산들이 실제로 실행되기전에 return되고 execution 순서는 undefined입니다. 각 연산은 이전의 연산의 종료에 의존하기때문에, 안타까운일입니다.

swapchain event들을 동기화하는 두 방법이 있습니다. fences와 semaphores입니다. 이 두 방법은 한개의 operation signal을 갖고 또다른 operation이 unsignaled에서 signaled state가 되기 위해 기다리도록 하여 operation을 coordinate하는 객체입니다.

차이점은 fences의 state들은 [`wait_for_fences`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/trait.DeviceV1_0.html#method.wait_for_fences)같은 호출을 사용하는 프로그램에서 접근이 가능하지만 semaphores는 그렇지 않은 것입니다. fences는 주로 애플리케이션을 rendering operation과 동기화하기위해 디자인되었습니다. 우리는 draw commands와 presentation의 queue operations들을 동기화하기를 원하므로, semaphores가 딱 맞습니다.

## Semaphores

한 이미지가 얻어졌고 rendering을 위해 준비가 됨을 signal하기 위해서 semaphore가 하나 필요합니다. 그리고 rendering이 끝나고 presentation이 일어남을 signal 위해서 또 하나가 필요합니다. 두 개의 `AppData`  필드를 생성해서 이 semaphore 오브젝트들을 저장하도록 합니다.

```rust
struct AppData {
    // ...
    image_available_semaphore: vk::Semaphore,
    render_finished_semaphore: vk::Semaphore,
}
```

semaphores를 생성하기 위해, 튜토리얼의 이 파트를 위한 마지막 `create` 함수를 추가할겁니다. `create_sync_objects` 함수를 추가합니다.

```rust
impl App {
    unsafe fn create(window: &Window) -> Result<Self> {
        // ...
        create_command_buffers(&device, &mut data)?;
        create_sync_objects(&device, &mut data)?;
        // ...
    }
}

unsafe fn create_sync_objects(device: &Device, data: &mut AppData) -> Result<()> {
    Ok(())
}
```

semaphore를 생성하는 것은 [`vk::SemaphoreCreateInfo`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.SemaphoreCreateInfo.html)를 채울 것을 요구하지만, API의 이 버전은 실제로 어떤 필드도 요구하지 않습니다.

```rust
unsafe fn create_sync_objects(device: &Device, data: &mut AppData) -> Result<()> {
    let semaphore_info = vk::SemaphoreCreateInfo::builder();

    Ok(())
}
```

Vulkan API 또는 extensions의 미래 버전은 아마 다른 구조체를 위해 했던것처럼, `flags`와 `p_next`를 위한 기능을 추가할 수도 있습니다. semaphore를 생성하는것은 비슷한 패턴을 따릅니다.

```rust
data.image_available_semaphore = device.create_semaphore(&semaphore_info, None)?;
data.render_finished_semaphore = device.create_semaphore(&semaphore_info, None)?;
```

semaphores는 프로그램의 끝에 청소되어야합니다. 모든 command들이 끝나고 더이상 synchronization이 필요하지 않을 때 합니다.

```rust
unsafe fn destroy(&mut self) {
    self.device.destroy_semaphore(self.data.render_finished_semaphore, None);
    self.device.destroy_semaphore(self.data.image_available_semaphore, None);
    // ...
}
```

## Acquiring an image from the swapchain

이전에 언급했듯이, `App::render` 함수에서 처음으로 해야할 것은 swapchain으로부터 image를 얻어오는 것입니다. swapchain이 extension feature임을 회상하면, `*_khr` 네이밍 컨벤션을 따르는 함수를 사용해야 합니다.

```rust
unsafe fn render(&mut self, window: &Window) -> Result<()> {
    let image_index = self
        .device
        .acquire_next_image_khr(
            self.data.swapchain,
            u64::MAX,
            self.data.image_available_semaphore,
            vk::Fence::null(),
        )?
        .0 as usize;

    Ok(())
}
```

[`acquire_next_image_khr`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/trait.KhrSwapchainExtension.html#method.acquire_next_image_khr)의 첫 번째 파라미터는 swapchain입니다. 이 swapchain으로부터 이미지를 얻기를 원합니다. 두 번째 파라미터는 이용할 수 있게 된 이미지에 대한 timeout을 nanoseconds로 지정합니다. 64 bit unsigned integer의 최대 값을 사용하면 timeout을 비활성화합니다.

다음 두 파라미터는 synchronization 오브젝트를 지정합니다. 이들은 image를 사용하여 presentation engine이 끝날 때 signaled됩니다. 이 시점이  presentation engine에 이미지를 그릴수 있게 된 시점입니다. semaphore, fence 또는 둘다 지정하는것이 가능합니다. 여기서 이 용도로 우리의 `image_avilable_semaphore`를 사용합니다.

이 함수는 이용가능한 swapchain image의 index를 반환합니다. 반환된 index는 우리의 `swapchain_images` 배열 안의 [`vk::Image`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.Image.html)를 가리킵니다. 적절한 command buffer를 선택하기 위해 이 index를 사용합니다.

## Submitting the command buffer

queue submission과 synchronization은 [`vk::SubmitInfo`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.SubmitInfo.html) 구조체안의 파라미터를 통해 구성됩니다.

```rust
let wait_semaphores = &[self.data.image_available_semaphore];
let wait_stages = &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
let command_buffers = &[self.data.command_buffers[image_index as usize]];
let signal_semaphores = &[self.data.render_finished_semaphore];
let submit_info = vk::SubmitInfo::builder()
    .wait_semaphores(wait_semaphores)
    .wait_dst_stage_mask(wait_stages)
    .command_buffers(command_buffers)
    .signal_semaphores(signal_semaphores);
```

처음 두 파라미터, `wait_semaphores`와 `wait_dst_stage_mask`는 execution 시작 전에 대기할 semaphores와 pipeline의 어느 stage(s)에서 대기할지 지정합니다. 이미지가 이용가능해질 때 까지 이미지에 색을 작성하는것을 미루고 싶으므로, color attachment에 write하는 graphics pipeline의 stage를 지정합니다. 이것은 이론상, 이미지가 아직 이용불가능하더라도, 구현이 미리 우리의 vertex shader 등의 실행을 시작할 수 있는 것을 의미합니다. `wait_stages` 배열의 각 entry는 `wait_semaphores`의 같은 index인 semaphore와 대응됩니다.

다음 `command_buffers` 파라미터는 실행을 위해 실제로 어떤 command buffer가 제출될지 지정합니다. 이전에 언급했듯이, color attachment로써 얻어진 swapchain image를 바인딩하는 command buffer를 제출해야합니다.

마지막으로 `signal_semaphores`는 command buffer(s)가 실행을 끝낼 때 signal할 semaphore를 지정합니다. 우리의 경우 이를 위해 `render_finished_sepahore`를 사용합니다.

```rust
self.device.queue_submit(
    self.data.graphics_queue, &[submit_info], vk::Fence::null())?;
```

이제 [`queue_submit`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/trait.DeviceV1_0.html#method.queue_submit)를 사용하여 graphics queue에 command buffer를 제출할 수 있습니다. 이 함수는 workload가 훨씬 클때 효율을 위해 [`vk::SubmitInfo`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.SubmitInfo.html) 구조체의 배열을 인수로 받습니다. 마지막 파라미터는 optional fence를 받고 이 fence는 command buffers가 실행을 끝낼 때 signaled됩니다. synchronization을 위해 semaphores를 썼으므로, 단지 `vk::Fence::null()`을 넘깁니다.

## Subpass dependencies

render pass에서 subpass는 자동으로 image layout transitions을 관리하는 것을 기억하세요. 이러한 transitions은 *subpass dependencies*에 의해 제어됩니다. 그리고 subpass dependencies는 subpasses사이에서 memory와 execution dependencies를 지정합니다. 당장은 한개의 single pass만 갖고있지만, 이 subpass 직전과 직후의 연산은 암시적으로 "subpasses"로 카운팅됩니다.

render pass의 시작과 끝에서 transition을 관리하는 두가지 built-in dependencies가 있지만, 전자는 적절한 시간에 발생하지 않습니다. pipeline의 시작에서 transition이 발생했지만, 그 시점에 이미지를 아직 얻지 못했다고 가정해봅시다. 이 문제를 다룰 두 가지 방법이 있습니다. `image_available_semaphore`를 위한 `wait_stages`를 [`vk::PipelineStageFlags::TOP_OF_PIPE`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.PipelineStageFlags.html#associatedconstant.TOP_OF_PIPE)로 변경하여 render passes가 이미지가 이용가능해질 때 까지 시작하지 않는 것을 보장하거나, render pass가 [`vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.PipelineStageFlags.html#associatedconstant.COLOR_ATTACHMENT_OUTPUT) stage를 기다리도록 할 수 있습니다. 저는 여기서 두 번째 옵션을 선택했습니다. 왜냐하면, subpass dependencies와 이 dependencies들이 어떻게 작동하는지 보기에 좋은 변명이 되기 때문입니다.

subpass dependencies는 [`vk::SubpassDependency`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.SubpassDependency.html) 구조체로 지정됩니다. `create_render_pass` 함수로 가서 하나 추가합니다.

```rust
let dependency = vk::SubpassDependency::builder()
    .src_subpass(vk::SUBPASS_EXTERNAL)
    .dst_subpass(0)
    // continued...
```

첫 두개의 필드는 dependency의 index들과 dependent subpass를 지정합니다. special value인 [`vk::SUBPASS_EXTERNAL`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/constant.SUBPASS_EXTERNAL.html)는 `src_subpass`나 `dst_subpass`에 지정되었는지에 따라 render pass의 전이나 후의 암시적인 subpass를 가리킵니다. index `0`은 우리의 subpass를 가리킵니다. 이 subpass는 첫번째고 오직 하나입니다. `dst_subpass`는 dependency graph에서 cycles을 방지하기 위해 항상 `src_subpass`보다 높아야합니다(subpass들 중 하나가 [`vk::SUBPASS_EXTERNAL`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/constant.SUBPASS_EXTERNAL.html)가 아닌 한).

```rust
    .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
    .src_access_mask(vk::AccessFlags::empty())
```

다음 두개의 필드는 기다릴 operations들과 이 operations들이 일어날 stages를 지정합니다. 우리가 이미지에 접근하기 전에 swapchain이 image로부터 읽기를 끝내기를 기다려야합니다. 이 과정은 color attachment output stage 자체에서 기다림으로써 가능합니다.

```rust
    .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
    .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE);
```

여기서 기다려야 할 operations은 color attachment stage안에 있고 color attachment의 작성을 포함합니다. 이러한 세팅은 실제로 필요할 때 까지(그리고 허용될 때 까지) transition이 가 발생하지 않도록 합니다. 즉, 우리가 color를 쓰려고 할 때 까지입니다.

```rust
let attachments = &[color_attachment];
let subpasses = &[subpass];
let dependencies = &[dependency];
let info = vk::RenderPassCreateInfo::builder()
    .attachments(attachments)
    .subpasses(subpasses)
    .dependencies(dependencies);
```

[`vk::RenderPassCreateInfo`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.RenderPassCreateInfo.html) 구조체는 dependencies의 배열을 지정할 필드를 가지고 있습니다.

## Presentation

frame을 그리는 마지막 단계는 결과를 swapchain에 반환하여 마침내 화면에 보여지도록 하는것입니다. Presentation `App::render` 함수의 끝에서 [`vk::PresentInfoKHR`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.PresentInfoKHR.html) 구조체를 통해 구성됩니다.

```rust
let swapchains = &[self.data.swapchain];
let image_indices = &[image_index as u32];
let present_info = vk::PresentInfoKHR::builder()
    .wait_semaphores(signal_semaphores)
    .swapchains(swapchains)
    .image_indices(image_indices);
```

첫 번째 파라미터는 presentation이 일어나기 전에, [`vk::SubmitInfo`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.SubmitInfo.html)같은, 어떤 semaphore에서 기다릴지 지정합니다.

다음 두개의 파라미터는 이미지를 표시할 swapchains을 지정하고 각 swapchain에 대한 이미지의 index를 지정합니다. 거의 항상 single one일겁니다.

마지막으로 `results`로 불리는 optional 파라미터가 있습니다. 이 파라미터는 [`vk::Result`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.Result.html) value의 배열을 지정하여 각 모든 swapchain에 대해 presentation이 성공적인지 확인할 수 있도록 합니다. single swapchain만 쓴다면 필수적이지 않습니다. 왜냐하면 단순히 present function의 리턴값만 사용할수 있기 때문입니다.

```rust
self.device.queue_present_khr(self.data.present_queue, &present_info)?;
```

[`queue_present_khr`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/trait.KhrSwapchainExtension.html#method.queue_present_khr) 함수는 swapchain에 이미지를 표시하기 위한 요청을 제출합니다. 다음 챕터에서 에러 핸들링을 위한 [`acquire_next_image_khr`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/trait.KhrSwapchainExtension.html#method.acquire_next_image_khr)과 [`queue_present_khr`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/trait.KhrSwapchainExtension.html#method.queue_present_khr)를 수정할겁니다. 왜냐하면, 이들의 실패는 지금까지 본 함수와 달리, 프로그램이 꼭 종료되어야함을 의미하지 않기 때문입니다.

여기까지 모든것이 올바르게 되었다면, 프로그램을 실행할 때 다음과 비슷한 무언가를 볼것입니다.

![triangle](https://kylemayes.github.io/vulkanalia/images/triangle.png)

> 이 colored triangle는 graphics tutorial에서 보기위해 사용한것과 약간 다르게 보일 수 있습니다. 이 튜토리얼은 linear color space에서 셰이더를 선형으로 보간하고 후에 sRGB color space로 변환하기 때문입니다. See [this blog post](https://medium.com/@heypete/hello-triangle-meet-swift-and-wide-color-6f9e246616d9) for a discussion of the difference.

와! 불행히도 validation layers가 활성화되어있다면, 프로그램을 닫자마자 크래시를 일으킬겁니다. `debug_callback`으로부터 터미널에 출력된 메세지는 이유를 알려줍니다.

![semaphore_in_use](https://kylemayes.github.io/vulkanalia/images/semaphore_in_use.png)

`App::render` 안의 모든 operations은 asynchronous임을 기억하세요. 이것은 `main`la에서 loop를 끝내기 전 `App::destroy`를 호출할 때 drawing과 presentation operations들이 여전히 진행중임을 의미합니다. 이러한 상황에서 리소스를 정리하는것은 나쁜 생각입니다.

이 문제를 해결하기 위해, `App::destroy`를 호출하기 전에 [`device_wait_idle`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/trait.DeviceV1_0.html#method.device_wait_idle)를 사용하여 logical device가 operations을 끝내기를 기다려야합니다.

```rust
Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
    destroying = true;
    *control_flow = ControlFlow::Exit;
    unsafe { app.device.device_wait_idle().unwrap(); }
    unsafe { app.destroy(); }
}
```

[`queue_wait_idle`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/trait.DeviceV1_0.html#method.queue_wait_idle)를 사용하여 특정 command queue의 operation이 끝날 때 까지 기다리는것 또한 가능합니다. 이런한 함수들은 synchronization을 수행하기위한 가장 기초적인 방법으로 사용됩니다. 윈도우를 닫을 때 프로그램이 더이상 크래시를 일으키지 않는 것을 볼 겁니다(그래도 validation layers가 활성화되어있다면, synchronization과 관련한 몇가지 오류를 볼겁니다.).

## Frames in flight

만약 validation layers를 활성화한채로 애플리케이션을 실행하면 아마 오류를 보거나 메모리 사용량이 천천히 높아진다는 notice를 볼겁니다. 이 이유는 애플리케이션이 `App::render` 함수에서 작업을 빠르게 제출하지만, 실제로는 그것이 끝났는지 체크하지 않기때문입니다. 만약 CPU가 GPU가 할수있는 것보다 빠르게 작업을 제출하는것이, queue는 천천히 작업으로 채워질겁니다. 심지어 더 안좋은일은 우리가 동시에 multiple frames에 대해 `image_available_semaphore`와 `render_finished_semaphore` semaphores, 그리고 command buffer를 재사용한다는 것입니다.

이 문제를 해결하기 위한 쉬운 방법은 작업을 제출후에 그것이 끝날때까지 대기하는 것입니다. 예를 들어, [`queue_wait_idle`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/trait.DeviceV1_0.html#method.queue_wait_idle)를 사용합니다. (note: 실제로 이렇게 변경하지 마세요)

```rust
unsafe fn render(&mut self, window: &Window) -> Result<()> {
    // ...

    self.device.queue_present_khr(self.data.present_queue, &present_info)?;
    self.device.queue_wait_idle(self.data.present_queue)?;

    Ok(())
}
```

그러나, 이 방법으로는 GPU를 최적으로 사용하지 않을것같습니다. 왜냐하면 전체 graphics pipeline은 당장은 한 time에 한개의 frame을 위해서만 사용되기 때문입니다. 현재 frame이 진행한 stage는 idle이고 이미 다음 프레임을 위해 사용될 수 있습니다. 이제 우리의 애플리케이션을 확장하여 여전히 누적되는 작업량을 제한하면서 multiple frames이  *in-flight*가 되도록 할겁니다.

프로그램의 상단에 얼마나 많은 frames이 동시에 실행되는지 정의하는 상수를 추가하면서 시작합니다.

```rust
const MAX_FRAMES_IN_FLIGHT: usize = 2;
```

각 frame은 `AppData`에서 자신만의 semaphores의 세트를 가져야합니다.

```rust
struct AppData {
    // ...
    image_available_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
}
```

`create_sync_objects` 함수는 이 모든것들을 생성하도록 수정되어야 합니다.

```rust
unsafe fn create_sync_objects(device: &Device, data: &mut AppData) -> Result<()> {
    let semaphore_info = vk::SemaphoreCreateInfo::builder();

    for _ in 0..MAX_FRAMES_IN_FLIGHT {
        data.image_available_semaphores
            .push(device.create_semaphore(&semaphore_info, None)?);
        data.render_finished_semaphores
            .push(device.create_semaphore(&semaphore_info, None)?);
    }

    Ok(())
}
```

비슷하게, 이것들 또한 청소되어야합니다.

```rust
unsafe fn destroy(&mut self) {
    self.data.render_finished_semaphores
        .iter()
        .for_each(|s| self.device.destroy_semaphore(*s, None));
    self.data.image_available_semaphores
        .iter()
        .for_each(|s| self.device.destroy_semaphore(*s, None));
    // ...
}
```

매 시간 적절한 semaphores의 쌍을 사용하기 위해서, 현재 frame의 track를 유지해야합니다. 이를위해 frame index를 사용할거고, `App`에 추가합니다(이 값을 `App::create`에서 `0`으로 초기화합니다).

```rust
struct App {
    // ...
    frame: usize,
}
```

이제 `App::render` 함수는 올바른 오브젝트를 쓰도록 수정됩니다.

```rust
unsafe fn render(&mut self, window: &Window) -> Result<()> {
    let image_index = self
        .device
        .acquire_next_image_khr(
            self.data.swapchain,
            u64::MAX,
            self.data.image_available_semaphores[self.frame],
            vk::Fence::null(),
        )?
        .0 as usize;

    // ...

    let wait_semaphores = &[self.data.image_available_semaphores[self.frame]];

    // ...

    let signal_semaphores = &[self.data.render_finished_semaphores[self.frame]];

    // ...

    Ok(())
}
```

물론, 매 시간 다음 frame으로 이동해야하는 것을 잊어서는 안됩니다.

```rust
unsafe fn render(&mut self, window: &Window) -> Result<()> {
    // ...

    self.frame = (self.frame + 1) % MAX_FRAMES_IN_FLIGHT;

    Ok(())
}
```

modular (%) 연산자를 사용함으로써, frame index가 매 `MAX_FRAMES_IN_FLIGHT`  frame이 enqueue된 후 순환되도록 보장합니다.

비록 이제 multiple frame이 동시에 처리되는것을 가능하도록 필요한 오브젝트를 설정했지만, 여전히 `MAX_FRAME_IN_FLIGHT` 보다 많은게 제출되는것을 실제로 막지는 않았습니다. 지금은 CPU-GPU synchronization만 있고 진행상황을 추적하기위한 CPU-GPU synchronization이 진행되고 있지 않습니다. 아마 여전히 frame #0 이 in-flight일때 frame #0 을 사용하고있을 수 있습니다.

CPU-GPU synchronization을 수행하기 위해, Vulkan은 *fences*로 불리는 synchronization primitive의 두번째 타입을 제공합니다. Fences는 이들이 singaled되고 그것을 기다릴수 있다는 점에서 semaphores와 유사하지만, 이번 시간에는 우리만의 코드에서 실제로 기다릴겁니다. 먼저 `AppData`에 각 frame을 위한 fence를 만듭니다.

```rust
struct AppData {
    // ...
    in_flight_fences: Vec<vk::Fence>,
}
```

`create_sync_object` 함수 안에서 fences를 semaphores와 함께 만들겁니다.

```rust
unsafe fn create_sync_objects(device: &Device, data: &mut AppData) -> Result<()> {
    let semaphore_info = vk::SemaphoreCreateInfo::builder();
    let fence_info = vk::FenceCreateInfo::builder();

    for _ in 0..MAX_FRAMES_IN_FLIGHT {
        data.image_available_semaphores
            .push(device.create_semaphore(&semaphore_info, None)?);
        data.render_finished_semaphores
            .push(device.create_semaphore(&semaphore_info, None)?);

        data.in_flight_fences.push(device.create_fence(&fence_info, None)?);
    }

    Ok(())
}
```

fences([`vk::Fence`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.Fence.html))의 생성은 semaphores의 생성과 매우 유사합니다. 또한 `App::destroy`에서 fences를 청소해야합니다.

```rust
unsafe fn destroy(&mut self) {
    self.data.in_flight_fences
        .iter()
        .for_each(|f| self.device.destroy_fence(*f, None));
    // ...
}
```

이제 남은것은 오직 `App::render`의 시작에서 frame이 끝나기를 기다리는 것입니다.

```rust
unsafe fn render(&mut self, window: &Window) -> Result<()> {
    self.device.wait_for_fences(
        &[self.data.in_flight_fences[self.frame]],
        true,
        u64::MAX,
    )?;

    self.device.reset_fences(&[self.data.in_flight_fences[self.frame]])?;

    // ...
}
```

[`wait_for_fences`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/trait.DeviceV1_0.html#method.wait_for_fences) 함수는 fences의 배열을 갖고 하나라도 또는 그 모든것들이 전부 반환되기전에 signaled 되기를 기다립니다. 여기서 넘긴 `true`는 모든 fences를 기다리기를 원한다는 것을 가리키지만, single fence인 이 경우에는 분명히 문제가 되지는 않습니다. [`acquire_next_image_khr`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/trait.KhrSwapchainExtension.html#method.acquire_next_image_khr)와 같이 이 함수는 timeout을 취합니다. semaphores와 다르게, 수동으로 [`reset_fences`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/trait.DeviceV1_0.html#method.reset_fences) call을 이용하여 리셋함으로써 fence를 unsignaled 상태로 복구해야합니다.

이제 프로그램을 실행하면 뭔가 이상한것을 알아챌겁니다. 애플리케이션이 더이상 아무것도 렌더링하지않고, 심지어는 아마 frozen일겁니다.

이것은 제출되지 않은 fence를 기다리고 있다는 것을 의미합니다. 문제는 이것입니다. 기본적으로, fences들은 unsignaled 상태로 생성됩니다. 이것이 [`wait_for_fences`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/trait.DeviceV1_0.html#method.wait_for_fences)가 fence를 이전에 사용하지 않았다면, 영원히 기다릴것이라는 것을 의미합니다. 이를 해결하기 위해, fence 생성을 완료된 초기 frame을 렌더링한것처럼 signaled 샅애로 초기화하도록 수정합니다.

```rust
unsafe fn create_sync_objects(device: &Device, data: &mut AppData) -> Result<()> {
    // ...

    let fence_info = vk::FenceCreateInfo::builder()
        .flags(vk::FenceCreateFlags::SIGNALED);

    // ...
}
```

memory leak은 이제 없어졌지만, 아직 프로그램이 올바르게 작동하지 않습니다. 만약 `MAX_FRAMES_IN_FLIGHT`가 swapchain image의 수보다 크거나, [`acquire_next_image_khr`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/trait.KhrSwapchainExtension.html#method.acquire_next_image_khr)가 out-of-order로 이미지를 반환한다면, 이미 *in flight*인 swapchain image에 렌더링을 시작할겁니다. 이것을 피하기 위해, frame이 이미 swapchain image를 사용중인지 추적해야합니다. 이 mapping은 in flight frames을 frame들의 fences를 통해 참조하므로 새로운 프레임이 해당 이미지를 사용할수 있게되기 전에 즉시 대기할 synchronization 오브젝트를 확보하게됩니다.

먼저 `AppData`에 `images_in_flight`라는 새로운 리스트를 추가하여 이를 추적할 수 있도록 합니다.

```rust
struct AppData {
    // ...
    in_flight_fences: Vec<vk::Fence>,
    images_in_flight: Vec<vk::Fence>,
}
```

`create_sync_objects`에서 준비합니다.

```rust
unsafe fn create_sync_objects(device: &Device, data: &mut AppData) -> Result<()> {
    // ...

    data.images_in_flight = data.swapchain_images
        .iter()
        .map(|_| vk::Fence::null())
        .collect();

    Ok(())
}
```

초기에는 single frame이 이미지를 사용하지 않으므로 명시적으로 *no fence*로 초기화합니다. 이제 `App::render`를 수정해서 다음 frame을 위해 할당한 이미지를 사용하는 어떤 이전 frame이던 기다리도록 합니다.

```rust
unsafe fn render(&mut self, window: &Window) -> Result<()> {
    // ...

    let image_index = self
        .device
        .acquire_next_image_khr(
            self.data.swapchain,
            u64::MAX,
            self.data.image_available_semaphores[self.frame],
            vk::Fence::null(),
        )?
        .0 as usize;

    if !self.data.images_in_flight[image_index as usize].is_null() {
        self.device.wait_for_fences(
            &[self.data.images_in_flight[image_index as usize]],
            true,
            u64::MAX,
        )?;
    }

    self.data.images_in_flight[image_index as usize] =
        self.data.in_flight_fences[self.frame];

    // ...
}
```

이제 더 많은 [`wait_for_fences`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/trait.DeviceV1_0.html#method.wait_for_fences) 호출이 있으므로, [`reset_fences`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/trait.DeviceV1_0.html#method.reset_fences) call은 **이동**되어야 합니다. 실제로 fence를 사용하기 직전에 호출하는것이 베스트입니다.

```rust
unsafe fn render(&mut self, window: &Window) -> Result<()> {
    // ...

    self.device.reset_fences(&[self.data.in_flight_fences[self.frame]])?;

    self.device.queue_submit(
        self.data.graphics_queue,
        &[submit_info],
        self.data.in_flight_fences[self.frame],
    )?;

    // ...
}
```

이제 모든 synchronization을 구현해서 enqueue된 작업 frame이 두개를 넘지 않고 이 frame들이 실수로 같은 이미지를 사용하지 않도록 보장했습니다. final cleanup 같은 코드의 다른 부분이 [`device_wait_idle`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/trait.DeviceV1_0.html#method.device_wait_idle)같은 더 rough synchronization에 의존하는것은 괜찮은것을 숙제하세요. 퍼포먼스 요구사하엥 기반한 어떤 접근을 써야하는지 결정해야합니다.

예제를 통해 synchronization에 대해 더 배우고싶다면, Khronos의 [this extensive overview](https://github.com/KhronosGroup/Vulkan-Docs/wiki/Synchronization-Examples#swapchain-image-acquire-and-present)를 살펴보세요

## Conclusion

600줄이 조금 넘는 (비어있지 않은)코드 우헤야, 마침에 스크린에 무언가를 띄우는 stage에 도달했습니다. Vulkan 프로그램을 Bootstrapping하는 것은 많응 양의 작업이지만, 핵심은 Vulkan이 그 명시적인 특성을 통해 엄청난 제어권을 제공한다는 것입니다. 이제 코드를 다시 읽고 프로그램에서 모든 Vulkan 오브젝트들의 목적과 그들이 서로 어떻게 상호작용하는지 이해하는 mental model을 구축하는데 시간을 투자하는것을 권장합니다. 이 지식을 바탕으로, 이 지점부터 프로그램의 기능을 확장할겁니다.

다음 장에서는 well-behaved Vulkan 프로그램을 위해 필요한 작은 사항을 하나 다룰겁니다.
