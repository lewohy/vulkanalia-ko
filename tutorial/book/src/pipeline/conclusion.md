# Conclusion

**Code:** [main.rs](https://github.com/KyleMayes/vulkanalia/tree/master/tutorial/src/12_graphics_pipeline_complete.rs)

이제 이전 챕터에서의 모든 구조체와 객체를 조합하여 graphics pipeline을 만들 수 있습니다! 여기에 우리가 가지고있는 오브젝트들의 타입이 있습니다. quick recap

- Shader stages – graphics pipeline의 programmable stage의 기능성을 정의하는 shader modules
- Fixed-function state – input assembly, rasterizer, viewport 그리고 color blending과 같은 pipeline의 fixed-function stage를 정의하는 모든 구조체
- Pipeline layout – draw time에 업데이트 가능한 shader에 의해 참조된 uniform과 push values
- Render pass – pipeline stage와 그것의 사용에 의해 참조되는 attachments

이 모든 것들의 조합은 graphics pipeline의 모든 기능성을 정의하므로 `create_pipeline` 함수의 끝(shader module이 파괴되기 전)에 [`vk::GraphicsPipelineCreateInfo`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.GraphicsPipelineCreateInfo.html)를 채우기 시작할 수 있습니다. `DeviceV1_0:::destroy_shader_module`전에 호출해야합니다. 왜냐하면 이것들은 여전히 creation에 사용되기 때문입니다.

```rust
let stages = &[vert_stage, frag_stage];
let info = vk::GraphicsPipelineCreateInfo::builder()
    .stages(stages)
    // continued...
```

[`vk::PipelineShaderStageCreateInfo`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.PipelineShaderStageCreateInfo.html) 구조체의 배열을 제공하면서 시작합니다.

```rust
    .vertex_input_state(&vertex_input_state)
    .input_assembly_state(&input_assembly_state)
    .viewport_state(&viewport_state)
    .rasterization_state(&rasterization_state)
    .multisample_state(&multisample_state)
    .color_blend_state(&color_blend_state)
```

그리고 fixed-function stage를 설명하는 구조체의 모든 구조체를 참조합니다.

```rust
    .layout(data.pipeline_layout)
```

그 후에 pipeline layout이 나옵니다. 참조대신 handle을 씁니다.

```rust
    .render_pass(data.render_pass)
    .subpass(0);
```

그리고 마지막으로 render pass에 대한 참조와 이 graphics pipeline이 사용될 sub pass의 index를 갖습니다. 이 specific instance대신 이 pipeline을 이용하여 다른 render passes를 사용하는것도 가능하지만, 이것들은 `render_pass`와 *호환되어야* 합니다. 호환성을 위한 요구사항은 [여기에](https://www.khronos.org/registry/vulkan/specs/1.2/html/vkspec.html#renderpass-compatibility) 설명되어 있지만, 이 튜토리얼에서 그런 기능을 쓰지는 않을것입니다.

```rust
    .base_pipeline_handle(vk::Pipeline::null()) // Optional.
    .base_pipeline_index(-1)                    // Optional.
```

실제로는 파라미터가 두개 더 있습니다. `base_pipeline_handle`과 `base_pipeline_index`입니다. Vulkan은 이미 있는 pipeline으로부터 파생된 새로운 graphics pipeline을 생성하게 해줍니다. pipeline derivatives의 아이디어는 기존 파이프라인과 많은 기능을 공통으로 가지고 있을 때 파이프라인을 설정하는 비용이 저렴하고 동일한 부모의 파이프라인 간 전환도 빨리 수행할 수 있다는 것입니다. `base_pipeline_handle`를 사용하여 이미 존재하는 pipeline의 핸들을 지정하거나 `base_pipeline_index`에 의해 생성될 또다른 pipeline에 대한 참조를 할 수 있습니다. 이제 오직 single pipeline만 있으므로 단순히 null handle과 invalid index를 지정합니다. 이 값들은 오직 [`vk::GraphicsPipelineCreateInfo`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.GraphicsPipelineCreateInfo.html) 의 `flag` 필드가 [`vk::PipelineCreateFlags::DERIVATIVE`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.PipelineCreateFlags.html#associatedconstant.DERIVATIVE)일때만 사용됩니다.

이제 마지막 단계를 위해 `AppData`에 필드를 생성하여 [`vk::Pipeline`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.Pipeline.html) 객제츷 저장하도록 합니다.

```rust
struct AppData {
    // ...
    pipeline: vk::Pipeline,
}
```

그리고 마침내 graphics pipeline을 생성합니다.

```rust
data.pipeline = device.create_graphics_pipelines(
    vk::PipelineCache::null(), &[info], None)?.0[0];
```

Vulkan에서 [`create_graphics_pipelines`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/trait.DeviceV1_0.html#method.create_graphics_pipelines) 함수는 실제로는 일반적인 오브젝트 생성 함수보다 더 많은 파라미터를 갖습니다. 이 함수는 single call에 여러 [`vk::GraphicsPipelineCreateInfo`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.GraphicsPipelineCreateInfo.html) 오브젝트들을 취하고 여러 [`vk::Pipeline`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.Pipeline.html) 오브젝트를 생성합니다.

`vk::PipelineCache::null()` 인수를 넘긴 첫 번째 파라미터는 optional [`vk::PipelineCache`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.PipelineCache.html) 오브젝트를 참조합니다. pipeline cache는 [`create_graphics_pipelines`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/trait.DeviceV1_0.html#method.create_graphics_pipelines)을 여러번 호출할때, 그리고 심지어는 cache가 file로 저장되어있다면 프로그램 실행동안 pipeline creation과 관련된 데이터를 저장하고 재활용할 수 있게 해줍니다. 이것은 이후에 pipeline creation의 속도를 상당히 올려주는걸 가능하게 합니다.

graphics pipeline은 모든 common drawing operation에 필요하므로, 이 또한 `App::destroy`에서 프로그램의 끝에서 파괴되어야합니다.

```rust
unsafe fn destroy(&mut self) {
    self.device.destroy_pipeline(self.data.pipeline, None);
    // ...
}
```

이제 프로그램을 실행해서 이 고된 작업이 성공적인 pipeline creation을 만들어내는지 확인하세요! 우리는 화면에 뭔가를 띄우는것을 보는것에 거의 근접해왔습니다. 다음 몇 챕터에서는 실제로 swapchain image로부터 framebuffers를 설정하고 drawing commands를 준비할겁니다.
