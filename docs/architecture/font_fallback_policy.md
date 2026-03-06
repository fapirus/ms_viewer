# Font Fallback Policy

## Goal
문서 지정 폰트가 없는 경우 플랫폼별 대체 규칙을 정의한다.

## Design rules
- 문서 지정 폰트를 우선 시도한다.
- OS에 폰트가 없으면 플랫폼별 curated fallback map을 사용한다.
- fallback은 문자 스크립트별로 다를 수 있다.
- 초기 MVP는 Latin과 CJK 우선으로 시작한다.

## Resolution order
1. requested font family exists on platform
2. explicit platform fallback mapping
3. script-based generic fallback
4. final generic fallback

## Initial platform mapping

### Windows
- `Calibri -> Arial`
- `Cambria -> Times New Roman`
- `Malgun Gothic -> Malgun Gothic`
- `Batang -> Malgun Gothic`

### macOS
- `Calibri -> Helvetica`
- `Cambria -> Times`
- `Malgun Gothic -> Apple SD Gothic Neo`
- `Batang -> Apple SD Gothic Neo`

### Android
- `Calibri -> Roboto`
- `Cambria -> Noto Serif`
- `Malgun Gothic -> Noto Sans CJK KR`
- `Batang -> Noto Sans CJK KR`

### iOS
- `Calibri -> Helvetica`
- `Cambria -> Times New Roman`
- `Malgun Gothic -> Apple SD Gothic Neo`
- `Batang -> Apple SD Gothic Neo`

## Script-based fallback
### Latin
- Windows: `Arial`
- macOS: `Helvetica`
- Android: `Roboto`
- iOS: `Helvetica`

### CJK
- Windows: `Malgun Gothic`
- macOS: `Apple SD Gothic Neo`
- Android: `Noto Sans CJK KR`
- iOS: `Apple SD Gothic Neo`

## MVP behavior
- fallback은 family 단위로만 해석한다.
- weight, italic, variable font axis는 후속 처리로 둔다.
- font substitution 결과는 layout 전에 확정한다.

## Deferred items
- emoji fallback chain
- symbol font fallback chain
- serif/sans/monospace generic family abstraction
- user-provided fallback override
