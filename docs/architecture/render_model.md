# Render Model

## Goal
Flutter가 그릴 수 있는 렌더 모델의 공통 구조를 정의한다.

## Design rules
- Rust는 최종 렌더링 결과가 아니라 Flutter가 그릴 수 있는 구조화된 모델을 반환한다.
- 포맷별 내부 구조는 숨기고, 공통 페이지 또는 슬라이드 렌더 모델로 변환한다.
- 검색 하이라이트와 텍스트 선택을 위해 모든 텍스트는 bounds와 anchor 정보를 가진다.
- 이미지와 도형은 공통 node 형태로 표현하고, 포맷별 세부 차이는 payload로 확장한다.

## Initial model

### PageRenderModel
하나의 페이지 또는 슬라이드에 대응한다.

Fields:
- `pageIndex`: 0-based page index
- `width`: logical width
- `height`: logical height
- `nodes`: render nodes in paint order
- `selectionAnchors`: selectable text anchor points

### RenderNode
공통 paint node.

#### Text node
- `text`
- `bounds`
- `style`
- `range`

#### Image node
- `bounds`
- `resourceId`
- `description`

#### Box node
- `bounds`
- `fillColor`
- `strokeColor`
- `strokeWidth`

## Coordinate system
- 좌측 상단 원점
- 단위는 logical pixel 유사 좌표
- 확대/축소는 Flutter가 담당하고, Rust는 base layout 좌표만 제공한다.

## Selection model
### SelectionAnchor
- `nodeIndex`
- `charIndex`
- `x`
- `y`

선택은 text node 내부 문자 오프셋 기준으로 계산한다.
표 셀, 이미지, 도형 자체 선택은 MVP 범위에서 제외한다.

## Hit testing model
초기에는 별도 hit-test payload를 만들지 않고 text node bounds와 selection anchor를 이용한다.
필요해지면 이후 `hitRegions`를 추가한다.

## Search model linkage
검색 결과는 render model의 text range와 연결되어야 한다.
초기에는 각 text node가 `start`/`end` range를 가지는 구조로 시작한다.

## Why this shape
- DOCX 페이지 모델에 바로 쓸 수 있다.
- PPTX 슬라이드도 거의 같은 구조로 표현 가능하다.
- XLSX는 셀 grid를 box/text 조합으로 투영할 수 있다.
- Flutter에서 paint order와 selection overlay를 구현하기 쉽다.

## Deferred items
- vector path node
- rich text decoration details
- dedicated hit-test regions
- transform matrix node
