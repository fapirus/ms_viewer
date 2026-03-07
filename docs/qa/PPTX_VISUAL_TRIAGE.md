# PPTX Visual Triage

이 문서는 `issue/powerpoint/*`에 저장한 실제 문서와 PDF/Viewer 비교 자료를 기준으로
현재 PPTX 시각 회귀를 분류한 기록이다.

## 분석 규칙
- `issue/`는 로컬 분석용 원본과 비교 자료를 보관한다.
- 원인이 고정되면 최소 재현 케이스를 synthetic test 또는 `fixtures/regression/`으로 승격한다.
- 한 이슈는 `증상 -> 원인 가설 -> 우선순위 -> 체크리스트 항목` 순서로 관리한다.

## 우선순위 표

| Case | 기준 자료 | 현재 증상 | 원인 가설 | 우선순위 | 난이도 | 체크리스트 항목 | 상태 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `fixday` 공통 | `issue/powerpoint/fixday/FIX발표자료.pptx`, `FIX발표자료.pdf` | 슬라이드 배경이 보이지 않음 | slide 자체만 그리고 layout/master 상속 배경과 장식 shape를 렌더하지 않음 | P0 | 높음 | `PPTX master/layout background inheritance 보정` | 이번 라운드 반영 |
| `fixday` slide 1~6 | 같은 자료 | 이미지와 텍스트 누락이 많음 | `p:grpSp` 내부 `sp/pic`를 재귀 탐색하지 않음 | P0 | 높음 | `PPTX group shape recursion 보정` | 이번 라운드 반영 |
| `fixday` slide 1 | 같은 자료 | 표와 표 내부 텍스트가 거의 비어 보임 | `p:graphicFrame/a:tbl`를 렌더 모델로 변환하지 않음 | P0 | 높음 | `PPTX graphicFrame/table 렌더 보정` | 이번 라운드 반영 |
| `fixday` slide 1 외 placeholder slide | 같은 자료 | 실제 입력 텍스트가 있는데 위치가 없어 렌더되지 않음 | slide placeholder가 layout placeholder의 bounds/bodyPr를 상속받지 못함 | P0 | 중상 | `PPTX placeholder layout inheritance 보정` | 이번 라운드 반영 |
| `fixday` slide 1, 4, 12 | 같은 자료 | 제목/본문 텍스트가 검정색 또는 기본 폰트로 보여 PDF와 크게 다름 | layout/master의 `lstStyle`/`txStyles`, theme font, gradient text fill을 해석하지 못함 | P1 | 높음 | `PPTX theme font와 기본 스타일 메트릭 보정` | 이번 라운드 반영 |
| `fixday` slide 1 | 같은 자료 | bullet, 문단 들여쓰기, 줄 간격이 달라 정보 블록이 헐거워 보임 | `buChar`, `buClr`, `marL`, `indent`, `lnSpc`, `spcBef/Aft`를 레이아웃에 반영하지 않음 | P1 | 높음 | `PPTX 텍스트 박스 정렬과 줄바꿈 보정` | 이번 라운드 반영 |
| `fixday` slide 1 | 같은 자료 | 우하단 표의 셀 배경색, 테두리, 일부 텍스트 스타일이 PDF와 다름 | `a:tbl` cell fill/border와 cell 내부 rich text style 상속이 불완전함 | P1 | 높음 | `PPTX 표 셀 배경색과 표 내부 텍스트 스타일 보정` | 이번 라운드 반영 |
| `fixday` slide 2~5 | 같은 자료 | 제목/본문 일부에 underline이 빠지고 `Client -> Clien`처럼 마지막 글자가 잘림 | underline decoration과 last-glyph width/box clipping 처리가 부족함 | P1 | 높음 | `PPTX 밑줄, 텍스트 clipping, 제목/본문 개행 보정` | 후속 |
| `fixday` slide 2, 5, 6 | 같은 자료 | 회색/흰색 오버레이, 반투명 마스크, 겹침 순서가 달라 내용이 어색하게 가려짐 | shape z-order, opacity, rounded corner, shadow/effect 일부가 미지원이거나 순서 계산이 다름 | P1 | 높음 | `PPTX shape z-order, opacity, rounded corner, overlay composition 보정` | 후속 |
| `fixday` slide 3, 5 | 같은 자료 | 이미지 방향, crop, fit, 위치가 PDF와 다름 | image transform/flip/crop/anchor 계산이 불완전함 | P1 | 중상 | `PPTX shape/image transform 및 crop 보정` | 후속 |
| `fixday` 공통 | viewer 수동 점검 | 페이지 클릭 시 일부 이미지가 반짝거림 | selection/highlight로 인한 rebuild 시 `Image.memory` 재구성 영향 가능성 | P2 | 중간 | `PPTX repaint flicker 완화` | `gaplessPlayback` 1차 반영, 후속 관찰 |
| `fixday` PDF 대비 잔차 | PDF/Viewer 비교 | 폰트, 줄 간격, 세부 배치가 PowerPoint/PDF와 완전히 같지 않음 | OS에 없는 원본 서체, exact glyph metrics, effect/shadow 미지원 때문에 최종 오차가 남음 | P2 | 매우 높음 | `PPTX visual parity acceptance pass` 이후 hardening backlog | 후속 |

## 2026-03-07 fixday 분석 메모
- 실제 문서 기준으로 slide 1에는 top-level text 3개 외에도 table, repeated icon images, master background가 존재한다.
- slide 2~6에는 `grpSp` 중첩이 많아 top-level만 읽으면 실제 자산 대부분을 놓친다.
- 이번 라운드에서는 아래 경로를 보강했다.
  - `grpSp` 재귀 탐색
  - `graphicFrame/table` 최소 렌더
  - slide/layout/master 조합 렌더
  - placeholder bounds/body inset 상속
  - `lstStyle`/`txStyles` 기본 문단 스타일 상속
  - theme font 해석(`+mn-*`, `+mj-*`)
  - gradient title text, bullet, paragraph spacing 반영
  - image `gaplessPlayback` 적용
- 아직 남은 것은 visual parity 품질 조정이다.
  - table cell fill/border/text style fidelity
  - underline, last-glyph clipping, mixed Korean/Latin wrapping
  - shape/image crop, rotation, z-order, opacity, rounded corner, effect/shadow
  - repaint flicker 최종 확인
  - exact glyph metrics

## 현재 결론
- 이번 라운드의 1차 목표는 `빈 화면/대량 누락`을 없애는 것이고, 그 범위는 구현했다.
- 현재 `acceptance`를 막는 항목은 아래 네 가지다.
  - 표 셀 배경색/표 내부 텍스트 스타일
  - underline, 텍스트 clipping, 제목/본문 개행
  - shape z-order/opacity/rounded corner/overlay composition
  - image transform/flip/crop/placement
- 아래 항목은 `Hardening`으로 넘겨도 된다.
  - exact glyph metrics
  - 서체 availability 차이에 따른 미세 spacing 오차
  - shadow/effect의 픽셀 단위 정밀도
