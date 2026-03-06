# Cache Policy

## Goal
대용량 문서를 archive-backed lazy loading 방식으로 다루기 위한 캐시 정책을 정의한다.

## Design rules
- 문서 전체를 한 번에 메모리에 올리지 않는다.
- ZIP archive index와 최소 메타데이터만 항상 유지한다.
- 페이지, 슬라이드, 시트 렌더 결과는 필요할 때만 만든다.
- eviction은 단순한 LRU 정책으로 시작한다.
- 성능보다 안정성을 우선하되, 첫 화면 반응은 빠르게 유지한다.

## Cache layers

### 1. Archive index cache
항상 유지한다.

Contains:
- ZIP central directory index
- part lookup table
- relationship lookup table
- content types summary

### 2. Parse cache
필요 파트만 유지한다.

Contains:
- parsed XML fragments
- shared resources
- style/theme summaries

### 3. Render cache
가시 영역 중심으로 유지한다.

Contains:
- page render model
- slide render model
- sheet window render model

### 4. Resource cache
공용 리소스를 캐시한다.

Contains:
- decoded image metadata
- shared strings summary
- font resolution result

## Initial eviction policy
- 현재 페이지 또는 슬라이드는 항상 유지
- 인접 1개 이전, 1개 이후 항목은 prefetch 허용
- viewport에서 멀어진 render cache는 eviction 대상
- archive index cache는 eviction하지 않음
- resource cache는 size 기반보다 count 기반으로 단순 시작

## Initial thresholds
- page render cache: max 3 entries
- slide render cache: max 3 entries
- sheet window cache: max 2 entries
- parsed part cache: max 32 entries
- decoded image metadata cache: max 64 entries

## Access strategy
1. open document
2. build archive index cache
3. load first visible page or slide only
4. prefetch adjacent viewport unit
5. evict least recently used invisible entries when threshold exceeded

## Viewport contract
Flutter는 현재 보고 있는 viewport unit을 엔진에 명시적으로 알려준다.
초기 단위는 아래와 같다.
- DOCX: page index
- PPTX: slide index
- XLSX: sheet index + visible window

## Why this shape
- DOCX 페이지 모델과 직접 맞물린다.
- PPTX는 slide 단위라 same policy를 거의 그대로 사용 가능하다.
- XLSX는 전체 시트를 그리지 않고 visible window 캐시로 확장 가능하다.

## Deferred items
- memory-byte based adaptive limit
- background decode priority queue
- image bitmap cache policy
- explicit warm-up strategy for long documents
