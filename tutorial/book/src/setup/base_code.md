# Base code

**Code:** [main.rs](https://github.com/KyleMayes/vulkanalia/tree/master/tutorial/src/00_base_code.rs)

`Development environment`에서 Cargo project를 만들었고, 필수적인 dependencies를 추가했습니다. 이 챕터에서는 `src/main.rs`안의 코드를 다음 코드로 바꿉니다.

```rust
#![allow(
    dead_code,
    unused_variables,
    clippy::too_many_arguments,
    clippy::unnecessary_wraps
)]

use anyhow::Result;
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

fn main() -> Result<()> {
    pretty_env_logger::init();

    // Window

    let event_loop = EventLoop::new()?;
    let window = WindowBuilder::new()
        .with_title("Vulkan Tutorial (Rust)")
        .with_inner_size(LogicalSize::new(1024, 768))
        .build(&event_loop)?;

    // App

    let mut app = unsafe { App::create(&window)? };
    event_loop.run(move |event, elwt| {
        match event {
            // Request a redraw when all events were processed.
            Event::AboutToWait => window.request_redraw(),
            Event::WindowEvent { event, .. } => match event {
                // Render a frame if our Vulkan app is not being destroyed.
                WindowEvent::RedrawRequested if !elwt.exiting() => unsafe { app.render(&window) }.unwrap(),
                // Destroy our Vulkan app.
                WindowEvent::CloseRequested => {
                    elwt.exit();
                    unsafe { app.destroy(); }
                }
                _ => {}
            }
            _ => {}
        }
    })?;

    Ok(())
}

/// Our Vulkan app.
#[derive(Clone, Debug)]
struct App {}

impl App {
    /// Creates our Vulkan app.
    unsafe fn create(window: &Window) -> Result<Self> {
        Ok(Self {})
    }

    /// Renders a frame for our Vulkan app.
    unsafe fn render(&mut self, window: &Window) -> Result<()> {
        Ok(())
    }

    /// Destroys our Vulkan app.
    unsafe fn destroy(&mut self) {}
}

/// The Vulkan handles and associated properties used by our Vulkan app.
#[derive(Clone, Debug, Default)]
struct AppData {}
```

먼저 `anyhow::Result`를 임포트해서 `anyhow`의 [Result](https://docs.rs/anyhow/latest/anyhow/type.Result.html)타입을 실패가능한 함수에 쓸 수 있도록 합니다. 그리고 window를 생성하고 그 window를 위한 event loop를 시작시키기 위해 필요한 `winit`의 모든 타입을 임포트합니다.

다음으로 `main`함수(`anyhow::Result`를 리턴합니다)로 갑니다. 이 함수는 로그를 콘솔에 출력하는 `pretty_env_logger`(이후에 설명될 것처럼)를 초기화하면서 시작합니다.

그러면, `winit`를 이용해 event loop와 렌더링할 window를 생성하고, `LogicalSize`를 통해 디스플레이의 DPI에 맞춰 창 크기를 조정합니다. UI 스케일링에 대해 더 알고싶다면 [`winit` 문서](https://docs.rs/winit/latest/winit/dpi/index.html)를 참고합니다.

다음으로 Vulkan app의 인스턴스를 만들고 rendering loop로 들어갑니다. 이 루프는 app이 파괴되고 프로그램이 종료되는, window를 닫는 요청을 할 때 까지 window에 scene을 계속 렌더링합니다. `destroying`플래그는 파괴된 Vulkan 리소스에 접근하는 시도 이후에 프로그램 crash를 만들어내는 가능성이 높은, app이 종료되는 동안 scene에 렌더링하는것을 방지하기 위해 필수적입니다.

마지막으로 `App`과 `AppData`로 옵니다. `App`은 따라오는 챕터의 코스를 빌드할 Vulkan프로그램을 위해 요구되는 setup, rendering 그리고 destruction 로직을 구현하는데 사용됩니다. `AppData`는 단순히 우리가 생성하고 초기화할 다수의 Vulkan 리소스 컨테이너 역할을 합니다. `AppData`는 Vulkan 리소스들이 쉽게 함수로 전달되어 읽거나 수정될 수 있도록 합니다. `AppData`는 [Default trait](https://doc.rust-lang.org/std/default/trait.Default.html)를 구현하므로, 쉽게 비어있거나 기본값으로 구조체의 인스턴스를 생성할 수 있습니다.

`&mut AppData`를 갖거나 Vulkan 리소스를 생성하고 초기화하는 함수들을 추가하는것으로 구성된 챕터들이 많기때문에, `AppData`는 유용합니다. 이 함수들은 Vulkan app을 set up하기 위해 `App:create` 생성자에서 호출됩니다. 그러면 프로그램이 종료되기 전에, Vulkan 리소스들은 `App:destroy`메소드에 의해 해제됩니다.

## A Note on Safety

모든 Vulkan command(raw command와 command wrapper)들은 `vulkanalia`에서 `unsafe`로 표시됩니다. 이것은 거의 Vulkan command들이 Rust에 의해 강제되는 방식으로 호출될 수 없기 때문입니다().(unless a higher-level interface that hides the Vulkan API is provided like in [`vulkano`](https://vulkano.rs/))

이 튜토리얼에서는 `unsafe`로 Vulkan command를 호출하는 모든 함수와 메소드를 마킹하는 방식으로 설명합니다. 이런 방식은 문법적 잡음을 최소화하지만, 실제 프로그램에서는 호출하는 Vulkan command에 불변 조건을 강제하는 인터페이스를 만들고, 자체적으로 만든 안전한 인터페이스를 노출시키는것이 좋습니다.

## Resource management

`malloc`를 이용해 C에서 할당된 메모리의 각 청크들이 대응하는 `free` 호출을 필요로하듯이, 우리가 만들 모든 Vulkan 오브젝트들은 더이상 필요가 없을 때 명시적으로 파괴되어야합니다. Rust에서는 아마 `Rc`또는 `Arc`같은 스마트 포인터와 결합된  [RAII](https://en.wikipedia.org/wiki/Resource_Acquisition_Is_Initialization)를 사용하여 자동으로 자원 관리를 수행하는것이 가능합니다. 그러나, [https://vulkan-tutorial.com](https://vulkan-tutorial.com/)의 저자는 이 튜토리얼에서 Vulkan 오브젝트의 allocation과 deallocation에 대하여 명시적인것을 선택했고 저도 같은 접근법을 결정했습니다. 어쨌든, Vulkan의 niche는 실수를 피하는 모든 연산에 대해 명시적이게 되는것이므로 API가 어떻게 작동하는지 배우기위해 오브젝트들의 lifetime에대해 명시적이게되는것은 좋습니다.

이 튜토리얼을 따른 후에, Vulkan 오브젝트를 감싼 Rust 구조체를 작성함으로써 automatic resource management를 구현하거나 거기에 `Drop`를 구현하여 해제하는것이 가능합니다. RAII는 큰 Vulkan 프로그램을 위한 추천되는 모델이지만, 학습의 목적에서는, scenes의 뒤에서 무슨 일이 일어나는지 아는것이 항상 좋습니다.

Vulkan 오브젝트들은 `create_xxx`같은 commands로 직접 생성되거나 `allocate_xxx`같은 commands로 다른 오브젝트를 통해 할당됩니다. 한 오브젝트가 더이상 어디서도 쓰이지 않는다는것을 확신한 후에, 대응하는 `destroy_xxx` 그리고 `free_xxx`를 사용하여 오브젝트를 파괴해야합니다. 이 commands를 위한 파라미터들은 일반적으로 오브젝트의 타입마다 다양하지만, 모든것에 공통으로 공유하는 한가지 `allocator` 파라미터가 있습니다. 이것은 optional 파라미터이고 custom memory allocator를 위한 callbacks을 지정할 수 있도록 해줍니다. 우리는 튜토리얼에서 이 파라미터를 무시할거고, 항상 `None`를 매개변수로 넘겨줄것입니다.
