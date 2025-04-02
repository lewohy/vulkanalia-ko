# Introduction

**Code:** [main.rs](https://github.com/KyleMayes/vulkanalia/tree/master/tutorial/src/08_graphics_pipeline.rs)

다음 몇 챕터동안 첫번째 삼각형을 그리기 위해 configure된 graphics pipeline을 설정할겁니다. graphics pipeline은 render target안에서 해당 픽셀에 이르기까지의 vertex들과 mesh의 texture를 취하는 연산의 과정입니다. 간략한 overview는 아래처럼 보입니다.

![pipeline](https://kylemayes.github.io/vulkanalia/images/vulkan_simplified_pipeline.svg)


*input assembler*는 지정한 buffer로부터 raw vertex data를 수집하고 vertex data 자체를 복사하지 않고 특정 element를 반복하기 위해 index buffer또한 사용합니다.

*vertex shader*는 모든 vertex에 대해 실행되고 일반적으로 vertex의 위치를 model space에서 screen space로의 transformation을 적용합니다. vertex shader는 per-vertex data를 pipeline으로 전달합니다.

*tessellation shader*는 mesh의 퀄리티를 높이기 위한 특정한 룰에 기반하여 geometry를 작게 나눌 수 있도록 해줍니다. 이 shader는 종종 근처의 덜 평평한 벽돌이나 계단의 surface를 만들기 위해 사용됩니다.

*geometry shader*는 모든 primitive(triangle, line, point)에 대해 실행되며 primitive가 들어온것을 버리거나 더 많은 primitive를 출력할 수 있습니다. 이 shader는 tessellation shader와 비슷하지만, 더 유연합니다. 그러나, 이 shader는 오늘날 애플리케이션에서는 많이 사용되지 않습니다. 왜냐하면 퍼포먼스가 Intel's integrated GPU를 제외한 많은 그래픽카드에서 그렇게 좋지 않기 때문입니디.

*rasterization* stage는 primitive들을 fragment들로 이산화합니다. fragment들은 framebuffer에서 채워질 픽셀 요소들입니다. 화면 밖으로 나간 어떤 fragment들은 폐기되며 vertex에 의해 출력된 attribute들은 그림에서 보이듯이 fragment간에 보간됩니다. 보통 다른 primitive fragment뒤에 있는 fragment들은 depth testing에 의해 폐기됩니다.

*color blending* stage는 framebuffer에서의 같은 픽셀에 매핑될 서로 다른 fragment들을 혼합하는 연산을 적용합니다. fragment들은 단순히 서로 overwrite하거나 add up 또는 transparency에 기반하여 혼합될 수 있습니다.

초록색 단계들은 *fixed-function* stage로 알려져 있습니다. 이 stage들은 파라미터를 사용하여 연산을 수정하게 해주지만, 이 단계들이 작동하는 방식은 사전 정의되어있습니다.

반면에 오렌지색 단계들은 *programmable*합니다. 이것은 자신만의 코드를 그래픽카드에 업로드해서 정확히 원하는 연산을 적용하도록 한다는 것을 의미합니다. 예를들어 이것은 fragment shader를 사용해서 texturing과 lighting에서부터 ray tracer까지 무엇이든 구현할 수 있게 합니다. 이 프로그램들은 많은 GPU core에서 동시에 실행되어 vertex들과 fragment같은 많은 오브젝트를 병렬적으로 처리합니다.

만약 OpenGL과 Direct3D같은 오래된 API들을 이전에 사용해봣다면, `glBlendFunc`와 `OMSetBlendState`같은 호출을 통해 자유롭게 pipeline setting을 변경하는게 가능했을겁니다. Vulkan에서 pipeline은 거의 완전히 immutable입니다. 그래서 만약 shader를 변경하거나 다른 framebuffer를 바인딩하거나 blend function을 변경하고싶다면, pipeline을 처음부터 다시 생성해야합니다. 불리한점은 렌더링 연산에서 사용하기를 원하는 state의 모든 서로다른 조합을 나타내는 수많은 pipeline을 생성해야하는 것입니다. 그러나 pipeline에서 할 연산의 모든것은 이미 잘 알려져있기 때문에, 드라이버는 훨씬 괜찮게 최적화할 수 있습니다.

몇가지 programmable stage는 뭘 하기를 원하는지에 따라 선택적입니다. 예를들어 tessellation과 geometry stage는 단순히 geometry를 그리는중이라면 비활성화 될 수 있습니다. 만약 오직 depth value에만 관심이 있다면, fragment shader stage를 비활성화 할 수 있습니다. 이것은 [shadow map](https://en.wikipedia.org/wiki/Shadow_mapping) 생성을 위해 유용합니다.

다음 챕터에서는 화면에 삼각형을 놓기 위해 필요한 두 가지 programmable stage를 생성할겁니다. vertex shader와 fragment shader입니다. blending mode, viewport, rasterization과 같은 fixed-function configuration은 그 이후에 설정될겁니다. Vulkan에서 graphics pipeline을 설정하는것의 마지막 부분은 input과 output framebuffer의 specification을 포함합니다.

`create_pipeline` 함수를 생성합니다. 이 함수는 `App::create`안에서 `create_swapchain_image_views` 바로 뒤에 호출됩니다. 이후 챕터에서 이 함수안에서 작업할겁니다.

```rust
impl App {
    unsafe fn create(window: &Window) -> Result<Self> {
        // ...
        create_swapchain_image_views(&device, &mut data)?;
        create_pipeline(&device, &mut data)?;
        // ...
    }
}

unsafe fn create_pipeline(device: &Device, data: &mut AppData) -> Result<()> {
    Ok(())
}
```
