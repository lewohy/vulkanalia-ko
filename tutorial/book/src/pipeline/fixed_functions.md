# Fixed functions

**Code:** [main.rs](https://github.com/KyleMayes/vulkanalia/tree/master/tutorial/src/10_fixed_functions.rs)

older graphics API들은 graphics pipeline의 단계의 대부분에 대해서 default state를 제공합니다. Vulkan에서는 모든것을 명시적으로 해야합니다. viewport size에서부터 color blending function까지. 이번 챕터에서는 구조체의 모든 것을 채워서 fixed-function 연산을을 구성할겁니다.

## Vertex input

[`vk::PipelineVertexInputStateCreateInfo`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.PipelineVertexInputStateCreateInfo.html) 구조체는 vertex shader로 넘겨질 vertex data의 포맷을 설명합니다. 대략 두 가지 방법을 설명합니다.

- Bindings – data사이의 spacing과 그 data가 per-vertex인지 per-instance인지 여부(see [instancing](https://en.wikipedia.org/wiki/Geometry_instancing))
- Attribute descriptions – vertex shader로 넘겨질 attribute들의 타입, 속성을 로드할 바인딩과 오프셋

vertex shader안에 vertex 데이터를 직접 하드코딩했기때문에, 이 구조체를 기본값으로 두어서 당장은 로드될 vertex data가 없음을 지정합니다. vertex buffer챕터에서 다시 돌아올겁니다. `create_pipeline` 함수의 [`vk::PipelineShaderStageCreateInfo`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.PipelineShaderStageCreateInfo.html) 구조체 바로 뒤에 추가합니다.

```rust
unsafe fn create_pipeline(device: &Device, data: &mut AppData) -> Result<()> {
    // ...

    let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::builder();
```

여기서 설정될 수 있는 이 구조체를 위한 `vertex_binding_descriptions`과 `vertex_attribute_descriptions` 필드는 앞서 설명한 vertex data로딩을 위해 필요한 디테일을 설명하는 구조체의 슬라이스입니다.

## Input assembly

[`vk::PipelineInputAssemblyStateCreateInfo`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.PipelineInputAssemblyStateCreateInfo.html) 구조체는 두가지를 설명합니다. 어떤 종류의 geometry가 vertex들로부터 그려질지와 primitive restart가 활성화되어야 하는지 여부입니다. 전자는 `topology`멤버에서 지정되고 다음과 같은 값을 가질 수 있습니다.

- [`vk::PrimitiveTopology::POINT_LIST`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.PrimitiveTopology.html#associatedconstant.POINT_LIST) – vertex들로부터 points
- [`vk::PrimitiveTopology::LINE_LIST`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.PrimitiveTopology.html#associatedconstant.LINE_LIST) –  reuse없이 모든 2개의 vertex마다 line
- [`vk::PrimitiveTopology::LINE_STRIP`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.PrimitiveTopology.html#associatedconstant.LINE_STRIP) – 모든 line의 끝 vertex는 다음 line의 시작 vertex로 사용됩니다.
- [`vk::PrimitiveTopology::TRIANGLE_LIST`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.PrimitiveTopology.html#associatedconstant.TRIANGLE_LIST) – reuse없이 모든 3개의 vertex마다 삼각형
- [`vk::PrimitiveTopology::TRIANGLE_STRIP`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.PrimitiveTopology.html#associatedconstant.TRIANGLE_STRIP) – 모든 삼각형의 두번쨰와 세번쨰 vertex가 다음 삼각형의 첫 두 vertex로 사용됩니다.

보통, vertex들은 vertex buffer에서 index에 의해 순차적으로 로드되지만, *element buffer*를 사용하면 사용할 index들을 직접 지정할 수 있습니다. element buffer는 vertex의 재사용같은 최적화를 수행할 수 있게 해줍니다. 만약 `primitive_restart_enable` 멤버를 `true`로 설정한다면, line들과 `_STRIP` topology modes안의 삼각형을 special index인 `0xFFFF` 또는 `0xFFFFFFFF`를 사용하여 구분할 수 있습니다.

이 튜토리얼동안에는 삼각형을 그리는것을 의도하고 다음의 구조체를 따를겁니다.

```rust
let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::builder()
    .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
    .primitive_restart_enable(false);
```

## Viewports and scissors

viewport는 기본적으로 output이 렌더링될 framebuffer의 region을 설명합니다. viewport는 거의 항상 `(0, 0)`에서 `(width, height)`이고 이 튜토리얼에서도 같은 경우일겁니다.

```rust
let viewport = vk::Viewport::builder()
    .x(0.0)
    .y(0.0)
    .width(data.swapchain_extent.width as f32)
    .height(data.swapchain_extent.height as f32)
    .min_depth(0.0)
    .max_depth(1.0);
```

swapchain과 swapchain의 image의 size는 window의 `WIDTH`와 `HEIGHT`랑 다를수도 있다는 것을 기억하세요. swapchain image는 나중에 framebuffer로 사용될겁니다. 그래서 swapchain image의 사이즈에 맞춰야합니다.

`min_depth`와 `max_depth`값은 framebuffer를 위해 사용할 depth value의 범위를 지정합니다. 이 값들은 `[0.0, 1.0]`범위안에 있습니다. 그러나 `min_depth`는 `max_depth`보다 클 수도 있습니다. 특별한 일을 하는게 아니라면 표준 값인 `0.0`과 `1.0`에 맞춥니다.

viewport가 image에서 framebuffer로의 transformation을 정의하는 반면, scissor rectangle는 실제로 어떤 범위의 픽셀이 저장될지를 정의합니다. scissor rectangle의 밖의 어떤 픽셀이던 rasterizer에 의해 폐기될겁니다. scissor rectangle들은 transformation보다는 filter처럼 작동합니다. 차이점은 밑에서 보여집니다. 왼쪽 scissor rectangle는 scissor rectangle이 viewport보다 큰 한, 저런 이미지를 생성하는 많은 가능성중 한가지임을 유의하세요.

![scissor](https://kylemayes.github.io/vulkanalia/images/viewports_scissors.png)

이 튜토리얼에서는 단순히 전체 framebuffer를 그리는것을 원하므로 scissor rectangle를 전체를 덮도록 지정할겁니다.

```rust
let scissor = vk::Rect2D::builder()
    .offset(vk::Offset2D { x: 0, y: 0 })
    .extent(data.swapchain_extent);
```

이제 viewport와 scissor rectangle은 [`vk::PipelineViewportStateCreateInfo`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.PipelineViewportStateCreateInfo.html) 구조체를 이용해서 viewport state로 조합되어야합니다. 몇 그래픽카드는 여러 viewport와 scissor rectangle를 사용하는것도 가능하므로, 이 구조체의 멤버는 저것들의 배열입니다. 여러개를 사용하는것은 GPU feature의 활성화를 요구합니다(logical device creation 참조).

```rust
let viewports = &[viewport];
let scissors = &[scissor];
let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
    .viewports(viewports)
    .scissors(scissors);
```

## Rasterizer

rasterizer는 vertex shader의 vertex들로부터 형성된 geometry를 가지고 이것들을 fragment shader에 의해 색이 입혀진 fragment들로 바꿉니다. rasterizer는 또한 [depth testing](https://en.wikipedia.org/wiki/Z-buffering), [face culling](https://en.wikipedia.org/wiki/Back-face_culling) 그리고 scissor test를 수행하고 전체 폴리곤을 채우거나 단순히 edges(wireframe rendering)을 출력하도록 구성될 수 있습니다. 이 모든 것은 [`vk::PipelineRasterizationStateCreateInfo`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.PipelineRasterizationStateCreateInfo.html) 구조체를 사용하여 구성될 수 있습니다.

```rust
let rasterization_state = vk::PipelineRasterizationStateCreateInfo::builder()
    .depth_clamp_enable(false)
    // continued...
```

만약 `depth_clamp_enable`이 `true`로 설정되었다면, near과 far plane너머에 있는 fragment들은 폐기되지 않고 clamp됩니다. 이것은 shadow map같은 특별한 케이스에서 유용합니다. 이걸 사용하기 위해서는 GPU feature을 활성화하는것을 요구합니다.

```rust
    .rasterizer_discard_enable(false)
```

만약 `rasterizer_discard_enable`가 `true`라면, geometry는 절대로 rasterizer stage로 넘겨지지 않습니다. 이것은 기본적으로 framebuffer으로의 출력을 비활성화합니다.

```rust
    .polygon_mode(vk::PolygonMode::FILL)
```

`polygon_mode`는 geometry를 위한 fragment가 어떻게 생성될지를 결정합니다. 다음의 모드들이 가능합니다.

- [`vk::PolygonMode::FILL`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.PolygonMode.html#associatedconstant.FILL) – polygon을 fragment로 채웁니다.
- [`vk::PolygonMode::LINE`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.PolygonMode.html#associatedconstant.LINE) – polygon의 edges만 line으로 그려집니다.
- [`vk::PolygonMode::POINT`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.PolygonMode.html#associatedconstant.POINT) – polygon vertices만 점으로 그려집니다.

다른 모드들을 사용하는것은 GPU feature의 활성화를 요구합니다.

```rust
    .line_width(1.0)
```

`line_width` 멤버는 straightforward합니다. 이것은 fragment의 숫자의 관점에서 line의 두꺼움정도를 설명합니다. 하드웨어에 의존하여 지원되는 최대 line width와 `1.0`보다 두꺼운 어떤 line이던 `wide_lines` GPU feature를 활성화하는것을 요구합니다.

```rust
    .cull_mode(vk::CullModeFlags::BACK)
    .front_face(vk::FrontFace::CLOCKWISE)
```

`cull_mode` 변수는 사용할 face culling의 타입을 결정합니다. culling을 비활성화하거나, front face를 cull하거나, back face를 cull하거나 둘다 그럴수도 있습니다. `front_face` 변수는 front-facing으로 고려된 face를 위한 vertex order를 지정하고 clockwise 또는 counterclockwise가 될 수 있습니다.

```rust
    .depth_bias_enable(false);
```

rasterizer는 constant value를 추가하거나 fragment의 slope에 따른 biasing을 함으로써 depth value를 수정할 수 있습니다. 가끔 shadow mapping을 위해 사용되지만, 사용하지 않을겁니다. 그냥 `depth_bias_enable`을 `false`로 설정합니다.

## Multisampling

[`vk::PipelineMultisampleStateCreateInfo`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.PipelineMultisampleStateCreateInfo.html)구조체는 multisampling을 구성합니다. 이 구조체는 [anti-aliasing](https://en.wikipedia.org/wiki/Multisample_anti-aliasing)을 수행하기 위한 방법 중 하나입니다. 이 구조체는 같은 픽셀에 rasterize된 여러 polygon의 fragment shader 결과를 조합하여 작동합니다. 대부분 가장자리를 따라 발생하며, 가장 눈에 띄는 aliasing artifact가 발생하는 곳이기도 합니다. 한개의 polygon이 한개의 픽셀에만 매핑된다면 fragment shader를 여러번 실행할 필요가 없기 때문에, 단순히 높은 resolution으로 렌더링하고 downscaling하는것보다 훨씬 저렴합니다. 이것을 활성화하기 위해서는 GPU feature를 활성화해야합니다.

```rust
let multisample_state = vk::PipelineMultisampleStateCreateInfo::builder()
    .sample_shading_enable(false)
    .rasterization_samples(vk::SampleCountFlags::_1);
```

이후 챕터에서 multisampling을 다시 방문할겁니다. 지금은 비활성화된 상태로 둡니다.

## Depth and stencil testing

만약 depth 그리고/또는 stencil buffer를 사용하고있다면, [`vk::PipelineDepthStencilStateCreateInfo`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.PipelineDepthStencilStateCreateInfo.html)를 사용하여 depth와 stencil을 구성해야합니다. 한개는 지금 하지는 않으므로 무시힙니다. depth buffering 챕터에서 다시 돌아올겁니다.

## Color blending

fragment shader가 색을 반환한 후에, framebuffer에 이미 들어있는 색과 조합되어야합니다. 이 transformation은 color blending으로 알려져있고 이것을 하기 위한 두 가지 방법이 있습니다.

- old와 new를 섞어서 final color을 만들어내기
- old와 new를 bitwise연산으로 조합하기

color blending을 구성하기 위한 두가지 타입의 구조체가 있습니다. 첫번째 구조체 [`vk::PipelineColorBlendAttachmentState`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.PipelineColorBlendAttachmentState.html)는 attached framebuffer마다 configuration을 포함합니다. 두 번째 구조체 [`vk::PipelineColorBlendStateCreateInfo`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.PipelineColorBlendStateCreateInfo.html)는 *global* color blending세팅을 포함합니다. 우리의 경우에서는 오직 한개의 framebuffer만 가질겁니다.

```rust
let attachment = vk::PipelineColorBlendAttachmentState::builder()
    .color_write_mask(vk::ColorComponentFlags::all())
    .blend_enable(false)
    .src_color_blend_factor(vk::BlendFactor::ONE)  // Optional
    .dst_color_blend_factor(vk::BlendFactor::ZERO) // Optional
    .color_blend_op(vk::BlendOp::ADD)              // Optional
    .src_alpha_blend_factor(vk::BlendFactor::ONE)  // Optional
    .dst_alpha_blend_factor(vk::BlendFactor::ZERO) // Optional
    .alpha_blend_op(vk::BlendOp::ADD);             // Optional
```

이러한 per-framebuffer 구조체는 color blending의 첫 번째 방법을 구성하도록 해줍니다. 이렇게 수행될 연산은 다음 pseudocode를 따라 잘 설명됩니다.

```rust
if blend_enable {
    final_color.rgb = (src_color_blend_factor * new_color.rgb)
        <color_blend_op> (dst_color_blend_factor * old_color.rgb);
    final_color.a = (src_alpha_blend_factor * new_color.a)
        <alpha_blend_op> (dst_alpha_blend_factor * old_color.a);
} else {
    final_color = new_color;
}

final_color = final_color & color_write_mask;
```

`blend_enable`이 `false`로 설정되어 있다면, fragment shader로부터 온 새 color은 수정되지 않은 채로 넘겨집니다. 그렇지 않은 경우, 두 mixing operation이 수행되어 새로운 색상을 계산해냅니다. resulting color는 `color_write_mask`와 AND연산되어 어떤 채널이 실제로 넘겨질지 결정합니다.

가장 일반적인 color blending을 사용하는 방법은 alpha blending을 구현하는 것입니다. 이 구현에서는 색의 opacity에 기반하여 new color가 old color와 blend되기를 원합니다. `final_color`는 다음과 같이 계산됩니다.

```rust
final_color.rgb = new_alpha * new_color + (1 - new_alpha) * old_color;
final_color.a = new_alpha.a;
```

이것은 다음과 같은 파라미터들로 달성될 수 있습니다.

```rust
let attachment = vk::PipelineColorBlendAttachmentState::builder()
    .color_write_mask(vk::ColorComponentFlags::all())
    .blend_enable(true)
    .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
    .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
    .color_blend_op(vk::BlendOp::ADD)
    .src_alpha_blend_factor(vk::BlendFactor::ONE)
    .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
    .alpha_blend_op(vk::BlendOp::ADD);
```

가능한 연산을 specification(또는 `vulkanalia`의 문서)안에서 [`vk::BlendFactor`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.BlendFactor.html)과 [`vk::BlendOp`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.BlendOp.html)의 enumeration에서 찾아볼 수 있습니다.

두 번째 구조체는 모든 framebuffer들의 구조체에 대한 배열을 참조합니다. 그리고 이 구조체는 앞서 말한 연산에서 blend factor로 사용할 수 있는 blend constant를 설정할 수 있게 해 줍니다.

```rust
let attachments = &[attachment];
let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
    .logic_op_enable(false)
    .logic_op(vk::LogicOp::COPY)
    .attachments(attachments)
    .blend_constants([0.0, 0.0, 0.0, 0.0]);
```

만약 blending의 두 번째 방법을 쓰고 싶다면 (bitwise combination), `logic_op_enable`을 `true`로 설정해야 합니다. bitwise 연산은 `logic_op` 필드에 지정할 수 있습니다. 이것은 자동으로 모든 attached framebuffer에 대해 `blend_enable`을 `false`하는것과 같이 첫 번째 방법을 비활성화한다는 것을 주목하세요. `color_write_mask`또한 이 모드에서는 framebuffer에서 어떤 채널이 실제로 영향을 받을지 결정하기 위해 사용됩니다. 여기서 한것처럼, 두 가지를 모두 비활성화하는것이 가능하고 이 경우 fragment color가 수정되지 않은 채로 framebuffer에 작성됩니다.

## Dynamic state (example, don't add)

이전 구조체에서 지정한 state의 제한된 양은 실제로 pipeline을 재생성하지 않고 수정하는것이 *가능합니다*. 예시로 viewport의 크기, line width, 그리고 blend constant가 있습니다. 만약 수정하기를 원한다면, [`vk::PipelineDynamicStateCreateInfo`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.PipelineDynamicStateCreateInfo.html)구조체를 다음과 같이 채워야합니다.

```rust
let dynamic_states = &[
    vk::DynamicState::VIEWPORT,
    vk::DynamicState::LINE_WIDTH,
];

let dynamic_state = vk::PipelineDynamicStateCreateInfo::builder()
    .dynamic_states(dynamic_states);
```

이것은 이 state의 값들이 무시되게 하고 drawing시간에 data를 지정할것을 요구합니다. 이후 챕터에서 다시 돌아올겁니다. 어떤 dynamic state도 원하지 않는다면, 이 구조체는 빼도 됩니다.

## Pipeline layout

shader에서 `uniform` 값을 사용할 수 있습니다. 이 값은 dynamic state와 유사한 global이고 shader를 재생성하지 않고 drawing time에 변경되어 shader의 동작을 바꿀 수 있습니다. 이 값들은 보통 vertex shader에 transformation matrix을 전달하기 위해 사용되거나, fragment shader에서 texture sampler를 생성하기 위해 사용됩니다.

이러한 uniform value들은 [`vk::PipelineLayout`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.PipelineLayout.html) 객체를 생성하여 pipeline creation동안에 지정되어야 합니다. 다음 챕터까지 사용하지는 않더라도, 여전히 empty pipeline layout을 생성하는것이 요구됩니다.

나중에 다른 함수에서 이 객체를 참조할것이므로, `AppData` 필드를 생성해서 이 객체를 저장합니다.

```rust
struct AppData {
    // ...
    pipeline_layout: vk::PipelineLayout,
}
```

그리고 `create_pipeline`함수에서 [`destroy_shader_module`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/trait.DeviceV1_0.html#method.destroy_shader_module) 의 호출 바로 위에 객체를 생성합니다.

```rust
unsafe fn create_pipeline(device: &Device, data: &mut AppData) -> Result<()> {
    // ...
    
    let layout_info = vk::PipelineLayoutCreateInfo::builder();
    
    data.pipeline_layout = device.create_pipeline_layout(&layout_info, None)?;
    
    device.destroy_shader_module(vert_shader_module, None);
    device.destroy_shader_module(frag_shader_module, None);
    
    Ok(())
}
```

또한 이 구조체는 *push constants*를 지정합니다. 이것은 이후 챕터에서 들어가볼 shader에 dynamic value를 전달하기 위한 또다른 방법입니다. 이 pipeline layout은 프로그램의 lifetime동안 참조될것이므로, `App:destroy`에서 파괴되어야합니다.

```rust
unsafe fn destroy(&mut self) {
    self.device.destroy_pipeline_layout(self.data.pipeline_layout, None);
    // ...
}
```

## Conclusion

이것이 fixed-function state에 대한 모든것입니다. 처음부터 설정하기위해 많은 작업이 필요하지만,. 이것의 이점은 graphics pipeline에서 진행되는 모든것들에 대한 거의 모든 것에 알고 있는 것입니다. 특정 component의 default state가 예상한것과 달라서 발생하는 unexpected behavior를 줄입니다.

그러나 graphics pipeline을 생성하기 위해 생성할 객체 하나가 더 있습니다. render pass입니다.
