# Shader modules

**Code:** [main.rs](https://github.com/KyleMayes/vulkanalia/tree/master/tutorial/src/09_shader_modules.rs) | [shader.vert](https://github.com/KyleMayes/vulkanalia/tree/master/tutorial/shaders/09/shader.vert) | [shader.frag](https://github.com/KyleMayes/vulkanalia/tree/master/tutorial/shaders/09/shader.frag)

이전의 API들과 달리, Vulkan에서 shader code는 [GLSL](https://en.wikipedia.org/wiki/OpenGL_Shading_Language)과 [HLSL](https://en.wikipedia.org/wiki/High-Level_Shading_Language)같은 human-readable 문법과 상반되게, bytecode format으로 지정되어야합니다.이 bytecode format은 [SPIR-V](https://www.khronos.org/spir) 으로 불리며, Vulkan과 OpenCL(둘 다 Khronos API)에서 사용됩니다. 이것은 graphics와 compute shader를 작성하기 위해 사용될 수 있는 포맷이지만, 이 튜토리얼에서는 Vulkan의 graphics pipeline에서 사용될 shader에 집중할겁니다.

bytecode format 사용의 이점은 shader코드를 native code로 바꾸기 위한 GPU vendor에 의해 작성된 컴파일러가 상당이 덜 복잡하다는 것입니다. 과거에는 GLSL같은 human-readable 문법을 보여주면, 일부 GPU vendor가 표준을 해석하는데 유연했습니다. 만약 이러한 vendor의 GPU로 사소하지 않은 shader를 작성한다면, 다른 vendor의 드라이버가 코드를 문법오류로 거부하거나, 컴파일러 버그에 의해 다르게 작동되는 위험을 각오해야합니다. SPIR-V같은 straightforward bytecode format을 사용하면 바라건데, 이런 경우를 피할 수 있습니다.

그러나, 이것이 직접 bytecode를 작성해야한다는 것을 의미하지는 않습니다. Khronos는 그들의 vendor-independent 컴파일러를 릴리즈했습니다. 이 컴파일러는 GLSL을 SPIR-V로 컴파일합니다. 그 컴파일러는 코드가 표준을 완전히 준수하는지 확인하고 프로그램에 실을 수 있는 SPIR-V binary를 만들어내기 위해 디자인되었습니다. 런타임에 SPIR-V를 만들어내기 위해 컴파일러를 라이브러리로 포함시킬 수 있지만, 이 튜토리얼에서는 그러지 않을겁니다. `glslangValidator.exe`를 통해 직접 컴파일러를 사용할 수 있지만, 대신 Google이 만든 `glslc.exe`를 사용할겁니다. `glslc`의 이점은 GCC와 Clang같은 잘 알려진 컴파일러과 같은 파라미터 포맷을 사용하며 *includes*같은 추가적인 기능성을 포함합니다. 저 두개는 이미 Vulkan SDK에 포함되어 있으므로, 추가로 뭔가를 다운로드할 필요는 없습니다.

GLSL은 C-style 문법의 shading language입니다. 이 문법으로 쓰인 프로그램은 `main` 함수를 갖고 이 함수는 모든 오브젝트에 대해 invoke됩니다. input으로 파라미터를 쓰고 output을 리턴하는 대신에, GLSL은 input과 output을 핸들링하기 위해 전역 변수를 사용합니다. 이 언어는 graphics programming에 도움을 주기위한 built-in vector와 matrix 기본요소가 있습니다. cross product, matrix-vector product 그리고 reflection around a vector같은 연산을 위한 함수가 포함되어 있습니다. vector 타입은 요소의 양을 나타내는 수와 함께 `vec`로 호출됩니다. 예를 들어, 3D position은 `vec3`에 저장될 수 있습니다. `.x`같은 필드를 사용하여 single component에 접근할 수 있지만, 동시에 multiple component로부터 새로운 vector를 생성하는것도 가능합니다. 예를 등러, 표현식 `vec3(1.0, 2.0, 3.0).xy`는 `vec2`를 만들어냅니다. 또한 vector의 생성자는 vector object와 scalar value의 조합을 취할 수도 있습니다. 예를 들어, `vec3`는 `vec3(vec2(1.0, 2.0), 3.0)`으로 생성될 수 있습니다.

이전 챕터에서 언급했듯이, 화면에 삼각형을 그리기 위해 vertex shader와 fragment shader를 작성해야합니다. 다음 두 섹션에서는 각각 두 shader의 GLSL코드를 다루고 그 후에 어떻게 두 SPIR-V 바이너리를 만들어낸 다음 프로그램에 로드하는지 보여줄겁니다.

## Vertex shader

vertex shader는 들어오는 각각의 vertex를 처리합니다. 이 shader는 vertex들의 world position, color, normal 그리고 texture coordinate같은 attribute를 input으로 가져옵니다. output은 clip coordinate에서의 마지막 position과 color과 texture coordinate같은 fragment shader로 넘겨져야할 attribute들입니다. 그러면 이 값들은 rasterizer에 의해 smooth gradient를 만들어내기 위해 fragment에서 보간될겁니다.

*clip coordinate*는 vertex shader로부터 온 4차원 vector입니다. 이 vector는 나중에 last component로 전체 vector를 나누어 *normalized device coordinate*로 바뀝니다. 이런 normalized device coordinate는 framebuffer를 아래처럼 보이는 `[-1, 1] by [-1, 1]` coordinate system에 매핑하는 [homogeneous coordinates](https://en.wikipedia.org/wiki/Homogeneous_coordinates)입니다.

![coordinates](https://kylemayes.github.io/vulkanalia/images/normalized_device_coordinates.svg)

만약 이전에 computer graphics를 접해보았다면 이미 이것에 친숙할겁니다. 이전에 OpenGL을 써봤다면, Y 좌표의 부호가 뒤집힌것을 확인할 수 있을겁니다. Z좌표는 지금 0에서 1까지 Direct3D와 같은 범위를 사용합니다.

어떤 transformation도 적용하지 않을 첫번째 삼각형을 위해, 다음과 같은 모양을 만들어내기 위해서 세 vertex들을 직접 normalized device coordinate의 위치로 지정할겁니다.

![normalized device coordinate](https://kylemayes.github.io/vulkanalia/images/triangle_coordinates.svg)

last component를 `1`로 설정한 vertex shader에서 normalized device coordinate를 clip coordinate로 출력하여 직접적으로 좌표를 출력할 수 있습니다. 이렇게 하면 clip coordinate 를 normalize device coordinate로 변환하는 division이 아무런 변경을 하지 않습니다.

보통 이런 좌표들은 vertex buffer에 저장되지만, Vulkan에서 vertex buffer를 생성하는 것과 vertex buffer를 채우는것은 사소한일이 아닙니다. 그러므로 이 과정을 화면에 삼각형을 띄우는 것을 보기전까지 연기하겠습니다. 그동안에 약간 색다른 작업을 하겠습니다. vertex shader에 coordinate를 직접 포함합니다. 코드는 다음과 같습니다.

```glsl
#version 450

vec2 positions[3] = vec2[](
    vec2(0.0, -0.5),
    vec2(0.5, 0.5),
    vec2(-0.5, 0.5)
);

void main() {
    gl_Position = vec4(positions[gl_VertexIndex], 0.0, 1.0);
}
```

`main` 함수는 모든 vertex에 대해 invoke됩니다. built-in `gl_VertexIndex` 변수는 현재 vertex의 index를 담고있습니다. 보통 이 값은 vertex buffer의 index이지만, 우리의 경우에는 하드코딩된 vertex data 배열의 index입니다. 각 vertex의 위치는 shader의 상수 배열에서 접근되고 더미 `z`와 `w` component에 의해 clip coordinate에서의 위치를 만들어내기 위해 조합됩니다. built-in 변수 `gl_Position`는 output으로써 작동합니다.

## Fragment shader

vertex shader로 생성된 위치로 형성된 삼각형은 fragment를 이용해서 스크린을 채웁니다. fragment shader는 이러한 fragment에 대해 color와 framebuffer(또는 framebuffers)를 위한 depth를 생성하기위해 invoke됩니다. 간단한 전체 삼각형을 빨간색으로 출력하기 위한 간단한 fragment shader는 다음과 같습니다.

```glsl
#version 450

layout(location = 0) out vec4 outColor;

void main() {
    outColor = vec4(1.0, 0.0, 0.0, 1.0);
}
```

vertex shader의 `main`함수가 모든 vertex에 대해 호출되는 것처럼 `main` 함수는 모든 fragment에 대해 호출됩니다. GLSL의 색상은 R, G, B 그리고 `[0, 1]`범위의 alpha채널을 가진 4-component vector입니다. vertex shader의 `gl_Position`과는 다르게 현재 fragment를 위해 색상을 출력할 built-in 변수는 없습니다. `layout(location = 0)` modifier가 framebuffer의 index를 지정하는 상황에서 각 모든 framebuffer 에 대한 자신만의 output 변수를 지정해야합니다. 빨간색은 `outColor` 변수에 쓰여집니다. 이 변수는 index가 `0`인 첫번째(유일한) framebuffer에 링크됩니다.

## Per-vertex colors

삼각형 전체를 빨간색으로 만드는것은 그다지 재미있지 않습니다. 다음과 같은게 더 보기 좋지 않나요?

![triangle](https://kylemayes.github.io/vulkanalia/images/triangle_coordinates_colors.png)

이런것을 만들기 위해 두 shader에 몇가지 수정을해야합니다. 우선, 각 세 vertex의 구분되는 색을 지정해야합니다. vertex shader는 다음과 같은 position들에 대한 색상 배열을 포함합니다.

```glsl
vec3 colors[3] = vec3[](
    vec3(1.0, 0.0, 0.0),
    vec3(0.0, 1.0, 0.0),
    vec3(0.0, 0.0, 1.0)
);
```

이제 이 per-vertex 색상들을 fragment shader에 넘기면 framebuffer에 보간된 색들의 값을 출력할수 있습니다. vertex shader에 대한 색상 출력을 추가하고 `main` 함수에서 그 out에 write합니다.

```glsl
layout(location = 0) out vec3 fragColor;

void main() {
    gl_Position = vec4(positions[gl_VertexIndex], 0.0, 1.0);
    fragColor = colors[gl_VertexIndex];
}
```

다음으로 fragment shader에서 matching input을 추가해야합니다.

```glsl
layout(location = 0) in vec3 fragColor;

void main() {
    outColor = vec4(fragColor, 1.0);
}
```

input변수는 꼭 이름이 같아야하는건 아닙니다. 그 변수들은 `location` directive에 의해 지정된 index들을 사용하여 링크됩니다. `main` 함수는 alpha 값에 따라 색상을 출력하기위해 수정됩니다. 위의 이미지에서 보이듯이, `fragColor`에 대한 값은 세 vertex사이에서 fragment에 대해 자동으로 보간되었고 smoothing gradient를 만들어냅니다.

## Compiling the shaders

`shaders` 디렉토리를 프로젝트의 루트 디렉토리에 만듭니다(`src`디렉토리와 가까운곳). 그 디렉토리에서 vertex shader를 `shader.vert`에 저장하고 fragment shader를 `shader.frag`에 저장합니다. GLSL shader는 공식적인 확장자가 없지만, 저 두가지가 보통 구분하기위해 사용됩니다.

`shader.vert`의 내용

```glsl
#version 450

layout(location = 0) out vec3 fragColor;

vec2 positions[3] = vec2[](
    vec2(0.0, -0.5),
    vec2(0.5, 0.5),
    vec2(-0.5, 0.5)
);

vec3 colors[3] = vec3[](
    vec3(1.0, 0.0, 0.0),
    vec3(0.0, 1.0, 0.0),
    vec3(0.0, 0.0, 1.0)
);

void main() {
    gl_Position = vec4(positions[gl_VertexIndex], 0.0, 1.0);
    fragColor = colors[gl_VertexIndex];
}
```

그리고 `shader.frag`의 내용

```glsl
#version 450

layout(location = 0) in vec3 fragColor;

layout(location = 0) out vec4 outColor;

void main() {
    outColor = vec4(fragColor, 1.0);
}
```

이제 이것들을 `glslc` 프로그램을 사용하여 SPIR-V bytecode로 변환할겁니다.

**Windows**

TODO

**Linux**


```sh
glslc shader.vert -o vert.spv
glslc shader.frag -o frag.spv
```

**macOS**

TODO

**End of platform-specific instructions**

이 두 명령어들은 compiler에게 GLSL소스파일을 읽어서 `-o` (output) 플래그를 사용하여 SPIR-V bytecode로 출력하는것을 말합니다.

만약 shader가 문법오류를 포함한다면, 컴파일러는 기대했듯이 line number와 프로그램을 알려줄겁니다. 예를들어 semicolon을 없애고 컴파일러 스크립트를 다시 실행해보세요. 또한 어떤 argument없이 컴파일러를 실행하여 어떤 플래그들이 지원되는지 확인해보세요. 예를 들어, 컴파일러는 bytecode를 human-readable format으로 출력해서 shader가 정확히  뭘 하는지 볼수있고 이 stage에서 어떤 optimization이 적용되었는지 볼 수 있습니다.

commandline에서 shader를 컴파일하는것은 가장 straightforward한 옵션입니다. 그리고 이 튜토리얼에서 사용할 방법입니다. 그러나 코드에서 직접 컴파일하는것도 가능합니다. Vulkan SDK는 [libshaderc](https://github.com/google/shaderc)를 포함하고 이것은 프로그램안에서 GLSL 코드를 SPIR-V로 컴파일해주는 라이브러리입니다.,

## Loading a shader

SPIR-V shader를 생성하는 방법을 알았으므로, SPIR-V를 프로그램으로 가져와서 graphics pipeline에 플러그해야합니다. Rust standard library의 [`include_bytes!`](https://doc.rust-lang.org/stable/std/macro.include_bytes.html)를 사용하여 shader를 위해 컴파일된 SPIR-V bytecode를 t실행파일에 포함합니다.

```rust
unsafe fn create_pipeline(device: &Device, data: &mut AppData) -> Result<()> {
    let vert = include_bytes!("../shaders/vert.spv");
    let frag = include_bytes!("../shaders/frag.spv");

    Ok(())
}
```

## Creating shader modules

코드를 pipeline에 넘기기전에, 코드를 [`vk::ShaderModule`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.ShaderModule.html) 오브젝트로 래핑해야합니다. helper function [`create_shader_module`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/trait.DeviceV1_0.html#method.create_shader_module)를 만들어서 래핑하도록 합니다.

```rust
unsafe fn create_shader_module(
    device: &Device,
    bytecode: &[u8],
) -> Result<vk::ShaderModule> {
}
```

이 함수는 bytecode를 포함하는 슬라이스를 파라미터로 가져옵니다. 그리고 logical device를 사용하는  [`vk::ShaderModule`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.ShaderModule.html)를 생성합니다.

shader module을 생성하는것은 간단합니다. bytecode의 length와 bytecode slice그 자체를 지정해주기만 하면 됩니다. 이 정보는 [`vk::ShaderModuleCreateInfo`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.ShaderModuleCreateInfo.html) 구조체에 지정합니다. 한 가지 문제는 bytecode의 크기가 bytes로 지정된다는 것이지만, 이 구조체에서 기대하는 bytecode slice는 `&[u8]`대신 `&[u32]`입니다. 그러므로 먼저 `&[u8]`을 `&[u32]`로 변환해야합니다.

`vulkanalia`는 [`Bytecode`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/bytecode/struct.Bytecode.html)라고 불리는 helper struct가 있습니다. 이 구조체는 shader bytecode를 `u32` 배열을 위해 올바른 alignment가 보장된 새로운 버퍼로 복사하는데 사용할겁니다. 이 helper struct를 위한 import를 추가합니다.

```rust
use vulkanalia::bytecode::Bytecode;
```

[`create_shader_module`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/trait.DeviceV1_0.html#method.create_shader_module) 함수로 돌아가서, `Bytecode::new`는 제공된 byte splice가 4배수가 아니거나 aligned buffer의 할당이 실패할 경우 오류를 반환할겁니다. 유효한 shader bytecode를 제공하는 한 문제가 되지는 않을겁니다. 그래서 단순히 결과에 `unwrap`를 합니다.

```rust
let bytecode = Bytecode::new(bytecode).unwrap();
```

그러면 [`vk::ShaderModuleCreateInfo`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.ShaderModuleCreateInfo.html)를 생성할 수 있고 [`create_shader_module`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/trait.DeviceV1_0.html#method.create_shader_module)를 호출하여 shader module을 생성하도록 할 수 있습니다.

```rust
let info = vk::ShaderModuleCreateInfo::builder()
    .code_size(bytecode.code_size())
    .code(bytecode.code());

Ok(device.create_shader_module(&info, None)?)
```

파라미터는 이전의 객체 생성함수의 파라미터들과 같습니다. create info 구조체와 optional custom allocator입니다.

shader module은 단순히 이전의 그 파일로부터 로드한 shader bytecode와 그 안에서 정의된 함수를 감싸는 작은 래퍼입니다. GPU에서의 실행을 위한 SPIR-V bytecode를 machine code로 컴파일하고 링킹하는것은 graphics pipeline이 생성되기 전까지 일어나지 않습니다. 이것은 pipeline creation이 끝나자마자 shader module을 다시 파괴할수 있게 하는것을 의미합니다. 그리고 이것은 `AppData`의 필드 대신 `create_pipeline`의 local 변수로 그것들을 저장할 이유입니다.

```rust
unsafe fn create_pipeline(device: &Device, data: &mut AppData) -> Result<()> {
    let vert = include_bytes!("../shaders/vert.spv");
    let frag = include_bytes!("../shaders/frag.spv");

    let vert_shader_module = create_shader_module(device, &vert[..])?;
    let frag_shader_module = create_shader_module(device, &frag[..])?;

    // ...
```

이 청소는 [`destroy_shader_module`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/trait.DeviceV1_0.html#method.destroy_shader_module)의 호출을 추가해서 함수의 끝에서 일어나도록 합니다. 이 챕터에너 남은것은 이 줄 사이에 들어갑니다.

```rust
    // ...

    device.destroy_shader_module(vert_shader_module, None);
    device.destroy_shader_module(frag_shader_module, None);

    Ok(())
}
```

## Shader stage creation

실제로 shader를 사용하기위해서, 실제 pipeline creation과정의 일부분으로 [`vk::PipelineShaderStageCreateInfo`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.PipelineShaderStageCreateInfo.html) 구조체를 특정 pipeline의 stage에 할당해야합니다.

vertex shader를 위한 구조체를 채우는것으로 시작합니다. 다시 `create_pipeline` 함수안으로 갑니다.

```rust
let vert_stage = vk::PipelineShaderStageCreateInfo::builder()
    .stage(vk::ShaderStageFlags::VERTEX)
    .module(vert_shader_module)
    .name(b"main\0");
```

첫번째 단계는 Vulkan에 shader가 사용되기 위해 어떤 pipeline stage에 갈지 알려주는것입니다. 이전 챕터에서 설명했던 각각의 programmable stage들을 위한 variant가 있습니다.

다음 두 필드는 코드를 포함하는 shader module과 *entrypoint*로 알려진 invoke할 함수를 지정합니다. 이것은 multiple fragment shader를 single shader module로 혼합하고 그들의 동작사이에서 서로다른 entry point를 사용할 수 있음을 의미합니다. 그러나 이 경우에는 표준 `main`에 고수합니다.

한가지 (optional) 멤버가 있습니다. `specialization_info`입니다. 이 멤버는 여기서는 쓰지 않지만, 논의할 가치는 있습니다. 이 멤버는 shader constant를 위한 value를 지정할 수 있게 해줍니다. pipeline 생성시에 사용된 상수에 대해 다른 값을 지정하는것으로 pipeline 생성시에 동작이 구성될 수 있는 상황에서 single shader module을 사용할 수 있습니다. 이것은 render time에 variable을 사용하는 shader를 구성하는 것 보다 효율적입니다. 왜냐하면 컴파일러가 이러한 값에 의존하여 `if`구문을 제거하는 것과 같은 최적화를 할 수 있기 때문입니다. 만약 그런 어떠한 상수도 없다면, 단순히 여기서 하는것처럼 단순히 그 세팅을 스킵해도 됩니다.

구조체를 수정해서 fragment shader를 fragment shader에 맞게 하는것은 쉽습니다.

```rust
let frag_stage = vk::PipelineShaderStageCreateInfo::builder()
    .stage(vk::ShaderStageFlags::FRAGMENT)
    .module(frag_shader_module)
    .name(b"main\0");
```

pipeline의 programmable stage를 설명하는것은 여기까지입니다. 다음챕터에서 fixed-function stage를 살펴볼겁니다.
