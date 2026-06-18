# Rendering Refactor — Token Dashboard

## 변경 배경
- 발생 환경: Tauri 2 / Linux X11 / WebKitGTK
- 문제 분류: 렌더링 합성 잔상, 기능 오류가 아님
- 해결 방향: 레이어 구조 재설계

## 변경 전 구조
- 숫자/레이블: SVG `<text>` 요소
- 디스크: `backdrop-filter` + `mask-image`
- 버튼 토글: `opacity` / `visibility`
- `clip-path`: 미적용

## 변경 후 구조
- 숫자/레이블: HTML `<span>` 요소, SVG 외부의 `.gauge-label` 안에서 갱신
- 디스크: `background: rgb()` + `border-radius: 50%`
- 버튼 토글: 고정 action row에서 제자리 갱신
- `clip-path: circle(50%)` 적용
- 대시보드 갱신: 고정 root에서 위젯 섹션만 추가/삭제/재정렬
- 창 구조: Claude / Codex / Pomodoro를 하나의 투명 창이 아니라 개별 투명 위젯 창으로 분리
- 배치/드래그: 세 위젯은 항상 하나의 anchored group으로 취급하고, 개별 분리 모드는 제공하지 않음
- Stale 표시: 디스크/숫자/arc를 opacity로 dim 하지 않고, update badge로만 stale 상태를 드러냄

## 변경별 근거

### 1. SVG `<text>` 제거
SVG text는 투명 WebKitGTK에서 숫자 glyph가 교체될 때 이전 raster가 남는 문제가 있었다. 숫자와 이름을 HTML span으로 옮겨, 텍스트 갱신을 SVG arc repaint와 분리했다.

### 2. `backdrop-filter` / `mask-image` 제거
blur + mask 조합은 Linux WebKitGTK에서 직사각형 damage region을 남기기 쉽다. 디스크는 불투명 원형 배경으로 바꾸고, 원형 경계는 `.gauge-wrapper`의 `clip-path: circle(50%)`로 명시했다.

### 3. Pomodoro 버튼 숨김 방식 변경
`opacity: 0` / `visibility` 전환은 투명 창에서 이전 페인트가 남는 원인이 될 수 있다. 버튼은 hover overlay가 아니라 Pomodoro 위젯 아래의 고정 action row로 유지하고, 버튼 노드는 교체하지 않고 label/class만 제자리에서 갱신한다.

### 4. `will-change` 적용 범위 제한
자주 갱신되는 레이어에만 `will-change`를 두고, 정적 요소에는 적용하지 않는다. 이렇게 해야 GPU 메모리 낭비를 줄이면서도 텍스트/버튼 합성 타이밍을 안정화할 수 있다.

### 5. 전창 remount clear plate 제거
설정 반영 때 `body::before`로 창 전체에 거의 투명한 사각형을 한 프레임 칠하는 방식은, Linux X11 투명 WebKitGTK에서 오히려 긴 직사각형 합성면을 드러낼 수 있었다. 이제 대시보드 root는 유지하고 위젯 섹션만 제자리에서 추가/삭제/재정렬해, 전체 창 단위 사각 repaint를 피한다.

### 6. Widget Window 분리
단일 투명 dashboard window 안에서 위젯 수가 바뀌면, X11/WebKitGTK가 제거된 형제 위젯의 alpha surface를 즉시 지우지 못하는 문제가 남았다. 최종 해법은 shape mask가 아니라, 각 provider를 독립 Tauri window로 분리해 “같은 top-level transparent surface 안에서 형제 위젯을 제거하는 상황” 자체를 없애는 것이다.

### 7. Grouped Layout 고정
개별 widget window는 X11 잔상을 줄이는 데 필요하지만, 사용자가 보기에는 하나의 dashboard row처럼 움직여야 한다. native `WindowEvent::Moved` 동기화는 위치 피드백 루프와 보이지 않는 창 문제를 만들었기 때문에, frontend pointer drag가 Rust `move_widget_windows` command를 호출하고 Rust가 anchor 기준으로 모든 enabled window를 재배치한다. 위젯 분리 모드는 제품 가치보다 회귀 위험이 커서 제거했다.

### 8. Stale 반투명 모드 제거
Stale 상태에서 숫자, 디스크, arc에 opacity/filter를 적용하면 Linux WebKitGTK 투명창에서 숫자 span의 rectangular backing layer가 남을 수 있었다. Stale 상태는 update badge로만 표시하고, 디스크는 `rgb(20, 20, 30)` 불투명 배경으로 고정한다.

## 알려진 제약
- `backdrop-filter` 제거로 블러 유리 효과는 포기한다. 이는 Linux WebKitGTK 호환성을 위한 의도된 트레이드오프다.
- `clip-path: circle(50%)`는 경계 밖 자식을 자른다. `.gauge-label`의 크기와 위치는 원형 경계 안에 들어가도록 잡아야 한다.
- `will-change` 남용은 GPU 메모리를 낭비한다. 이 문서에 적은 요소 외에는 임의로 추가하지 않는다.
- 위젯 창 분리 후에는 provider별 위치/크기 조정이 창 단위 책임이 된다. 즉, 이전처럼 한 dashboard DOM에서 형제 위젯 간 간격을 맞추는 문제가 아니라 각 창의 `position`과 `inner_size` 계약이 중요하다.
- 디스크 edge fade와 stale dimming은 포기한다. Linux X11/WebKitGTK의 투명 창에서는 alpha 기반 시각 효과보다 불투명 paint surface가 더 안정적이다.

## 검증 방법
변경 후 다음 세 항목을 직접 확인한다:
1. 숫자가 초마다 바뀔 때 이전 숫자가 보이지 않는다
2. Codex 위젯 hover 후 네모 잔상이 없다
3. Pomodoro hover 해제 시 버튼이 즉시 사라진다
4. 설정에서 위젯 표시 구성을 바꿔도 계기판 바깥에 긴 사각 불투명 블럭이 생기지 않는다
5. 설정에서 위젯 on/off를 반복해도 제거된 옛 게이지가 다른 현재 게이지와 같은 창 안에 겹쳐 남지 않는다
6. 세 위젯을 드래그하면 하나의 row처럼 부드럽게 함께 이동한다
7. 시간이 지나 stale 상태가 되어도 숫자 뒤에 사각 박스가 남지 않는다
