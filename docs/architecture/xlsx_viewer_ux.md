# XLSX Viewer UX

## Why the current UX is not enough
- 현재 `XLSX`는 공통 `PageRenderModel` 위에 dense grid를 그려서 "보이게" 만드는 단계다.
- 이 구조는 엔진 검증과 demo smoke에는 유용하지만, 실제 사용자가 기대하는 spreadsheet UX와는 다르다.
- `XLSX`는 `DOCX/PPTX`처럼 page 중심 문서가 아니라, `2차원 grid를 탐색하는 UI`가 본질이다.

## Market and technical findings

### 1. Flutter 기본 `DataTable`은 large-sheet viewer에 맞지 않는다
- Flutter 공식 `DataTable` 문서는 large data에서 비용이 크다고 설명한다.
- 이유:
  - column sizing을 위해 두 번 측정
  - `SingleChildScrollView`는 보이는 부분만이 아니라 전체 child를 mount/paint
- Flutter 공식 문서는 large amounts of data에는 `TableView`나 `CustomScrollView`를 보라고 안내한다.

### 2. Flutter 공식 2D scrolling stack은 `TableView` 중심이다
- Flutter 공식 `two_dimensional_scrollables` 패키지의 `TableView`는:
  - lazy viewport
  - infinite rows/columns
  - pin rows/columns
  - merged cells
  를 지원한다.
- 즉, 현재 우리 요구사항과 가장 직접적으로 맞는 기반은 `DocumentPageView`가 아니라 `TableView` 계열이다.

### 3. Excel의 global max를 그대로 그리면 안 된다
- Excel 공식 제한은 worksheet당 `1,048,576 rows x 16,384 columns`다.
- 이 수치는 address validity와 문서 호환성의 상한이지, 초기 render 범위가 아니다.
- viewer는 이 전체 범위를 paint 기준으로 잡으면 안 된다.

### 4. SpreadsheetML은 `used range` 힌트를 준다
- Open XML의 `dimension`은 worksheet의 used range를 나타낸다.
- used cells에는 값, 수식, 텍스트뿐 아니라 서식도 포함될 수 있다.
- 다만 `dimension`은 optional이므로, viewer는 이것만 믿으면 안 된다.
- 실제 cell records, row/column metrics, merges, frozen pane metadata를 함께 봐서 effective bounds를 계산해야 한다.

### 5. Freeze panes는 grid UX의 핵심이다
- Excel 공식 문서는 freeze panes를 "선택 셀의 위 row와 왼쪽 column을 고정"하는 개념으로 설명한다.
- 따라서 우리 viewer도 단순 overlay가 아니라:
  - 좌상단 고정 코너
  - 상단 고정 header strip
  - 좌측 고정 row strip
  - 본문 scroll viewport
  형태의 split viewport 모델로 가는 편이 맞다.

### 6. 실제 spreadsheet 제품은 grid-first interaction을 채택한다
- Rows 공식 자료를 보면:
  - elastic grid
  - sort/filter
  - view mode
  - keyboard navigation
  을 핵심 경험으로 둔다.
- 특히 keyboard shortcut 문서에서 `current data area` 끝으로 이동하는 패턴이 나온다.
- 이건 viewer가 full sheet보다 "현재 데이터 영역"을 중심으로 UX를 잡아야 한다는 신호다.

### 7. Flutter ecosystem의 pragmatic reference
- Syncfusion Flutter DataGrid는:
  - frozen rows/columns
  - column resizing
  - stacked headers
  같은 spreadsheet UX를 제공한다.
- 우리는 Syncfusion을 그대로 채택할 필요는 없지만, UX 체크리스트 기준으로는 참고 가치가 높다.

## Product decision
- `XLSX`는 공통 `DocumentPageView`를 최종 UX로 삼지 않는다.
- 최종 방향은 `grid-first spreadsheet viewer`다.
- 즉:
  - `DOCX/PPTX`: page/slide viewer
  - `XLSX`: 2D sheet viewport viewer

## Recommended architecture

### Flutter side
- 전용 `SheetViewport` 위젯 도입
- 후보 기반:
  - 1순위: `two_dimensional_scrollables`의 `TableView`
  - 2순위: custom 2D viewport
- 필수 UI 구성:
  - corner cell
  - pinned column headers
  - pinned row headers
  - scrollable body grid
  - frozen pane split
  - desktop scrollbar
  - workbook sheet tabs

### Rust side
- 현재 `PageRenderModel` 기반 sheet window는 transitional wire model로 유지 가능
- 하지만 장기적으로는 아래 정보를 가진 dedicated sheet viewport contract가 더 적절하다:
  - visible row range
  - visible column range
  - row metrics map
  - column metrics map
  - merged cell regions
  - frozen pane metadata
  - visible cell records
  - selection/search anchors

## How to determine the render window

### Do not
- Excel의 global max row/column를 initial viewport로 사용하지 않는다.
- `dimension`만 단독으로 사용하지 않는다.

### Do
- `effective bounds`를 아래 union으로 계산한다:
  - `dimension ref`
  - 실제 non-empty cells
  - custom row heights
  - custom column widths
  - merged ranges
  - frozen panes `topLeftCell`

### Initial viewport policy
- 기본 시작 셀:
  - frozen pane이 있으면 `topLeftCell`
  - 없으면 `A1`
- 기본 visible budget:
  - desktop first pass: `rows 40+`, `columns 16+`
  - mobile first pass: `rows 18+`, `columns 6+`
  - 첫 frame 이후 viewport 크기를 보고 visible count + overscan 기준으로 window를 확장한다
- 기본 overscan:
  - desktop: `+20 rows`, `+4 columns`
  - mobile: `+12 rows`, `+2 columns`

### Sheet navigation policy
- `XLSX`는 `next/previous`로 탐색하지 않는다.
- workbook의 visible sheet를 하단 탭으로 노출한다.
- 탭 label은 sheet name을 사용한다.
- hidden / veryHidden sheet는 기본 탭 strip에 노출하지 않는다.
- active sheet가 있으면 그 시트를 먼저 연다.
- 시트 전환은 같은 viewer shell 안에서 viewport만 교체한다.

### Scroll policy
- scroll 시 viewport window만 다시 요청
- full sheet render 금지
- overscan은 row/column metric 변동을 고려해 보수적으로 유지
- used range까지만 clamp하지 않고 blank grid도 함께 유지한다
- trailing edge에 닿으면 window를 뒤로 미는 대신 `endRow/endColumn`을 먼저 늘려 누적 확장한다
- leading edge에서 이전 영역이 필요하면 `startRow/startColumn`을 앞쪽으로 넓힌다
- 즉 기본 전략은 `sliding window`보다 `cumulative expanding window`에 가깝다
- 장기적으로 메모리 상한이 필요해지면 far range prune/LRU를 hardening 단계에서 추가한다
- 이후 zoom이 들어오면 visible count 계산은 scale factor를 반영하도록 확장한다

### Demo engine performance policy
- desktop demo bridge는 request마다 `cargo run`을 다시 호출하지 않는다.
- `viewer_cli`는 persistent `serve` 프로세스로 떠 있고 stdin/stdout JSON 프로토콜로 요청을 직렬 처리한다.
- `XLSX`는 open 시점에 workbook/shared strings/styles/cells/metrics/merge/frozen pane까지 한 번 파싱해 캐시한다.
- 이후 visible window, search, selection 요청은 cached package를 재사용한다.
- 즉 large-sheet 성능 병목은 "프로세스 재시작 + ZIP/XML 재파싱"이 아니라 실제 viewport diff와 painting 비용으로 좁혀야 한다.

### Flutter painting policy
- `XLSX` body는 cell마다 개별 widget을 쌓는 방식보다 sheet 전용 painter를 우선 사용한다.
- blank/default cell까지 전부 widget tree에 올리면 large-sheet scroll에서 rebuild/paint 비용이 급격히 커진다.
- 따라서 row/column header 외 본문은 가능한 한 적은 widget 수로 유지하고, 보이는 rect에 겹치는 node만 paint/cull한다.
- viewport size가 커지면 scroll edge를 기다리지 않고 minimum visible window를 다시 평가해 빈 영역이 남지 않게 한다.

### Interaction policy
- `XLSX`는 `DOCX/PPTX`처럼 문자 range selection을 기본 interaction으로 두지 않는다.
- search 결과는 `sheet + cell(row/column)` 기준으로 반환하는 편이 맞다.
- search 결과 선택 시:
  - 해당 sheet를 연다
  - target cell이 현재 window 밖이면 viewport를 그 셀 중심으로 재요청한다
  - target cell rect를 highlight한다
- drag selection도 장기적으로는 text range가 아니라 cell range selection으로 전환한다.

## UX priorities

### P0
- pinned row/column headers
- 2D scroll
- visible sheet tabs
- accurate column widths / row heights
- frozen panes
- merged cell placement
- large-sheet virtualization
- cell-centric search and selection

### P1
- active cell highlight
- search jump to cell
- read-only cell detail bar
- number/date display fidelity

### P2
- keyboard navigation
- resize affordance
- stacked headers
- sheet tabs polish

## Immediate plan change
- `XLSX demo real integration acceptance`는 "열린다"만으로 닫지 않는다.
- 아래 조건이 있어야 acceptance로 본다:
  - sheet가 page처럼 보이지 않는다
  - row/column headers가 있다
  - 2D scroll이 가능하다
  - frozen pane이 자연스럽다
  - visible window가 scroll에 맞춰 갱신된다

## Sources
- Flutter `DataTable` performance guidance:
  - https://api.flutter.dev/flutter/material/DataTable-class.html
- Flutter `two_dimensional_scrollables` / `TableView`:
  - https://pub.dev/packages/two_dimensional_scrollables
  - https://pub.dev/documentation/two_dimensional_scrollables/latest/
- Excel specifications and limits:
  - https://support.microsoft.com/en-us/office/excel-specifications-and-limits-1672b34d-7043-467e-8e27-269d656771c3
- Excel freeze panes:
  - https://support.microsoft.com/en-au/office/freeze-panes-to-lock-rows-and-columns-dab2ffc9-020d-4026-8121-67dd25f2508f
- SpreadsheetML `dimension` / used range:
  - https://learn.microsoft.com/en-us/dotnet/api/documentformat.openxml.spreadsheet.sheetdimension?view=openxml-3.0.1
- Rows product and keyboard navigation:
  - https://rows.com/features
  - https://rows.com/docs/keyboard-shortcuts
- Syncfusion Flutter DataGrid references:
  - https://help.syncfusion.com/flutter/datagrid/freeze-panes
  - https://help.syncfusion.com/flutter/datagrid/columns-resizing
  - https://help.syncfusion.com/flutter/datagrid/stacked-headers
