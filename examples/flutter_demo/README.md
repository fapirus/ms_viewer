# flutter_demo

MS 문서 뷰어 데모 앱입니다.

현재 검증 가능한 경로
- bundled DOCX fixture open
- file picker DOCX open
- desktop drop DOCX open

현재 엔진 연결 방식
- desktop: Rust `viewer_cli`를 `cargo run`으로 호출하는 개발용 bridge
- mobile: 아직 실연동 전

실행 예시
```bash
cd examples/flutter_demo
fvm flutter run -d macos
```

수동 점검 포인트
- fixture 목록에서 `docx_plain_text.docx` 열기
- `Open File`로 `.docx` 선택 후 첫 페이지 표시 확인
- desktop drop으로 `.docx` 드롭 후 첫 페이지 표시 확인
- 검색 입력 후 결과 목록과 페이지 이동 확인
- 드래그 선택 하이라이트 확인
