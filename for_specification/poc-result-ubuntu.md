# PoC 결과 — Ubuntu
- 실행일시 / 우분투 버전: 2026-06-10T20:07:05+09:00 / Ubuntu 22.04.5 LTS (jammy)
- 세션 타입(XDG_SESSION_TYPE) / 데스크톱 환경 / 모니터 구성: x11 / ubuntu:GNOME / 확인 불가 (`xrandr`: `Can't open display :1`)
- 도구 버전: claude 2.1.170 / codex-cli 0.139.0 / node v25.6.1 / jq 1.6 / rustc 1.96.0 / cargo 1.96.0
## 토큰 저장 위치
- Claude 파일 존재/권한: `/home/junon/.claude/.credentials.json` 존재, 권한 `600`
- Claude 구조(키 목록): top-level `["claudeAiOauth","organizationUuid"]`, `claudeAiOauth` keys `["accessToken","refreshToken","expiresAt","scopes","subscriptionType","rateLimitTier"]`
- Claude access token 상태: `expiresAt` = `2026-06-09T17:39:15Z`로 현재 실행 시각 기준 만료됨
- Codex auth.json 구조(키 목록)/권한: `/home/<redacted>/.codex/auth.json` 존재, 권한 `600`
- Codex top-level keys: `["auth_mode","OPENAI_API_KEY","tokens","last_refresh"]`
- Codex `.tokens` keys: `["id_token","access_token","refresh_token","account_id"]`
## Claude usage API
- 결과: 성공
- HTTP 코드: `200`
- 응답 JSON (마스킹):
```json
{
  "five_hour": {
    "utilization": 34.0,
    "resets_at": "2026-06-10T13:40:00.531727+00:00"
  },
  "seven_day": {
    "utilization": 8.0,
    "resets_at": "2026-06-16T11:00:00.531748+00:00"
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
 
- `five_hour`/`seven_day` 필드명과 `resets_at` 형식: 확인됨. `resets_at`은 ISO 8601 offset datetime 문자열.
## 폴링 안정성 (3분 간격 20회)
- 시간 제약으로 20회 완주는 생략.
- 짧은 재측정 결과: 3분 간격 7회 연속 `200`, `429` 0회, 기타 0회.
- 로그 파일: `/home/junon/Token_Dashboard/poll-test.clean-20.log`
- 판정: 단기 폴링 안정성은 통과. 1시간 장기 안정성까지 공식 주장하려면 20회 완주가 필요함.
## Codex usage API
- 사용 도구: `npx codex-check --auth ~/.codex/auth.json --json`
- 결과: 성공
- 결과 JSON (마스킹):
```json
[
  {
    "authFile": "/home/<redacted>/.codex/auth.json",
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
        "percentUsed": 45,
        "percentRemaining": 55,
        "windowMinutes": 300,
        "resetsAt": "2026-06-10T09:07:44.000Z",
        "raw": {
          "used_percent": 45,
          "limit_window_seconds": 18000,
          "reset_after_seconds": 147,
          "reset_at": 1781082464
        }
      },
      "secondary": {
        "label": "weekly",
        "percentUsed": 23,
        "percentRemaining": 77,
        "windowMinutes": 10080,
        "resetsAt": "2026-06-16T09:47:13.000Z",
        "raw": {
          "used_percent": 23,
          "limit_window_seconds": 604800,
          "reset_after_seconds": 520916,
          "reset_at": 1781603233
        }
      }
    },
    "fetchedAt": "2026-06-10T09:05:18.307Z",
    "lastRefresh": "2026-06-09T09:49:36.892204687Z"
  }
]
```
 
- 5시간/7일 필드명: `windows.primary` (`label: "5h"`), `windows.secondary` (`label: "weekly"`)
- 리셋 시간 형식: ISO 문자열 `resetsAt`, 원본 epoch seconds `raw.reset_at`
- 토큰 자동갱신 여부: `lastRefresh` 값은 확인됨. 이번 실행 중 새로 갱신됐다는 증거는 없음.
## Tauri 환경
- 의존성 설치 결과: 사용자가 직접 apt 설치 완료 확인.
- `jq`: 설치됨 (`jq-1.6`)
- `libwebkit2gtk-4.1-dev`: 설치됨. `pkg-config --modversion webkit2gtk-4.1` = `2.50.4`
- `javascriptcoregtk-4.1`: `2.50.4`
- `libsoup-3.0`: `3.0.7`
- Rust: 설치됨. `rustc 1.96.0`, `cargo 1.96.0`
- (선택) 투명창 스파이크 결과: 환경변수 적용 후 창 표시 성공.
  - `npm run tauri dev` 빌드 완료: `Finished dev profile`
  - 최초 실행 실패 메시지: `Could not create GBM EGL display: EGL_NOT_INITIALIZED. Aborting...`
  - 재시도 권장 환경변수: `WEBKIT_DISABLE_DMABUF_RENDERER=1`
  - 재시도 후 화면: `Welcome to Tauri` 기본 화면 표시 성공.
  - `decorations:false`: 스크린샷 기준 OS 타이틀바/테두리 없음으로 적용된 것으로 보임.
  - `transparent:true`: CSS 배경 제거/반투명 패널 적용 후 투명 표시 확인됨.
  - `alwaysOnTop` / `skipTaskbar` / 드래그 이동: 사용자 육안 확인 필요.
## 특이사항 / 막힌 부분
- `jq` 및 Tauri WebKit 계열 apt 의존성은 설치 완료.
- Rust 설치 및 현재 사용자 PATH 확인 완료.
- `XDG_SESSION_TYPE=x11`, `XDG_CURRENT_DESKTOP=ubuntu:GNOME`, `DISPLAY=:1`, `XAUTHORITY=/run/user/1000/gdm/Xauthority`이나 `xrandr`/`glxinfo`가 `Can't open display :1`로 실패함.
- Claude usage API는 재로그인/토큰 갱신 후 단일 호출 `200` 성공 확인됨.
- 정상적인 폴링 안정성 검증은 기존 실패/오염 로그를 폐기하고 3분 간격 20회로 재수행해야 함.
- Tauri 투명창 스파이크는 최초 WebKitGTK EGL/GBM 초기화 실패가 있었으나 `WEBKIT_DISABLE_DMABUF_RENDERER=1` 적용 후 창 표시 성공. CSS 배경 제거/반투명 패널 적용 후 투명 표시 확인됨.
