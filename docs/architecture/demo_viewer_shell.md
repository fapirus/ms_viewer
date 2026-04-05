# Demo Viewer Shell Refactor

## Why now

현재 demo 앱은 `문서 목록 rail + preview pane` 구조를 기본으로 둔다.

- [examples/flutter_demo/lib/src/demo_home_page.dart](/Users/ultramarine/Documents/Workspace/Fapirus/ms_viewer/examples/flutter_demo/lib/src/demo_home_page.dart) 가 source rail과 preview pane을 한 화면에 같이 렌더한다.
- [packages/ms_viewer/lib/src/widgets/ms_document_view.dart](/Users/ultramarine/Documents/Workspace/Fapirus/ms_viewer/packages/ms_viewer/lib/src/widgets/ms_document_view.dart) 는 포맷별 viewer surface를 제공하지만, 상위 shell은 여전히 demo 브라우저 성격이 강하다.

이 구조는 다음 문제를 만든다.

- `XLSX`의 2D viewport, frozen pane, large-sheet scroll을 정확히 검증하기 어렵다.
- `DOCX/PPTX`도 실제 viewer가 아니라 preview shell처럼 느껴진다.
- 최종 제품이 제공해야 할 `문서 전용 화면` UX를 미리 검증할 수 없다.

## Decision

`XLSX visual parity acceptance`를 닫기 전에 demo 앱 shell을 refactor 한다.

핵심 결정은 다음과 같다.

1. 문서 목록 화면과 뷰어 화면을 분리한다.
2. `DOCX/PPTX`는 엔진의 page/slide index는 유지하되, UI는 버튼 paging이 아니라 연속 스크롤 기반 viewer로 바꾼다.
3. `XLSX`는 viewer 화면에서 가능한 한 full-bleed에 가깝게 viewport를 제공한다.
4. 검색, 파일 열기, back navigation은 viewer chrome으로 옮긴다.

## Non-goals

- 지금 단계에서 최종 제품 수준의 annotation/editing을 넣지 않는다.
- document management 기능을 demo 앱에 과도하게 넣지 않는다.
- format engine 동작을 viewer shell refactor와 같이 뒤엎지 않는다.

## Target structure

### 1. Library screen

역할:

- fixture 목록
- picked/dropped file 목록
- 문서 메타 정보 요약
- 문서 open entry point

특징:

- `demo home = library`
- 문서를 선택하면 별도 viewer route로 push

### 2. Viewer screen

역할:

- 문서 열람 전용
- 검색
- back
- 포맷별 viewer controls

공통 chrome:

- top app bar
- title
- search affordance
- open file / close / back

포맷별 surface:

- `DOCX`: vertical continuous page scroll
- `PPTX`: vertical slide list scroll
- `XLSX`: full-screen 2D sheet viewport

## Format-specific UI policy

### DOCX

- 엔진은 page model 유지
- Flutter는 `ListView` 또는 sliver 기반으로 page stack을 세로로 렌더
- page 번호는 secondary 정보로 유지
- next/previous 버튼은 제거하거나 보조 기능으로 내린다

### PPTX

- 엔진은 slide index 유지
- Flutter는 slide list를 세로 스크롤로 렌더
- 필요하면 slide navigator strip을 후속으로 추가

### XLSX

- viewer screen에서 side rail, preview card, 불필요한 padding을 제거
- row/column header + grid viewport가 화면의 주 surface가 된다
- large-sheet scroll, frozen pane, search jump를 이 화면에서 검증한다

## Refactor phases

### Phase A: Route split

- library screen / viewer screen 분리
- document selection 시 viewer route push

### Phase B: Viewer chrome

- 공통 app bar
- search placement 재조정
- error/loading/password UI를 viewer shell 안으로 이동

### Phase C: Continuous document surfaces

- DOCX continuous page stack
- PPTX continuous slide stack
- XLSX full-screen viewport

### Phase D: Acceptance

- `DOCX/PPTX/XLSX` 각각 viewer shell 기준 수동 확인
- issue screenshot 비교는 viewer screen 기준으로 다시 수집

## Acceptance criteria

이 refactor는 아래가 충족되면 닫는다.

1. demo 앱 첫 화면은 document library 역할만 한다.
2. 문서를 열면 별도의 viewer screen으로 이동한다.
3. `DOCX/PPTX`는 button paging이 아니라 scroll viewing으로 동작한다.
4. `XLSX`는 list rail 없이 전용 viewport로 보인다.
5. 검색과 back navigation이 viewer chrome 안에서 동작한다.
