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
- 디스크: `background: rgba()` + `border-radius: 50%`
- 버튼 토글: `display: none` 기반, hover 시만 표시
- `clip-path: circle(50%)` 적용

## 변경별 근거

### 1. SVG `<text>` 제거
SVG text는 투명 WebKitGTK에서 숫자 glyph가 교체될 때 이전 raster가 남는 문제가 있었다. 숫자와 이름을 HTML span으로 옮겨, 텍스트 갱신을 SVG arc repaint와 분리했다.

### 2. `backdrop-filter` / `mask-image` 제거
blur + mask 조합은 Linux WebKitGTK에서 직사각형 damage region을 남기기 쉽다. 디스크는 단순 반투명 원형 배경으로 바꾸고, 원형 경계는 `.gauge-wrapper`의 `clip-path: circle(50%)`로 명시했다.

### 3. Pomodoro 버튼 숨김 방식 변경
`opacity: 0` / `visibility` 전환은 투명 창에서 이전 페인트가 남는 원인이 될 수 있다. 버튼은 `display: none`을 기본으로 두고 hover 시에만 표시하며, hover 해제 시에는 레이아웃 flush를 강제해 잔상을 줄인다.

### 4. `will-change` 적용 범위 제한
자주 갱신되는 레이어에만 `will-change`를 두고, 정적 요소에는 적용하지 않는다. 이렇게 해야 GPU 메모리 낭비를 줄이면서도 텍스트/버튼 합성 타이밍을 안정화할 수 있다.

## 알려진 제약
- `backdrop-filter` 제거로 블러 유리 효과는 포기한다. 이는 Linux WebKitGTK 호환성을 위한 의도된 트레이드오프다.
- `clip-path: circle(50%)`는 경계 밖 자식을 자른다. `.gauge-label`의 크기와 위치는 원형 경계 안에 들어가도록 잡아야 한다.
- `will-change` 남용은 GPU 메모리를 낭비한다. 이 문서에 적은 요소 외에는 임의로 추가하지 않는다.

## 검증 방법
변경 후 다음 세 항목을 직접 확인한다:
1. 숫자가 초마다 바뀔 때 이전 숫자가 보이지 않는다
2. Codex 위젯 hover 후 네모 잔상이 없다
3. Pomodoro hover 해제 시 버튼이 즉시 사라진다
