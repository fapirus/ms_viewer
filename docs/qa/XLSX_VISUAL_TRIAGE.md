# XLSX Visual Triage

## Scope
- 기준 자료:
  - `issue/excel/template-a/template-xlsx-a.xlsx`
  - `issue/excel/template-b/template-xlsx-b.xlsx`
- 비교 자료:
  - MS Excel 캡처
  - demo 앱 캡처

## Case classification

### template-a / Sheet1
- 현상:
  - worksheet 본문 grid는 보이지만 chart/drawing 계열 시각 요소가 누락된다
- 원인:
  - 현재 엔진이 worksheet drawing, chart part, chart image를 해석하지 않는다
- 우선순위:
  - 중간
- 처리:
  - `Phase 4` backlog로 이관

### template-a / Sheet2, Sheet3
- 현상:
  - 데이터가 거의 없을 때 시트가 한 셀처럼 보이거나 지나치게 작게 보인다
- 원인:
  - used range까지 render bounds를 clamp해서 blank grid가 사라진다
- 우선순위:
  - 높음
- 처리:
  - `Phase 3.6`에서 수정

### template-b / Purchase Order
- 현상:
  - blank area가 부족하고 initial viewport가 spreadsheet답지 않게 보인다
- 원인:
  - visible window minimum budget은 있었지만 Rust render 단계가 실제 blank cell window를 유지하지 못했다
- 우선순위:
  - 높음
- 처리:
  - `Phase 3.6`에서 수정

## Current decision
- 즉시 수정:
  - blank grid window 유지
  - 최소 visible cell budget 유지
  - scrolling 시 window를 lazy하게 더 확장
- 후속 지원:
  - worksheet drawing/image parse
  - chart placeholder/render support
