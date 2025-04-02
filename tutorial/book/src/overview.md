# Overview

이 챕터는 Vulkan의 소개와 Vulkan이 다루는 문제로 시작합니다. 그 후, 첫 번째 삼각형을 위해 요구되는 요소들을 살펴봅니다. 이 과정은 여러분에게 big picture를 제공하고, 이 안에서 subsequent 챕터들을 배치할수 있습니다. 우리는 `vulkanalia`에 의해 구현된 Vulkan API의 구조체로 커버함으로써 결론지을겁니다.

## Origin of Vulkan

기존의 graphics APIs같이, Vulkan은 [GPUs](https://en.wikipedia.org/wiki/Graphics_processing_unit)위의 cross-platform abstraction으로 디자인되었습니다. 대부분의 이런 APIs의 문제는 이런 APIs가 설계된 시대는 구성 가능한 fixed functionality에 제한된 graphics hardware에 특화되어있다는 것입니다. 프로그래머는 standard format으로 vertex data를 넘겨주어야 했고, lighting과 shading 옵션과 관련해서는 GPU 제조사의 자비에 달려있었습니다.

graphics card 아키텍쳐가 성숙해지면서, 점점 더 많은 programmable functionality를 제공하기 시작했습니다. 이런 모든 functionality는 기존의 API들과 어떻게든 통합되어야 했습니다. 이로 인해 이상적이지 못한 추상화와 프로그래머의 의도를 모던 graphics 아키텍쳐에 매핑하기 위해 graphics driver에 많은 추측 작업을 초래했습니다. 이것이 게임의 퍼포먼스를 향상하기 위해 많은 드라이버 업데이트가 필요한 이유이며, 때로는 상당한 성능 향상을 가져오는 이유입니다. 이러한 드라이버의 복잡성때문에, 애플리케이션 개발자들은 [shaders](https://en.wikipedia.org/wiki/Shader)에 허용되는 문법과 같은 vendor들 사이의 비일관성을 처리해야합니다. 새로운 기능 외에도, 지난 10년은 파워풀한 graphics hardware을 탑재한 모바일 기기의 influx를 보여주었습니다. 이러한 모바일 GPU들은 기기들의 에너지와 공간 요구사항에 따라 다른 아키텍져를 가지고 있습니다. 한 가지 예시는 [tiled rendering](https://en.wikipedia.org/wiki/Tiled_rendering)입니다. 이것은 functionality에 대한 더 많은 저워를 프로그래머에게 제공함으로써 향상된 퍼포먼스의 이득을 볼 수 있습니다. 이런 API들의 시대에서 비롯된 또다른 지한사항은 multi-threading 지원입니다. 이것은 CPU측에서 병목현상이 발생할 수 있습니다.

Vulkan은 이 modern graphics 아키텍쳐를 위해 처음부터 다시 설계함으로써 이러한 문제를 해결합니다. Vulkan은 더 많은 verbose API를 사용하여 프로그래머들의 의도를 명확히 지정할 수 있게 함으로써 driver overhead를 줄입니다. 그리고 multiple threads가 commands를 동식에 제출할 수 있도록 해줍니다. 단일 컴파일러를 사용하여 byte coder format으로 변경함으로써 shader compilation 불일치를 줄입니다. 마지막으로, Vulkan은 graphics와 compute functionality 를 단일 API로 통합함으로써 modern graphics cards의 general purpose processing capabilities를 인정합니다.

## What it take to draw a triangle

이제 잘 동작하는 Vulkan 프로그램에서 삼각형을 그리기위해 필요한 모든 단계의 개요를 살펴볼겁니다. 여기서 소개된 모든 컨셉들은 다음 챕터에서 정교해집니다.이것은 각 개별 컴포넌트와 연관지을 big picture를 제공할 뿐입니다.

### Step 1  - Instance and physical device selection

Vulkan 애플리케이션은 [`VkInstance`](https://www.khronos.org/registry/vulkan/specs/1.4-extensions/man/html/VkInstance.html)를 통해 Vulkan API를 세팅하는 것으로부터 시작합니다. 한 instance는 애플리케이션은 설명하고 사용할 API extensions들을 설명함으로써 생성됩니다. instance 생성 후, Vulkan supported hardware를 query하고 operation을 위해 사용할 한개 이상의 [`VkPhysicalDevice`](https://www.khronos.org/registry/vulkan/specs/1.4-extensions/man/html/VkPhysicalDevice.html)를 선택할 수 있습니다. 요구된 장치를 선택하기 위해 VRAM size와 device capabilities같은 프로퍼티들을 쿼리할 수 있습니다. 예를들어 dedicated graphics cards를 사용하는것을 선호하기 위해 쿼리할 수 있습니다.

### Step 2 - Logical device and queue families

사용할 적절한 device를 선택한 수에, [`VkDevice`](https://www.khronos.org/registry/vulkan/specs/1.4-extensions/man/html/VkDevice.html) (logical device)를 생성해야 합니다. 여기에서는 viewport rendering과 64-bit floats같은 어떤 [`VkPhysicalDeviceFeatures`](https://www.khronos.org/registry/vulkan/specs/1.4-extensions/man/html/VkPhysicalDeviceFeatures.html)를 사용할지 더 구체적으로 설명해야합니다. 또한, 어떤 queue families를 사용할지도 지정해야합니다. draw commands와 memory operations같은 Vulkan으로 수행되는 대부분의 operations은 [`VkQueue`](https://www.khronos.org/registry/vulkan/specs/1.4-extensions/man/html/VkQueue.html)에 제출됨으로써 비동기적으로 실행됩니다. queues는 queue families로부터 할당됩니다. 그리고 각 queue family는 각 queues에서의 특정 연산 집합을 지원합니다. 예를 들어, graphics, compute 그리고 memory transfer operations들을 위한 분리된 queue families가 있을 수 있습니다. queue families의 이용가능성은 physical device 선택할 때 구분할 요소로 사용될 수도 있습니다. Vulkan support device가 어떠한 graphics functionality도 제공하지 않을 수도 있습니다. 그러나 오늘날 모든 Vulkan을 지원하는 그래픽카드는 일반적으로 우리가 관심있는 모든 queue operations을 지원합니다.

### Step 3 - Window surface and swapchain

offscreen rendering에만 관심있는게 아닌 한, 렌더링된 이미지를 표시할 window를 생성해야합니다. windows는 native platform APIs 또는 [GLFW](http://www.glfw.org/),  [SDL](https://www.libsdl.org/), [`winit`](https://github.com/rust-windowing/winit) 크레이트를 사용하여 생성될 수 있습니다. 이 튜토리얼에서는 `winit` 크레이트를 사용할거지만, 다음 장에서 더 자세히 봅니다.

실제로 window에 렌더링하기 위해서는 두가지 컴포넌트가 더 필요합니다. window surface ([`VkSurfaceKHR`](https://www.khronos.org/registry/vulkan/specs/1.4-extensions/man/html/VkSurfaceKHR.html))과 swapchain ([`VkSwapchainKHR`](https://www.khronos.org/registry/vulkan/specs/1.4-extensions/man/html/VkSwapchainKHR.html))입니다. `KHR` postfix를 주목하세요. 이것은 이런 오브젝트가 Vulkan extension의 일부라는것을 의미합니다. Vulkan API는 그 자체로 완전히 플랫폼 불가지론적입니다. 이것이 왜 standardized WSI (Window System Interface) extension을 사용하여 window manager와 상호작용하는 이유입니다. surface는 렌더링할 윈도우 위의 cross-platform abstraction이고, 일반적으로 native window handle에 대한 참조를 제공하면서 instantiated됩니다. 예를 들어, Windows에서는 `HWND`입니다. 그러나, `vulkanalia`는 `winit` crate와 optional integration을 갖고있습니다. 우리는 이 integration을 사용해서 window 그리고 연관된 surface의 platform-specific detail를 핸들링할겁니다.

swapchain은 render target의 collection입니다. swapchain의 기본 목적은 현재 렌더링할 이미지가 현재 화면에 보인 이미지와 다름을 보장하기 위함입니다. 이것은 완전한 이미지만 보여주는것을 확신하기 위해 중요합니다. 매번 frame을 그리려고 할 때, swapchain에 렌더링할 이미지를 우리한데 제공해달라고 요청해야합니다. frame을 그리는것이 끝날 때, 그 이미지는 swapchain으로 반환되어서 어느 시점에 화면에 표현됩니다. render target과 화면에 이미지를 표시하는것을 끝내기위한 conditions들의 수는 present mode에 의존합니다. common present mode는 double buffering (vsync)과 triple buffering입니다. 이건 swapchain creation chapter에서 살펴봅니다.

몇 platforms은 어떠한 window manger의 상호작용 없이 [`VK_KHR_display`](https://www.khronos.org/registry/vulkan/specs/1.4-extensions/man/html/VK_KHR_display.html)과 [`VK_KHR_display_swapchain`](https://www.khronos.org/registry/vulkan/specs/1.4-extensions/man/html/VK_KHR_display_swapchain.html) extension들을 통해 디스프레이에 직접 렌더링을 할 수 있게 해줍니다. 이 extensions은 예를 들어 전체 스크린을 표현하는 surface를 생성할 수 있게 해주고 자신만의 window manager를 구현할 수 있게 해 줍니다.

### Step 4 - Image views and framebuffers

swapchain으로부터 얻은 이미지를 그리기 위해서, 그 이미지를 [`VkImageView`](https://www.khronos.org/registry/vulkan/specs/1.4-extensions/man/html/VkImageView.html)과 [`VkFramebuffer`](https://www.khronos.org/registry/vulkan/specs/1.4-extensions/man/html/VkFramebuffer.html)로 래핑해야합니다. image view는 사용될 이미지의 특정 부분을 참조합니다. 그리고 framebuffer는 color, depth 그리고 stencil target을 위해 사용될 image view를 참조합니다. swapchain안에는 서로다른 많은 이미지가 있기때문에, 미리 각 이미지에 대한 image view와 framebuffer를 만들어두고 draw time에 적절한 것을 선택해야합니다.

### Step 5 - Render passes

Vulkan에서 render passes는 렌더링 연산동안 사용될 이미지의 타입을 설명합니다. 이미지가 어떻게 사용될지, 이미지의 contents가 어떻게 취급될지 설명해야합니다. 우리의 첫 삼각형 렌더링 애플리케이션에서는, single image를 color target로 사용하고 drawing 연산이 끝난직후 solid color로 clear될것을 Vulkan에게 말해줄것입니다. render pass가 오직 이미지의 타입만을 설명하는 반면\, [`VkFramebuffer`](https://www.khronos.org/registry/vulkan/specs/1.4-extensions/man/html/VkFramebuffer.html)는 실제로 특정 이미지를 이러한 slots에 바인딩합니다.

### Step 6 - Graphics pipeline

Vulkan에서 graphics pipeline은 [`VkPipeline`](https://www.khronos.org/registry/vulkan/specs/1.4-extensions/man/html/VkPipeline.html) 오브젝트를 생성함으로써 세팅됩니다. 이 오브젝트는 viewport size, buffer operation 그리고 [`VkShaderModule`](https://www.khronos.org/registry/vulkan/specs/1.4-extensions/man/html/VkShaderModule.html)를 사용하는 programmable state와 같은 그래픽카드의 구성가능한 state를 설명합니다. [`VkShaderModule`](https://www.khronos.org/registry/vulkan/specs/1.4-extensions/man/html/VkShaderModule.html) 오브젝트는 shader byte code로부터 생성됩니다. driver 또한 어떤 render targets이 pipeline에서 사용될 지 알아야합니다. 그리고 이것은 render pass를 참조시킴으로써 지정합니다.

기존의 APIs와 비교하여 가장 독특한 features중 하나는 graphics pipeline의 거의 모든 configuration들이 사전에 설정되어야 하는 것입니다. 이 특징은 만약 다른 shader로 변경하거나 vertex layout에 약간의 변경을 하고싶다면, graphics pipeline 전체를 재생성해야 하는것을 의미합니다. 이것은 rendering operations을 위해 필요한 서로다른 모든 조합에 대해 미리 수많은 [`VkPipeline`](https://www.khronos.org/registry/vulkan/specs/1.4-extensions/man/html/VkPipeline.html) 오브젝트를 만들어야하는것도 의미합니다. viewport size와 clear color같은 몇가지 단순한 configuration만 동적으로 변경될 수 있습니다. 모든 state는 또한 명시적으로 설명되어야합니다. 예를 들어 default color blend state는 없습니다.

좋은 소식은 just-in-time compilation대신 ahead-of-time compilation에 해당하는 방식을 사용하고 있기 때문에, driver에 대한 더 많은 최적화 기회가 있으며 runtime성능도 더 예측 가능해진다는 것입니다. 왜냐하면, 서로 다른 graphics pipeline으로 전환하는 것과 같은 큰 state변화가 매우 명시적으로 이루어지기 때문입니다.

### Step 7 - Command pools and command buffers

이전에 언급했듯이, drawing operation같은 Vulkan의 실행하고 싶은 연산들의 대부분은 queue에 제출되어야합니다. 이러한 연산들은 제출되기 전에 [`VkCommandBuffer`](https://www.khronos.org/registry/vulkan/specs/1.4-extensions/man/html/VkCommandBuffer.html)에 먼저 기록되어야합니다. 이러한 command buffer들은 특정 queue family와 연관된 [`VkCommandPool`](https://www.khronos.org/registry/vulkan/specs/1.4-extensions/man/html/VkCommandPool.html)로부터 할당됩니다. 단순한 삼각형을 그리기 위해서, 다음과 같은 연산을 통해 command buffer를 기록해야합니다.

- render pass 시작
- graphics pipeline 바인딩
- 3개의 vertex 그리기
- render pass 종료

framebuffer의 이미지는 swapchain이 우리에게 줄 특정 이미지에 의존하기 때문에, 각 가능한 이미지들을 위해 command buffer를 기록하고 draw time에 적절한것을 선택해야합니다. 대안은 모든 frame마다 command buffer를 다시 그리는것인데, 효율적이지 않습니다.

### Step 8 - Main loop

drawing commands가 command buffer로 래핑되어졌으므로, main loop는 꽤 직관적입니다. 먼저 [`vkAcquireNextImageKHR`](https://www.khronos.org/registry/vulkan/specs/1.4-extensions/man/html/vkAcquireNextImageKHR.html)를 이용하여 swapchain으로부터 이미지를 얻어옵니다. 그러면 그 이미지를 위한 적절한 command buffer를 선택할 수 있고 [`vkQueueSubmit`](https://www.khronos.org/registry/vulkan/specs/1.4-extensions/man/html/vkQueueSubmit.html)를 이용하여 실행할 수 있습니다. 마지막으로 화면에 프레젠테이션을 위하여 [`vkQueuePresentKHR`](https://www.khronos.org/registry/vulkan/specs/1.4-extensions/man/html/vkQueuePresentKHR.html)를 이용해 swapchain에 반환합니다.

queues에 제출된 operations은 비동기적으로 실행됩니다. 그러므로 semaphores같은 synchronization 오브젝트를 사용해서 올바른 실행 순서를 보장해야합니다. draw command buffer의 실행은 이미지 습득이 끝날때까지 기다리도록 설정되어야 합니다. 그렇지 않으면 화면에 프레젠테이션을 위해 아직 읽고 있는 이미지에 렌더링을 발생시킬 수도 있습니다. [`vkQueuePresentKHR`](https://www.khronos.org/registry/vulkan/specs/1.4-extensions/man/html/vkQueuePresentKHR.html) call은 렌더링이 끝날때까지 기다려야 하며, 이를 위해, 렌더링이 끝난 후 시그널될 두 번쨰 세마포어를 사용할겁니다.

### Summary

이 정신없는 여행은 첫 번째 삼각형을 그리기 위해 앞으로 해야할 일들의 기본적인 이해를 줍니다. real-world 프로그램은 vertex buffers 할당, uniform buffers 생성, 이후 챕터에서 커버될 texture images 업로드같은 더 많은 스텝을 포함합니다. 그러나 Vulkan은 이미 충분한 가파른 학습 곡선을 갖고있으므로, 간단하게 시작할겁니다. 우리는 처음에 vertex coordinate를 vertex buffer대신 vertex shader에 포함시키는 치트를 할 것에 주목하세요. 왜냐하면 vertex buffers를 관리하는것은 먼저 command buffer에 익숙해지는것을 요구하기 때문입니다.

그래서 요약하자면, 첫 삼각형을 그리기위해 필요한것들은 다음과 같습니다.

- [`VkInstance`](https://www.khronos.org/registry/vulkan/specs/1.4-extensions/man/html/VkInstance.html) 생성
- 지원되는 그래픽카드 선택 ([`VkPhysicalDevice`](https://www.khronos.org/registry/vulkan/specs/1.4-extensions/man/html/VkPhysicalDevice.html))
- drawing과 presentation을 위한 [`VkDevice`](https://www.khronos.org/registry/vulkan/specs/1.4-extensions/man/html/VkDevice.html)과 [`VkQueue`](https://www.khronos.org/registry/vulkan/specs/1.4-extensions/man/html/VkQueue.html) 생성
- window, window surface 그리고 swapchain 생성
- swapchain images를 [`VkImageView`](https://www.khronos.org/registry/vulkan/specs/1.4-extensions/man/html/VkImageView.html)로 래핑
- render target과 usage를 지정하는 render pass 생성
- render pass를 위한 framebuffers 생성
- graphics pipeline 설정
- 모든 가능한 swapchain image에 대한 draw commands를 사용하여 command buffer를 할당하고 기록
- 적절한 draw command buffer를 제출하고 images를 swapchain에 반환하여 얻어진 이미지로 frames을 그리기

많은 단계지만, 각 개별 스텝의 목적은 이후 챕터에서 매우 단순하고 명확해질겁니다. 만약 전체 프로그램과 비교하여 단일 단계의 관계에 대해 혼란스럽다면, 이 챕터를 다시 참조하세요

## API concepts

Vulkan API는 C programming language의 관점에서 정의되었습니다. Vulkan API의 canonical version은 Vulkan API Registry에서 정의되었고 이것은 [XML 파일](https://github.com/KhronosGroup/Vulkan-Docs/blob/main/xml/vk.xml)입니다. 이 파일은 Vulkan API을 machine readable definition의 역할을 합니다.

이후 챕터에서 설치할 Vulkan SDK의 일부인 [Vulkan headers](https://github.com/KhronosGroup/Vulkan-Headers)은 Vulkan API Registry로부터 생성됩니다. 그러나, 이 headers를 직/간접적으로 사용하지는 않을겁니다. 왜냐하면 `vulkanlia`가 Vulkan API registry로부터 생성된 Vulkan API에 대한 interface를 포함하기 때문입니다. 이 interface는 Vulkan SDK에 의해 제공된 C interface와는 독립되어있습니다.

`vulkanlia`의 foundation은 [`vulkanalia-sys`](https://docs.rs/vulkanalia-sys) 크레이트입니다. 이것은 Vulkan API Registry에 의해 정의된 raw types (commands, enums, bitmasks, structs, etc.)를 정의합니다. 이러한 raw types은 Vulkan API Registry로부터 생성된 다른 items과 마찬가지로 [`vk`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/index.html)모듈의 `vulkanlia` crate로부터 re-exported됩니다. 그리고 이 raw types은 introduction에서 이전에 언급한 Vulkan API를 둘러싸는 얇은 래퍼의 역할을 합니다.

### Type Names

Rust는 C와 다르게 namespaces에 대한 지원이 있기때문에, `vulkanlia` API는 C에서 namespacing 목적으로 사용되던 Vulkan type names의 일부를 생략합니다. 더 구체적으로, structs, unions 그리고 enums과 같은 Vulkan types은 그들의 `Vk` prefix를 잃습니다. 예를 들어, [`VkInstanceCreateInfo`](https://www.khronos.org/registry/vulkan/specs/1.4-extensions/man/html/VkInstanceCreateInfo.html) 구조체는 `vulknalia`에서 [`InstanceCreateInfo`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.InstanceCreateInfo.html)가 되고, 이전에 언급한 [`vk`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/index.html) 모듈에서 찾을 수 있습니다.

앞으로 이 튜토리얼은 `vk::` module prefix를 사용하여 `vulkanlia`에 의해 정의된 Vulkan types을 가리키도록 하여 Vulkan API Registry로부터 생성된 무언가를 가리키는것을 명확하게 할겁니다.

이러한 type names은 각각 referenced type에 대한 `vulkanlia` documentation과 연결됩니다. Vulkan types을 위한 `vulkanlia` documentation은 또한 그 type의 목적이나 사용에 관해 더 알아보기위해 사용할수 있는 타입에 대한 Vulkan specification으로의 링크를 포함합니다.

몇가지 type name 예시입니다.

- [`vk::Instance`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.Instance.html)
- [`vk::InstanceCreateInfo`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.InstanceCreateInfo.html)
- [`vk::InstanceCreateFlags`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.InstanceCreateFlags.html)

## Enums

`vulkanlia`은 Vulkan enums을 structs로 모델링합니다,. 그리고 variants를 이러한 구조체에 대한 연관된 상수로 모델링합니다. Rust enums은 Vulkan enums을 위해 사용되지 않습니다. 왜냐하면 FFI에서 Rust enums의 사용은 [undefined behavior](https://github.com/rust-lang/rust/issues/36927)로 이어질 수 있기 때문입니다.

연관된 상수들이 구조체에 대해 namespace되므로, C에서처럼 서로다른 Vulkan enums (또는 다른 라이브러리의 enums) 의 값 사이세어의 충돌을 걱정할 필요가 없습니다. 따라서 type names과 마찬가지로, `vulkanlia`는 namespacing 목적으로 쓰이던 variant names들의 일부를 생략합니다.

예를 들어, `VK_OBJECT_TYPE_INSTANCE` variant는 [`VkObjectType`](https://www.khronos.org/registry/vulkan/specs/1.4-extensions/man/html/VkObjectType.html) enum에 대한 `INSTANCE` 입니다. `vulkanalia`에서 이 variant는 [`vk::ObjectType::INSTANCE`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.ObjectType.html#associatedconstant.INSTANCE)가 됩니다.

## Bitmasks

`vulkanlia`는 Vulkan bitmasks를 structs로 모델링하고, bitflags를 이러한 structs과 연관된 상수로 모델링합니다. 이러한 구조체 그리고 연관 상수들은 [`bitflags`](https://github.com/bitflags/bitflags) crate의 `bitflags!` 매크로를 통해 생성됩니다.

variants처럼, namespacing 목적으로 사용된 bitmask names의 일부는 생략됩니다.

예를 들어, `VK_BUFFER_USAGE_TRANSFER_SRC_BIT` bitflag는 [`VkBufferUsageFlags`](https://www.khronos.org/registry/vulkan/specs/1.4-extensions/man/html/VkBufferUsageFlags.html) bitmask에 대한 `TRANSFER_SRC`bitflag입니다. `vulkanlia`에서 이것은 [`vk::BufferUsageFlags::TRANSFER_SRC`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.BufferUsageFlags.html#associatedconstant.TRANSFER_SRC)가 됩니다.

## Commands

[`vkCreateInstance`](https://www.khronos.org/registry/vulkan/specs/1.4-extensions/man/html/vkCreateInstance.html)같은 raw Vulkan commands를 위한 타입은 `vulkanaila`에서 `PFN_` (pointer to function) prefix가 있는 function pointer type aliases로 정의됩니다. 따라서 [`vkCreateInstance`](https://www.khronos.org/registry/vulkan/specs/1.4-extensions/man/html/vkCreateInstance.html)에 대한 `vulkanalia` type alias는 [`vk::PFN_vkCreateInstance`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/type.PFN_vkCreateInstance.html)입니다.

이러한 function pointer types은 그것만으로는 Vulkan commands를 호출하기에 충분하지 않습니다. 먼저 이러한 타입들로 설명된 commands를 로드해야합니다. Vulkan specification에는 어떻게 이 과정을 완료하는지에 대한 [detailed description](https://www.khronos.org/registry/vulkan/specs/1.2-extensions/html/vkspec.html#initialization-functionpointers)이 있습니다. 그러나 여기서는 simplified version을 보여줄것입니다.

처음으로 로드해야 할 Vulkan command는 [`vkGetInstanceProcAddr`](https://www.khronos.org/registry/vulkan/specs/1.4-extensions/man/html/vkGetInstanceProcAddr.html)입니다. 이 command는 platform-specific한 방법으로 로드되지만, `vulkanalia`는 [`libloading`](https://crates.io/crates/libloading)과의 optional integration을 제공합니다. 이 튜토리얼에서는 Vulkan shared library에서 이 command를 로드하기 위해 사용할겁니다. [`vkGetInstanceProcAddr`](https://www.khronos.org/registry/vulkan/specs/1.4-extensions/man/html/vkGetInstanceProcAddr.html)는 그러면 호출하고싶은 다른 Vulkan commands를 로드하기위해 사용할 수 있습니다.

그러나, 각자의 시스템에서 Vulkan 구현에 따른 이용가능한 다양한 버전의 Vulkan commands가 있습니다. 예를 들어, 만약 시스템이 dedicated NVIDIA GPU와 integrated Intel GPU를 갖고있다면, 각 디바이스를 위한 [`allocate_memory`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/trait.DeviceV1_0.html#method.allocate_memory)같은 구분된 device-specific Vulkan commands의 구현이 있을겁니다. 이 경우에는, [`vkGetInstanceProcAddr`](https://www.khronos.org/registry/vulkan/specs/1.4-extensions/man/html/vkGetInstanceProcAddr.html)가 사용중인 디바이스에 따라 적절한 device-specific command의 호출을 dispatch하는 command를 반환할겁니다.

이 dispatch의 runtime overhead를 피하기 위해서, [`vkGetDeviceProcAddr`](https://www.khronos.org/registry/vulkan/specs/1.4-extensions/man/html/vkGetDeviceProcAddr.html) command를 직접 device-specific Vulkan commands를 로드하도록 사용할 수 있습니다. 이 command는 [`vkGetInstanceProcAddr`](https://www.khronos.org/registry/vulkan/specs/1.4-extensions/man/html/vkGetInstanceProcAddr.html)처럼 같은 방식으로 로드될 수 있습니다.

이 튜토리얼에서 여러개의 Vulkan commands를 호출할겁니다. 운이 좋게도 commands를 수동으로 로드하지는 않을겁니다. `vulkanaila`는 4개의 카테고리중 하나의 Vulkan commands를 쉽게 로드할 수 있는 구조체를 제공합니다.

- [`vk::StaticCommands`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.StaticCommands.html) – 다른 commands를 로드하기 위해 사용될 수 있는 platform-specific한 방식으로 로드된 Vulkan commands (i.e., [`vkGetInstanceProcAddr`](https://www.khronos.org/registry/vulkan/specs/1.4-extensions/man/html/vkGetInstanceProcAddr.html) and [`vkGetDeviceProcAddr`](https://www.khronos.org/registry/vulkan/specs/1.4-extensions/man/html/vkGetDeviceProcAddr.html))
- [`vk::EntryCommands`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.EntryCommands.html) – [`vkGetInstanceProcAddr`](https://www.khronos.org/registry/vulkan/specs/1.4-extensions/man/html/vkGetInstanceProcAddr.html)과 null Vulkan instance를 사용하여 로드된 Vulkan commands. 이러한 commands는 특정 Vulkan instance에 묶이지 않고 instance support를 query하는것과 instances를 생성하기위해 사용됩니다.
- [`vk::InstanceCommands`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.InstanceCommands.html) – [`vkGetInstanceProcAddr`](https://www.khronos.org/registry/vulkan/specs/1.4-extensions/man/html/vkGetInstanceProcAddr.html)과 valid Vulkan instance를 사용하여 로드된 Vulkan commands. 이러한 commands는 특정 Vulkan instance에 묶여있고, 무엇보다도 device support를 query하고 devices를 생성하기 위해 사용됩니다.
- [`vk::DeviceCommands`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.DeviceCommands.html) – [`vkGetDeviceProcAddr`](https://www.khronos.org/registry/vulkan/specs/1.4-extensions/man/html/vkGetDeviceProcAddr.html)과 valid Vulkan instance를 사용하여 로드된 Vulkan commands. 이러한 commands는 특정 Vulkan device에 묶여있고 graphics APIP로부터 기대할 수 있는 대부분의 functionality를 노출시킵니다.

이러한 구조체들은 Rust에서 Vulkan commands를 쉽게 로드하고 호출할 수 있도록 해주지만, `vulkanalia`는 raw Vulkan commands에 대한 래퍼를 제공합니다. 이것은 Rust에서 commands를 호출하는 것을 쉽게 해주고 less error-prone가 되도록 합니다.

## Command wrappers

C에서 통상적인 Vulkan command signature의 예시는 다음과 같이 보입니다.

```C
VkResult vkEnumerateInstanceExtensionProperties(
    const char* pLayerName,
    uint32_t* pPropertyCount,
    VkExtensionProperties* pProperties
);
```

Vulkan API의 컨벤션과 익숙한 누군가는 이 시그니처가 중요한 정보를 포함하지 않았더라도, 이것만으로 이 command가 어떻게 사용될지 빠르게 알 수 있습니다.

Vulkan API에 처음인사람들을 위해서, 이 command를 위한 [documentation](https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkEnumerateInstanceExtensionProperties.html)를 살펴보면 더 잘 이해가 될겁니다. 문서에서 이 command의 동작에 대한 설명은 Vulkan instance에 대한 이용가능한 extensions을 리스팅하는 이 command를 사용하는 것은 multi-step 프로세스가 될 것임을 암시합니다.

1. extensions의 수를 얻기 위해 command 호출
2. outputted number of extensions을 포함할 수 있는 buffer 할당
3. extensions를 사용하여 buffer에 붙이기 위해 command를 다시 호출

따라서 C++에서는 아래처럼 보일겁니다 (간단함을 위해 command의 result는 무시하는 중입니다).

```C++
// 1.
uint32_t pPropertyCount;
vkEnumerateInstanceExtensionProperties(NULL, &pPropertyCount, NULL);

// 2.
std::vector<VkExtensionProperties> pProperties{pPropertyCount};

// 3.
vkEnumerateInstanceExtensionProperties(NULL, &pPropertyCount, pProperties.data());
```

[`vkEnumerateInstanceExtensionProperties`](https://www.khronos.org/registry/vulkan/specs/1.4-extensions/man/html/vkEnumerateInstanceExtensionProperties.html)를 위한 wrapper의 Rust signature는 다음과 같습니다.

```rust
unsafe fn enumerate_instance_extension_properties(
    &self,
    layer_name: Option<&[u8]>,
) -> VkResult<Vec<ExtensionProperties>>;
```

이 command wrapper는 Rust에서 [`vkEnumerateInstanceExtensionProperties`](https://www.khronos.org/registry/vulkan/specs/1.4-extensions/man/html/vkEnumerateInstanceExtensionProperties.html)의 사용을 쉽고, less error-prone, 그리고 다음과 같은 여러 방면에서 더 idiomatic하게 만들어줍니다.

- `layer_name` 파라미터의 optionality function signature로 인코드됩니다. 이 파라미터가 optional임은 C function signature에 캡쳐되지 않습니다. 이 정보에 대해서는 Vulkan specification을 확인해야합니다.
- command의 fallibility는 `Result`를 반환함으로써 모델링됩니다 ([`VkResult<T>`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/type.VkResult.html)는 `Result<T, vk::ErrorCode>`의 type alias입니다). 이것은 Rust의 강력한 error handling 능력의 이점을 갖을 뿐만 아니라 fallible command의 result를 방치하는지에 대해 컴파일러로부터 경고를 받을 수 있습니다.
- command wrapper는 위에 설명된 세 가지 프로세스를 내부적으로 핸들링하고 extension properties를 포함하는 `Vec`를 반환합니다.

command wrapper가 여전히 `unsafe`임을 주목하세요. 왜냐하면 `vulkanalia`가 특정 클래스의 오류를 지울 수 있지만 (e.g., ), 여전히 끔찌갛게 잘못될 수 있는 일이 많고 segfaults같은 재밌는 일이 발생하기 때문입니다. 언제든지 Vulkan document의 `Valid Usage` 섹션을 확인해서 command에 대해 해당 command를 올바르게 호출하기 위해 유지해야할 불변성을 확인할 수 있습니다.

아마 위의 command wrapper에서 `&self`를 눈치챘을수도 있습니다. 이러한 command wrappers는 `vulkanalia`에 의해 노출된 타입을 위해 구현한 traits에 정의되어있습니다. 이러한 traits는 두가지 카테고리로 구분됩니다. version traits와 extension traits입니다. version traits은 Vulkan의 standard part인 commands를 위한 command wrapper를 제공하는 반면, extension traits는 Vulkan extension의 부분으로 정의된 commands를 위한 command wrappers를 제공합니다.

예를 들어, [`enumerate_instance_extension_properties`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/trait.EntryV1_0.html#method.enumerate_instance_extension_properties)는 [`vk::EntryV1_0`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/trait.EntryV1_0.html) trait안에 있습니다. 왜냐하면 이것은 Vulkan 1.0의 일부인 non-extension Vulkan command이고 Vulkan instance또는 device에 의존하지 않기 때문입니다. Vulkan 1.2에서 추가되었고 Vulkan device에 의존하는 [`cmd_draw_indirect_count`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/trait.DeviceV1_2.html#method.cmd_draw_indirect_count)같은 Vulkan command는 [`vk::DeviceV1_2`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/trait.DeviceV1_2.html) trait안에 있습니다.

이러한 version과 extension traits은 loaded commands와 요구된 Vulkan instance또는 device (if any) 모두를 포함하는 types을 위해 정의되었습니다. 이러한 types은 사랑스럼게 수작업으로 만들어졌고 `vulkanalia`의 `vk` module안에 생성된 Vulkan bindings의 일부가 아닙니다. 이 타입들은 이후 챕터에서 사용될거고 [`Entry`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/struct.Entry.html), [`Instance`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/struct.Instance.html) 그리고 [`Device`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/struct.Device.html) 구조체들입니다.

앞으로, 이 튜토리얼은 이 섹션에서처럼 (e.g. [`create_instance`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/trait.EntryV1_0.html#method.create_instance)) command wrappers를 이름으로 직접 참조하면서 진행할겁니다. command wrapper가 정의된 trait같은 더 많은 정보를 위해 command wrapper에 대한 `vulkanalia` documentation를 방문할 수 있습니다.

## Builders

Vulkan API는 Vulkan commands를 위해 심하게 structs를 parameters로 utilize합니다. command parameters로 사용되는 Vulkan structs는 구조체의 타입을 가리키는 필드를 갖고 있습니다. C API에서는, 이 필드 (`sType`)가 명시적으로 설정되어야합니다. 예를 들어, 여기 [`VkInstanceCreateInfo`](https://www.khronos.org/registry/vulkan/specs/1.4-extensions/man/html/VkInstanceCreateInfo.html)의 instance를 이동시키고 C++이것을 사용해 [`vkCreateInstance`](https://www.khronos.org/registry/vulkan/specs/1.4-extensions/man/html/vkCreateInstance.html)를 호출하는것을 보여줍니다.

```C++
std::vector<const char*> extensions{/* 3 extension names */};

VkInstanceCreateInfo info;
info.sType = VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO;
info.enabledExtensionCount = static_cast<uint32_t>(extensions.size());
info.ppEnabledExtensionNames = extensions.data();

VkInstance instance;
vkCreateInstance(&info, NULL, &instance);
```

여전히 `vulkanalia`를 쓸 때도 이러한 방식으로 parameter를 이동시킬 수 있지만, `vulkanalia`는 이러한 parameter structs를 생성을 간소화하는 builder를 제공합니다. 이 builder를 사용하면, 위의 코드는 다음과 같이 됩니다.

```rust
let extensions = &[/* 3 extension names */];

let info = vk::InstanceCreateInfo::builder()
    .enabled_extension_names(extensions)
    .build();

let instance = entry.create_instance(&info, None).unwrap();
```

다음과 같은 차이점을 주목하세요

- `s_type` 필드에 대한 값이 제공되지 않습니다. 이는 builder 이 필드 ([`vk::StructureType::INSTANCE_CREATE_INFO`](https://docs.rs/vulkanalia/0.26.0/vulkanalia/vk/struct.StructureType.html#associatedconstant.INSTANCE_CREATE_INFO)) 에 대한 올바른 값을 자동으로 제공하기 떄문입니다.
- `enabled_extensions_count` 필드에 대한 값이 제공되지 않습니다. 이는 `enabled_extension_names` builder method가 이 필드를 설정하기 위해 제공된 slice의 길이를 자동으로 사용하기 때문입니다.

그러나, 위의 Rust 코드는 어느정도 위험을 수반합니다. builders는 lifetimes을 갖고 이 lifetime는 builders안에 저장된 references가 빌더 자체만큼 오래 살도록  강제합니다. 위의 예시에서는, 이것이 Rust compiler가 `enabled_extension_names`로 넘겨진 slice가 builder만큼 오래 살아있는것을 확신해 줄것을 의미합니다. 그러나, 기본 [`vk::InstanceCreateInfo`](https://docs.rs/vulkanalia/0.27.0/vulkanalia/vk/struct.InstanceCreateInfo.html) 구조체를 얻기 위해 `.build()`를 호출하자마자 builder의 lifetime은 폐기됩니다. 이것은 Rust compiler가 더 이상 존재하지 않는 slice에 대해 역참조하려고 할때 우리의 발에 총을 쏘려는 것을 막아주지 못한다는 것을 의미합니다.

다음의 코드는 (바라건데) 크래시가 일어납니다. 왜냐하면 `enabled_extension_names`로 넘어간 임시 `Vec`가 [`vk::InstanceCreateInfo`](https://docs.rs/vulkanalia/0.27.0/vulkanalia/vk/struct.InstanceCreateInfo.html) 구조체를 이용하여 [`create_instance`](https://docs.rs/vulkanalia/0.27.0/vulkanalia/vk/trait.EntryV1_0.html#method.create_instance)를 호출할 때 drop되기 때문입니다.

```rust
let info = vk::InstanceCreateInfo::builder()
    .enabled_extension_names(&vec![/* 3 extension names */])
    .build();

let instance = entry.create_instance(&info, None).unwrap();
```

운이좋게도, `vulkanalia`는 이에 대한 솔루션이 있습니다. 간단하게 `build()`를 호출하지 말고 대신 builder를 command wrapper에 넘기세요! command wrapper안에서 Vulkan 구조체가 기대되는 어디서든, 관련된 builder를 대신 제공해줄 수 있습니다. 위의 코드에서 `build()`를 지운다면 Rust compiler는 builder의 lifetimes을 이용여 이 bad code를 `error[E0716]: temporary value dropped while borrowed`로 거부할수 있습니다.

## Preludes

`vulkanalia`는 [prelude modules](https://docs.rs/vulkanalia/0.27.0/vulkanalia/prelude/index.html)를 제공합니다. 이것은 이 crate를 사용하기 위핸 필요한 기본 types을 노출시킵니다. 한개의 prelude module는 각 Vulkan version마다 이용가능하고 각각은 관련 command traits과 함께 다른 자주 사용되는 타입들을 제공합니다.

```rust
// Vulkan 1.0
use vulkanalia::prelude::v1_0::*;

// Vulkan 1.1
use vulkanalia::prelude::v1_1::*;

// Vulkan 1.2
use vulkanalia::prelude::v1_2::*;
```

## Validation layers

이전에 언급했듯이, Vulkan은 high performance와 low driver overhead를 위해 디자인되었습니다. 그래서 Vulkan은 기본적으로 매우 제한된 error checking과 debugging capabilities를 포함합니다. driver는 종종 뭔가 잘못하면 error code를 반환하는 대신 crash가 일어납니다. 또는 더 안좋은 경우 작업이 당신의 그래픽카드에서는 작동하지만 다른 그래픽카드에서는 완전히 실패합니다.

Vulkan은 *validation layers*로 알려진 feature를 사용하여 광범위한 검새를 활성화 할 수 있습니다. Validation layers는 API와 graphics driver사이에 삽입되는 pieces of code입니다. 함수 파라미터에 대한 추가적인 검사와 메모리 관리 문제 추적같은 일을 합니다. 좋은 점은, 개발동안에만 활성화하고 애플리케이션을 배포시에는 zero overhead를 위해 완전히 비활성화할 수 있습니다. 누구든지 자신만의 validation layers를 작성할 수 있지만, LunarG의 Vulkan SDK는 표준 validation layers를 제공합니다. 이 튜토리얼에서는 이것을 사용할겁니다. 또한 이 layers로부터 debug message를 받기 위해 callback function을 register해야합니다.

Vulkan은 모든 연산에 꽤 명시적이고 validation layers는 꽤 광범위하기 때문에, OpenGL과 Direct3D와 비교하여 왜 화면이 검정색인지 찾기에는 실제로 더 쉬울겁니다.
