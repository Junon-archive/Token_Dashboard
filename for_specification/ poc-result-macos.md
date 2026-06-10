# PoC 결과 — macOS
 
- **실행일시**: 2026-06-10T09:21:25Z
- **macOS 버전**: 26.5.1 (Build 25F80)
- **아키텍처**: arm64 (Apple Silicon)
- **도구 버전**:
  - claude: 2.1.138 (Claude Code)
  - codex: 0.139.0 (codex-cli)
  - node: v24.15.0
  - jq: 1.7.1-apple
  - rustc: 1.96.0 (ac68faa20 2026-05-25)
- **CLI 로그인 상태**:
  - claude: 로그인됨 (Keychain 토큰 존재 확인, usage API HTTP 200 성공)
  - codex: 로그인됨 (`codex login status` 출력 없음·exit 0, usage API 정상 응답 확인)
---
 
## 토큰 저장 위치
 
- **Claude**: **키체인(Keychain)** — 서비스명 `"Claude Code-credentials"`
  - `security find-generic-password -s "Claude Code-credentials" -w` 로 JSON 문자열 획득
  - 최상위 키: `["claudeAiOauth", "trustedDeviceToken"]`
  - `claudeAiOauth` 하위 키: `["accessToken", "expiresAt", "rateLimitTier", "refreshToken", "scopes", "subscriptionType"]`
  - ※ `~/.claude/.credentials.json` 파일은 존재하지 않음 → **키체인 단일 경로**
- **Codex**: `~/.codex/auth.json` (파일 방식)
  - 최상위 키: `["OPENAI_API_KEY", "auth_mode", "last_refresh", "tokens"]`
  - `tokens` 하위 키: `["access_token", "account_id", "id_token", "refresh_token"]`
  - `auth_mode`: `"chatgpt"`
---
 
## Claude usage API
 
- **결과**: 성공 (HTTP 200)
- **엔드포인트**: `https://api.anthropic.com/api/oauth/usage`
- **헤더**: `anthropic-beta: oauth-2025-04-20`
- **사용한 User-Agent**: `claude-code/2.1.138`
- **응답 JSON** (토큰성 필드 없음, 마스킹 불필요):
```json
{
  "five_hour": {
    "utilization": 4.0,
    "resets_at": "2026-06-10T13:40:00.931160+00:00"
  },
  "seven_day": {
    "utilization": 6.0,
    "resets_at": "2026-06-16T11:00:00.931180+00:00"
  },
  "seven_day_oauth_apps": null,
  "seven_day_opus": null,
  "seven_day_sonnet": null,
  "seven_day_cowork": null,
  "seven_day_omelette": null,
  "tangelo": null,
  "iguana_necktie": null,
  "omelette_promotional": null,
  "cinder_cove": null,
  "extra_usage": {
    "is_enabled": true,
    "monthly_limit": 3000,
    "used_credits": 1115.0,
    "utilization": 37.166666666666664,
    "currency": "USD",
    "disabled_reason": null
  }
}
```
 
**필드 구조 메모**:
- 5시간 창: `five_hour.utilization` (%), `five_hour.resets_at` (ISO 8601 with timezone offset `+00:00`)
- 7일 창: `seven_day.utilization` (%), `seven_day.resets_at` (ISO 8601 with timezone offset `+00:00`)
- 추가 사용량: `extra_usage.used_credits` / `extra_usage.monthly_limit` (USD), `extra_usage.utilization` (%)
- 나머지 모델별 필드(`seven_day_opus` 등)는 현재 `null`
---
 
## Codex usage API
 
- **사용 도구**: `npx codex-check` (`codex-check` npm 패키지)
- **호출 방법**: `npx codex-check --auth ~/.codex/auth.json --json`
- **결과 JSON** (토큰성 필드 마스킹 적용):
```json
[
  {
    "authFile": "/Users/<redacted>/.codex/auth.json",
    "baseUrl": "https://chatgpt.com/backend-api",
    "account": {
      "email": "<redacted@example.invalid>",
      "accountId": "<redacted-account-id>",
      "planFromToken": "plus",
      "planFromUsage": "plus"
    },
    "allowed": true,
    "limitReached": false,
    "windows": {
      "primary": {
        "label": "5h",
        "percentUsed": 4,
        "percentRemaining": 96,
        "windowMinutes": 300,
        "resetsAt": "2026-06-10T14:07:49.000Z",
        "raw": {
          "used_percent": 4,
          "limit_window_seconds": 18000,
          "reset_after_seconds": 17334,
          "reset_at": 1781100469
        }
      },
      "secondary": {
        "label": "weekly",
        "percentUsed": 24,
        "percentRemaining": 76,
        "windowMinutes": 10080,
        "resetsAt": "2026-06-16T09:47:13.000Z",
        "raw": {
          "used_percent": 24,
          "limit_window_seconds": 604800,
          "reset_after_seconds": 520097,
          "reset_at": 1781603233
        }
      }
    },
    "fetchedAt": "2026-06-10T09:18:56.849Z",
    "lastRefresh": "2026-06-10T09:11:45.099333Z"
  }
]
```
 
**필드 구조 메모**:
- 5시간 창: `windows.primary.percentUsed` (%), `windows.primary.resetsAt` (ISO 8601 UTC `Z`)
  - raw: `used_percent`, `limit_window_seconds` (18000 = 5h), `reset_after_seconds`, `reset_at` (Unix timestamp)
- 7일 창: `windows.secondary.percentUsed` (%), `windows.secondary.resetsAt` (ISO 8601 UTC `Z`)
  - raw: `limit_window_seconds` (604800 = 7일), `reset_at` (Unix timestamp)
- **토큰 갱신 발생 여부**: `lastRefresh`(09:11Z) vs `fetchedAt`(09:18Z) — 호출 시점에 토큰 갱신 없음 (기존 토큰 재사용). `codex-check`는 내부적으로 만료 시 자동 갱신 처리함.
---
 
## Tauri 환경
 
- **Xcode CLT**: 설치됨 (`/Library/Developer/CommandLineTools`)
- **Rust/Cargo**: 설치됨 — rustc 1.96.0 / cargo 1.96.0 (rustup, `$HOME/.cargo/env` source 필요)
- **Tauri 버전**: tauri 2.5.1, tauri-build 2.2.0
- **첫 빌드 소요 시간**: 약 4분 37초 (arm64 Apple Silicon, dev profile)
### 투명창 스파이크 결과: **성공**
 
**적용 설정** (`src-tauri/tauri.conf.json` → `app.windows[0]`):
```json
"transparent": true,
"decorations": false,
"alwaysOnTop": true,
"skipTaskbar": true
```
그리고 `app` 레벨에:
```json
"macOSPrivateApi": true
```
 
**`src-tauri/Cargo.toml`**:
```toml
tauri = { version = "2", features = ["macos-private-api"] }
```
 
**CSS** (`src/styles.css`):
```css
html, body { background: transparent; }
.container {
  background: rgba(20, 20, 30, 0.75);
  backdrop-filter: blur(12px);
  -webkit-app-region: drag;   /* 드래그 이동 */
}
```
 
**실행 결과**: `npm run tauri dev` 성공, PID 98043/99902로 프로세스 기동 확인.
투명 배경 + 프레임 없는 창 + 항상 위 + 드래그 가능 — 모두 정상 동작.
 
**주의사항**:
- `macOSPrivateApi: true` 없이 `transparent: true`만 적용하면 배경이 검게 나옴 (macOS 한정)
- Tauri v2에서 `macOSPrivateApi`는 `app.windows[0]` 안이 아니라 **`app` 최상위 레벨**에 위치함
- `-webkit-app-region: drag` CSS로 프레임 없는 창을 드래그 이동 가능하게 함
---
 
## 특이사항 / 막힌 부분
 
1. **Claude 토큰 저장소가 키체인 전용**: macOS에서 `~/.claude/.credentials.json`이 없고 Keychain에만 저장됨. 앱에서 읽으려면 `security find-generic-password` 또는 macOS Security framework API 사용 필요. Linux/Windows에서는 파일 경로가 다를 수 있음.
2. **Claude usage API resets_at 형식**: `+00:00` suffix (RFC 3339), Codex는 `Z` suffix — 파싱 시 양쪽 모두 처리 필요.
3. **Codex raw.reset_at은 Unix timestamp**: `windows.primary.raw.reset_at`은 Unix epoch 정수. `resetsAt` (ISO 8601)과 동일한 시각이므로 ISO 필드 사용 권장.
4. **npm 캐시 권한 문제**: `/Users/junonlee/.npm/_cacache/content-v2/sha512/d5/` 디렉터리가 `root` 소유로 설정돼 있어 `npm create` 실패. `npm config set cache ~/.npm-cache` 로 캐시 경로를 변경해 우회.
5. **Claude extra_usage**: `monthly_limit: 3000 USD` 기반의 추가 사용 크레딧 필드 존재 — 대시보드에 USD 사용량 표시 가능. 현재 $1115 / $3000 사용 (37.2%).
6. **Codex plan**: `"plus"` 플랜 확인됨 (`planFromToken`, `planFromUsage` 모두 `"plus"`).
 
