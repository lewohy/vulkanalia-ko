# Cargo project

먼저, 우리의 Cargo project를 만듭니다.

```sh
cargo new vulkan-tutorial
```

명령이 실행되면 `vulkan-tutorial`이라는 폴더가 생성됩니다. 여기에는 Rust 실행파일을 생성하는 minimal Cargo project가 있습니다.

폴더 안의 `Cargo.toml`파일을 열어서 `[dependencies]`부분에 dependencies를 추가합니다

```toml
anyhow = "1"
log = "0.4"
cgmath = "0.18"
png = "0.17"
pretty_env_logger = "0.5"
thiserror = "1"
tobj = { version = "3", features = ["log"] }
vulkanalia = { version = "=0.26.0", features = ["libloading", "provisional", "window"] }
winit = "0.29"
```

- `anyhow` – 간단한 error handling을 위해 사용됩니다.
- `log` – logging statements를 위해 사용됩니다.
- `cgmath` – [GLM](https://glm.g-truc.net/0.9.9/index.html)(graphics math library)를 위한 러스트 대체제로 사용됩니다.
- `png` – 텍스쳐로 사용하기 위한 PNG를 로딩하기 위해 사용됩니다.
- `pretty_env_logger` – 로그를 콘솔에 출력하기 위해 사용됩니다.
- `thiserror` – boilerplate없이 custom error 타입을 정의하기 위해 사용됩니다.
- `tobj` – 3D모델을 [Wavefront .obj 포맷](https://en.wikipedia.org/wiki/Wavefront_.obj_file)으로 로딩하기위해 사용됩니다.
- `vulkanalia` – Vulkan API를 호출하기 위해 사용됩니다.
- `winit` – used to create a window to render to



## Vulkan SDK

### Windows

TODO

### Linux

> <https://kylemayes.github.io/vulkanalia/development_environment.html#linux> 에는 Ubuntu유저를 위한 설명에 맞춰져 있습니다.

Linux에서 Vulkan 애플리케이션을 개발하기위해 필요한것중 가장 중요한 컴포넌트는 Vulkan loader, validation layer 그리고 자신의 기기가 Vulkan-capable한지 테스트하기 위한 몇가지 command-line 유틸리티입니디

#### Arch

> <https://wiki.archlinux.org/title/Vulkan> 를 참고합니다.

- `vulkan-tools` - 가장 중요한 `vulkaninfo`와 `vkcube`가 있는Command-line 유틸리티들입니다. 이것들을 실행해서 자신의 기기가 Vulkan을 지원하는지 테스트합니다.
- `libvulkan-dev` - Vulkan loader를 설치합니다. 이 loader는 런타입에 driver에서 함수들을 찾습니다. OpenGL의 GLEW와 비슷합니다.
- `vulkan-validationlayers-dev` - standard validation layers를 설치합니다. 이것들은 Vulkan 애플리케이션을 디버깅하는데 중요합니다. 그리고 우리는 이후 챕터에서 다시 다룹니다.

설치가 성공적이라면, you should be all set with the Vulkan portion.
`vkcube`를 실행해서 윈도우가 뜨는지 확인합니다.

![vkcube](https://kylemayes.github.io/vulkanalia/images/cube_demo_nowindow.png)

If you receive an error message then ensure that your drivers are up-to-date, include the Vulkan runtime and that your graphics card is supported. See the introduction chapter for links to drivers from the major vendors.

