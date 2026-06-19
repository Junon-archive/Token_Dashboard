# SPEC.md — 데스크톱 사용량 게이지 위젯 (초안 v0.1)

> **문서 상태**: Draft. `[결정 필요]` 표식 항목은 미확정.
> **작성 기준일**: 2026-06-10
> **근거 자료**: poc-result-macos.md (2026-06-10, macOS 26.5.1 arm64), poc-result-ubuntu.md (2026-06-10, Ubuntu 22.04.5 X11), design-reference.html

---

## 1. 개요 및 목표 / 비목표

### 1.1 제품 요약

Claude Code(CC)와 Codex CLI의 5시간/7일 사용 한도를 자동차 계기판 메타포의 원형 게이지로 표시하는 **데스크톱 플로팅 위젯 앱**. 로컬 뽀모도로 타이머 위젯 포함. 위젯 3종(CC / Codex / Pomodoro)은 개별 on/off 가능하며, 투명·프레임 없는·항상 위 창으로 바탕화면에 떠 있는다.

- 게이지는 **잔량 표시**: 100% 남았을 때 링이 가득 차고, 0%에 가까울수록 줄어든다.
- 게이지 중앙에는 **리셋까지 남은 시간**(H:MM)을 표시한다.
- 대상 사용자: 제작자 본인 + 연구실 동료 + 외부 오픈소스 배포.

### 1.2 목표

1. CLI 토큰을 로컬에서 직접 읽어 공식 CLI가 쓰는 usage 엔드포인트만 호출하는 **로컬 에이전트** — 토큰은 머신 밖으로 나가지 않는다.
2. macOS(Apple Silicon) / Ubuntu(X11) 단일 코드베이스(Tauri 2).
3. 비공식(미문서화) API 변동에 대해 **크래시 없는 강등**(stale 상태)으로 견디는 견고성.
4. 뽀모도로는 네트워크/토큰 의존성 0으로, CC·Codex API가 전부 실패해도 정상 동작.

### 1.3 비목표 (Out of Scope, v1) — 과잉 구현 차단

다음은 **v1에서 구현하지 않는다**. PR/이슈에서 v1 범위로 들어오면 거부한다.

| 항목 | 처리 |
|---|---|
| 다중 계정 (CLI 프로필 여러 개) | 미지원. 기본 경로 1개만 |
| Windows 빌드 | 환경 검증 전까지 미지원 (확정) |
| iPhone/모바일 표시 | 개인용 부가 프로젝트로 분리. §12.1 부록 개요만 |
| 중앙 서버 / 계정 시스템 / 텔레메트리 | 영구 미지원 (아키텍처 결정 B-1) |
| 사용량 히스토리 그래프 / 시계열 저장 | 미지원. 현재 스냅숏만 표시 |
| 웹 스크래핑 기반 데이터 수집 | 금지 (아키텍처 결정 B-2) |
| Wayland 네이티브 지원 | X11만 검증. Wayland는 XWayland 동작 여부만 README에 기재 |
| 자동 업데이트 자가 설치 | macOS/Linux 모두 "알림만". 자가 교체 설치 없음 |

---

## 2. 검증 노트 (PoC 실측 요약 및 프롬프트 가정과의 충돌)

### 2.1 OS별 토큰 저장 위치 (실측)

| 항목 | macOS (26.5.1 arm64) | Ubuntu 22.04.5 (X11) |
|---|---|---|
| Claude 토큰 | **키체인 전용**. 서비스명 `"Claude Code-credentials"`. `~/.claude/.credentials.json` **존재하지 않음** | 파일 `~/.claude/.credentials.json`, 권한 `600` |
| Claude JSON 최상위 키 | `["claudeAiOauth", "trustedDeviceToken"]` | `["claudeAiOauth", "organizationUuid"]` |
| `claudeAiOauth` 하위 키 | `accessToken, expiresAt, rateLimitTier, refreshToken, scopes, subscriptionType` (양 OS 동일) | 동일 |
| Codex 토큰 | 파일 `~/.codex/auth.json` | 파일 `~/.codex/auth.json`, 권한 `600` |
| Codex 최상위 키 | `["OPENAI_API_KEY", "auth_mode", "last_refresh", "tokens"]` (양 OS 동일, `auth_mode: "chatgpt"`) | 동일 |
| Codex `tokens` 하위 키 | `access_token, account_id, id_token, refresh_token` (양 OS 동일) | 동일 |

### 2.2 API 응답 실측 스키마 요약

- **Claude** `GET https://api.anthropic.com/api/oauth/usage` → HTTP 200 (양 OS).
  - `five_hour.utilization`(%), `five_hour.resets_at`(ISO 8601, `+00:00` offset), `seven_day.*` 동일 구조.
  - `extra_usage`: `monthly_limit`(USD), `used_credits`, `utilization`(%), `currency`, `is_enabled`.
  - 모델별 필드(`seven_day_opus`, `seven_day_sonnet` 등)와 `tangelo`, `iguana_necktie`, `omelette_promotional`, `cinder_cove`는 실측 시점에 모두 `null` — **파서는 null 허용 + 미지(unknown) 필드 무시**여야 한다.
- **Codex**: PoC는 `npx codex-check --auth ~/.codex/auth.json --json` 래퍼로 검증. `baseUrl: https://chatgpt.com/backend-api`. 5h 창 = `windows.primary`(`label:"5h"`, `limit_window_seconds:18000`), 주간 창 = `windows.secondary`(`label:"weekly"`, `limit_window_seconds:604800`). 리셋 시각은 ISO `resetsAt`(`Z` suffix)과 raw epoch `reset_at` 양쪽 제공.

### 2.3 폴링 안정성 수치

- Ubuntu: 3분 간격 **7회 연속 HTTP 200, 429 0회** (단기 통과). **20회 완주는 미수행** — 장기 안정성은 미검증.
- macOS: 단일 호출 200 성공만 확인.

### 2.4 프롬프트 가정 ↔ PoC 충돌 및 미확정 사항 (중요)

1. **[충돌] macOS Claude 토큰은 파일이 아니라 키체인 전용.** 프롬프트 기본값의 "원본 파일(또는 macOS 키체인)" 문구 중 macOS는 **키체인 단일 경로**가 실측 사실. `security find-generic-password -s "Claude Code-credentials" -w` 또는 Security framework로 읽어야 하며, 파일 폴백 코드는 macOS에서 dead path다(다만 방어적으로 파일 경로도 2차 폴백으로 유지 — §8.1).
2. **[충돌] Claude 토큰 만료가 현실적으로 발생.** Ubuntu PoC에서 저장된 access token이 만료 상태였고(`expiresAt` 과거), CLI 재로그인 후에야 200을 받았다. "매 폴링 시 원본에서 읽으면 CLI 갱신을 자동으로 따라간다"는 가정은 **CLI가 최근에 실행된 경우에만** 성립한다. 앱 자체의 refresh 수행 여부는 [결정 필요] D-8 (§4.4)로 승격한다.
3. **[충돌 가능] 네트워크 허용 목적지 2곳 vs 토큰 refresh.** 아키텍처 결정 B-1은 목적지를 `api.anthropic.com`, `chatgpt.com` 백엔드 2곳으로 제한하지만, Codex refresh는 별도 인증 호스트(codex-check/codex-cli 구현상 OpenAI auth 서버)로 나갈 가능성이 높다. **PoC에서 refresh가 실제 발생하지 않아 미확인** (`lastRefresh` < `fetchedAt`, 갱신 없음). M1에서 codex-check/CodexBar 소스로 refresh 엔드포인트 호스트를 확정하고, 필요 시 허용 목록에 인증 호스트를 명시적으로 추가해야 한다. → 리스크 R-3.
4. **[미확정] Codex usage의 정확한 직접 호출 경로/헤더.** PoC는 codex-check 래퍼를 통했으므로 실제 HTTP 경로·필수 헤더는 미실측. v1 구현은 codex-check / codex-cli-usage / CodexBar 소스를 1차 레퍼런스로 직접 호출을 구현하되, **응답 스키마는 codex-check raw 필드(§4.3)를 기준으로 한다.** → M1 산출물에 "직접 호출 1회 실측 로그" 포함.
5. **[미확정] 폴링 장기 안정성.** 20회(1시간) 완주가 안 됐으므로, 기본 폴링 180초의 장기 무429 보장은 아직 "주장 불가". → M1 종료 조건에 포함.
6. **타임스탬프 형식 불일치 (실측).** Claude `resets_at`은 `+00:00` offset, Codex `resetsAt`은 `Z` suffix. 파서는 RFC 3339 양 표기를 모두 처리해야 한다 (Rust `chrono`/`time`의 RFC3339 파서는 양쪽 처리 가능).
7. **[충돌] 상태 머신 vs 디자인 레퍼런스 상태 집합.** 프롬프트 표는 7개 상태(NORMAL/WARN/CRITICAL/STALE/NOT_LOGGED_IN/AUTH_ERROR/RATE_LIMITED), 디자인 레퍼런스는 8개 비주얼(NORMAL/LOW/CRITICAL/**DEPLETED**/HOVER/STALE/NOT_LOGGED_IN/AUTH_ERROR)을 정의한다. 본 명세는 **논리 상태 7개를 유지**하고, `DEPLETED`(사용률 100%)는 CRITICAL의 비주얼 변형, `HOVER`는 상태와 무관한 인터랙션 레이어로 정리한다 (§5.3). 디자인 레퍼런스의 "LOW = 잔량 ≤ 20%"는 프롬프트의 "WARN = 사용률 ≥ 80%"와 동일 임계값이므로 충돌 아님(명칭만 통일: 논리명 WARN, 비주얼 클래스명 low).
8. **Linux WebKitGTK 렌더링 이슈 (실측).** 최초 실행 시 `Could not create GBM EGL display: EGL_NOT_INITIALIZED` 크래시. `WEBKIT_DISABLE_DMABUF_RENDERER=1` 적용 후 정상. 앱이 런처/데스크톱 엔트리에서 이 환경변수를 자동 주입해야 한다 (§8.2).
9. **Linux `alwaysOnTop`/`skipTaskbar`/드래그 이동은 육안 미검증.** 빌드·투명·decorations:false까지만 확인. → M2 검증 항목.
10. **macOS 투명창은 private API 필수 (실측).** `app.macOSPrivateApi: true`(주의: `app.windows[0]`가 아닌 **app 최상위**) + Cargo feature `macos-private-api` 없이는 배경이 검게 나온다.

---

## 3. 아키텍처

### 3.1 모듈 다이어그램 (텍스트)

```
┌────────────────────────────── Tauri App ──────────────────────────────┐
│                                                                        │
│  [Rust 코어]                                                           │
│  ┌──────────────────────────────────────────────┐                     │
│  │ UsageProvider (trait)                        │                     │
│  │  ├─ ClaudeProvider                           │                     │
│  │  │   token_source: Keychain(macOS) | File(Linux)                   │
│  │  │   fetch(): GET api.anthropic.com/api/oauth/usage                │
│  │  ├─ CodexProvider                            │                     │
│  │  │   token_source: ~/.codex/auth.json        │                     │
│  │  │   fetch(): chatgpt.com 백엔드 usage (+ refresh)                  │
│  │  └─ 공통: 폴링 스케줄러 / 캐시(메모리) / 백오프 / 상태 머신          │
│  └──────────────┬───────────────────────────────┘                     │
│                 │ Tauri event (snapshot: UsageSnapshot JSON)           │
│  ┌──────────────▼───────────────┐   ┌───────────────────────────┐     │
│  │ 위젯 렌더링 (웹/JS+SVG)       │   │ 뽀모도로 모듈 (웹, 독립)    │     │
│  │  CC 위젯 / Codex 위젯         │   │  타이머 로직 + 알림         │     │
│  └──────────────┬───────────────┘   └───────────┬───────────────┘     │
│                 │                                │                     │
│  ┌──────────────▼────────────────────────────────▼──────────────┐     │
│  │ 설정/영속화 (Rust): config.json 읽기/쓰기, 창 위치 저장,       │     │
│  │ 자동 시작 등록, 데스크톱 알림 발송                              │     │
│  └───────────────────────────────────────────────────────────────┘     │
└────────────────────────────────────────────────────────────────────────┘
   패키징/배포: GitHub Actions → .dmg(무서명) / .deb + AppImage
```

### 3.2 데이터 흐름

1. 폴링 틱(기본 180s) → Provider가 **매번 원본 저장소에서 토큰을 새로 읽음**(앱 내 토큰 영속 복사본 없음) → usage 엔드포인트 호출.
2. 응답 파싱 → 정규화된 `UsageSnapshot` 생성 → 상태 머신 평가 → Tauri 이벤트로 프론트엔드에 push.
3. 프론트엔드는 snapshot만 렌더링(데이터 호출 없음). 리셋 카운트다운은 snapshot의 `resets_at`(UTC 절대시각) − 현재시각을 **매분 로컬 재계산**. 서버의 상대시간 텍스트 파싱 금지.
4. 뽀모도로는 위 흐름과 완전 분리(타이머는 프론트엔드, 알림 발송만 Rust 커맨드 경유).

### 3.3 정규화 스키마 `UsageSnapshot` (Provider → UI 공통 계약)

```json
{
  "provider": "claude | codex",
  "state": "NORMAL | WARN | CRITICAL | STALE | NOT_LOGGED_IN | AUTH_ERROR | RATE_LIMITED",
  "primary":   { "used_pct": 34.0, "resets_at": "2026-06-10T13:40:00Z" },
  "secondary": { "used_pct": 8.0,  "resets_at": "2026-06-16T11:00:00Z" },
  "extra": { "used_credits": 1115.0, "monthly_limit": 3000, "currency": "USD" },
  "fetched_at": "2026-06-10T09:21:25Z",
  "is_stale": false,
  "error": null
}
```

- `extra`는 Claude 전용(없으면 null). UI는 hover 툴팁/설정 화면에서만 노출(중앙 디스크에는 표시하지 않음 — 디자인 결정 "중앙은 시간만").
- 완료 조건(AC): 두 Provider의 실측 응답(§4)을 입력으로 한 단위 테스트가 위 스키마로의 변환을 검증한다.

---

## 4. 데이터 소스 명세 (PoC 실측 기반)

### 4.1 공통 원칙 (변경 금지)

- 데이터 소스는 CLI가 쓰는 OAuth usage 엔드포인트만. 웹 스크래핑 금지.
- 토큰 허용 목적지: `api.anthropic.com`, `chatgpt.com` 백엔드 (+ refresh 인증 호스트는 R-3 확정 후 명시 추가).
- 엔드포인트 URL·헤더는 하드코딩하지 않고 **설정 파일에서 오버라이드 가능**(§7, `endpoints` 키) — 비공식 API 변경 대비.
- 응답 스키마가 예상과 다르면 크래시 대신 STALE 강등 + 마지막 정상값 유지.

### 4.2 ClaudeProvider

**토큰 읽기**

| OS | 소스 | 방법 |
|---|---|---|
| macOS | Keychain, 서비스명 `Claude Code-credentials` | Security framework(권장, Rust `security-framework` crate) 또는 `security find-generic-password -s "Claude Code-credentials" -w` 서브프로세스. 2차 폴백: `~/.claude/.credentials.json` (실측상 미존재하나 방어적 유지) |
| Linux | `~/.claude/.credentials.json` (권한 600) | 파일 읽기. 권한이 600보다 느슨하면 경고 로그(토큰값 미포함) |

파싱 경로: `claudeAiOauth.accessToken`, `claudeAiOauth.expiresAt`. 최상위의 OS별 부가 키(`trustedDeviceToken` / `organizationUuid`)는 무시한다.

**요청 (실측 검증됨, HTTP 200)**

```
GET https://api.anthropic.com/api/oauth/usage
Authorization: Bearer <accessToken>
anthropic-beta: oauth-2025-04-20
User-Agent: claude-code/<로컬 claude CLI 버전, 탐지 실패 시 PoC 검증값 2.1.138>
```

**응답 (실측 스키마)** — 사용 필드만 발췌:

| 필드 | 타입 | 의미 |
|---|---|---|
| `five_hour.utilization` | number(%) | 5h 창 사용률 → `primary.used_pct` |
| `five_hour.resets_at` | string, RFC 3339 (`+00:00`) | 5h 리셋 절대시각 |
| `seven_day.utilization` / `.resets_at` | 동일 | 7일 창 → `secondary` |
| `extra_usage.{used_credits, monthly_limit, utilization, currency, is_enabled}` | — | 추가 크레딧(USD). 실측: $1115/$3000 (37.2%) |
| 기타 (`seven_day_opus` 등 + 미지 필드) | null/any | **무시. 존재·부재·신규 추가 모두 비파괴적으로 처리** |

**완료 조건(AC)**
- [ ] macOS에서 키체인 경유, Linux에서 파일 경유로 각각 200 응답을 받고 `UsageSnapshot`을 생성한다.
- [ ] 응답에서 `five_hour`가 누락된 변조 fixture 입력 시 크래시 없이 STALE로 강등된다.
- [ ] `+00:00`/`Z` 양 표기의 `resets_at`을 동일 시각으로 파싱한다.

### 4.3 CodexProvider

**토큰 읽기**: `~/.codex/auth.json` → `tokens.access_token`, `tokens.refresh_token`, `tokens.account_id`, `last_refresh`. `OPENAI_API_KEY` 키는 v1에서 무시(chatgpt auth_mode만 지원, `auth_mode != "chatgpt"`이면 NOT_LOGGED_IN 취급 + 툴팁 안내).

**요청**: `https://chatgpt.com/backend-api` 하위 usage 엔드포인트. 정확한 경로/헤더는 codex-check, codex-cli-usage, CodexBar 소스에서 추출해 구현하고(§2.4-4), 설정 `endpoints.codex_usage`로 오버라이드 가능하게 한다.

**응답 (codex-check raw 기준, 실측)**

| 필드 (raw) | 의미 |
|---|---|
| `used_percent` | 창 사용률(%) |
| `limit_window_seconds` | 18000 = 5h(primary), 604800 = 7d(secondary). **라벨 문자열이 아니라 이 값으로 창을 식별**한다 |
| `reset_after_seconds` | 리셋까지 남은 초 (사용하지 않음 — 절대시각 우선 원칙) |
| `reset_at` | Unix epoch 초 → UTC 절대시각으로 변환해 `resets_at`으로 사용. ISO 필드가 함께 있으면 ISO 우선(PoC 권고) |

**토큰 갱신 (Codex)**: access token은 약 1시간 주기로 만료되는 것이 **정상 동작**이다. 만료 감지(401 또는 토큰 exp 클레임) 시 `refresh_token`으로 자동 갱신한다. **refresh 성공 시에는 어떤 경고도 표시하지 않는다.** 경고등(AUTH_ERROR)은 refresh까지 실패했을 때만. 갱신된 토큰의 `auth.json` 재기록 여부는 codex-cli 동작과의 충돌 위험이 있으므로:
- **권고안**: 갱신 토큰은 **메모리에만 보관**하고 파일에 쓰지 않는다(CLI 소유권 존중). 단, 메모리 토큰도 다음 폴링에서 파일이 더 새로우면(`last_refresh` 비교) 파일 우선.

**갱신 시퀀스**

```
poll → auth.json 읽기 → access_token 만료? 
  ├─ 아니오 → usage 호출
  └─ 예 → 메모리 캐시에 유효 토큰? 
        ├─ 예 → 그걸로 usage 호출
        └─ 아니오 → refresh 호출
              ├─ 성공 → 메모리 보관 → usage 호출  (경고 없음)
              └─ 실패(401/403) → AUTH_ERROR, 백오프 재시도
```

**완료 조건(AC)**
- [ ] codex-check 없이(npx 의존 없이) Rust 네이티브 HTTP로 usage 200 응답 1회 실측 → 로그를 M1 산출물에 포함.
- [ ] access token을 인위적으로 만료시킨 상태에서 refresh → usage 성공이 경고 없이 이뤄진다.
- [ ] refresh도 실패하는 fixture에서 AUTH_ERROR로 전이된다.

### 4.4 D-8: Claude 토큰 자체 refresh 수행 여부

PoC에서 Ubuntu의 Claude access token이 만료 상태였다(§2.4-2). 선택지:

| 후보 | 내용 | 장단점 |
|---|---|---|
| A. refresh 안 함 | 만료 시 AUTH_ERROR 표시, "claude CLI를 한 번 실행하세요" 툴팁 | 단순·안전. 단 CLI를 안 쓴 날엔 위젯이 죽은 화면 |
| B. 메모리 한정 refresh (**권고**) | `refreshToken`으로 앱이 직접 갱신하되 키체인/파일에 **쓰지 않음**. Codex와 동일한 파일-우선 규칙 | UX 최선. refresh 엔드포인트/클라이언트ID를 레퍼런스 소스에서 확정해야 하며 허용 목적지 목록 영향(R-3) |
| C. refresh + 원본 재기록 | CLI와 동일하게 저장소 갱신 | CLI와 경합·손상 위험. 비권고 |

→ **권고: B.** 단, refresh 호출 대상 호스트가 `api.anthropic.com` 밖이면 보안 원칙 B-1의 허용 목록 개정과 함께 결정할 것.

→ B으로 진행 결정

---

## 5. 상태 머신 명세 (위젯별 독립)

### 5.1 상태 표 (프롬프트 기본 + PoC 보강)

| 상태 | 진입 조건 | 게이지 표현 | 경고등 | 폴링 동작 | PoC 보강 |
|---|---|---|---|---|---|
| NORMAL | 최근 폴링 성공, 사용률 < 80% | 정상 색 (브랜드 컬러) | 꺼짐 | 기본 주기(180s) | 실측 정상값: Claude 5h 4~34% |
| WARN | 사용률 ≥ 80% | 경고 색 `#EBB13E` | 꺼짐 | 기본 주기 | 디자인 비주얼명 `low`(잔량≤20%와 동일 임계) |
| CRITICAL | 잔량 ≤ 5% / 사용률 ≥ 95% | 위험 색 `#E5484D` + 절제된 1.8s 펄스 | 꺼짐 | 기본 주기 | 사용률 100%면 비주얼 변형 `depleted`(링 빈 채 적색 트랙, 리셋시간 유지) |
| STALE | 마지막 성공 폴링 후 10분 경과 | 마지막 값 + 회색조/흐림 + "Nm ago" 배지 | stale 표시 | 계속 시도(기본 주기) | — |
| NOT_LOGGED_IN | 토큰 파일/키체인 항목 없음 | 반투명(아크 opacity .16) | **ON** 호박색 키 램프 "Sign in" | 60초마다 존재 재확인 | macOS는 키체인 항목 부재 = 미로그인. Codex는 `auth_mode != "chatgpt"`도 포함 |
| AUTH_ERROR | 토큰 만료 + 갱신 실패(401/403) | 반투명 | **ON** 적색 경고 삼각형 "Auth" | 백오프 후 재시도 | Codex 1h 만료는 정상이므로 refresh **성공** 시 절대 경고 금지 |
| RATE_LIMITED | 429 수신 | 마지막 값 + 흐림 | stale 표시 | 지수 백오프 3→6→12→24분, 상한 30분 | PoC 7회 연속 무429이나 장기 미검증(§2.4-5) |

### 5.2 전이 규칙

- 모든 상태는 **최신 폴링 결과 1건으로 재평가**된다(이력 누적 없음). 단 STALE 진입은 타이머(마지막 성공 +10분), RATE_LIMITED 해제는 다음 성공 응답.
- AUTH_ERROR/NOT_LOGGED_IN에서 성공 응답 수신 → 즉시 NORMAL/WARN/CRITICAL 중 사용률에 맞는 상태로 복귀.
- 시스템 절전 복귀·네트워크 복구 이벤트 → 즉시 1회 폴링(백오프 무시하되 RATE_LIMITED 중이면 백오프 유지).
- 위젯 간 상태 독립: Claude AUTH_ERROR가 Codex/뽀모도로에 영향 주지 않는다.
- CRITICAL 상태에서만 pulse 모션을 활성화한다. NORMAL/WARN/STALE/NOT_LOGGED_IN/AUTH_ERROR/RATE_LIMITED 및 Pomodoro 상태에는 적용하지 않는다.

### 5.3 비주얼-논리 매핑 (충돌 정리, §2.4-7)

| 논리 상태 | 비주얼 클래스 (design-reference.html) |
|---|---|
| NORMAL | (기본) |
| WARN | `.low` |
| CRITICAL | `.critical` / 사용률 100%일 때 `.depleted` |
| STALE, RATE_LIMITED | `.stale` (+ "Nm ago" 배지) |
| NOT_LOGGED_IN | `.notin` (키 램프) |
| AUTH_ERROR | `.autherr` (삼각형 램프) |
| — (상태 아님) | `.hoverdemo`/`:hover` — 마우스오버 인터랙션 레이어, `.paused`·`.break` — 뽀모도로 전용 |

**완료 조건(AC)**
- [ ] mock Provider로 7개 논리 상태 각각의 렌더링 스냅숏 테스트가 통과한다 (§10).
- [ ] Codex refresh 성공 경로에서 어떤 프레임에서도 경고등이 점등되지 않는다.
- [ ] STALE에서 마지막 정상 수치가 유지되어 표시된다(0이나 빈값으로 대체 금지).

---

## 6. UI/UX 명세

### 6.0 참조
/home/junon/Token_Dashboard/for_specification/design-reference.html

### 6.1 위젯 레이아웃 (140px 기준 단위)

- 위젯 3종 가로 나열(확정). 각 위젯은 140×140px 기준으로 설계, **크기 조절 가능**(확정) — 설정의 `widget_scale`(0.75~2.0)로 균일 스케일.
- 메인 링 = 5시간 한도 **잔량**(12시 시작, 시계방향 감소). 안쪽 가는 보조 링 = 7일 한도 잔량(확정: 보조 링 형태).
- 중앙 = 5시간 창 리셋까지 남은 시간 **H:MM 통일 포맷**, `tabular-nums`. 중앙에 퍼센트 숫자 없음(디자인 결정).
- 뽀모도로: 보조 링 없음, 메인 링 = 현재 인터벌 잔량, 중앙 = 남은 분(단일 숫자), 상태 3종(FOCUS/BREAK/PAUSED).
- 경고등 비주얼: 디자인 레퍼런스의 중앙 텔테일 방식(확정) — 미로그인 = 호박색 키 램프, 인증오류 = 적색 경고 삼각형. 데이터 부재 상태이므로 숫자 대신 램프가 중앙 점유.

### 6.2 기하 (디자인 레퍼런스 추출, 140px base)

| 요소 | 값 |
|---|---|
| Frosted disk | 지름 128px, border 1px, `rgba(20,21,26,.52)` + blur(10px) |
| Tick ring | 48개, 반경 62.5→66 (길이 3.5), 두께 1 |
| Main gauge (5h) | 반경 55, stroke 7, linecap round, 둘레 ≈ 345.6 |
| Secondary gauge (7d) | 반경 43, stroke 3, opacity 0.5, 둘레 ≈ 270.2 |
| 중앙 숫자 | 30px / weight 560 / ls −1 / tnum |
| 서비스 라벨 | 9px / 600 / ls 1.4 / uppercase |
| 경고 램프 | 26×26px, 라벨 9px |

### 6.3 디자인 토큰 표 (확정 — design-reference.html에서 추출)

**색**

| Role | Hex | 사용처 |
|---|---|---|
| Claude brand | `#D97757` / bright `#E88A63` | Claude 메인·보조 아크 |
| Codex brand | `#46C2D4` / bright `#62D4E3` | Codex 메인·보조 아크 |
| Pomodoro focus | `#F0563D` / bright `#FF6B52` | 집중 링 |
| Pomodoro break | `#5FB98C` / bright `#76CC9F` | 휴식 링 |
| Caution (system) | `#EBB13E` | WARN 아크 · NOT_LOGGED_IN 램프 |
| Danger (system) | `#E5484D` | CRITICAL/DEPLETED 아크 · AUTH_ERROR 램프 |
| Text primary | `#F3F4F6` | 중앙 숫자 |
| Label | `rgba(244,246,250,.55)` | 서비스 라벨 |
| Track main / sec | `rgba(255,255,255,.12)` / `.07` | 링 미충전부 |
| Tick | `rgba(255,255,255,.16)` | 눈금 48개 |
| Disk substrate / bezel | `rgba(20,21,26,.52)`+blur10 / `rgba(255,255,255,.07)` 1px | frosted 디스크 |

**타이포그래피**: `-apple-system, BlinkMacSystemFont, "Segoe UI", system-ui, sans-serif` — 웹폰트 의존 0.

**모션**

| 트리거 | 값 |
|---|---|
| 상태 전환 | 240ms `cubic-bezier(.4,0,.2,1)` |
| Hover | 160ms 동일 ease — 글로우 + 디스크 상승, **스케일 변형 없음** |
| CRITICAL 펄스 | 1800ms infinite, cubic-bezier(.45, 0, .2, 1), opacity 또는 glow 강도만 미세 변화 — **유일한 상시 모션** |
| 모션 감소 | `@media (prefers-reduced-motion: reduce)` 환경에서는 CRITICAL pulse 비활성화 |
| 그 외 | 상시 애니메이션 없음 (상주 앱 전력/주의 배려) |

### 6.4 동작 요구

| ID | 요구사항 | 완료 조건(AC) |
|---|---|---|
| UI-1 | 위젯별 표시 토글 | 설정에서 끈 위젯은 창에서 제거되고 재시작 후에도 유지 |
| UI-2 | 창 위치·설정 영속화 | 드래그 이동 후 재시작 시 동일 위치 복원. 경로: Linux `~/.config/<앱이름>/`, macOS `~/Library/Application Support/<앱이름>/` |
| UI-3 | 다중 모니터 위치 복구 | 저장 위치가 어떤 모니터 영역에도 없으면 주 모니터 중앙으로 복귀 |
| UI-4 | 로그인 시 자동 시작 | 옵션 제공, **기본 off** |
| UI-5 | 마지막 갱신 시각 표시 | hover 또는 stale 배지로 "Nm ago" 노출 |
| UI-6 | 데스크톱 알림 | v1에서는 제공하지 않음. 사용률은 게이지 상태로만 확인 |
| UI-7 | 클릭 통과(click-through) | v1 설정 화면에서는 노출하지 않음. 기존 config 호환 필드는 유지 |
| UI-8 | 드래그 이동 | `-webkit-app-region: drag` (PoC macOS 검증, Linux는 M2에서 육안 검증 §2.4-9) |
| UI-9 | 설정 화면 | 별도 일반 창(decorations 있는 표준 창). 항목: 위젯 토글 3종, 폴링 주기, 위젯 스케일, 자동 시작, 뽀모도로 시간, 엔드포인트 오버라이드(고급, 접힘), 앱 종료 |
| UI-10 | 앱 종료 | 설정 창에서 `Quit` 버튼 제공. frameless/skip-taskbar 위젯만 떠 있어도 사용자가 앱을 종료할 수 있어야 함 |
| UI-11 | 앱 썸네일/아이콘 | macOS Dock/앱 전환기 및 Linux 작업표시줄/창 목록에 표시되는 앱 이미지는 `assets/Thumbnail.png`를 사용 |

---

## 7. 설정/영속화 스키마

위치: Linux `~/.config/<앱이름>/config.json`, macOS `~/Library/Application Support/<앱이름>/config.json`. **토큰은 절대 이 파일에 기록하지 않는다.**

```json
{
  "version": 1,
  "widgets": {
    "claude":   { "enabled": true,  "position": { "x": 120, "y": 80 } },
    "codex":    { "enabled": true,  "position": { "x": 280, "y": 80 } },
    "pomodoro": { "enabled": true,  "position": { "x": 440, "y": 80 } }
  },
  "widget_scale": 1.0,
  "polling": {
    "interval_sec": 180,
    "min_interval_sec": 120
  },
  "click_through": false,
  "autostart": false,
  "pomodoro": { "focus_min": 20, "break_min": 5, "dial_full_min": 60 },
  "endpoints": {
    "claude_usage": "https://api.anthropic.com/api/oauth/usage",
    "claude_beta_header": "oauth-2025-04-20",
    "codex_base": "https://chatgpt.com/backend-api",
    "codex_usage_path": null
  },
  "advanced": {
    "claude_token_source_override": null,
    "codex_auth_path": "~/.codex/auth.json"
  }
}
```

규칙:
- `interval_sec < 120` 입력 시 120으로 강제 클램프(레이트리밋 보호, 변경 금지 정책).
- 알 수 없는 키는 보존(roundtrip), 누락 키는 기본값. `version` 마이그레이션 훅 포함.
- 뽀모도로 기본: **집중 20분 / 다이얼 한 바퀴 60분**(확정). `break_min` 기본 5는 pomotier 사양 이식 시 원본 값으로 대체. [결정 필요 아님 — pomotier 사양 확인 후 채움]

**완료 조건(AC)**: 손상된 config.json(파싱 불가) → 백업(`config.json.bak`) 후 기본값 재생성, 크래시 없음.

---

## 8. 플랫폼별 구현 노트

### 8.1 macOS (실측: 26.5.1, arm64)

- **토큰**: Keychain `"Claude Code-credentials"` 단일 경로(실측). Security framework API 우선, `security` CLI 폴백. 첫 접근 시 키체인 권한 다이얼로그가 뜰 수 있음 → README에 스크린샷 안내.
- **투명창 (실측 검증)**: `tauri.conf.json`의 `app.windows[0]`에 `transparent:true, decorations:false, alwaysOnTop:true, skipTaskbar:true`, **그리고 `app` 최상위에 `macOSPrivateApi: true`** (windows[0] 안에 두면 무효 — PoC 주의사항). `Cargo.toml`: `tauri = { version = "2", features = ["macos-private-api"] }`. 미적용 시 배경 검게 나옴.
- **드래그**: `-webkit-app-region: drag` CSS (실측 정상).
- private API 사용 → Mac App Store 배포 불가(어차피 비목표). Gatekeeper: 무서명 배포이므로 §9 참조.

### 8.2 Ubuntu / Linux X11 (실측: 22.04.5, GNOME/X11)

- **토큰**: `~/.claude/.credentials.json`(600), `~/.codex/auth.json`(600). 권한 600 초과 개방 시 경고.
- **WebKitGTK**: 빌드 의존성 `libwebkit2gtk-4.1-dev`(실측 2.50.4), `libsoup-3.0`. **런타임에 `WEBKIT_DISABLE_DMABUF_RENDERER=1`을 앱이 자동 설정**(실측: 미설정 시 `EGL_NOT_INITIALIZED` 크래시). 구현: 프로세스 시작 직후 env set 또는 .desktop `Exec=env WEBKIT_DISABLE_DMABUF_RENDERER=1 <bin>`.
- `alwaysOnTop`/`skipTaskbar`/드래그: X11에서 미검증(§2.4-9) → M2 수동 검증 체크리스트. 시스템 트레이: v1은 GNOME 기본 트레이 부재를 감안해 **트레이 없이 설정 창 진입은 위젯 우클릭 메뉴**로 제공. [결정 필요] D-9: AppIndicator 의존(트레이) 추가 여부 — 권고: v1 제외.
- Wayland: 비목표. `XDG_SESSION_TYPE=wayland` 감지 시 "X11 세션에서 검증됨" 1회 경고만.

---

## 9. 빌드/배포 파이프라인

- **CI (GitHub Actions)**: 태그 `v*` 푸시 → matrix 빌드(macos-latest arm64, ubuntu-22.04) → 산출물: `.dmg`(무서명), `.deb`, `.AppImage` → GitHub Releases 자동 업로드. PR마다 lint + 단위테스트(상태 머신 mock 테스트 포함). **실 API 호출 테스트는 CI에서 금지** — 네트워크 차단 환경에서도 전 테스트 통과해야 함.
- **macOS 배포**: Apple Developer Program 미사용(확정). GitHub Releases + Homebrew tap. tap formula에 `--no-quarantine` 캐스크 안내. README에 Gatekeeper 우회(우클릭→열기 / `xattr -dr com.apple.quarantine`) **스크린샷 포함 필수**.
- **자동 업데이트**: macOS는 "새 버전 알림만"(릴리스 RSS/API 폴링, 일 1회), Linux는 알림 + 수동 설치 안내. 자가 교체 없음.
- **라이선스/고지**: MIT. README 최상단에 "비공식 도구이며 Anthropic/OpenAI와 무관. 미문서화 API 사용으로 예고 없이 동작이 중단될 수 있음" 고지. **앱 이름·번들ID에 Claude/Codex 상표 미사용**(확정). 위젯 라벨 텍스트("Claude"/"Codex")는 데이터 소스 식별 표기로 한정.

**완료 조건(AC)**: 클린 머신(macOS arm64 / Ubuntu 22.04)에서 릴리스 산출물 설치 → 앱 실행 → CLI 로그인 상태에서 NORMAL 게이지 표시까지 외부 수동 개입(환경변수 수동 설정 포함) 0회.

---

## 10. 테스트 계획

| 레벨 | 내용 | 필수 여부 |
|---|---|---|
| 단위 (Rust) | 실측 응답 fixture(§4) → `UsageSnapshot` 변환, RFC3339 양 표기 파싱, epoch 변환, 스키마 변조 fixture → STALE 강등, 백오프 시퀀스(3→6→12→24→30분 상한) | 필수 |
| 단위 (Rust) | 토큰 마스킹: 디버그 로그 HTTP 덤프에서 `Authorization` 헤더가 항상 마스킹됨을 검증하는 테스트 | 필수 |
| 상태 머신 | **UsageProvider mock으로 7개 논리 상태 각각의 렌더링 검증** (프론트 스냅숏/DOM 클래스 단언: `.low/.critical/.depleted/.stale/.notin/.autherr` 매핑) | 필수 (명시 요구) |
| 통합 (로컬 전용) | 실 토큰으로 실 API 1회 호출 스모크 — **CI 금지**, 개발자 로컬 스크립트로만 | 선택 |
| 수동 (M2) | Linux X11: alwaysOnTop / skipTaskbar / 드래그 / 클릭통과 육안 체크리스트. 다중 모니터 위치 복구. 절전 복귀 즉시 폴링 | 필수 |
| 폴링 안정성 | 3분 간격 20회(1시간) 무429 완주 — Ubuntu PoC 미완(§2.4-5) 보완. M1 종료 조건 | 필수 |
| 뽀모도로 격리 | 네트워크 차단 + 토큰 삭제 환경에서 뽀모도로 전 기능 정상 | 필수 |

---

## 11. 마일스톤

| 단계 | 산출물 | 종료 조건 |
|---|---|---|
| **M1 — 데이터 레이어 (CLI 검증판)** | GUI 없는 Rust 바이너리: 두 Provider가 토큰 읽기→(필요시 refresh)→usage 호출→`UsageSnapshot` JSON을 stdout 출력 | ① 양 OS에서 Claude/Codex snapshot 출력 성공 ② Codex **직접 호출**(래퍼 미사용) 실측 로그 ③ 폴링 20회 무429 완주 ④ refresh 엔드포인트 호스트 확정 → 허용 목적지 목록 개정(R-3 해소) ⑤ D-8 결정 |
| **M2 — 단일 위젯 UI** | Claude 위젯 1종 투명창 렌더링 + 7상태 mock 테스트 | ① 디자인 토큰 1:1 구현 ② Linux 미검증 창 속성 육안 통과 ③ 상태 머신 테스트 green |
| **M3 — 3위젯 + 설정** | Codex·뽀모도로 위젯, 설정 창, 영속화, 알림, 자동시작, 클릭통과 | ① UI-1~UI-9 AC 전부 통과 ② 뽀모도로 격리 테스트 통과 |
| **M4 — 패키징/배포** | GitHub Actions 릴리스 파이프라인, README(Gatekeeper 스크린샷 포함), Homebrew tap | §9 AC(클린 머신 설치) 통과, v0.1.0 태그 릴리스 |

---

## 12. 부록

### 12.1 iPhone 개인용 릴레이 (v1 비범위 — 개요만)

상시 가동 PC에서 M1 데이터 레이어 바이너리를 데몬으로 돌려 snapshot JSON을 로컬 HTTP로 노출 → Tailscale 사설망으로만 접근 → iPhone Scriptable 위젯이 주기 fetch해 잠금화면/홈 위젯 렌더링. 토큰은 여전히 PC 밖으로 나가지 않으며(스냅숏 수치만 전송), 본 명세의 어떤 모듈에도 의존성을 추가하지 않는다. 별도 저장소로 분리.

### 12.2 리스크 목록

| ID | 리스크 | 영향 | 완화 |
|---|---|---|---|
| R-1 | 미문서화 usage API의 스키마/엔드포인트 변경 | 게이지 전면 불능 | 설정 기반 엔드포인트 오버라이드(§7), 스키마 불일치 시 STALE 강등, README 고지 |
| R-2 | 레이트리밋 정책 강화(180s 폴링이 429 유발) | RATE_LIMITED 상시화 | 하한 120s 강제 + 지수 백오프, M1 폴링 20회 검증으로 베이스라인 확보 |
| R-3 | **refresh 인증 호스트가 허용 목적지 2곳 밖** (§2.4-3) | 보안 원칙 B-1과 충돌 / refresh 불능 | M1에서 codex-check·CodexBar 소스로 호스트 확정 → 허용 목록을 명시적 개정(목적지별 사유 문서화) |
| R-4 | macOS private API 의존 (`macOSPrivateApi`) | OS 업데이트로 투명창 파손 가능 | 릴리스 전 최신 macOS 스모크, 파손 시 불투명 폴백 모드 |
| R-5 | Linux WebKitGTK 렌더러 이슈 재발/변형 | 시작 크래시 | `WEBKIT_DISABLE_DMABUF_RENDERER=1` 자동 주입(§8.2) + README 트러블슈팅 |
| R-6 | Claude 토큰 만료 + CLI 미사용 기간 | 위젯 장기 AUTH_ERROR | D-8 결정(권고 B: 메모리 한정 refresh) |
| R-7 | 상표/약관 리스크 (비공식 API 사용) | 배포 중단 요청 가능성 | 중립 앱 이름, 비공식 고지, 토큰 로컬 한정 설계 문서화 |

### 12.3 [결정 필요] 목록 총괄

| ID | 항목 | 후보 / 권고 |
|---|---|---|
| D-1 | **앱 이름** (Claude/Codex 상표 미사용) | 'Token Dashboard'[결정 완료]|
| D-8 | Claude 토큰 자체 refresh (§4.4) | 권고 B(메모리 한정 refresh)으로 진행. **[결정 완료]** |
| D-9 | Linux 트레이(AppIndicator) 추가 (§8.2) | 권고: v1 제외, 위젯 우클릭 메뉴로 대체. **[결정 완료]** |

확정 완료된 항목(재논의 불요): 7일 게이지 = 보조 링 / 디자인 토큰 = §6.3 표 / 위젯 크기 조절 가능 + 가로 나열 / 경고등 = 중앙 텔테일 / 뽀모도로 20분 기본·60분 한 바퀴 / Windows 미지원.
