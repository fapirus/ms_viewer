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
| `fixday` 공통 | viewer 수동 점검 | 페이지 클릭 시 일부 이미지가 반짝거림 | selection/highlight로 인한 rebuild 시 `Image.memory` 재구성 영향 가능성 | P2 | 중간 | `PPTX repaint flicker 완화` | `gaplessPlayback` 1차 반영, 후속 관찰 |
| `fixday` PDF 대비 잔차 | PDF/Viewer 비교 | 폰트, 줄 간격, 세부 배치가 PowerPoint/PDF와 완전히 같지 않음 | theme font/default style, exact glyph metrics, line break 정밀도가 아직 부족 | P2 | 매우 높음 | `PPTX theme font와 기본 스타일 메트릭 보정` | 후속 |

## 2026-03-07 fixday 분석 메모
- 실제 문서 기준으로 slide 1에는 top-level text 3개 외에도 table, repeated icon images, master background가 존재한다.
- slide 2~6에는 `grpSp` 중첩이 많아 top-level만 읽으면 실제 자산 대부분을 놓친다.
- 이번 라운드에서는 아래 경로를 보강했다.
  - `grpSp` 재귀 탐색
  - `graphicFrame/table` 최소 렌더
  - slide/layout/master 조합 렌더
  - placeholder bounds/body inset 상속
  - image `gaplessPlayback` 적용
- 아직 남은 것은 visual parity 품질 조정이다.
  - exact font metrics
  - theme/default text style 해석
  - repaint flicker 최종 확인

## 현재 결론
- 이번 라운드의 1차 목표는 `빈 화면/대량 누락`을 없애는 것이고, 그 범위는 구현했다.
- 다음 수동 점검에서는 `배경 복구 여부`, `group 이미지 누락 해소 여부`, `table 텍스트/셀 가시성`, `클릭 반짝임 잔존 여부`를 우선 확인하면 된다.
- 그 이후에 남는 차이는 `visual parity` 단계에서 폰트와 간격 정밀도로 넘기는 편이 맞다.
